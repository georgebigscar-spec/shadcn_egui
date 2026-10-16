//! Commit graph lanes, drawn one row at a time (for a DataTable column).

use egui::epaint::CubicBezierShape;
use egui::{Color32, Pos2, Rect, Stroke, Ui};

use crate::theme::Theme;

/// Lane colors, cycled by lane index.
const LANES: [Color32; 6] = [
    Color32::from_rgb(59, 130, 246),
    Color32::from_rgb(168, 85, 247),
    Color32::from_rgb(34, 197, 94),
    Color32::from_rgb(249, 115, 22),
    Color32::from_rgb(236, 72, 153),
    Color32::from_rgb(20, 184, 166),
];

#[derive(Clone, Debug, Default)]
struct Row {
    lane: usize,
    /// Lines from the top edge to the middle: (lane at top, lane at middle).
    top: Vec<(usize, usize)>,
    /// Lines from the middle to the bottom edge: (lane at middle, lane at bottom).
    bottom: Vec<(usize, usize)>,
    merge: bool,
}

/// The lanes of a commit graph, laid out from commits in display order
/// (newest first, as `git log` prints them), each with its parent ids.
///
/// Build it once when the history loads and paint each row from
/// [`crate::DataTable::cell_ui`]:
///
/// ```ignore
/// let graph = CommitGraph::new(commits.iter().map(|c| (c.id.as_str(), c.parents.as_slice())));
/// // ...
/// .cell_ui(|ui, row, col| match col {
///     0 => { graph.paint(ui, row); true }
///     _ => false,
/// })
/// ```
#[derive(Clone, Debug, Default)]
pub struct CommitGraph {
    rows: Vec<Row>,
    lanes: usize,
    /// Distance between lanes in points (default 14).
    pub lane_width: f32,
}

impl CommitGraph {
    pub fn new<'a, P: AsRef<str> + 'a>(commits: impl IntoIterator<Item = (&'a str, &'a [P])>) -> Self {
        // `active[l]` is the commit that lane `l` is waiting for.
        let mut active: Vec<Option<String>> = Vec::new();
        let mut rows = Vec::new();
        let mut lanes = 0;
        for (id, parents) in commits {
            let mut row = Row::default();
            let lane = match active.iter().position(|a| a.as_deref() == Some(id)) {
                Some(l) => l,
                None => free_slot(&mut active),
            };
            row.lane = lane;
            for (l, a) in active.iter().enumerate() {
                match a.as_deref() {
                    Some(x) if x == id => row.top.push((l, lane)),
                    Some(_) => row.top.push((l, l)),
                    None => {}
                }
            }
            // Lanes that converge into this commit are free from here on.
            for a in active.iter_mut() {
                if a.as_deref() == Some(id) {
                    *a = None;
                }
            }
            let mut targets = Vec::new();
            let mut moved = Vec::new();
            for (k, parent) in parents.iter().enumerate() {
                let parent = parent.as_ref();
                let existing = active.iter().position(|a| a.as_deref() == Some(parent));
                let l = match existing {
                    // Keep the first parent in this lane; the other lane bends into it.
                    Some(l) if k == 0 && l > lane => {
                        active[l] = None;
                        moved.push(l);
                        lane
                    }
                    Some(l) => l,
                    None if k == 0 => lane,
                    None => free_slot(&mut active),
                };
                active[l] = Some(parent.to_owned());
                targets.push(l);
            }
            row.merge = parents.len() > 1;
            for (l, a) in active.iter().enumerate() {
                if a.is_none() {
                    continue;
                }
                if targets.contains(&l) {
                    row.bottom.push((lane, l));
                }
                if l != lane && row.top.contains(&(l, l)) {
                    row.bottom.push((l, l));
                }
            }
            for m in moved {
                row.bottom.push((m, lane));
            }
            while active.last().is_some_and(Option::is_none) {
                active.pop();
            }
            lanes = lanes.max(active.len()).max(lane + 1);
            rows.push(row);
        }
        Self { rows, lanes, lane_width: 14.0 }
    }

    /// Number of lanes; the column needs about `lanes() * lane_width + 12` points.
    pub fn lanes(&self) -> usize {
        self.lanes
    }

    /// Width the graph needs.
    pub fn width(&self) -> f32 {
        self.lanes as f32 * self.lane_width + 12.0
    }

    /// Paints row `row` into the whole of `ui.max_rect()` (a DataTable cell).
    pub fn paint(&self, ui: &Ui, row: usize) {
        let Some(r) = self.rows.get(row) else { return };
        let t = Theme::get(ui.ctx());
        let rect = ui.max_rect();
        self.paint_in(ui.painter(), rect, r, &t);
    }

    fn paint_in(&self, p: &egui::Painter, rect: Rect, r: &Row, t: &Theme) {
        let x = |lane: usize| rect.left() + 6.0 + lane as f32 * self.lane_width;
        let (top, mid, bottom) = (rect.top(), rect.center().y, rect.bottom());
        let color = |lane: usize| LANES[lane % LANES.len()];
        let line = |a: Pos2, b: Pos2, c: Color32| {
            let stroke = Stroke::new(2.0, c);
            if (a.x - b.x).abs() < 0.5 {
                p.line_segment([a, b], stroke);
            } else {
                let dy = (b.y - a.y) * 0.6;
                let curve = CubicBezierShape::from_points_stroke(
                    [a, Pos2::new(a.x, a.y + dy), Pos2::new(b.x, b.y - dy), b],
                    false,
                    Color32::TRANSPARENT,
                    stroke,
                );
                p.add(curve);
            }
        };
        for &(from, to) in &r.top {
            line(Pos2::new(x(from), top), Pos2::new(x(to), mid), color(from));
        }
        for &(from, to) in &r.bottom {
            line(Pos2::new(x(from), mid), Pos2::new(x(to), bottom), color(to));
        }
        let c = Pos2::new(x(r.lane), mid);
        if r.merge {
            p.circle(c, 4.0, t.background, Stroke::new(2.0, color(r.lane)));
        } else {
            p.circle_filled(c, 4.0, color(r.lane));
        }
    }
}

fn free_slot(active: &mut Vec<Option<String>>) -> usize {
    match active.iter().position(Option::is_none) {
        Some(l) => l,
        None => {
            active.push(None);
            active.len() - 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(commits: &[(&'static str, &[&'static str])]) -> CommitGraph {
        CommitGraph::new(commits.iter().map(|(id, p)| (*id, *p)))
    }

    #[test]
    fn linear_history_uses_one_lane() {
        let g = graph(&[("c", &["b"]), ("b", &["a"]), ("a", &[])]);
        assert_eq!(g.lanes(), 1);
        assert!(g.rows.iter().all(|r| r.lane == 0));
    }

    #[test]
    fn merge_opens_and_closes_a_lane() {
        // m merges feature (f) into main (b); both come from a.
        let g = graph(&[("m", &["b", "f"]), ("f", &["a"]), ("b", &["a"]), ("a", &[])]);
        assert_eq!(g.lanes(), 2);
        assert_eq!(g.rows[0].bottom, vec![(0, 0), (0, 1)]);
        assert_eq!(g.rows[1].lane, 1);
        assert_eq!(g.rows[2].lane, 0);
        // Below b, the feature lane bends back into main, so a stays in lane 0.
        assert_eq!(g.rows[2].bottom, vec![(0, 0), (1, 0)]);
        assert_eq!(g.rows[3].lane, 0);
        assert_eq!(g.rows[3].top, vec![(0, 0)]);
        assert!(g.rows[3].bottom.is_empty());
    }
}
