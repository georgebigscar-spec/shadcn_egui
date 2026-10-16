//! Small display components: Badge, Label, Separator, Kbd, Skeleton, Progress, Spinner, Empty, Avatar.

use egui::{
    Color32, CornerRadius, FontId, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
    Widget,
};

use crate::theme::Theme;

// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
}

/// `<Badge>`: a small pill with a label.
#[must_use = "add it with `ui.add(...)`"]
pub struct Badge {
    text: String,
    variant: BadgeVariant,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), variant: BadgeVariant::Default }
    }
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn secondary(self) -> Self {
        self.variant(BadgeVariant::Secondary)
    }
    pub fn destructive(self) -> Self {
        self.variant(BadgeVariant::Destructive)
    }
    pub fn outline(self) -> Self {
        self.variant(BadgeVariant::Outline)
    }
}

impl Widget for Badge {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let (fill, fg, stroke) = match self.variant {
            BadgeVariant::Default => (t.primary, t.primary_foreground, Stroke::NONE),
            BadgeVariant::Secondary => (t.secondary, t.secondary_foreground, Stroke::NONE),
            BadgeVariant::Destructive => (t.destructive, t.destructive_foreground, Stroke::NONE),
            BadgeVariant::Outline => (Color32::TRANSPARENT, t.foreground, Stroke::new(1.0, t.border)),
        };
        let galley = ui.painter().layout_no_wrap(self.text, FontId::proportional(12.0), fg);
        let size = galley.size() + Vec2::new(16.0, 6.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().rect(rect, t.radius_md(), fill, stroke, StrokeKind::Inside);
            ui.painter().galley(rect.center() - galley.size() / 2.0, galley, fg);
        }
        response
    }
}

// ---------------------------------------------------------------------------

/// `<Label>`: form label text.
pub fn label(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(14.0).color(t.foreground))
}

/// Secondary, muted text (`text-muted-foreground text-sm`).
pub fn muted(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(14.0).color(t.muted_foreground))
}

/// Typography helpers in the style of the shadcn typography page.
pub fn h1(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(32.0).strong().color(t.foreground))
}
pub fn h2(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(24.0).strong().color(t.foreground))
}
pub fn h3(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(18.0).strong().color(t.foreground))
}

// ---------------------------------------------------------------------------

/// `<Separator>`: a 1px line in the border color.
pub fn separator(ui: &mut Ui) -> Response {
    let t = Theme::get(ui.ctx());
    let vertical = ui.layout().main_dir().is_horizontal();
    let size = if vertical {
        Vec2::new(1.0, ui.available_height().min(ui.spacing().interact_size.y))
    } else {
        Vec2::new(ui.available_width(), 1.0)
    };
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, 0.0, t.border);
    response
}

// ---------------------------------------------------------------------------

/// `<Kbd>`: a keyboard key hint.
pub fn kbd(ui: &mut Ui, keys: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    let galley = ui
        .painter()
        .layout_no_wrap(keys.into(), FontId::proportional(12.0), t.muted_foreground);
    let size = Vec2::new((galley.size().x + 8.0).max(20.0), 20.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, t.radius_sm(), t.muted);
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, t.muted_foreground);
    response
}

// ---------------------------------------------------------------------------

/// `<Skeleton>`: a pulsing placeholder block.
#[must_use = "add it with `ui.add(...)`"]
pub struct Skeleton {
    size: Vec2,
    round: bool,
}

impl Skeleton {
    pub fn new(size: impl Into<Vec2>) -> Self {
        Self { size: size.into(), round: false }
    }
    /// `rounded-full`, for avatar placeholders.
    pub fn circle(diameter: f32) -> Self {
        Self { size: Vec2::splat(diameter), round: true }
    }
}

impl Widget for Skeleton {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
        let time = ui.input(|i| i.time);
        let pulse = 0.75 + 0.25 * (time * std::f64::consts::TAU / 2.0).cos() as f32;
        let radius = if self.round {
            CornerRadius::same((self.size.min_elem() / 2.0) as u8)
        } else {
            t.radius_md()
        };
        ui.painter().rect_filled(rect, radius, t.accent.gamma_multiply(pulse));
        ui.ctx().request_repaint();
        response
    }
}

// ---------------------------------------------------------------------------

/// `<Progress value={..}>`, with `fraction` in `0..=1`.
#[must_use = "add it with `ui.add(...)`"]
pub struct Progress {
    fraction: f32,
    width: Option<f32>,
}

impl Progress {
    pub fn new(fraction: f32) -> Self {
        Self { fraction: fraction.clamp(0.0, 1.0), width: None }
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for Progress {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 8.0), Sense::hover());
        let radius = CornerRadius::same(4);
        ui.painter().rect_filled(rect, radius, t.primary.gamma_multiply(0.2));
        let shown = ui
            .ctx()
            .animate_value_with_time(response.id, self.fraction, 0.3);
        if shown > 0.0 {
            let mut fill = rect;
            fill.set_width(rect.width() * shown);
            ui.painter().rect_filled(fill, radius, t.primary);
        }
        response
    }
}

// ---------------------------------------------------------------------------

/// `<Spinner>`: a rotating arc, for work in progress such as a fetch or push.
#[must_use = "add it with `ui.add(...)`"]
pub struct Spinner {
    size: f32,
    color: Option<Color32>,
}

impl Spinner {
    pub fn new() -> Self {
        Self { size: 16.0, color: None }
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ProgressIndicator, true, "Loading"));
        let color = self.color.unwrap_or(t.muted_foreground);
        let radius = self.size / 2.0 - 1.5;
        let start = ui.input(|i| i.time) as f32 * std::f32::consts::TAU * 1.2;
        let points: Vec<egui::Pos2> = (0..=24)
            .map(|k| {
                let a = start + k as f32 / 24.0 * std::f32::consts::TAU * 0.75;
                rect.center() + radius * Vec2::angled(a)
            })
            .collect();
        ui.painter().line(points, Stroke::new(1.75, color));
        ui.ctx().request_repaint();
        response
    }
}

// ---------------------------------------------------------------------------

/// `<Empty>`: a centered placeholder for an empty list or view, with an
/// optional icon, description and actions.
///
/// ```ignore
/// Empty::new("No changes")
///     .icon("✓")
///     .description("Your working tree is clean.")
///     .show(ui, |ui| { ui.add(Button::outline("Fetch")); });
/// ```
pub struct Empty {
    title: String,
    description: Option<String>,
    icon: Option<String>,
}

impl Empty {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), description: None, icon: None }
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    /// A short glyph shown in a muted square above the title.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Fills the available width; `add_actions` draws buttons below the text.
    pub fn show<R>(self, ui: &mut Ui, add_actions: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
        let t = Theme::get(ui.ctx());
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            if let Some(icon) = &self.icon {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                ui.painter().rect_filled(rect, t.radius_lg(), t.muted);
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, icon, FontId::proportional(18.0), t.foreground);
                ui.add_space(8.0);
            }
            ui.label(RichText::new(&self.title).size(16.0).color(t.foreground));
            if let Some(d) = &self.description {
                ui.add_space(-4.0);
                ui.label(RichText::new(d).size(14.0).color(t.muted_foreground));
            }
            ui.add_space(8.0);
            let r = ui.horizontal_top(|ui| {
                // Centre the actions: lay them out once to measure, then offset.
                let id = ui.id().with("empty_actions_w");
                let w: f32 = ui.data(|d| d.get_temp(id)).unwrap_or(0.0);
                ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                let start = ui.cursor().left();
                let r = add_actions(ui);
                let width = ui.min_rect().right() - start;
                if (width - w).abs() > 0.5 {
                    ui.data_mut(|d| d.insert_temp(id, width));
                    ui.ctx().request_discard("Empty: measured actions");
                }
                r
            });
            ui.add_space(24.0);
            r.inner
        })
    }
}

// ---------------------------------------------------------------------------

/// `<Avatar>` with an initials fallback, or an image when one is given.
#[must_use = "add it with `ui.add(...)`"]
pub struct Avatar<'a> {
    fallback: String,
    image: Option<egui::ImageSource<'a>>,
    size: f32,
}

impl<'a> Avatar<'a> {
    pub fn new(fallback: impl Into<String>) -> Self {
        Self { fallback: fallback.into(), image: None, size: 32.0 }
    }
    pub fn image(mut self, source: impl Into<egui::ImageSource<'a>>) -> Self {
        self.image = Some(source.into());
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Widget for Avatar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        let radius = CornerRadius::same((self.size / 2.0) as u8);
        ui.painter().rect_filled(rect, radius, t.muted);
        let mut drew_image = false;
        if let Some(source) = self.image {
            let image = egui::Image::new(source).corner_radius(radius);
            if let Ok(egui::load::TexturePoll::Ready { .. }) =
                image.load_for_size(ui.ctx(), rect.size())
            {
                image.paint_at(ui, rect);
                drew_image = true;
            }
        }
        if !drew_image {
            let font = FontId::proportional(self.size * 0.4);
            let galley = ui.painter().layout_no_wrap(self.fallback, font, t.foreground);
            ui.painter().galley(rect.center() - galley.size() / 2.0, galley, t.foreground);
        }
        response
    }
}

/// Draws a small check mark centered in `rect`.
pub(crate) fn paint_check(painter: &egui::Painter, rect: Rect, color: Color32, width: f32) {
    let c = rect.center();
    let s = rect.width() / 2.0;
    painter.line(
        vec![
            c + Vec2::new(-0.55 * s, 0.0),
            c + Vec2::new(-0.15 * s, 0.4 * s),
            c + Vec2::new(0.6 * s, -0.45 * s),
        ],
        Stroke::new(width, color),
    );
}

/// Draws a chevron pointing down (or right when `right` is true).
pub(crate) fn paint_chevron(painter: &egui::Painter, center: egui::Pos2, angle: f32, color: Color32) {
    let rot = egui::emath::Rot2::from_angle(angle);
    let pts = [Vec2::new(-4.0, -2.0), Vec2::new(0.0, 2.0), Vec2::new(4.0, -2.0)]
        .map(|p| center + rot * p);
    painter.line(pts.to_vec(), Stroke::new(1.5, color));
}
