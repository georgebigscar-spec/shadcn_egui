//! A viewer for unified diffs (`git diff` output).

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use egui::{Align2, Color32, FontId, Id, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    File,
    Hunk,
    Context,
    Add,
    Del,
}

#[derive(Clone, Debug)]
struct Line {
    kind: Kind,
    old: Option<u32>,
    new: Option<u32>,
    text: String,
}

/// One row of the side-by-side view: indices into the lines.
#[derive(Clone, Copy, Debug)]
enum SplitRow {
    Full(usize),
    Pair(Option<usize>, Option<usize>),
}

struct Parsed {
    hash: u64,
    lines: Vec<Line>,
    split: Vec<SplitRow>,
    max_chars: usize,
    max_number: u32,
}

fn parse(diff: &str, hash: u64) -> Parsed {
    let mut lines = Vec::new();
    let (mut old, mut new) = (0u32, 0u32);
    let mut in_hunk = false;
    let mut file_shown = false;
    for raw in diff.lines() {
        let raw = raw.trim_end_matches('\r');
        if let Some(rest) = raw.strip_prefix("diff --git ") {
            let path = rest.rsplit_once(" b/").map_or(rest, |(_, b)| b);
            lines.push(Line { kind: Kind::File, old: None, new: None, text: path.to_owned() });
            in_hunk = false;
            file_shown = true;
            continue;
        }
        if let Some(rest) = raw.strip_prefix("@@ ") {
            let mut nums = rest.split_whitespace();
            let start = |s: Option<&str>| {
                s.and_then(|s| s[1..].split(',').next()?.parse::<u32>().ok()).unwrap_or(1)
            };
            old = start(nums.next());
            new = start(nums.next());
            lines.push(Line { kind: Kind::Hunk, old: None, new: None, text: raw.to_owned() });
            in_hunk = true;
            continue;
        }
        if !in_hunk {
            // File metadata: only `+++` matters, when there was no `diff --git` line.
            if let Some(path) = raw.strip_prefix("+++ ") {
                if !file_shown {
                    let path = path.strip_prefix("b/").unwrap_or(path);
                    lines.push(Line { kind: Kind::File, old: None, new: None, text: path.to_owned() });
                }
                file_shown = false;
            }
            continue;
        }
        let (kind, text) = match raw.as_bytes().first() {
            Some(b'+') => (Kind::Add, &raw[1..]),
            Some(b'-') => (Kind::Del, &raw[1..]),
            Some(b' ') => (Kind::Context, &raw[1..]),
            Some(b'\\') => continue, // "\ No newline at end of file"
            None => (Kind::Context, ""),
            _ => {
                in_hunk = false;
                continue;
            }
        };
        let (o, n) = match kind {
            Kind::Add => (None, Some(new)),
            Kind::Del => (Some(old), None),
            _ => (Some(old), Some(new)),
        };
        if o.is_some() {
            old += 1;
        }
        if n.is_some() {
            new += 1;
        }
        lines.push(Line { kind, old: o, new: n, text: text.replace('\t', "    ") });
    }

    // Side by side: pair each run of deletions with the additions after it.
    let mut split = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        match lines[i].kind {
            Kind::File | Kind::Hunk => {
                split.push(SplitRow::Full(i));
                i += 1;
            }
            Kind::Context => {
                split.push(SplitRow::Pair(Some(i), Some(i)));
                i += 1;
            }
            Kind::Add | Kind::Del => {
                let dels: Vec<usize> = (i..lines.len()).take_while(|&j| lines[j].kind == Kind::Del).collect();
                let after = i + dels.len();
                let adds: Vec<usize> = (after..lines.len()).take_while(|&j| lines[j].kind == Kind::Add).collect();
                for k in 0..dels.len().max(adds.len()) {
                    split.push(SplitRow::Pair(dels.get(k).copied(), adds.get(k).copied()));
                }
                i = after + adds.len();
            }
        }
    }
    let max_chars = lines.iter().map(|l| l.text.chars().count()).max().unwrap_or(0);
    let max_number = old.max(new);
    Parsed { hash, lines, split, max_chars, max_number }
}

/// Shows the output of `git diff` (unified format): file headers, hunk headers,
/// old and new line numbers, green additions and red deletions. Only the rows
/// in view are drawn, and the parsed diff is cached until the text changes.
///
/// ```ignore
/// DiffView::new("diff", &diff_text).split(self.side_by_side).show(ui);
/// ```
pub struct DiffView<'a> {
    id: Id,
    text: &'a str,
    split: bool,
    max_height: Option<f32>,
}

impl<'a> DiffView<'a> {
    pub fn new(id_salt: impl egui::AsId, diff: &'a str) -> Self {
        Self { id: Id::new(id_salt), text: diff, split: false, max_height: None }
    }
    /// Side by side (old on the left, new on the right) instead of unified.
    pub fn split(mut self, split: bool) -> Self {
        self.split = split;
        self
    }
    /// Height before it scrolls; defaults to the available height.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let t = Theme::get(ui.ctx());
        let hash = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            self.text.hash(&mut h);
            h.finish()
        };
        let parsed = match ui.data(|d| d.get_temp::<Arc<Parsed>>(self.id)) {
            Some(p) if p.hash == hash => p,
            _ => {
                let p = Arc::new(parse(self.text, hash));
                ui.data_mut(|d| d.insert_temp(self.id, p.clone()));
                p
            }
        };

        let font = FontId::monospace(12.5);
        let char_w = ui.ctx().fonts_mut(|f| f.glyph_width(&font, '0'));
        let row_h = 20.0;
        let digits = parsed.max_number.max(1).to_string().len() as f32;
        let num_w = digits * char_w + 16.0;
        let green = if t.dark { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(22, 163, 74) };
        let red = t.destructive;
        let colors = Colors {
            add_bg: green.gamma_multiply(if t.dark { 0.15 } else { 0.12 }),
            del_bg: red.gamma_multiply(if t.dark { 0.18 } else { 0.10 }),
            green,
            red,
        };

        let height = self.max_height.unwrap_or_else(|| ui.available_height());
        let scroll = if self.split { egui::ScrollArea::vertical() } else { egui::ScrollArea::both() };
        let rows = if self.split { parsed.split.len() } else { parsed.lines.len() };
        // `show_rows` adds item spacing to the row height, so there must be none.
        let saved_spacing = ui.spacing().item_spacing.y;
        ui.spacing_mut().item_spacing.y = 0.0;
        let out = scroll
            .id_salt(self.id.with("scroll"))
            .max_height(height)
            .auto_shrink([false, true])
            .show_rows(ui, row_h, rows, |ui, range| {
                let view_w = ui.available_width();
                let content_w = if self.split {
                    view_w
                } else {
                    (2.0 * num_w + 20.0 + parsed.max_chars as f32 * char_w + 16.0).max(view_w)
                };
                for n in range {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(content_w, row_h), Sense::hover());
                    let p = ui.painter();
                    if self.split {
                        match parsed.split[n] {
                            SplitRow::Full(i) => paint_header(p, rect, &parsed.lines[i], &font, &t),
                            SplitRow::Pair(l, r) => {
                                let half = rect.width() / 2.0;
                                let left = Rect::from_min_size(rect.min, Vec2::new(half, row_h));
                                let right = Rect::from_min_size(rect.min + Vec2::new(half, 0.0), Vec2::new(half, row_h));
                                for (side, idx, is_old) in [(left, l, true), (right, r, false)] {
                                    match idx {
                                        Some(i) => {
                                            let line = &parsed.lines[i];
                                            let number = if is_old { line.old } else { line.new };
                                            paint_line(p, side, line, &[number], num_w, &font, &t, &colors);
                                        }
                                        None => {
                                            p.rect_filled(side, 0.0, t.muted.gamma_multiply(0.5));
                                        }
                                    }
                                }
                                p.vline(right.left(), rect.y_range(), Stroke::new(1.0, t.border));
                            }
                        }
                    } else {
                        let line = &parsed.lines[n];
                        match line.kind {
                            Kind::File | Kind::Hunk => paint_header(p, rect, line, &font, &t),
                            _ => paint_line(p, rect, line, &[line.old, line.new], num_w, &font, &t, &colors),
                        }
                    }
                }
            });
        ui.spacing_mut().item_spacing.y = saved_spacing;
        ui.interact(out.inner_rect, self.id.with("area"), Sense::hover())
    }
}

struct Colors {
    add_bg: Color32,
    del_bg: Color32,
    green: Color32,
    red: Color32,
}

fn paint_header(p: &egui::Painter, rect: Rect, line: &Line, font: &FontId, t: &Theme) {
    let (fill, color) = match line.kind {
        Kind::File => (t.muted, t.foreground),
        _ => (t.muted.gamma_multiply(0.5), t.muted_foreground),
    };
    p.rect_filled(rect, 0.0, fill);
    p.text(Pos2::new(rect.left() + 8.0, rect.center().y), Align2::LEFT_CENTER, &line.text, font.clone(), color);
}

#[allow(clippy::too_many_arguments)]
fn paint_line(
    p: &egui::Painter,
    rect: Rect,
    line: &Line,
    numbers: &[Option<u32>],
    num_w: f32,
    font: &FontId,
    t: &Theme,
    c: &Colors,
) {
    let (bg, sign, sign_color) = match line.kind {
        Kind::Add => (c.add_bg, "+", c.green),
        Kind::Del => (c.del_bg, "-", c.red),
        _ => (Color32::TRANSPARENT, " ", t.muted_foreground),
    };
    if bg != Color32::TRANSPARENT {
        p.rect_filled(rect, 0.0, bg);
    }
    let p = p.with_clip_rect(rect.intersect(p.clip_rect()));
    let mut x = rect.left();
    for n in numbers {
        if let Some(n) = n {
            p.text(Pos2::new(x + num_w - 8.0, rect.center().y), Align2::RIGHT_CENTER, n.to_string(), font.clone(), t.muted_foreground);
        }
        x += num_w;
    }
    p.text(Pos2::new(x + 4.0, rect.center().y), Align2::LEFT_CENTER, sign, font.clone(), sign_color);
    p.text(Pos2::new(x + 20.0, rect.center().y), Align2::LEFT_CENTER, &line.text, font.clone(), t.foreground);
}
