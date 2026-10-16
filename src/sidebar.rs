//! Sidebar navigation.

use egui::{Align2, Color32, CursorIcon, FontId, Id, Pos2, Rect, Response, Sense, Ui, Vec2};

use crate::basic::paint_chevron;
use crate::theme::{mix, paint_focus_ring, Theme};

/// `<Sidebar>` menu: groups with a label and items, one of which is selected.
/// Groups can be collapsed by clicking their label. Draw it into a panel, for
/// example the first panel of a [`crate::Resizable`]; [`Sidebar::fill`] gives
/// shadcn's sidebar background color.
///
/// ```ignore
/// Sidebar::new("nav").show(ui, &mut self.view, |sb| {
///     sb.item(View::Changes, "Changes", Some("4"));
///     sb.item(View::History, "History", None);
///     sb.group("Branches", |sb| {
///         for b in &branches { sb.item(View::Branch(b.clone()), b, None); }
///     });
/// });
/// ```
pub struct Sidebar {
    id: Id,
}

impl Sidebar {
    pub fn new(id_salt: impl egui::AsId) -> Self {
        Self { id: Id::new(id_salt) }
    }

    /// The sidebar background (`--sidebar`), a shade between background and muted.
    pub fn fill(theme: &Theme) -> Color32 {
        mix(theme.background, theme.muted, if theme.dark { 0.6 } else { 0.5 })
    }

    pub fn show<T: PartialEq, R>(
        self,
        ui: &mut Ui,
        selected: &mut T,
        add_items: impl FnOnce(&mut SidebarUi<'_, T>) -> R,
    ) -> egui::InnerResponse<R> {
        let mut changed = false;
        let mut inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            let mut sb = SidebarUi { ui, selected, id: self.id, changed: false };
            let r = add_items(&mut sb);
            changed = sb.changed;
            r
        });
        if changed {
            inner.response.mark_changed();
        }
        inner
    }
}

/// Adds groups and items inside [`Sidebar::show`].
pub struct SidebarUi<'a, T> {
    ui: &'a mut Ui,
    selected: &'a mut T,
    id: Id,
    changed: bool,
}

impl<T: PartialEq> SidebarUi<'_, T> {
    /// The `Ui` the sidebar is drawn into, for anything custom.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    /// `<SidebarGroup>` with a collapsible `<SidebarGroupLabel>`; open by default.
    pub fn group(&mut self, label: &str, add_items: impl FnOnce(&mut Self)) -> Response {
        let t = Theme::get(self.ui.ctx());
        let id = self.id.with(("group", label));
        let open_id = id.with("open");
        let mut open: bool = self.ui.data(|d| d.get_temp(open_id)).unwrap_or(true);
        self.ui.add_space(8.0);
        let width = self.ui.available_width();
        let (rect, _) = self.ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::hover());
        let r = self.ui.interact(rect, id, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, true, label));
        if r.clicked() {
            open = !open;
            self.ui.data_mut(|d| d.insert_temp(open_id, open));
        }
        let openness = self.ui.ctx().animate_bool_with_time(open_id, open, 0.15);
        let p = self.ui.painter();
        if r.hovered() {
            p.rect_filled(rect, t.radius_md(), t.accent);
        }
        p.text(Pos2::new(rect.left() + 8.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(12.0), t.muted_foreground);
        let angle = -std::f32::consts::FRAC_PI_2 * (1.0 - openness);
        paint_chevron(p, Pos2::new(rect.right() - 14.0, rect.center().y), angle, t.muted_foreground);
        if r.has_focus() {
            paint_focus_ring(p, rect, t.radius_md(), &t);
        }
        if open {
            add_items(self);
        }
        r
    }

    /// `<SidebarMenuButton>` with an optional `<SidebarMenuBadge>` (a count) on
    /// the right. Clicking selects `value`.
    pub fn item(&mut self, value: T, label: &str, badge: Option<&str>) -> Response {
        let t = Theme::get(self.ui.ctx());
        let width = self.ui.available_width();
        let (rect, _) = self.ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::hover());
        let r = self.ui.interact(rect, self.id.with(("item", label)), Sense::click());
        let is_selected = *self.selected == value;
        r.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, is_selected, label));
        if r.clicked() && !is_selected {
            *self.selected = value;
            self.changed = true;
        }
        let p = self.ui.painter();
        if is_selected || r.hovered() {
            p.rect_filled(rect, t.radius_md(), t.accent);
        }
        let fg = if is_selected || r.hovered() { t.accent_foreground } else { t.foreground };
        let right = match badge {
            Some(b) => {
                let g = p.layout_no_wrap(b.to_owned(), FontId::proportional(12.0), t.muted_foreground);
                let pos = Pos2::new(rect.right() - 8.0 - g.size().x, rect.center().y - g.size().y / 2.0);
                let x = pos.x;
                p.galley(pos, g, t.muted_foreground);
                x - 8.0
            }
            None => rect.right() - 8.0,
        };
        let g = p.layout_no_wrap(label.to_owned(), FontId::proportional(14.0), fg);
        let clip = Rect::from_min_max(rect.min, Pos2::new(right, rect.bottom()));
        p.with_clip_rect(clip.intersect(p.clip_rect()))
            .galley(Pos2::new(rect.left() + 8.0, rect.center().y - g.size().y / 2.0), g, fg);
        if r.has_focus() {
            paint_focus_ring(p, rect, t.radius_md(), &t);
        }
        r
    }
}
