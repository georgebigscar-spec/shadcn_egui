//! Text styles from the shadcn typography page, and a themed ScrollArea.

use egui::{FontId, Margin, Pos2, Response, RichText, Sense, Stroke, Ui, Vec2};

use crate::theme::{mix, Theme};

// ---------------------------------------------------------------------------

pub fn h4(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(16.0).strong().color(t.foreground))
}

/// A paragraph: body text that wraps at the available width.
pub fn p(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.add(egui::Label::new(RichText::new(text).size(14.0).color(t.foreground)).wrap())
}

/// Larger muted intro text under a heading (`.lead`).
pub fn lead(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.add(egui::Label::new(RichText::new(text).size(18.0).color(t.muted_foreground)).wrap())
}

/// Emphasised text a size up (`.large`).
pub fn large(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(16.0).strong().color(t.foreground))
}

/// Small text, for captions and fine print (`.small`).
pub fn small(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    ui.label(RichText::new(text).size(12.0).color(t.foreground))
}

/// A quote with a line on the left, in italics.
pub fn blockquote(ui: &mut Ui, text: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    let response = egui::Frame::new()
        .inner_margin(Margin { left: 24, ..Margin::ZERO })
        .show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(text).size(14.0).italics().color(t.foreground)).wrap());
        })
        .response;
    ui.painter().vline(response.rect.left() + 1.0, response.rect.y_range(), Stroke::new(2.0, t.border));
    response
}

/// A bulleted list; each item wraps under its own text.
pub fn list<S: AsRef<str>>(ui: &mut Ui, items: &[S]) -> Response {
    let t = Theme::get(ui.ctx());
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        for item in items {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let row_h = ui.text_style_height(&egui::TextStyle::Body);
                let (r, _) = ui.allocate_exact_size(Vec2::new(24.0, row_h), Sense::hover());
                ui.painter().circle_filled(Pos2::new(r.left() + 12.0, r.center().y), 2.0, t.foreground);
                ui.add(egui::Label::new(RichText::new(item.as_ref()).size(14.0).color(t.foreground)).wrap());
            });
        }
    })
    .response
}

/// Inline code on a muted background. Put it in `ui.horizontal_wrapped` to
/// mix it with text.
pub fn inline_code(ui: &mut Ui, code: impl Into<String>) -> Response {
    let t = Theme::get(ui.ctx());
    let galley = ui.painter().layout_no_wrap(code.into(), FontId::monospace(13.0), t.foreground);
    let size = galley.size() + Vec2::new(8.0, 4.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, egui::CornerRadius::same(4), t.muted);
        ui.painter().galley(rect.center() - galley.size() / 2.0, galley, t.foreground);
    }
    response
}

// ---------------------------------------------------------------------------

/// `<ScrollArea>`: egui's [`egui::ScrollArea`] with shadcn's scroll bars. They
/// float over the content, show up when the pointer is over the area, and are
/// a rounded thumb in the border color with no track.
///
/// ```ignore
/// ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
///     for tag in &tags { ui.label(tag); }
/// });
/// ```
#[must_use = "call `.show(ui, ..)`"]
pub struct ScrollArea {
    inner: egui::ScrollArea,
    always_visible: bool,
}

impl ScrollArea {
    fn wrap(inner: egui::ScrollArea) -> Self {
        Self { inner, always_visible: false }
    }
    pub fn vertical() -> Self {
        Self::wrap(egui::ScrollArea::vertical())
    }
    pub fn horizontal() -> Self {
        Self::wrap(egui::ScrollArea::horizontal())
    }
    pub fn both() -> Self {
        Self::wrap(egui::ScrollArea::both())
    }
    pub fn max_height(mut self, height: f32) -> Self {
        self.inner = self.inner.max_height(height);
        self
    }
    pub fn max_width(mut self, width: f32) -> Self {
        self.inner = self.inner.max_width(width);
        self
    }
    pub fn id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.inner = self.inner.id_salt(salt);
        self
    }
    /// Shrink to the content instead of filling the available space.
    pub fn auto_shrink(mut self, shrink: impl Into<egui::Vec2b>) -> Self {
        self.inner = self.inner.auto_shrink(shrink);
        self
    }
    pub fn stick_to_bottom(mut self, stick: bool) -> Self {
        self.inner = self.inner.stick_to_bottom(stick);
        self
    }
    /// Keep the thumb visible instead of only on hover (`type="always"`).
    pub fn always_visible(mut self) -> Self {
        self.always_visible = true;
        self
    }
    /// Any other `egui::ScrollArea` setting.
    pub fn with(mut self, f: impl FnOnce(egui::ScrollArea) -> egui::ScrollArea) -> Self {
        self.inner = f(self.inner);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::scroll_area::ScrollAreaOutput<R> {
        let t = Theme::get(ui.ctx());
        let content_style = ui.style().clone();
        ui.scope(|ui| {
            // The scroll bars are painted with the outer style, the content
            // gets the original one back.
            let style = ui.style_mut();
            style.spacing.scroll = egui::style::ScrollStyle {
                floating: true,
                bar_width: 10.0,
                floating_width: 6.0,
                floating_allocated_width: 0.0,
                bar_inner_margin: 2.0,
                bar_outer_margin: 0.0,
                handle_min_length: 20.0,
                foreground_color: false,
                dormant_background_opacity: 0.0,
                active_background_opacity: 0.0,
                interact_background_opacity: 0.0,
                dormant_handle_opacity: if self.always_visible { 1.0 } else { 0.0 },
                active_handle_opacity: 1.0,
                interact_handle_opacity: 1.0,
                ..egui::style::ScrollStyle::floating()
            };
            let w = &mut style.visuals.widgets;
            let thumb_hover = mix(t.border, t.muted_foreground, 0.35);
            w.inactive.bg_fill = t.border;
            w.hovered.bg_fill = thumb_hover;
            w.active.bg_fill = thumb_hover;
            let mut inner = self.inner;
            if self.always_visible {
                inner = inner.scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
            }
            inner.show(ui, |ui| {
                ui.set_style(content_style);
                add_contents(ui)
            })
        })
        .inner
    }
}
