//! Resizable panel groups.

use egui::{CursorIcon, Id, Pos2, Rect, Sense, Stroke, Ui, UiBuilder, Vec2};

use crate::theme::Theme;

/// `<ResizablePanelGroup>`: panels side by side (or stacked) with draggable
/// handles between them.
///
/// `sizes` holds one fraction per panel (they are normalized to sum to 1) and is
/// updated while dragging, so you can store it in your app state.
///
/// ```ignore
/// // sidebar | commits | diff
/// Resizable::horizontal("main", &mut self.sizes).min_size(160.0).show(ui, |ui, panel| match panel {
///     0 => sidebar(ui),
///     1 => commits(ui),
///     _ => diff(ui),
/// });
/// ```
pub struct Resizable<'a> {
    id: Id,
    sizes: &'a mut [f32],
    vertical: bool,
    min_size: f32,
    handle: bool,
    size: Option<f32>,
}

impl<'a> Resizable<'a> {
    /// Panels left to right.
    pub fn horizontal(id_salt: impl egui::AsId, sizes: &'a mut [f32]) -> Self {
        Self { id: Id::new(id_salt), sizes, vertical: false, min_size: 48.0, handle: false, size: None }
    }
    /// Panels top to bottom.
    pub fn vertical(id_salt: impl egui::AsId, sizes: &'a mut [f32]) -> Self {
        Self { vertical: true, ..Self::horizontal(id_salt, sizes) }
    }
    /// Smallest width (or height) of a panel in points (default 48).
    pub fn min_size(mut self, min: f32) -> Self {
        self.min_size = min;
        self
    }
    /// Draws a grip on each handle (`<ResizableHandle withHandle />`).
    pub fn with_handle(mut self) -> Self {
        self.handle = true;
        self
    }
    /// Size across the panels: the height of a horizontal group or the width
    /// of a vertical one. Defaults to all the available space.
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    /// `add_panel(ui, index)` is called once per panel with a `Ui` clipped to it.
    pub fn show(self, ui: &mut Ui, mut add_panel: impl FnMut(&mut Ui, usize)) -> egui::Response {
        let t = Theme::get(ui.ctx());
        let n = self.sizes.len();
        let avail = ui.available_size_before_wrap();
        let outer = if self.vertical {
            Vec2::new(self.size.unwrap_or(avail.x), avail.y)
        } else {
            Vec2::new(avail.x, self.size.unwrap_or(avail.y))
        };
        let (rect, response) = ui.allocate_exact_size(outer, Sense::hover());
        if n == 0 {
            return response;
        }
        // Main-axis helpers.
        let along = |v: Vec2| if self.vertical { v.y } else { v.x };
        let start = if self.vertical { rect.top() } else { rect.left() };
        let total = along(rect.size());
        let min = self.min_size.min(total / n as f32);

        let sum: f32 = self.sizes.iter().map(|s| s.max(0.0)).sum();
        for s in self.sizes.iter_mut() {
            *s = if sum > 0.0 { s.max(0.0) / sum } else { 1.0 / n as f32 };
        }

        let mut pos = start;
        for i in 0..n {
            let len = total * self.sizes[i];
            let panel = if self.vertical {
                Rect::from_min_size(Pos2::new(rect.left(), pos), Vec2::new(rect.width(), len))
            } else {
                Rect::from_min_size(Pos2::new(pos, rect.top()), Vec2::new(len, rect.height()))
            };
            pos += len;
            let inner = if self.vertical { panel.shrink2(Vec2::new(0.0, 0.5)) } else { panel.shrink2(Vec2::new(0.5, 0.0)) };
            let mut child = ui.new_child(UiBuilder::new().id_salt(self.id.with(("panel", i))).max_rect(inner));
            child.set_clip_rect(inner.intersect(ui.clip_rect()));
            add_panel(&mut child, i);
        }

        // Handles after the panels, so they win hit-testing at the edges.
        for i in 0..n - 1 {
            let edge = start + total * self.sizes[..=i].iter().sum::<f32>();
            let hit = self.handle_rect(rect, edge, 8.0);
            let r = ui.interact(hit, self.id.with(i), Sense::drag());
            let r = r.on_hover_and_drag_cursor(if self.vertical {
                CursorIcon::ResizeVertical
            } else {
                CursorIcon::ResizeHorizontal
            });
            if r.dragged() {
                let delta = along(r.drag_delta()) / total;
                let pair = self.sizes[i] + self.sizes[i + 1];
                let lo = (min / total).min(pair / 2.0);
                let a = (self.sizes[i] + delta).clamp(lo, pair - lo);
                self.sizes[i] = a;
                self.sizes[i + 1] = pair - a;
            }
            let active = r.hovered() || r.dragged();
            let edge = start + total * self.sizes[..=i].iter().sum::<f32>();
            let line = self.handle_rect(rect, edge, 1.0);
            let color = if active { t.ring } else { t.border };
            ui.painter().rect_filled(line, 0.0, color);
            if self.handle {
                let c = line.center();
                let grip = Rect::from_center_size(c, if self.vertical { Vec2::new(16.0, 12.0) } else { Vec2::new(12.0, 16.0) });
                ui.painter().rect(grip, t.radius_sm(), t.border, Stroke::NONE, egui::StrokeKind::Inside);
                for k in [-1.0, 0.0, 1.0] {
                    for s in [-1.5, 1.5] {
                        let off = if self.vertical { Vec2::new(k * 4.0, s) } else { Vec2::new(s, k * 4.0) };
                        ui.painter().circle_filled(c + off, 0.9, t.muted_foreground);
                    }
                }
            }
        }

        response
    }

    fn handle_rect(&self, rect: Rect, edge: f32, thickness: f32) -> Rect {
        if self.vertical {
            Rect::from_min_max(Pos2::new(rect.left(), edge - thickness / 2.0), Pos2::new(rect.right(), edge + thickness / 2.0))
        } else {
            Rect::from_min_max(Pos2::new(edge - thickness / 2.0, rect.top()), Pos2::new(edge + thickness / 2.0, rect.bottom()))
        }
    }
}
