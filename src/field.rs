//! Form layout and list rows: Field, FieldSet, field separator, Item.

use egui::{
    Align, Color32, CursorIcon, FontId, Frame, InnerResponse, Layout, Margin, Response, RichText,
    Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType,
};

use crate::theme::Theme;

// ---------------------------------------------------------------------------

/// `<Field>`: a label, the control, a description and an error message,
/// spaced like shadcn forms.
///
/// ```ignore
/// let error = (!email.contains('@')).then_some("Enter a valid email address.");
/// Field::new("Email")
///     .description("We'll never share your email.")
///     .error(error)
///     .show(ui, |ui| ui.add(Input::new(&mut email).invalid(error.is_some())));
///
/// // Label and description on the left, the control on the right:
/// Field::new("Auto fetch").description("Fetch remotes every 5 minutes.").horizontal()
///     .show(ui, |ui| ui.add(Switch::new(&mut auto_fetch)));
/// ```
pub struct Field {
    label: String,
    description: Option<String>,
    error: Option<String>,
    required: bool,
    horizontal: bool,
}

impl Field {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), description: None, error: None, required: false, horizontal: false }
    }
    /// Muted help text under the control (next to the label when horizontal).
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    /// An error message; when set, the label turns red and the error replaces
    /// the description. Takes an `Option` so you can pass your validation result.
    pub fn error(mut self, error: Option<impl Into<String>>) -> Self {
        self.error = error.map(Into::into);
        self
    }
    /// A red `*` after the label.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    /// Label and description on the left, the control on the right. Good for
    /// switches and checkboxes in settings pages.
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    fn label_text(&self, t: &Theme) -> egui::WidgetText {
        let color = if self.error.is_some() { t.destructive } else { t.foreground };
        let mut job = egui::text::LayoutJob::default();
        RichText::new(&self.label).size(14.0).color(color).append_to(
            &mut job,
            &egui::Style::default(),
            egui::FontSelection::Default,
            Align::Center,
        );
        if self.required {
            RichText::new(" *").size(14.0).color(t.destructive).append_to(
                &mut job,
                &egui::Style::default(),
                egui::FontSelection::Default,
                Align::Center,
            );
        }
        job.into()
    }

    fn notes(&self, ui: &mut Ui, t: &Theme) {
        if let Some(e) = &self.error {
            ui.add(egui::Label::new(RichText::new(e).size(14.0).color(t.destructive)).wrap());
        } else if let Some(d) = &self.description {
            ui.add(egui::Label::new(RichText::new(d).size(14.0).color(t.muted_foreground)).wrap());
        }
    }

    pub fn show<R>(self, ui: &mut Ui, add_control: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = Theme::get(ui.ctx());
        if self.horizontal {
            return ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let inner = add_control(ui);
                    ui.add_space(16.0);
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(self.label_text(&t));
                        self.notes(ui, &t);
                    });
                    inner
                })
                .inner
            });
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.label(self.label_text(&t));
            let inner = add_control(ui);
            self.notes(ui, &t);
            inner
        })
    }
}

// ---------------------------------------------------------------------------

/// `<FieldSet>` with `<FieldLegend>`: a titled group of fields.
///
/// ```ignore
/// FieldSet::new("Remote").description("Where to push and pull from.").show(ui, |ui| {
///     Field::new("Name").show(ui, |ui| ui.add(Input::new(&mut name)));
///     Field::new("URL").show(ui, |ui| ui.add(Input::new(&mut url)));
/// });
/// ```
pub struct FieldSet {
    legend: String,
    description: Option<String>,
}

impl FieldSet {
    pub fn new(legend: impl Into<String>) -> Self {
        Self { legend: legend.into(), description: None }
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// The fields inside are spaced 24px apart, as in shadcn's `FieldGroup`.
    pub fn show<R>(self, ui: &mut Ui, add_fields: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = Theme::get(ui.ctx());
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.label(RichText::new(&self.legend).size(16.0).color(t.foreground));
            if let Some(d) = &self.description {
                ui.add(egui::Label::new(RichText::new(d).size(14.0).color(t.muted_foreground)).wrap());
            }
            ui.add_space(16.0);
            ui.spacing_mut().item_spacing.y = 24.0;
            add_fields(ui)
        })
    }
}

/// `<FieldSeparator>`: a horizontal line between groups of fields, with
/// optional text in the middle such as `"Or continue with"`.
pub fn field_separator(ui: &mut Ui, text: Option<&str>) -> Response {
    let t = Theme::get(ui.ctx());
    let height = if text.is_some() { 20.0 } else { 1.0 };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let y = rect.center().y;
    let stroke = Stroke::new(1.0, t.border);
    match text {
        Some(text) => {
            let galley = ui.painter().layout_no_wrap(text.to_owned(), FontId::proportional(14.0), t.muted_foreground);
            let half = galley.size().x / 2.0 + 8.0;
            let c = rect.center().x;
            ui.painter().hline(rect.left()..=c - half, y, stroke);
            ui.painter().hline(c + half..=rect.right(), y, stroke);
            ui.painter().galley(rect.center() - galley.size() / 2.0, galley, t.muted_foreground);
        }
        None => {
            ui.painter().hline(rect.x_range(), y, stroke);
        }
    }
    response
}

// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemVariant {
    /// No border or background.
    #[default]
    Default,
    /// A 1px border.
    Outline,
    /// A muted background.
    Muted,
}

enum ItemMedia {
    Icon(String),
    Avatar(String),
}

/// `<Item>`: a list row with an icon or avatar, a title, a description and
/// actions on the right. Good for lists of remotes, stashes or accounts.
///
/// ```ignore
/// let item = Item::new("origin")
///     .description("git@github.com:org/repo.git")
///     .icon("☁")
///     .outline()
///     .show(ui, |ui| { ui.add(Button::outline("Fetch").size(ButtonSize::Sm)); });
/// ```
pub struct Item {
    title: String,
    description: Option<String>,
    media: Option<ItemMedia>,
    variant: ItemVariant,
    small: bool,
    clickable: bool,
}

pub struct ItemResponse<R> {
    /// The whole row. It only senses clicks with [`Item::clickable`].
    pub response: Response,
    /// What the actions closure returned.
    pub inner: R,
}

impl Item {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            media: None,
            variant: ItemVariant::Default,
            small: false,
            clickable: false,
        }
    }
    /// Muted text under the title, cut to two lines' width.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    /// A glyph in a muted, bordered square on the left.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.media = Some(ItemMedia::Icon(icon.into()));
        self
    }
    /// A round avatar with initials on the left.
    pub fn avatar(mut self, initials: impl Into<String>) -> Self {
        self.media = Some(ItemMedia::Avatar(initials.into()));
        self
    }
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn outline(self) -> Self {
        self.variant(ItemVariant::Outline)
    }
    pub fn muted(self) -> Self {
        self.variant(ItemVariant::Muted)
    }
    /// `size="sm"`: less padding.
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
    /// The row highlights on hover and senses clicks, like an `<a>` item.
    pub fn clickable(mut self) -> Self {
        self.clickable = true;
        self
    }

    /// Fills the available width; `add_actions` draws on the right, right to left.
    pub fn show<R>(self, ui: &mut Ui, add_actions: impl FnOnce(&mut Ui) -> R) -> ItemResponse<R> {
        let t = Theme::get(ui.ctx());
        let id = ui.auto_id_with("shadcn_item");

        // Sense the row before its content so buttons inside stay on top.
        let prev_rect: Option<egui::Rect> = ui.data(|d| d.get_temp(id));
        let row = prev_rect.map(|rect| {
            let r = ui.interact(rect, id, if self.clickable { Sense::click() } else { Sense::hover() });
            if self.clickable {
                r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), &self.title));
                r.on_hover_cursor(CursorIcon::PointingHand)
            } else {
                r
            }
        });
        let hovered = self.clickable && row.as_ref().is_some_and(|r| r.contains_pointer());

        let (fill, stroke) = match self.variant {
            ItemVariant::Default => (Color32::TRANSPARENT, Stroke::new(1.0, Color32::TRANSPARENT)),
            ItemVariant::Outline => (Color32::TRANSPARENT, Stroke::new(1.0, t.border)),
            ItemVariant::Muted => (t.muted.gamma_multiply(0.5), Stroke::new(1.0, Color32::TRANSPARENT)),
        };
        let fill = if hovered { t.accent } else { fill };
        let margin = if self.small { Margin::symmetric(16, 12) } else { Margin::same(16) };
        let gap = if self.small { 10.0 } else { 16.0 };

        let mut prepared = Frame::new()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(t.radius_md())
            .inner_margin(margin)
            .begin(ui);
        let inner = {
            let ui = &mut prepared.content_ui;
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                match &self.media {
                    Some(ItemMedia::Icon(icon)) => {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
                        ui.painter().rect(r, t.radius_sm(), t.muted, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
                        ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, icon, FontId::proportional(16.0), t.foreground);
                    }
                    Some(ItemMedia::Avatar(initials)) => {
                        ui.add(crate::Avatar::new(initials.as_str()).size(if self.small { 32.0 } else { 40.0 }));
                    }
                    None => {}
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let inner = add_actions(ui);
                    ui.add_space(gap - 8.0);
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.add(egui::Label::new(RichText::new(&self.title).size(14.0).color(t.foreground)).truncate());
                        if let Some(d) = &self.description {
                            ui.add(egui::Label::new(RichText::new(d).size(14.0).color(t.muted_foreground)).truncate());
                        }
                    });
                    inner
                })
                .inner
            })
            .inner
        };
        let space = prepared.allocate_space(ui);
        prepared.paint(ui);
        if prev_rect != Some(space.rect) {
            // First frame, or the row moved: sense it at the new place next frame.
            ui.data_mut(|d| d.insert_temp(id, space.rect));
            ui.ctx().request_repaint();
        }
        let response = row.unwrap_or(space);
        ItemResponse { response, inner }
    }
}
