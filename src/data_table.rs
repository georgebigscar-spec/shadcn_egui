//! `<DataTable>`: the shadcn data table recipe (sorting, filtering, row selection, pagination).

use std::cmp::Ordering;
use std::collections::HashSet;

use egui::{Align2, CursorIcon, FontId, Frame, Id, Margin, Pos2, Rect, Sense, Stroke, Ui, UiBuilder, Vec2};

use crate::button::{Button, ButtonSize};
use crate::form::{Checkbox, Input};
use crate::theme::{paint_focus_ring, Theme};

/// One column of a [`DataTable`].
pub struct Column {
    header: String,
    weight: f32,
    sortable: bool,
    right: bool,
}

impl Column {
    pub fn new(header: impl Into<String>) -> Self {
        Self { header: header.into(), weight: 1.0, sortable: false, right: false }
    }
    /// Relative width (default 1.0).
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
    /// Clicking the header sorts ascending, then descending, then unsorted.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
    /// Right-aligned text, for amounts and numbers.
    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

/// What happened in a [`DataTable`] this frame.
pub struct DataTableResponse {
    /// Index (into `rows`) of a clicked row.
    pub clicked: Option<usize>,
    /// True if the selection changed.
    pub selection_changed: bool,
}

#[derive(Clone, Default)]
struct State {
    sort: Option<(usize, bool)>, // (column, ascending)
    filter: String,
    page: usize,
}

/// A table with a filter input, sortable headers, a checkbox column and
/// Previous/Next pagination, like shadcn's data table example.
///
/// Sort, filter and page are kept in egui memory; the selection is yours
/// (indices into `rows`, so it survives sorting and filtering).
pub struct DataTable<'a> {
    id: Id,
    columns: Vec<Column>,
    selection: Option<&'a mut HashSet<usize>>,
    filter_hint: Option<String>,
    page_size: Option<usize>,
}

impl<'a> DataTable<'a> {
    pub fn new(id_salt: impl egui::AsId, columns: Vec<Column>) -> Self {
        Self { id: Id::new(id_salt), columns, selection: None, filter_hint: None, page_size: None }
    }
    /// Adds a checkbox column; checked rows are stored in `selected`.
    pub fn selection(mut self, selected: &'a mut HashSet<usize>) -> Self {
        self.selection = Some(selected);
        self
    }
    /// Adds a filter input above the table; rows match if any cell contains the text.
    pub fn filter(mut self, placeholder: impl Into<String>) -> Self {
        self.filter_hint = Some(placeholder.into());
        self
    }
    /// Rows per page; adds Previous/Next buttons below the table.
    pub fn page_size(mut self, rows: usize) -> Self {
        self.page_size = Some(rows.max(1));
        self
    }

    pub fn show<S: AsRef<str>>(mut self, ui: &mut Ui, rows: &[Vec<S>]) -> DataTableResponse {
        let t = Theme::get(ui.ctx());
        let mut state: State = ui.data(|d| d.get_temp(self.id)).unwrap_or_default();
        let mut response = DataTableResponse { clicked: None, selection_changed: false };
        let saved_spacing = ui.spacing().item_spacing.y;

        if let Some(hint) = &self.filter_hint {
            let before = state.filter.clone();
            ui.add(Input::new(&mut state.filter).placeholder(hint.as_str()).width(ui.available_width().min(280.0)));
            if state.filter != before {
                state.page = 0;
            }
            ui.add_space(4.0);
        }

        // Filter, then sort, then cut out the current page.
        let needle = state.filter.to_lowercase();
        let mut visible: Vec<usize> = (0..rows.len())
            .filter(|&i| needle.is_empty() || rows[i].iter().any(|c| c.as_ref().to_lowercase().contains(&needle)))
            .collect();
        if let Some((col, asc)) = state.sort {
            let cell = |i: usize| rows[i].get(col).map(|c| c.as_ref()).unwrap_or("");
            visible.sort_by(|&a, &b| {
                let o = compare_cells(cell(a), cell(b));
                if asc { o } else { o.reverse() }
            });
        }
        let page_count = self.page_size.map_or(1, |n| visible.len().div_ceil(n).max(1));
        state.page = state.page.min(page_count - 1);
        let page_rows: Vec<usize> = match self.page_size {
            Some(n) => visible.iter().copied().skip(state.page * n).take(n).collect(),
            None => visible.clone(),
        };

        let check_w = if self.selection.is_some() { 40.0 } else { 0.0 };
        let font = FontId::proportional(14.0);
        Frame::new()
            .stroke(Stroke::new(1.0, t.border))
            .corner_radius(t.radius_md())
            .inner_margin(Margin::ZERO)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let total = ui.available_width();
                let sum: f32 = self.columns.iter().map(|c| c.weight).sum();
                let widths: Vec<f32> = self.columns.iter().map(|c| c.weight / sum * (total - check_w)).collect();

                // Header.
                let (rect, _) = ui.allocate_exact_size(Vec2::new(total, 40.0), Sense::hover());
                if let Some(selected) = self.selection.as_deref_mut() {
                    let all = !page_rows.is_empty() && page_rows.iter().all(|i| selected.contains(i));
                    let mut checked = all;
                    let cb = Rect::from_center_size(Pos2::new(rect.left() + 20.0, rect.center().y), Vec2::splat(16.0));
                    if ui.new_child(UiBuilder::new().max_rect(cb)).add(Checkbox::new(&mut checked)).changed() {
                        for i in &page_rows {
                            if checked { selected.insert(*i) } else { selected.remove(i) };
                        }
                        response.selection_changed = true;
                    }
                }
                let mut x = rect.left() + check_w;
                for (c, (col, w)) in self.columns.iter().zip(&widths).enumerate() {
                    let cell = Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(*w, rect.height()));
                    x += w;
                    let sorted = state.sort.filter(|(sc, _)| *sc == c).map(|(_, asc)| asc);
                    if col.sortable {
                        let galley = ui.painter().layout_no_wrap(col.header.clone(), font.clone(), t.foreground);
                        let bw = galley.size().x + 16.0 + 24.0;
                        let bx = if col.right { cell.right() - bw - 8.0 } else { cell.left() + 8.0 };
                        let button = Rect::from_min_size(Pos2::new(bx, cell.center().y - 16.0), Vec2::new(bw, 32.0));
                        let r = ui
                            .interact(button, self.id.with(("sort", c)), Sense::click())
                            .on_hover_cursor(CursorIcon::PointingHand);
                        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), &col.header));
                        if r.clicked() {
                            state.sort = match sorted {
                                None => Some((c, true)),
                                Some(true) => Some((c, false)),
                                Some(false) => None,
                            };
                            state.page = 0;
                        }
                        let p = ui.painter();
                        if r.hovered() {
                            p.rect_filled(button, t.radius_md(), t.accent);
                        }
                        let fg = if r.hovered() || sorted.is_some() { t.foreground } else { t.muted_foreground };
                        let text_pos = Pos2::new(button.left() + 8.0, button.center().y - galley.size().y / 2.0);
                        let arrow_x = text_pos.x + galley.size().x + 12.0;
                        p.galley(text_pos, galley, fg);
                        paint_sort_icon(p, Pos2::new(arrow_x, button.center().y), sorted, fg);
                        if r.has_focus() {
                            paint_focus_ring(p, button, t.radius_md(), &t);
                        }
                    } else {
                        let (pos, align) = text_anchor(cell, col.right);
                        ui.painter().text(pos, align, &col.header, font.clone(), t.muted_foreground);
                    }
                }
                ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, t.border));

                // Body.
                if page_rows.is_empty() {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(total, 96.0), Sense::hover());
                    ui.painter().text(rect.center(), Align2::CENTER_CENTER, "No results.", font.clone(), t.muted_foreground);
                }
                for (n, &i) in page_rows.iter().enumerate() {
                    let (rect, r) = ui.allocate_exact_size(Vec2::new(total, 44.0), Sense::click());
                    if r.clicked() {
                        response.clicked = Some(i);
                    }
                    let is_selected = self.selection.as_deref().is_some_and(|s| s.contains(&i));
                    if is_selected {
                        ui.painter().rect_filled(rect, 0.0, t.muted);
                    } else if r.hovered() {
                        ui.painter().rect_filled(rect, 0.0, t.muted.gamma_multiply(0.5));
                    }
                    if let Some(selected) = self.selection.as_deref_mut() {
                        let mut checked = is_selected;
                        let cb = Rect::from_center_size(Pos2::new(rect.left() + 20.0, rect.center().y), Vec2::splat(16.0));
                        if ui.new_child(UiBuilder::new().max_rect(cb)).add(Checkbox::new(&mut checked)).changed() {
                            if checked { selected.insert(i) } else { selected.remove(&i) };
                            response.selection_changed = true;
                        }
                    }
                    let p = ui.painter();
                    let mut x = rect.left() + check_w;
                    for (col, (text, w)) in self.columns.iter().zip(rows[i].iter().zip(&widths)) {
                        let cell = Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(*w, rect.height()));
                        x += w;
                        let (pos, align) = text_anchor(cell, col.right);
                        p.with_clip_rect(cell.shrink2(Vec2::new(4.0, 0.0)))
                            .text(pos, align, text.as_ref(), font.clone(), t.foreground);
                    }
                    if n + 1 < page_rows.len() {
                        p.hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, t.border));
                    }
                }
            });

        // Footer: selection count and pagination.
        if self.selection.is_some() || self.page_size.is_some() {
            ui.spacing_mut().item_spacing.y = saved_spacing;
            ui.horizontal(|ui| {
                ui.set_min_height(36.0);
                let p = ui.painter();
                let font = FontId::proportional(14.0);
                let text = match self.selection.as_deref() {
                    Some(s) => format!("{} of {} row(s) selected.", s.iter().filter(|i| visible.contains(i)).count(), visible.len()),
                    None => format!("Page {} of {}", state.page + 1, page_count),
                };
                let galley = p.layout_no_wrap(text, font, t.muted_foreground);
                let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
                ui.painter().galley(rect.min, galley, t.muted_foreground);
                if self.page_size.is_some() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(state.page + 1 < page_count, Button::outline("Next").size(ButtonSize::Sm)).clicked() {
                            state.page += 1;
                        }
                        if ui.add_enabled(state.page > 0, Button::outline("Previous").size(ButtonSize::Sm)).clicked() {
                            state.page -= 1;
                        }
                    });
                }
            });
        }

        ui.spacing_mut().item_spacing.y = saved_spacing;
        ui.data_mut(|d| d.insert_temp(self.id, state));
        response
    }
}

fn text_anchor(cell: Rect, right: bool) -> (Pos2, Align2) {
    if right {
        (Pos2::new(cell.right() - 16.0, cell.center().y), Align2::RIGHT_CENTER)
    } else {
        (Pos2::new(cell.left() + 16.0, cell.center().y), Align2::LEFT_CENTER)
    }
}

/// Numbers compare by value (ignoring `$`, `,`, `%` and spaces), everything else
/// case-insensitively.
fn compare_cells(a: &str, b: &str) -> Ordering {
    let num = |s: &str| s.chars().filter(|c| !matches!(c, '$' | '€' | '£' | ',' | '%' | ' ')).collect::<String>().parse::<f64>().ok();
    match (num(a), num(b)) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// Lucide `arrow-up-down` when unsorted, a single arrow when sorted.
fn paint_sort_icon(p: &egui::Painter, center: Pos2, sorted: Option<bool>, color: egui::Color32) {
    let stroke = Stroke::new(1.3, color);
    let arrow = |x: f32, up: bool| {
        let (tip, tail) = if up { (-5.0, 5.0) } else { (5.0, -5.0) };
        let d = if up { 3.0 } else { -3.0 };
        p.line_segment([center + Vec2::new(x, tail), center + Vec2::new(x, tip)], stroke);
        p.line(
            vec![center + Vec2::new(x - 3.0, tip + d), center + Vec2::new(x, tip), center + Vec2::new(x + 3.0, tip + d)],
            stroke,
        );
    };
    match sorted {
        None => {
            arrow(-3.0, false);
            arrow(3.0, true);
        }
        Some(asc) => arrow(0.0, asc),
    }
}
