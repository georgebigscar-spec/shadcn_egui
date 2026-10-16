//! Command (a search field over a filtered list), its dialog form, and Combobox.

use egui::{
    Align2, Color32, Context, CursorIcon, EventFilter, FontId, Frame, Id, Key, Margin, Modal,
    Modifiers, Pos2, Rect, Response, RichText, Sense, Stroke, TextEdit, Ui, Vec2,
    Widget, WidgetInfo, WidgetType,
};

use crate::basic::paint_check;
use crate::overlay::paint_select_trigger;
use crate::theme::{paint_focus_ring, Theme};

/// Case-insensitive subsequence match, like cmdk: `"fbr"` matches `"Feature branch"`.
pub fn fuzzy_match(query: &str, text: &str) -> bool {
    let mut chars = text.chars().flat_map(char::to_lowercase);
    query
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .all(|q| chars.any(|c| c == q))
}

#[derive(Clone, Default)]
struct State {
    query: String,
    /// Index of the highlighted item among the visible ones.
    active: usize,
    /// Visible items last frame, for wrapping the arrow keys.
    count: usize,
}

/// `<Command>`: a search input over a list of items that filters as you type.
/// Arrow keys move the highlight, Enter picks it.
///
/// ```ignore
/// Command::new("palette").show(ui, |cmd| {
///     cmd.group("Branch", |cmd| {
///         if cmd.item("Checkout…", Some("⌘B")) { /* ... */ }
///         if cmd.item("Merge into current", None) { /* ... */ }
///     });
/// });
/// ```
pub struct Command {
    id: Id,
    placeholder: String,
    empty: String,
    max_height: f32,
    autofocus: bool,
}

/// What happened in a [`Command`] this frame.
pub struct CommandResponse<R> {
    pub inner: R,
    /// An item was clicked or picked with Enter.
    pub chosen: bool,
}

impl Command {
    pub fn new(id_salt: impl egui::AsId) -> Self {
        Self {
            id: Id::new(id_salt),
            placeholder: "Type a command or search...".into(),
            empty: "No results found.".into(),
            max_height: 300.0,
            autofocus: false,
        }
    }
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    /// Text shown when nothing matches (`<CommandEmpty>`).
    pub fn empty(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }
    /// Height of the list before it scrolls (default 300).
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }
    /// Keeps keyboard focus in the search field while nothing else has it.
    pub fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    /// Clears the query and highlight, as when the command is opened anew.
    pub fn reset(ctx: &Context, id_salt: impl egui::AsId) {
        Self::reset_id(ctx, Id::new(id_salt));
    }

    /// [`Self::reset`] for an already-built id (hashing it again would miss the state).
    fn reset_id(ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.remove::<State>(id));
    }

    pub fn show<R>(self, ui: &mut Ui, add_items: impl FnOnce(&mut CommandUi<'_>) -> R) -> CommandResponse<R> {
        let t = Theme::get(ui.ctx());
        let mut state: State = ui.data(|d| d.get_temp(self.id)).unwrap_or_default();
        let input_id = self.id.with("input");
        let width = ui.available_width();

        // Take the navigation keys before the text field sees them.
        let focused = ui.memory(|m| m.has_focus(input_id));
        let (down, up, enter) = if focused {
            ui.input_mut(|i| {
                (
                    i.consume_key(Modifiers::NONE, Key::ArrowDown),
                    i.consume_key(Modifiers::NONE, Key::ArrowUp),
                    i.consume_key(Modifiers::NONE, Key::Enter),
                )
            })
        } else {
            (false, false, false)
        };
        if state.count > 0 {
            if down {
                state.active = (state.active + 1) % state.count;
            }
            if up {
                state.active = (state.active + state.count - 1) % state.count;
            }
        }

        let saved_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = Vec2::ZERO;

        // Search row: magnifier, borderless text field, bottom border.
        let (row, _) = ui.allocate_exact_size(Vec2::new(width, 40.0), Sense::hover());
        paint_search_icon(ui.painter(), Pos2::new(row.left() + 18.0, row.center().y), t.muted_foreground);
        let field = Rect::from_min_max(Pos2::new(row.left() + 32.0, row.top()), Pos2::new(row.right() - 8.0, row.bottom()));
        let edit = ui
            .scope_builder(egui::UiBuilder::new().max_rect(field), |ui| {
                ui.add(
                    TextEdit::singleline(&mut state.query)
                        .id(input_id)
                        .frame(Frame::NONE)
                        .font(FontId::proportional(14.0))
                        .text_color(t.foreground)
                        .hint_text(RichText::new(&self.placeholder).color(t.muted_foreground))
                        .vertical_align(egui::Align::Center)
                        .min_size(field.size())
                        .desired_width(field.width()),
                )
            })
            .inner;
        if edit.has_focus() {
            // Keep the arrow keys for the list instead of moving focus.
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    input_id,
                    EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() },
                )
            });
        }
        if self.autofocus && !edit.has_focus() && ui.memory(|m| m.focused().is_none()) {
            edit.request_focus();
        }
        if edit.changed() {
            state.active = 0;
        }
        ui.painter().hline(row.x_range(), row.bottom() - 0.5, Stroke::new(1.0, t.border));

        let query = state.query.clone();
        let (inner, count, chosen) = egui::ScrollArea::vertical()
            .id_salt(self.id.with("list"))
            .max_height(self.max_height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::same(4))
                    .show(ui, |ui| {
                        ui.set_width(width - 8.0);
                        let mut cmd = CommandUi {
                            ui,
                            theme: t.clone(),
                            query: &query,
                            active: state.active,
                            hovered: None,
                            index: 0,
                            heading: None,
                            shown_any: false,
                            enter,
                            scroll: down || up,
                            chosen: false,
                        };
                        let inner = add_items(&mut cmd);
                        let (count, chosen, hovered) = (cmd.index, cmd.chosen, cmd.hovered);
                        if count == 0 {
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(width - 8.0, 64.0), Sense::hover());
                            ui.painter().text(rect.center(), Align2::CENTER_CENTER, &self.empty, FontId::proportional(14.0), t.muted_foreground);
                        }
                        if let Some(h) = hovered {
                            state.active = h;
                        }
                        (inner, count, chosen)
                    })
                    .inner
            })
            .inner;

        ui.spacing_mut().item_spacing = saved_spacing;
        state.count = count;
        state.active = state.active.min(count.saturating_sub(1));
        ui.data_mut(|d| d.insert_temp(self.id, state));
        CommandResponse { inner, chosen }
    }

    /// `<CommandDialog>`: the command in a centered modal, for Ctrl+K style
    /// palettes. Closes on Escape, a click outside, or when an item is picked.
    ///
    /// ```ignore
    /// if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::K))) {
    ///     self.palette_open = !self.palette_open;
    /// }
    /// Command::new("palette").show_dialog(ctx, &mut self.palette_open, |cmd| { .. });
    /// ```
    pub fn show_dialog<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        add_items: impl FnOnce(&mut CommandUi<'_>) -> R,
    ) -> Option<R> {
        let id = self.id;
        if !*open {
            Command::reset_id(ctx, id);
            return None;
        }
        let t = Theme::get(ctx);
        let modal = Modal::new(id.with("dialog"))
            .backdrop_color(Color32::from_black_alpha(128))
            .frame(
                Frame::new()
                    .fill(t.popover)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(t.radius_lg())
                    .shadow(t.shadow_lg()),
            )
            .show(ctx, |ui| {
                ui.set_width(512.0f32.min(ctx.content_rect().width() - 32.0));
                self.autofocus().show(ui, add_items)
            });
        if modal.inner.chosen || modal.should_close() {
            *open = false;
            Command::reset_id(ctx, id);
        }
        Some(modal.inner.inner)
    }
}

/// Builds the list inside a [`Command`]: groups and items, filtered by the query.
pub struct CommandUi<'a> {
    ui: &'a mut Ui,
    theme: Theme,
    query: &'a str,
    active: usize,
    hovered: Option<usize>,
    /// Visible items so far.
    index: usize,
    /// Heading of the current group, drawn before its first visible item.
    heading: Option<String>,
    shown_any: bool,
    enter: bool,
    scroll: bool,
    chosen: bool,
}

impl CommandUi<'_> {
    /// The current search text.
    pub fn query(&self) -> &str {
        self.query
    }

    /// The `Ui` the list is drawn into, for anything custom.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    /// `<CommandGroup>`: a heading over the items added in `add_items`; hidden
    /// when none of them match. Groups are separated by a line.
    pub fn group<R>(&mut self, heading: &str, add_items: impl FnOnce(&mut Self) -> R) -> R {
        self.heading = Some(heading.to_owned());
        let r = add_items(self);
        self.heading = None;
        r
    }

    /// `<CommandItem>`: returns `true` when it is clicked or picked with Enter.
    pub fn item(&mut self, label: &str, shortcut: Option<&str>) -> bool {
        self.row(label, label, shortcut, false)
    }

    /// Like [`Self::item`], but also matches `keywords`, e.g. `"checkout switch"`.
    pub fn item_with_keywords(&mut self, label: &str, keywords: &str, shortcut: Option<&str>) -> bool {
        let haystack = format!("{label} {keywords}");
        self.row(label, &haystack, shortcut, false)
    }

    fn row(&mut self, label: &str, haystack: &str, shortcut: Option<&str>, checked: bool) -> bool {
        if !fuzzy_match(self.query, haystack) {
            return false;
        }
        let t = &self.theme;
        if let Some(heading) = self.heading.take() {
            if self.shown_any {
                let (rect, _) = self.ui.allocate_exact_size(Vec2::new(self.ui.available_width(), 9.0), Sense::hover());
                self.ui.painter().hline((rect.left() - 4.0)..=(rect.right() + 4.0), rect.center().y, Stroke::new(1.0, t.border));
            }
            let (rect, _) = self.ui.allocate_exact_size(Vec2::new(self.ui.available_width(), 28.0), Sense::hover());
            self.ui.painter().text(
                Pos2::new(rect.left() + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                heading,
                FontId::proportional(12.0),
                t.muted_foreground,
            );
        }
        self.shown_any = true;
        let index = self.index;
        self.index += 1;

        let width = self.ui.available_width();
        let (rect, response) = self.ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::click());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
        if response.hovered() && self.ui.input(|i| i.pointer.delta() != Vec2::ZERO) {
            self.hovered = Some(index);
        }
        let active = index == self.active;
        if active && self.scroll {
            response.scroll_to_me(None);
        }
        let p = self.ui.painter();
        if active {
            p.rect_filled(rect, t.radius_sm(), t.accent);
        }
        let fg = if active { t.accent_foreground } else { t.popover_foreground };
        let g = p.layout_no_wrap(label.to_owned(), FontId::proportional(14.0), fg);
        p.galley(Pos2::new(rect.left() + 8.0, rect.center().y - g.size().y / 2.0), g, fg);
        if let Some(s) = shortcut {
            p.text(Pos2::new(rect.right() - 8.0, rect.center().y), Align2::RIGHT_CENTER, s, FontId::proportional(12.0), t.muted_foreground);
        }
        if checked {
            let c = Rect::from_center_size(Pos2::new(rect.right() - 16.0, rect.center().y), Vec2::splat(14.0));
            paint_check(p, c, fg, 1.5);
        }
        let picked = response.clicked() || (active && self.enter);
        self.chosen |= picked;
        picked
    }
}

pub(crate) fn paint_search_icon(p: &egui::Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let c = center + Vec2::new(-1.0, -1.0);
    p.circle_stroke(c, 5.0, stroke);
    p.line_segment([c + Vec2::splat(3.6), c + Vec2::splat(7.0)], stroke);
}

// ---------------------------------------------------------------------------

/// `<Combobox>`: a [`crate::Select`]-like trigger whose list has a search field,
/// for long lists such as branches or remotes.
#[must_use = "add it with `ui.add(...)`"]
pub struct Combobox<'a, S> {
    selected: &'a mut Option<usize>,
    options: &'a [S],
    placeholder: String,
    search: String,
    empty: String,
    width: f32,
}

impl<'a, S: AsRef<str>> Combobox<'a, S> {
    pub fn new(selected: &'a mut Option<usize>, options: &'a [S]) -> Self {
        Self {
            selected,
            options,
            placeholder: "Select…".into(),
            search: "Search...".into(),
            empty: "No results found.".into(),
            width: 200.0,
        }
    }
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    /// Placeholder of the search field inside the list.
    pub fn search_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.search = placeholder.into();
        self
    }
    /// Text shown when nothing matches.
    pub fn empty(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl<S: AsRef<str>> Widget for Combobox<'_, S> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(self.width, 36.0), Sense::click());
        let current = self.selected.and_then(|i| self.options.get(i)).map(|s| s.as_ref());
        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::ComboBox, ui.is_enabled(), current.unwrap_or(&self.placeholder))
        });
        response = response.on_hover_cursor(CursorIcon::PointingHand);
        match current {
            Some(s) => paint_select_trigger(ui, rect, s, t.foreground, &t),
            None => paint_select_trigger(ui, rect, &self.placeholder, t.muted_foreground, &t),
        }

        let cmd_id = response.id.with("command");
        let mut changed = false;
        let popup = egui::Popup::menu(&response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .width(self.width)
            .gap(4.0)
            .frame(t.popover_frame().inner_margin(Margin::ZERO))
            .show(|ui| {
                let r = Command::new(cmd_id)
                    .placeholder(&self.search)
                    .empty(&self.empty)
                    .autofocus()
                    .show(ui, |cmd| {
                        for (i, option) in self.options.iter().enumerate() {
                            let option = option.as_ref();
                            if cmd.row(option, option, None, *self.selected == Some(i)) {
                                *self.selected = Some(i);
                                changed = true;
                            }
                        }
                    });
                if r.chosen {
                    ui.close();
                }
            });
        if popup.is_none() {
            Command::reset(ui.ctx(), cmd_id);
        }
        if response.has_focus() || popup.is_some() {
            paint_focus_ring(ui.painter(), rect, t.radius_md(), &t);
        }
        if changed {
            response.mark_changed();
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_dialog_clears_the_query() {
        let ctx = Context::default();
        let id = Id::new("palette");
        ctx.data_mut(|d| d.insert_temp(id, State { query: "feat".into(), active: 2, count: 3 }));
        let mut open = false;
        Command::new("palette").show_dialog(&ctx, &mut open, |_| ());
        assert!(ctx.data(|d| d.get_temp::<State>(id)).is_none());
    }
}
