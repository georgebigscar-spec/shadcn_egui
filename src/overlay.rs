//! Floating UI: Select, DropdownMenu, ContextMenu, Menubar, Popover, HoverCard, Sheet, Tooltip, Dialog, Toaster.

use egui::{
    Align2, Color32, Context, CursorIcon, FontId, Frame, Id, Margin, Modal, Popup,
    PopupCloseBehavior, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
    Widget, WidgetInfo, WidgetType,
};

use crate::basic::{paint_check, paint_chevron};
use crate::button::Button;
use crate::theme::{paint_focus_ring, Theme};

// ---------------------------------------------------------------------------

/// `<Select>`: a trigger that looks like an input and opens a list of options.
#[must_use = "add it with `ui.add(...)`"]
pub struct Select<'a> {
    selected: &'a mut Option<usize>,
    options: &'a [&'a str],
    placeholder: String,
    width: f32,
}

impl<'a> Select<'a> {
    pub fn new(selected: &'a mut Option<usize>, options: &'a [&'a str]) -> Self {
        Self { selected, options, placeholder: "Select…".into(), width: 180.0 }
    }
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Widget for Select<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(self.width, 36.0), Sense::click());
        let current = self.selected.and_then(|i| self.options.get(i).copied());
        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::ComboBox, ui.is_enabled(), current.unwrap_or(&self.placeholder))
        });
        response = response.on_hover_cursor(CursorIcon::PointingHand);

        match current {
            Some(s) => paint_select_trigger(ui, rect, s, t.foreground, &t),
            None => paint_select_trigger(ui, rect, &self.placeholder, t.muted_foreground, &t),
        }

        let mut changed = false;
        let popup = Popup::menu(&response)
            .width(self.width)
            .gap(4.0)
            .frame(t.popover_frame())
            .show(|ui| {
                ui.set_min_width(self.width);
                ui.spacing_mut().item_spacing.y = 0.0;
                for (i, option) in self.options.iter().enumerate() {
                    let is_sel = *self.selected == Some(i);
                    if menu_row(ui, option, None, is_sel).clicked() {
                        *self.selected = Some(i);
                        changed = true;
                        ui.close();
                    }
                }
            });
        if response.has_focus() || popup.is_some() {
            paint_focus_ring(ui.painter(), rect, t.radius_md(), &t);
        }
        if changed {
            response.mark_changed();
        }
        response
    }
}

/// The input-like box of a Select or Combobox: text on the left, chevron on the right.
pub(crate) fn paint_select_trigger(ui: &Ui, rect: Rect, text: &str, color: Color32, t: &Theme) {
    let p = ui.painter();
    let fill = if t.dark { t.input.gamma_multiply(0.3) } else { Color32::TRANSPARENT };
    p.add(t.shadow_sm().as_shape(rect, t.radius_md()));
    p.rect(rect, t.radius_md(), fill.to_opaque_or(t.background), Stroke::new(1.0, t.input), StrokeKind::Inside);
    let g = p.layout_no_wrap(text.to_owned(), FontId::proportional(14.0), color);
    let clip = rect.shrink2(Vec2::new(12.0, 0.0)).with_max_x(rect.right() - 28.0);
    p.with_clip_rect(clip.intersect(p.clip_rect()))
        .galley(Pos2::new(rect.left() + 12.0, rect.center().y - g.size().y / 2.0), g, color);
    paint_chevron(p, Pos2::new(rect.right() - 16.0, rect.center().y), 0.0, t.muted_foreground);
}

trait OpaqueOr {
    fn to_opaque_or(self, fallback: Color32) -> Color32;
}
impl OpaqueOr for Color32 {
    fn to_opaque_or(self, fallback: Color32) -> Color32 {
        if self.a() == 0 { fallback } else { self }
    }
}

/// One row inside a menu or select list: 32px, accent on hover, optional
/// shortcut on the right, check mark when `checked`.
fn menu_row(ui: &mut Ui, text: &str, shortcut: Option<&str>, checked: bool) -> Response {
    let t = Theme::get(ui.ctx());
    let text_width = |s: &str, size: f32| {
        ui.painter().layout_no_wrap(s.to_owned(), FontId::proportional(size), t.popover_foreground).size().x
    };
    let mut desired = 8.0 + text_width(text, 14.0) + 8.0;
    if let Some(s) = shortcut {
        desired += 24.0 + text_width(s, 12.0);
    }
    if checked {
        desired += 24.0;
    }
    // During the popup's sizing pass take only what the content needs, so the
    // menu ends up as wide as its widest row; afterwards fill that width.
    let width = if ui.is_sizing_pass() { desired } else { ui.available_width().max(desired) };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), text));
    let p = ui.painter();
    if response.hovered() || response.has_focus() {
        p.rect_filled(rect, t.radius_sm(), t.accent);
    }
    let fg = if response.hovered() { t.accent_foreground } else { t.popover_foreground };
    let g = p.layout_no_wrap(text.to_owned(), FontId::proportional(14.0), fg);
    p.galley(Pos2::new(rect.left() + 8.0, rect.center().y - g.size().y / 2.0), g, fg);
    if let Some(s) = shortcut {
        p.text(
            Pos2::new(rect.right() - 8.0, rect.center().y),
            Align2::RIGHT_CENTER,
            s,
            FontId::proportional(12.0),
            t.muted_foreground,
        );
    }
    if checked {
        let c = Rect::from_center_size(Pos2::new(rect.right() - 16.0, rect.center().y), Vec2::splat(14.0));
        paint_check(p, c, fg, 1.5);
    }
    response
}

// ---------------------------------------------------------------------------

/// `<DropdownMenu>` attached to a trigger. Build the contents with
/// [`menu_item`], [`menu_label`] and [`menu_separator`].
pub fn dropdown_menu<R>(
    trigger: &Response,
    width: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let t = Theme::get(&trigger.ctx);
    Popup::menu(trigger)
        .width(width)
        .gap(4.0)
        .frame(t.popover_frame())
        .show(|ui| {
            ui.set_min_width(width);
            ui.spacing_mut().item_spacing.y = 0.0;
            add_contents(ui)
        })
        .map(|r| r.inner)
}

/// `<ContextMenu>`: the same menu as [`dropdown_menu`], opened by a right click
/// on `target` at the pointer. `target` must sense clicks (buttons, tree rows,
/// or `ui.interact(rect, id, Sense::click())`).
///
/// ```ignore
/// let row = tree.leaf(path, name);
/// context_menu(&row, 220.0, |ui| {
///     if menu_item(ui, "Stage", None).clicked() { /* ... */ }
///     menu_item(ui, "Discard changes", None);
/// });
/// ```
pub fn context_menu<R>(
    target: &Response,
    width: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let t = Theme::get(&target.ctx);
    Popup::context_menu(target)
        .width(width)
        .frame(t.popover_frame())
        .show(|ui| {
            ui.set_min_width(width);
            ui.spacing_mut().item_spacing.y = 0.0;
            add_contents(ui)
        })
        .map(|r| r.inner)
}

/// A clickable menu entry; closes the menu when clicked.
pub fn menu_item(ui: &mut Ui, text: &str, shortcut: Option<&str>) -> Response {
    let r = menu_row(ui, text, shortcut, false);
    if r.clicked() {
        ui.close();
    }
    r
}

/// A checkable menu entry (`DropdownMenuCheckboxItem`).
pub fn menu_checkbox(ui: &mut Ui, text: &str, checked: &mut bool) -> Response {
    let mut r = menu_row(ui, text, None, *checked);
    if r.clicked() {
        *checked = !*checked;
        r.mark_changed();
    }
    r
}

/// A non-interactive heading inside a menu.
pub fn menu_label(ui: &mut Ui, text: &str) {
    let t = Theme::get(ui.ctx());
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(RichText::new(text).size(14.0).strong().color(t.popover_foreground));
    });
    ui.add_space(6.0);
}

pub fn menu_separator(ui: &mut Ui) {
    let t = Theme::get(ui.ctx());
    let width = if ui.is_sizing_pass() { 0.0 } else { ui.available_width() };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 9.0), Sense::hover());
    ui.painter().hline(
        (rect.left() - 4.0)..=(rect.right() + 4.0),
        rect.center().y,
        Stroke::new(1.0, t.border),
    );
}

// ---------------------------------------------------------------------------

/// `<Popover>`: free-form content that opens when `trigger` is clicked and
/// closes when clicking outside.
pub fn popover<R>(
    trigger: &Response,
    width: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let t = Theme::get(&trigger.ctx);
    Popup::from_toggle_button_response(trigger)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .width(width)
        .gap(4.0)
        .frame(t.popover_frame().inner_margin(Margin::same(16)))
        .show(add_contents)
        .map(|r| r.inner)
}

// ---------------------------------------------------------------------------

/// `<Menubar>`: a row of menus like File / Edit / View. Once one is open,
/// hovering another trigger switches to it.
///
/// ```ignore
/// menubar(ui, "main_menu", |mb| {
///     mb.menu("Repository", |ui| {
///         if menu_item(ui, "Fetch", Some("⌘F")).clicked() { /* ... */ }
///     });
///     mb.menu("Branch", |ui| { menu_item(ui, "New branch…", None); });
/// });
/// ```
pub fn menubar<R>(ui: &mut Ui, id_salt: impl egui::AsId, add_menus: impl FnOnce(&mut MenubarUi<'_>) -> R) -> R {
    let t = Theme::get(ui.ctx());
    let id = Id::new(id_salt);
    let open: Option<usize> = ui.data(|d| d.get_temp(id)).flatten();
    let frame = Frame::new()
        .fill(t.background)
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(t.radius_md())
        .inner_margin(Margin::same(3))
        .shadow(t.shadow_sm());
    let (inner, new_open) = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let mut mb = MenubarUi { ui, id, open, new_open: None, index: 0 };
                let r = add_menus(&mut mb);
                (r, mb.new_open)
            })
            .inner
        })
        .inner;
    ui.data_mut(|d| d.insert_temp(id, new_open));
    inner
}

/// Adds menus inside [`menubar`].
pub struct MenubarUi<'a> {
    ui: &'a mut Ui,
    id: Id,
    open: Option<usize>,
    new_open: Option<usize>,
    index: usize,
}

impl MenubarUi<'_> {
    /// `<MenubarMenu>` with a trigger labelled `title`. Build the contents with
    /// [`menu_item`], [`menu_checkbox`], [`menu_label`] and [`menu_separator`].
    pub fn menu<R>(&mut self, title: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let t = Theme::get(self.ui.ctx());
        let i = self.index;
        self.index += 1;
        let g = self.ui.painter().layout_no_wrap(title.to_owned(), FontId::proportional(14.0), Color32::PLACEHOLDER);
        let (rect, _) = self.ui.allocate_exact_size(Vec2::new(g.size().x + 16.0, 28.0), Sense::hover());
        let r = self.ui.interact(rect, self.id.with(i), Sense::click());
        r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, title));
        let mut open = self.open == Some(i);
        if r.clicked() {
            open = !open;
        } else if r.hovered() && self.open.is_some() && !open {
            open = true;
        }
        let p = self.ui.painter();
        if open || r.hovered() {
            p.rect_filled(rect, t.radius_sm(), t.accent);
        }
        let fg = if open || r.hovered() { t.accent_foreground } else { t.foreground };
        p.galley(rect.center() - g.size() / 2.0, g, fg);
        if r.has_focus() {
            paint_focus_ring(p, rect, t.radius_sm(), &t);
        }
        let out = Popup::from_response(&r)
            .kind(egui::PopupKind::Menu)
            .layout(egui::Layout::top_down_justified(egui::Align::Min))
            .align(egui::RectAlign::BOTTOM_START)
            .open_bool(&mut open)
            .width(220.0)
            .gap(6.0)
            .frame(t.popover_frame())
            .show(|ui| {
                ui.set_min_width(220.0);
                ui.spacing_mut().item_spacing.y = 0.0;
                add_contents(ui)
            })
            .map(|r| r.inner);
        if open {
            self.new_open = Some(i);
        }
        out
    }
}

// ---------------------------------------------------------------------------

/// `<HoverCard>`: a popover that appears after hovering `target` for a moment,
/// for previews such as a commit behind its hash.
///
/// ```ignore
/// let hash = ui.label(RichText::new("4f1c2a9").monospace());
/// hover_card(&hash, 300.0, |ui| { ui.label("Merge branch 'feature/git-widgets'"); });
/// ```
pub fn hover_card<R>(target: &Response, width: f32, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let t = Theme::get(&target.ctx);
    let mut tip = egui::Tooltip::for_enabled(target);
    tip.popup = tip.popup.gap(6.0).width(width).frame(t.popover_frame().inner_margin(Margin::same(16)));
    tip.show(|ui| {
        ui.set_width(width - 32.0);
        add_contents(ui)
    })
    .map(|r| r.inner)
}

// ---------------------------------------------------------------------------

/// Which edge a [`Sheet`] slides in from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// `<Sheet>`: a panel that slides in from an edge of the window over a dimmed
/// backdrop, for details or settings. Closes on Escape, a click on the
/// backdrop or the ✕ button.
///
/// ```ignore
/// Sheet::new("details", "Commit 4f1c2a9").description("George · 2 hours ago")
///     .show(ctx, &mut self.details_open, |ui| { /* ... */ });
/// ```
pub struct Sheet {
    id: Id,
    title: String,
    description: Option<String>,
    side: SheetSide,
    size: f32,
}

impl Sheet {
    pub fn new(id_salt: impl egui::AsId, title: impl Into<String>) -> Self {
        Self { id: Id::new(id_salt), title: title.into(), description: None, side: SheetSide::Right, size: 384.0 }
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }
    /// Width (left/right) or height (top/bottom); default 384, like `sm:max-w-sm`.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Shows the sheet while `open` is true; sets it to false when dismissed.
    pub fn show<R>(self, ctx: &Context, open: &mut bool, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let shown = ctx.animate_bool_with_time(self.id.with("anim"), *open, 0.2);
        if !*open {
            return None;
        }
        let t = Theme::get(ctx);
        let screen = ctx.content_rect();
        let vertical = matches!(self.side, SheetSide::Left | SheetSide::Right);
        let size = if vertical { Vec2::new(self.size, screen.height()) } else { Vec2::new(screen.width(), self.size) };
        let hidden = (1.0 - shown) * self.size;
        let (pivot, pos) = match self.side {
            SheetSide::Right => (Align2::RIGHT_TOP, screen.right_top() + Vec2::new(hidden, 0.0)),
            SheetSide::Left => (Align2::LEFT_TOP, screen.left_top() - Vec2::new(hidden, 0.0)),
            SheetSide::Top => (Align2::LEFT_TOP, screen.left_top() - Vec2::new(0.0, hidden)),
            SheetSide::Bottom => (Align2::LEFT_BOTTOM, screen.left_bottom() + Vec2::new(0.0, hidden)),
        };
        let area = egui::Area::new(self.id)
            .kind(egui::UiKind::Modal)
            .sense(Sense::hover())
            .order(egui::Order::Foreground)
            .interactable(true)
            .pivot(pivot)
            .fixed_pos(pos);
        let mut close_clicked = false;
        let modal = Modal::new(self.id)
            .area(area)
            .backdrop_color(Color32::from_black_alpha((128.0 * shown) as u8))
            .frame(
                Frame::new()
                    .fill(t.background)
                    .stroke(Stroke::new(1.0, t.border))
                    .inner_margin(Margin::same(24))
                    .shadow(t.shadow_lg()),
            )
            .show(ctx, |ui| {
                ui.set_min_size(size - Vec2::splat(48.0));
                ui.set_max_width(size.x - 48.0);
                let top = ui.max_rect().right_top();
                let close = Rect::from_center_size(top + Vec2::new(-4.0, 4.0), Vec2::splat(20.0));
                let r = ui.interact(close, self.id.with("close"), Sense::click());
                r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Close"));
                let c = if r.hovered() { t.foreground } else { t.muted_foreground };
                let m = close.center();
                let s = 4.5;
                ui.painter().line_segment([m + Vec2::new(-s, -s), m + Vec2::new(s, s)], Stroke::new(1.5, c));
                ui.painter().line_segment([m + Vec2::new(-s, s), m + Vec2::new(s, -s)], Stroke::new(1.5, c));
                close_clicked = r.clicked();
                ui.label(RichText::new(&self.title).size(16.0).color(t.foreground));
                if let Some(d) = &self.description {
                    ui.add_space(-4.0);
                    ui.label(RichText::new(d).size(14.0).color(t.muted_foreground));
                }
                ui.add_space(8.0);
                add_contents(ui)
            });
        if modal.should_close() || close_clicked {
            *open = false;
        }
        if shown < 1.0 {
            ctx.request_repaint();
        }
        Some(modal.inner)
    }
}

// ---------------------------------------------------------------------------

/// `<Tooltip>`: `ui.add(button).tooltip("text")`.
pub trait TooltipExt {
    fn tooltip(self, text: impl Into<String>) -> Self;
}

impl TooltipExt for Response {
    fn tooltip(self, text: impl Into<String>) -> Self {
        let t = Theme::get(&self.ctx);
        let mut tip = egui::Tooltip::for_enabled(&self);
        tip.popup = tip.popup.gap(4.0).frame(
            Frame::new()
                .fill(t.primary)
                .corner_radius(t.radius_md())
                .inner_margin(Margin::symmetric(12, 6)),
        );
        let text = text.into();
        tip.show(|ui| {
            ui.label(RichText::new(text).size(12.0).color(t.primary_foreground));
        });
        self
    }
}

// ---------------------------------------------------------------------------

/// `<Dialog>`: a centered modal with title, description and a close button.
pub struct Dialog {
    id: Id,
    title: String,
    description: Option<String>,
    width: f32,
}

/// What happened in a [`Dialog`] this frame.
pub struct DialogResponse<R> {
    pub inner: R,
    /// Escape, a click on the backdrop, or the ✕ button.
    pub should_close: bool,
}

impl Dialog {
    pub fn new(id_salt: impl egui::AsId, title: impl Into<String>) -> Self {
        Self { id: Id::new(id_salt), title: title.into(), description: None, width: 425.0 }
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R) -> DialogResponse<R> {
        let t = Theme::get(ctx);
        let mut close_clicked = false;
        let modal = Modal::new(self.id)
            .backdrop_color(Color32::from_black_alpha(128))
            .frame(
                Frame::new()
                    .fill(t.background)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(t.radius_lg())
                    .inner_margin(Margin::same(24))
                    .shadow(t.shadow_lg()),
            )
            .show(ctx, |ui| {
                ui.set_width(self.width - 48.0);
                let top = ui.max_rect().right_top();
                let close = Rect::from_center_size(top + Vec2::new(-4.0, 4.0), Vec2::splat(20.0));
                let r = ui.interact(close, self.id.with("close"), Sense::click());
                let c = if r.hovered() { t.foreground } else { t.muted_foreground };
                let p = ui.painter();
                let s = 4.5;
                let m = close.center();
                p.line_segment([m + Vec2::new(-s, -s), m + Vec2::new(s, s)], Stroke::new(1.5, c));
                p.line_segment([m + Vec2::new(-s, s), m + Vec2::new(s, -s)], Stroke::new(1.5, c));
                close_clicked = r.clicked();

                ui.label(RichText::new(&self.title).size(18.0).strong().color(t.foreground));
                if let Some(d) = &self.description {
                    ui.add_space(-2.0);
                    ui.label(RichText::new(d).size(14.0).color(t.muted_foreground));
                }
                ui.add_space(8.0);
                add_contents(ui)
            });
        DialogResponse { should_close: modal.should_close() || close_clicked, inner: modal.inner }
    }
}

/// Right-aligned footer row for a dialog or card (`<DialogFooter>`).
pub fn footer<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    ui.add_space(8.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_contents).inner
}

/// Convenience: an `<AlertDialog>` with Cancel / Continue. Returns
/// `Some(true)` on continue, `Some(false)` on cancel or dismiss.
pub fn alert_dialog(
    ctx: &Context,
    id_salt: impl egui::AsId,
    title: &str,
    description: &str,
    action: &str,
) -> Option<bool> {
    let resp = Dialog::new(id_salt, title).description(description).show(ctx, |ui| {
        footer(ui, |ui| {
            if ui.add(Button::new(action)).clicked() {
                return Some(true);
            }
            if ui.add(Button::outline("Cancel")).clicked() {
                return Some(false);
            }
            None
        })
    });
    resp.inner.or(resp.should_close.then_some(false))
}

// ---------------------------------------------------------------------------

/// The look of a [`Toast`]: an icon on the left, and for errors a red title.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastKind {
    #[default]
    Default,
    Success,
    Info,
    Warning,
    Error,
    /// A spinner; stays until it is updated (see [`Toast::update`]) or dismissed.
    Loading,
}

/// Names a queued toast, to update it or to tell which action was clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(u64);

/// A Sonner-style toast with an optional icon, description and action button.
///
/// ```ignore
/// let id = Toast::new("Pushing to origin…").loading().show(ctx);
/// // later, when the push is done:
/// Toast::new("Pushed to origin").success().update(ctx, id);
///
/// let undo = Toast::new("Changes discarded").action("Undo").show(ctx);
/// // every frame:
/// if Toaster::show(ctx) == Some(undo) { /* restore */ }
/// ```
///
/// Toasts close after 4 seconds (see [`Toast::duration`]) unless they are
/// loading; hovering any toast pauses the timers, clicking one dismisses it.
#[must_use = "queue it with `.show(ctx)`"]
#[derive(Clone, Debug)]
pub struct Toast {
    title: String,
    description: Option<String>,
    kind: ToastKind,
    action: Option<String>,
    duration: Option<f64>,
}

impl Toast {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), description: None, kind: ToastKind::Default, action: None, duration: None }
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn success(self) -> Self {
        self.kind(ToastKind::Success)
    }
    pub fn info(self) -> Self {
        self.kind(ToastKind::Info)
    }
    pub fn warning(self) -> Self {
        self.kind(ToastKind::Warning)
    }
    pub fn error(self) -> Self {
        self.kind(ToastKind::Error)
    }
    pub fn loading(self) -> Self {
        self.kind(ToastKind::Loading)
    }
    /// A button on the right; [`Toaster::show`] returns this toast's id when it is clicked.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        self
    }
    /// Seconds before it closes (default 4).
    pub fn duration(mut self, seconds: f64) -> Self {
        self.duration = Some(seconds);
        self
    }

    /// Queues the toast.
    pub fn show(self, ctx: &Context) -> ToastId {
        ctx.data_mut(|d| {
            let s = d.get_temp_mut_or_default::<ToastState>(toast_state_id());
            s.next_id += 1;
            let id = ToastId(s.next_id);
            s.toasts.push(Queued { id, toast: self, elapsed: 0.0 });
            id
        })
    }

    /// Replaces toast `id` with this one and restarts its timer, for example to
    /// turn a loading toast into a success. If `id` was already closed, the
    /// toast is shown again.
    pub fn update(self, ctx: &Context, id: ToastId) {
        ctx.data_mut(|d| {
            let s = d.get_temp_mut_or_default::<ToastState>(toast_state_id());
            match s.toasts.iter_mut().find(|q| q.id == id) {
                Some(q) => {
                    q.toast = self;
                    q.elapsed = 0.0;
                }
                None => s.toasts.push(Queued { id, toast: self, elapsed: 0.0 }),
            }
        });
    }

    /// Closes toast `id`.
    pub fn dismiss(ctx: &Context, id: ToastId) {
        ctx.data_mut(|d| d.get_temp_mut_or_default::<ToastState>(toast_state_id()).toasts.retain(|q| q.id != id));
    }

    fn lifetime(&self) -> f64 {
        match (self.duration, self.kind) {
            (Some(d), _) => d,
            (None, ToastKind::Loading) => f64::INFINITY,
            (None, _) => 4.0,
        }
    }
}

#[derive(Clone)]
struct Queued {
    id: ToastId,
    toast: Toast,
    /// Seconds shown, not counting time with the pointer over the toasts.
    elapsed: f64,
}

#[derive(Clone, Default)]
struct ToastState {
    next_id: u64,
    toasts: Vec<Queued>,
    last_time: Option<f64>,
    hovered: bool,
}

/// Draws the toasts queued with [`Toast::show`], [`toast`] and [`toast_error`].
pub struct Toaster;

fn toast_state_id() -> Id {
    Id::new("shadcn_egui::toasts")
}

/// Queues a toast with an optional description.
pub fn toast(ctx: &Context, title: impl Into<String>, description: Option<&str>) {
    let mut t = Toast::new(title);
    t.description = description.map(str::to_owned);
    let _ = t.show(ctx);
}

/// Queues a red error toast.
pub fn toast_error(ctx: &Context, title: impl Into<String>, description: Option<&str>) {
    let mut t = Toast::new(title).error();
    t.description = description.map(str::to_owned);
    let _ = t.show(ctx);
}

impl Toaster {
    /// Draws the toasts; call it once per frame. Returns the toast whose action
    /// button was clicked this frame (that toast closes).
    pub fn show(ctx: &Context) -> Option<ToastId> {
        let t = Theme::get(ctx);
        let now = ctx.input(|i| i.time);
        let mut state = ctx.data(|d| d.get_temp::<ToastState>(toast_state_id())).unwrap_or_default();
        let dt = state.last_time.map_or(0.0, |last| (now - last).max(0.0));
        state.last_time = Some(now);
        if !state.hovered {
            for q in &mut state.toasts {
                q.elapsed += dt;
            }
        }
        state.toasts.retain(|q| q.elapsed < q.toast.lifetime());
        if state.toasts.is_empty() {
            state.hovered = false;
            ctx.data_mut(|d| d.insert_temp(toast_state_id(), state));
            return None;
        }
        let mut dismissed = None;
        let mut clicked_action = None;
        let mut hovered = false;
        let screen = ctx.content_rect();
        let mut bottom = screen.bottom() - 16.0;
        for q in state.toasts.iter().rev().take(3) {
            let toast = &q.toast;
            let age = q.elapsed as f32;
            let left = (toast.lifetime() - q.elapsed) as f32;
            let fade = (age / 0.15).min(1.0).min(left / 0.3).clamp(0.0, 1.0);
            let area = egui::Area::new(Id::new(("shadcn_toast", q.id.0)))
                .order(egui::Order::Foreground)
                .pivot(Align2::RIGHT_BOTTOM)
                .fixed_pos(Pos2::new(screen.right() - 16.0, bottom + (1.0 - fade) * 12.0))
                .show(ctx, |ui| {
                    ui.multiply_opacity(fade);
                    Frame::new()
                        .fill(t.popover)
                        .stroke(Stroke::new(1.0, t.border))
                        .corner_radius(t.radius_lg())
                        .inner_margin(Margin::same(16))
                        .shadow(t.shadow_lg())
                        .show(ui, |ui| {
                            ui.set_width(324.0);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if let Some(label) = &toast.action {
                                    if ui.add(Button::new(label.as_str()).size(crate::ButtonSize::Sm)).clicked() {
                                        clicked_action = Some(q.id);
                                    }
                                    ui.add_space(4.0);
                                }
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                    if toast.kind != ToastKind::Default {
                                        let (rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 20.0), Sense::hover());
                                        paint_toast_icon(ui, rect, toast.kind, &t);
                                        ui.add_space(4.0);
                                    }
                                    ui.vertical(|ui| {
                                        let title_color = if toast.kind == ToastKind::Error { t.destructive } else { t.popover_foreground };
                                        ui.label(RichText::new(&toast.title).size(14.0).strong().color(title_color));
                                        if let Some(d) = &toast.description {
                                            ui.add_space(-4.0);
                                            ui.label(RichText::new(d).size(13.0).color(t.muted_foreground));
                                        }
                                    });
                                });
                            });
                        });
                });
            hovered |= area.response.contains_pointer();
            if clicked_action.is_none() && toast.kind != ToastKind::Loading && area.response.interact(Sense::click()).clicked() {
                dismissed = Some(q.id);
            }
            bottom -= area.response.rect.height() + 12.0;
        }
        state.hovered = hovered;
        if let Some(id) = dismissed.or(clicked_action) {
            state.toasts.retain(|x| x.id != id);
        }
        ctx.data_mut(|d| d.insert_temp(toast_state_id(), state));
        ctx.request_repaint();
        clicked_action
    }
}

fn paint_toast_icon(ui: &Ui, rect: Rect, kind: ToastKind, t: &Theme) {
    let p = ui.painter();
    let c = rect.center();
    let r = 7.0;
    let (green, amber, blue) = if t.dark {
        (Color32::from_rgb(74, 222, 128), Color32::from_rgb(250, 204, 21), Color32::from_rgb(96, 165, 250))
    } else {
        (Color32::from_rgb(22, 163, 74), Color32::from_rgb(202, 138, 4), Color32::from_rgb(37, 99, 235))
    };
    let stroke = |color| Stroke::new(1.5, color);
    match kind {
        ToastKind::Default => {}
        ToastKind::Success => {
            p.circle_stroke(c, r, stroke(green));
            paint_check(p, Rect::from_center_size(c, Vec2::splat(8.0)), green, 1.5);
        }
        ToastKind::Error => {
            p.circle_stroke(c, r, stroke(t.destructive));
            let d = 2.6;
            p.line_segment([c + Vec2::new(-d, -d), c + Vec2::new(d, d)], stroke(t.destructive));
            p.line_segment([c + Vec2::new(d, -d), c + Vec2::new(-d, d)], stroke(t.destructive));
        }
        ToastKind::Info => {
            p.circle_stroke(c, r, stroke(blue));
            p.line_segment([c + Vec2::new(0.0, -0.5), c + Vec2::new(0.0, 3.5)], stroke(blue));
            p.circle_filled(c + Vec2::new(0.0, -3.2), 0.9, blue);
        }
        ToastKind::Warning => {
            let pts = vec![c + Vec2::new(0.0, -7.0), c + Vec2::new(7.5, 6.0), c + Vec2::new(-7.5, 6.0)];
            p.add(egui::Shape::closed_line(pts, stroke(amber)));
            p.line_segment([c + Vec2::new(0.0, -2.5), c + Vec2::new(0.0, 1.5)], stroke(amber));
            p.circle_filled(c + Vec2::new(0.0, 3.8), 0.9, amber);
        }
        ToastKind::Loading => {
            let start = ui.input(|i| i.time) as f32 * std::f32::consts::TAU * 1.2;
            let points: Vec<Pos2> = (0..=24)
                .map(|k| c + (r - 0.5) * Vec2::angled(start + k as f32 / 24.0 * std::f32::consts::TAU * 0.75))
                .collect();
            p.line(points, stroke(t.muted_foreground));
        }
    }
}
