//! A resolver for merge conflicts: the `<<<<<<<` / `=======` / `>>>>>>>` blocks
//! git leaves in a file.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use egui::{Align2, Color32, CursorIcon, FontId, Id, Pos2, Rect, Response, RichText, Sense, Ui, Vec2};

use crate::button::{Button, ButtonSize};
use crate::theme::Theme;

/// How one conflict is resolved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Resolution {
    #[default]
    Unresolved,
    /// Keep the current side (ours, `HEAD`).
    Current,
    /// Keep the incoming side (theirs, the branch being merged).
    Incoming,
    /// Keep both: current first, then incoming.
    Both,
    /// Keep the common ancestor (only offered when the file has `|||||||` base sections).
    Base,
}

#[derive(Clone, Debug)]
enum Segment {
    Common(Vec<String>),
    Conflict(Conflict),
}

#[derive(Clone, Debug)]
struct Conflict {
    current: Vec<String>,
    base: Option<Vec<String>>,
    incoming: Vec<String>,
    current_label: String,
    incoming_label: String,
}

impl Conflict {
    fn lines(&self, r: Resolution) -> Vec<&str> {
        fn v(l: &[String]) -> Vec<&str> {
            l.iter().map(String::as_str).collect()
        }
        match r {
            Resolution::Unresolved => Vec::new(),
            Resolution::Current => v(&self.current),
            Resolution::Incoming => v(&self.incoming),
            Resolution::Both => self.current.iter().chain(&self.incoming).map(String::as_str).collect(),
            Resolution::Base => self.base.as_deref().map(v).unwrap_or_default(),
        }
    }
}

struct Parsed {
    hash: u64,
    segments: Vec<Segment>,
    conflicts: usize,
    trailing_newline: bool,
}

fn parse(text: &str, hash: u64) -> Parsed {
    #[derive(PartialEq)]
    enum In {
        Common,
        Current,
        Base,
        Incoming,
    }
    let mut segments = Vec::new();
    let mut common = Vec::new();
    let mut c = Conflict { current: vec![], base: None, incoming: vec![], current_label: String::new(), incoming_label: String::new() };
    let mut state = In::Common;
    let label = |l: &str| l[7..].trim().to_owned();
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        match state {
            In::Common if line.starts_with("<<<<<<<") => {
                if !common.is_empty() {
                    segments.push(Segment::Common(std::mem::take(&mut common)));
                }
                c.current_label = label(line);
                state = In::Current;
            }
            In::Current if line.starts_with("|||||||") => {
                c.base = Some(Vec::new());
                state = In::Base;
            }
            In::Current | In::Base if line == "=======" => state = In::Incoming,
            In::Incoming if line.starts_with(">>>>>>>") => {
                c.incoming_label = label(line);
                let done = std::mem::replace(
                    &mut c,
                    Conflict { current: vec![], base: None, incoming: vec![], current_label: String::new(), incoming_label: String::new() },
                );
                segments.push(Segment::Conflict(done));
                state = In::Common;
            }
            In::Common => common.push(line.to_owned()),
            In::Current => c.current.push(line.to_owned()),
            In::Base => c.base.get_or_insert_with(Vec::new).push(line.to_owned()),
            In::Incoming => c.incoming.push(line.to_owned()),
        }
    }
    if state != In::Common {
        // An unterminated block is not a conflict: keep its text as it was.
        common.push(format!("<<<<<<< {}", c.current_label));
        common.extend(c.current);
        if let Some(base) = c.base {
            common.push("|||||||".to_owned());
            common.extend(base);
        }
        if state == In::Incoming {
            common.push("=======".to_owned());
            common.extend(c.incoming);
        }
    }
    if !common.is_empty() {
        segments.push(Segment::Common(common));
    }
    let conflicts = segments.iter().filter(|s| matches!(s, Segment::Conflict(_))).count();
    Parsed { hash, segments, conflicts, trailing_newline: text.ends_with('\n') }
}

fn parsed(ctx: &egui::Context, id: Id, text: &str) -> Arc<Parsed> {
    let hash = {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut h);
        h.finish()
    };
    match ctx.data(|d| d.get_temp::<Arc<Parsed>>(id)) {
        Some(p) if p.hash == hash => p,
        _ => {
            let p = Arc::new(parse(text, hash));
            ctx.data_mut(|d| d.insert_temp(id, p.clone()));
            p
        }
    }
}

/// Number of conflict blocks in `text`.
pub fn count_conflicts(text: &str) -> usize {
    parse(text, 0).conflicts
}

/// The file with every conflict replaced by its resolution, or `None` while
/// any is [`Resolution::Unresolved`] (missing entries count as unresolved).
pub fn resolve_conflicts(text: &str, resolutions: &[Resolution]) -> Option<String> {
    let p = parse(text, 0);
    let mut out: Vec<&str> = Vec::new();
    let mut k = 0;
    for s in &p.segments {
        match s {
            Segment::Common(lines) => out.extend(lines.iter().map(String::as_str)),
            Segment::Conflict(c) => {
                let r = resolutions.get(k).copied().unwrap_or_default();
                if r == Resolution::Unresolved {
                    return None;
                }
                out.extend(c.lines(r));
                k += 1;
            }
        }
    }
    let mut text = out.join("\n");
    if p.trailing_newline && !text.is_empty() {
        text.push('\n');
    }
    Some(text)
}

/// What [`MergeView::show`] returns.
pub struct MergeViewResponse {
    pub response: Response,
    /// Conflict blocks in the file.
    pub conflicts: usize,
    /// How many of them have a resolution.
    pub resolved: usize,
    /// True if a button changed a resolution this frame.
    pub changed: bool,
}

/// Shows a file with merge conflicts: unchanged lines around each block
/// (longer runs fold into a clickable "N unchanged lines" row), and for each
/// conflict the current and incoming sides (and the base, for `diff3`-style
/// markers) with Accept current / Accept incoming / Accept both buttons.
/// A resolved block shows the lines it keeps and an Undo button.
///
/// ```ignore
/// let r = MergeView::new("merge", &file_text).show(ui, &mut self.resolutions);
/// if r.resolved == r.conflicts {
///     let merged = resolve_conflicts(&file_text, &self.resolutions).unwrap();
///     std::fs::write(path, merged)?; // then `git add path`
/// }
/// ```
///
/// `resolutions` holds one entry per conflict, in file order; it is grown to
/// the number of conflicts as needed.
pub struct MergeView<'a> {
    id: Id,
    text: &'a str,
    max_height: Option<f32>,
    scroll_to: Option<usize>,
}

impl<'a> MergeView<'a> {
    pub fn new(id_salt: impl egui::AsId, text: &'a str) -> Self {
        Self { id: Id::new(id_salt), text, max_height: None, scroll_to: None }
    }
    /// Height before it scrolls; defaults to the available height.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }
    /// Scrolls conflict `index` (from 0) into view whenever the index changes.
    pub fn scroll_to_conflict(mut self, index: usize) -> Self {
        self.scroll_to = Some(index);
        self
    }

    pub fn show(self, ui: &mut Ui, resolutions: &mut Vec<Resolution>) -> MergeViewResponse {
        let t = Theme::get(ui.ctx());
        let parsed = parsed(ui.ctx(), self.id, self.text);
        if resolutions.len() < parsed.conflicts {
            resolutions.resize(parsed.conflicts, Resolution::Unresolved);
        }
        let expanded_id = self.id.with("expanded");
        let mut expanded: HashSet<usize> = ui.data(|d| d.get_temp(expanded_id)).unwrap_or_default();
        let scrolled_id = self.id.with("scrolled_to");
        let scroll_now = self.scroll_to.filter(|&i| ui.data(|d| d.get_temp::<usize>(scrolled_id)) != Some(i));
        if let Some(i) = scroll_now {
            ui.data_mut(|d| d.insert_temp(scrolled_id, i));
        }

        let font = FontId::monospace(12.5);
        let blue = if t.dark { Color32::from_rgb(96, 165, 250) } else { Color32::from_rgb(37, 99, 235) };
        let purple = if t.dark { Color32::from_rgb(192, 132, 252) } else { Color32::from_rgb(147, 51, 234) };
        let green = if t.dark { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(22, 163, 74) };
        let tint = |c: Color32| c.gamma_multiply(if t.dark { 0.14 } else { 0.08 });
        let mut changed = false;
        let height = self.max_height.unwrap_or_else(|| ui.available_height());

        let out = egui::ScrollArea::both().id_salt(self.id.with("scroll")).max_height(height).auto_shrink([false, true]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let width = ui.available_width();
            let code = |ui: &mut Ui, text: &str, bar: Option<Color32>, fill: Color32| {
                let galley = ui.painter().layout_no_wrap(text.to_owned(), font.clone(), t.foreground);
                let (rect, _) = ui.allocate_exact_size(Vec2::new((galley.size().x + 32.0).max(width), 20.0), Sense::hover());
                if ui.is_rect_visible(rect) {
                    let p = ui.painter();
                    if fill != Color32::TRANSPARENT {
                        p.rect_filled(rect, 0.0, fill);
                    }
                    if let Some(bar) = bar {
                        p.rect_filled(Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())), 0.0, bar);
                    }
                    p.galley(Pos2::new(rect.left() + 16.0, rect.center().y - galley.size().y / 2.0), galley, t.foreground);
                }
            };
            let section = |ui: &mut Ui, title: &str, color: Color32| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::hover());
                if ui.is_rect_visible(rect) {
                    let p = ui.painter();
                    p.rect_filled(rect, 0.0, color.gamma_multiply(if t.dark { 0.22 } else { 0.14 }));
                    p.rect_filled(Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())), 0.0, color);
                    p.text(Pos2::new(rect.left() + 16.0, rect.center().y), Align2::LEFT_CENTER, title, FontId::proportional(12.0), color);
                }
            };

            let last = parsed.segments.len().saturating_sub(1);
            let mut k = 0;
            for (n, segment) in parsed.segments.iter().enumerate() {
                match segment {
                    Segment::Common(lines) => {
                        // Fold long runs, keeping a few lines next to each conflict.
                        const KEEP: usize = 3;
                        let head = if n == 0 { 0 } else { KEEP };
                        let tail = if n == last { 0 } else { KEEP };
                        let fold = !expanded.contains(&n) && lines.len() > head + tail + 2;
                        for (i, line) in lines.iter().enumerate() {
                            if fold && i == head {
                                let hidden = lines.len() - head - tail;
                                let (rect, r) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::click());
                                let r = r.on_hover_cursor(CursorIcon::PointingHand);
                                r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, format!("Show {hidden} unchanged lines")));
                                let fill = if r.hovered() { t.accent } else { t.muted.gamma_multiply(0.6) };
                                ui.painter().rect_filled(rect, 0.0, fill);
                                ui.painter().text(rect.left_center() + Vec2::new(16.0, 0.0), Align2::LEFT_CENTER, format!("…  {hidden} unchanged lines"), FontId::proportional(12.0), t.muted_foreground);
                                if r.clicked() {
                                    expanded.insert(n);
                                }
                            }
                            if fold && i >= head && i < lines.len() - tail {
                                continue;
                            }
                            code(ui, line, None, Color32::TRANSPARENT);
                        }
                    }
                    Segment::Conflict(c) => {
                        let r = resolutions[k];
                        let header = egui::Frame::new()
                            .fill(t.muted)
                            .inner_margin(egui::Margin::symmetric(12, 6))
                            .show(ui, |ui| {
                                ui.set_width(width - 24.0);
                                ui.spacing_mut().item_spacing.x = 6.0;
                                ui.horizontal(|ui| {
                                    let status = match r {
                                        Resolution::Unresolved => format!("Conflict {} of {}", k + 1, parsed.conflicts),
                                        Resolution::Current => format!("Conflict {}: kept current", k + 1),
                                        Resolution::Incoming => format!("Conflict {}: kept incoming", k + 1),
                                        Resolution::Both => format!("Conflict {}: kept both", k + 1),
                                        Resolution::Base => format!("Conflict {}: kept base", k + 1),
                                    };
                                    let color = if r == Resolution::Unresolved { t.foreground } else { green };
                                    ui.label(RichText::new(status).size(13.0).strong().color(color));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let mut pick = |ui: &mut Ui, label: &str, value: Resolution, primary: bool| {
                                            let b = if primary { Button::outline(label) } else { Button::ghost(label) };
                                            if ui.add(b.size(ButtonSize::Sm)).clicked() {
                                                resolutions[k] = value;
                                                changed = true;
                                            }
                                        };
                                        if r == Resolution::Unresolved {
                                            pick(ui, "Accept both", Resolution::Both, false);
                                            if c.base.is_some() {
                                                pick(ui, "Accept base", Resolution::Base, false);
                                            }
                                            pick(ui, "Accept incoming", Resolution::Incoming, true);
                                            pick(ui, "Accept current", Resolution::Current, true);
                                        } else {
                                            pick(ui, "Undo", Resolution::Unresolved, false);
                                        }
                                    });
                                });
                            });
                        if scroll_now == Some(k) {
                            header.response.scroll_to_me(Some(egui::Align::TOP));
                        }
                        if r == Resolution::Unresolved {
                            let current = if c.current_label.is_empty() { "Current".to_owned() } else { format!("Current · {}", c.current_label) };
                            section(ui, &current, blue);
                            for line in &c.current {
                                code(ui, line, Some(blue), tint(blue));
                            }
                            if let Some(base) = &c.base {
                                section(ui, "Base", t.muted_foreground);
                                for line in base {
                                    code(ui, line, Some(t.muted_foreground), tint(t.muted_foreground));
                                }
                            }
                            let incoming = if c.incoming_label.is_empty() { "Incoming".to_owned() } else { format!("Incoming · {}", c.incoming_label) };
                            section(ui, &incoming, purple);
                            for line in &c.incoming {
                                code(ui, line, Some(purple), tint(purple));
                            }
                        } else {
                            let kept = c.lines(r);
                            if kept.is_empty() {
                                code(ui, "(no lines)", Some(green), tint(green));
                            }
                            for line in kept {
                                code(ui, line, Some(green), tint(green));
                            }
                        }
                        ui.add_space(1.0);
                        k += 1;
                    }
                }
            }
        });
        ui.data_mut(|d| d.insert_temp(expanded_id, expanded));
        if changed {
            ui.ctx().request_repaint();
        }
        let resolved = resolutions.iter().take(parsed.conflicts).filter(|r| **r != Resolution::Unresolved).count();
        let response = ui.interact(out.inner_rect, self.id.with("area"), Sense::hover());
        MergeViewResponse { response, conflicts: parsed.conflicts, resolved, changed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = concat!(
        "fn main() {\n",
        "<<<<<<< HEAD\n",
        "    let x = 1;\n",
        "||||||| base\n",
        "    let x = 0;\n",
        "=======\n",
        "    let x = 2;\n",
        "    let y = 3;\n",
        ">>>>>>> feature\n",
        "}\n",
    );

    #[test]
    fn parses_diff3_markers() {
        let p = parse(FILE, 0);
        assert_eq!(p.conflicts, 1);
        let Segment::Conflict(c) = &p.segments[1] else { panic!() };
        assert_eq!(c.current, ["    let x = 1;"]);
        assert_eq!(c.base.as_deref(), Some(&["    let x = 0;".to_owned()][..]));
        assert_eq!(c.incoming.len(), 2);
        assert_eq!((c.current_label.as_str(), c.incoming_label.as_str()), ("HEAD", "feature"));
    }

    #[test]
    fn resolves() {
        assert_eq!(resolve_conflicts(FILE, &[]), None);
        assert_eq!(resolve_conflicts(FILE, &[Resolution::Current]).unwrap(), "fn main() {\n    let x = 1;\n}\n");
        assert_eq!(
            resolve_conflicts(FILE, &[Resolution::Both]).unwrap(),
            "fn main() {\n    let x = 1;\n    let x = 2;\n    let y = 3;\n}\n"
        );
        assert_eq!(resolve_conflicts(FILE, &[Resolution::Base]).unwrap(), "fn main() {\n    let x = 0;\n}\n");
    }

    #[test]
    fn unterminated_block_is_text() {
        let text = "a\n<<<<<<< HEAD\nb\n";
        assert_eq!(count_conflicts(text), 0);
        assert_eq!(resolve_conflicts(text, &[]).unwrap(), text);
    }
}
