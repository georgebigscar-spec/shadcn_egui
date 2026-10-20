//! A viewer for unified diffs (`git diff` output).

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use egui::{Align2, Color32, CursorIcon, FontId, Id, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

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
    /// Index of this line in the diff text (`diff.lines()`), the key of a selection.
    raw: usize,
    /// Changed words, as char ranges of `text`, when the line is paired with a
    /// similar line on the other side.
    emph: Vec<(usize, usize)>,
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
    /// Row of the side-by-side view that shows each line.
    split_row: Vec<usize>,
    max_chars: usize,
    max_number: u32,
    additions: usize,
    deletions: usize,
}

fn parse(diff: &str, hash: u64) -> Parsed {
    let mut lines = Vec::new();
    let (mut old, mut new) = (0u32, 0u32);
    let mut in_hunk = false;
    let mut file_shown = false;
    for (raw_index, raw) in diff.lines().enumerate() {
        let raw = raw.trim_end_matches('\r');
        let line = |kind, old, new, text: String| Line { kind, old, new, text, raw: raw_index, emph: Vec::new() };
        if let Some(rest) = raw.strip_prefix("diff --git ") {
            let path = rest.rsplit_once(" b/").map_or(rest, |(_, b)| b);
            lines.push(line(Kind::File, None, None, path.to_owned()));
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
            lines.push(line(Kind::Hunk, None, None, raw.to_owned()));
            in_hunk = true;
            continue;
        }
        if !in_hunk {
            // File metadata: only `+++` matters, when there was no `diff --git` line.
            if let Some(path) = raw.strip_prefix("+++ ") {
                if !file_shown {
                    let path = path.strip_prefix("b/").unwrap_or(path);
                    lines.push(line(Kind::File, None, None, path.to_owned()));
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
        lines.push(line(kind, o, n, text.replace('\t', "    ")));
    }

    // Side by side: pair each run of deletions with the additions after it.
    // The same pairs get word-level highlights.
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
                    let (d, a) = (dels.get(k).copied(), adds.get(k).copied());
                    if let (Some(d), Some(a)) = (d, a) {
                        if let Some((ed, ea)) = word_diff(&lines[d].text, &lines[a].text) {
                            lines[d].emph = ed;
                            lines[a].emph = ea;
                        }
                    }
                    split.push(SplitRow::Pair(d, a));
                }
                i = after + adds.len();
            }
        }
    }
    let mut split_row = vec![0; lines.len()];
    for (row, s) in split.iter().enumerate() {
        match *s {
            SplitRow::Full(i) => split_row[i] = row,
            SplitRow::Pair(l, r) => {
                for i in l.into_iter().chain(r) {
                    split_row[i] = row;
                }
            }
        }
    }
    let max_chars = lines.iter().map(|l| l.text.chars().count()).max().unwrap_or(0);
    let max_number = old.max(new);
    let additions = lines.iter().filter(|l| l.kind == Kind::Add).count();
    let deletions = lines.iter().filter(|l| l.kind == Kind::Del).count();
    Parsed { hash, lines, split, split_row, max_chars, max_number, additions, deletions }
}

/// Splits a line into words, runs of spaces and single punctuation characters,
/// as char ranges.
fn tokens(chars: &[char]) -> Vec<(usize, usize)> {
    let class = |c: char| {
        if c.is_alphanumeric() || c == '_' {
            0
        } else if c.is_whitespace() {
            1
        } else {
            2
        }
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let k = class(chars[i]);
        let mut j = i + 1;
        if k != 2 {
            while j < chars.len() && class(chars[j]) == k {
                j += 1;
            }
        }
        out.push((i, j));
        i = j;
    }
    out
}

type Ranges = Vec<(usize, usize)>;

/// The changed parts of two similar lines, as char ranges of each, or `None`
/// when the lines have too little in common for word highlights to help.
fn word_diff(a: &str, b: &str) -> Option<(Ranges, Ranges)> {
    let (ac, bc): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (ta, tb) = (tokens(&ac), tokens(&bc));
    if ta.is_empty() || tb.is_empty() || ta.len() * tb.len() > 40_000 {
        return None;
    }
    let same = |i: usize, j: usize| ac[ta[i].0..ta[i].1] == bc[tb[j].0..tb[j].1];
    // Longest common subsequence of tokens, filled from the end.
    let w = tb.len() + 1;
    let mut lcs = vec![0u32; (ta.len() + 1) * w];
    for i in (0..ta.len()).rev() {
        for j in (0..tb.len()).rev() {
            lcs[i * w + j] =
                if same(i, j) { lcs[(i + 1) * w + j + 1] + 1 } else { lcs[(i + 1) * w + j].max(lcs[i * w + j + 1]) };
        }
    }
    let (mut keep_a, mut keep_b) = (vec![false; ta.len()], vec![false; tb.len()]);
    let (mut i, mut j) = (0, 0);
    while i < ta.len() && j < tb.len() {
        if same(i, j) {
            keep_a[i] = true;
            keep_b[j] = true;
            i += 1;
            j += 1;
        } else if lcs[(i + 1) * w + j] >= lcs[i * w + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    let common: usize = ta.iter().zip(&keep_a).filter(|(_, k)| **k).map(|(t, _)| t.1 - t.0).sum();
    let longest = ac.len().max(bc.len());
    if common * 2 < longest {
        return None;
    }
    let ranges = |toks: &[(usize, usize)], keep: &[bool]| {
        let mut out: Vec<(usize, usize)> = Vec::new();
        for (t, k) in toks.iter().zip(keep) {
            if *k {
                continue;
            }
            match out.last_mut() {
                Some(last) if last.1 == t.0 => last.1 = t.1,
                _ => out.push(*t),
            }
        }
        out
    };
    Some((ranges(&ta, &keep_a), ranges(&tb, &keep_b)))
}

/// Search hits: (line, first char, end char), in line order.
struct Hits {
    hash: u64,
    query: String,
    hits: Vec<(usize, usize, usize)>,
}

fn find_hits(parsed: &Parsed, query: &str) -> Vec<(usize, usize, usize)> {
    let lower = |s: &str| s.chars().map(|c| c.to_lowercase().next().unwrap_or(c)).collect::<Vec<char>>();
    let q = lower(query);
    let mut hits = Vec::new();
    if q.is_empty() {
        return hits;
    }
    for (n, line) in parsed.lines.iter().enumerate() {
        if matches!(line.kind, Kind::File | Kind::Hunk) {
            continue;
        }
        let text = lower(&line.text);
        let mut i = 0;
        while i + q.len() <= text.len() {
            if text[i..i + q.len()] == q[..] {
                hits.push((n, i, i + q.len()));
                i += q.len();
            } else {
                i += 1;
            }
        }
    }
    hits
}

/// What [`DiffView::show`] returns.
pub struct DiffViewResponse {
    pub response: Response,
    /// Number of search hits (see [`DiffView::search`]).
    pub matches: usize,
    /// True if a click changed the selection (see [`DiffView::selection`]).
    pub selection_changed: bool,
    /// Added and deleted lines in the whole diff.
    pub additions: usize,
    pub deletions: usize,
}

/// Shows the output of `git diff` (unified format): file headers, hunk headers,
/// old and new line numbers, green additions and red deletions, with the changed
/// words of modified lines highlighted. Only the rows in view are drawn, and the
/// parsed diff is cached until the text changes.
///
/// ```ignore
/// DiffView::new("diff", &diff_text).split(self.side_by_side).show(ui);
/// ```
///
/// With [`DiffView::selection`], clicking added and deleted lines selects them
/// (Shift+click selects a range, a click on a hunk header the whole hunk), and
/// [`partial_patch`] turns the selection into a patch for `git apply --cached`,
/// to stage single lines.
pub struct DiffView<'a> {
    id: Id,
    text: &'a str,
    split: bool,
    max_height: Option<f32>,
    selection: Option<&'a mut BTreeSet<usize>>,
    query: &'a str,
    current: Option<usize>,
}

impl<'a> DiffView<'a> {
    pub fn new(id_salt: impl egui::AsId, diff: &'a str) -> Self {
        Self { id: Id::new(id_salt), text: diff, split: false, max_height: None, selection: None, query: "", current: None }
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
    /// Makes added and deleted lines selectable. The set holds line indices of
    /// the diff text (as `diff.lines()` numbers them), ready for [`partial_patch`].
    pub fn selection(mut self, selected: &'a mut BTreeSet<usize>) -> Self {
        self.selection = Some(selected);
        self
    }
    /// Highlights every case-insensitive occurrence of `query`.
    /// [`DiffViewResponse::matches`] says how many there are.
    pub fn search(mut self, query: &'a str) -> Self {
        self.query = query;
        self
    }
    /// Highlights search hit `index` (from 0) more strongly, and scrolls to it
    /// whenever the index or the query changes.
    pub fn current_match(mut self, index: usize) -> Self {
        self.current = Some(index);
        self
    }

    pub fn show(self, ui: &mut Ui) -> DiffViewResponse {
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
        let hits_id = self.id.with("hits");
        let hits = match ui.data(|d| d.get_temp::<Arc<Hits>>(hits_id)) {
            Some(h) if h.hash == hash && h.query == self.query => h,
            _ => {
                let h = Arc::new(Hits { hash, query: self.query.to_owned(), hits: find_hits(&parsed, self.query) });
                ui.data_mut(|d| d.insert_temp(hits_id, h.clone()));
                h
            }
        };
        let current = self.current.filter(|&c| c < hits.hits.len());

        let font = FontId::monospace(12.5);
        let char_w = ui.ctx().fonts_mut(|f| f.glyph_width(&font, '0'));
        let row_h = 20.0;
        let digits = parsed.max_number.max(1).to_string().len() as f32;
        let num_w = digits * char_w + 16.0;
        let green = if t.dark { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(22, 163, 74) };
        let red = t.destructive;
        let amber = if t.dark { Color32::from_rgb(250, 204, 21) } else { Color32::from_rgb(234, 179, 8) };
        let colors = Colors {
            add_bg: green.gamma_multiply(if t.dark { 0.15 } else { 0.12 }),
            del_bg: red.gamma_multiply(if t.dark { 0.18 } else { 0.10 }),
            add_emph: green.gamma_multiply(if t.dark { 0.35 } else { 0.30 }),
            del_emph: red.gamma_multiply(if t.dark { 0.40 } else { 0.25 }),
            hit: amber.gamma_multiply(0.35),
            hit_current: amber.gamma_multiply(0.85),
            green,
            red,
        };
        let pc = PaintCtx { num_w, char_w, font, t: &t, c: &colors };

        let height = self.max_height.unwrap_or_else(|| ui.available_height());
        let mut scroll = if self.split { egui::ScrollArea::vertical() } else { egui::ScrollArea::both() };
        // Scroll the current hit into view when it changes.
        let scrolled_id = self.id.with("scrolled_to");
        if let Some(c) = current {
            let key = (self.query.to_owned(), c);
            if ui.data(|d| d.get_temp::<(String, usize)>(scrolled_id)).as_ref() != Some(&key) {
                ui.data_mut(|d| d.insert_temp(scrolled_id, key));
                let (line, start, _) = hits.hits[c];
                let row = if self.split { parsed.split_row[line] } else { line };
                let y = (row as f32 * row_h - height / 2.0).max(0.0);
                // Scroll sideways only when the hit would be out of view.
                let hit_x = 2.0 * num_w + 20.0 + (start + self.query.chars().count()) as f32 * char_w;
                let view_w = ui.available_width();
                let x = if self.split || hit_x + 16.0 < view_w { 0.0 } else { hit_x - view_w / 2.0 };
                scroll = scroll.scroll_offset(Vec2::new(x, y));
            }
        }
        let rows = if self.split { parsed.split.len() } else { parsed.lines.len() };
        let selectable = self.selection.is_some();
        let selection = self.selection;
        let empty = BTreeSet::new();
        let selected: &BTreeSet<usize> = selection.as_deref().unwrap_or(&empty);
        let mut clicked: Option<usize> = None; // index into lines
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
                let line_hits = |i: usize| {
                    let from = hits.hits.partition_point(|h| h.0 < i);
                    let to = hits.hits.partition_point(|h| h.0 <= i);
                    (from, &hits.hits[from..to])
                };
                let is_selected = |i: usize| selected.contains(&parsed.lines[i].raw);
                let changed = |i: Option<usize>| i.filter(|&i| matches!(parsed.lines[i].kind, Kind::Add | Kind::Del | Kind::Hunk));
                for n in range {
                    let sense = if selectable { Sense::click() } else { Sense::hover() };
                    let (rect, response) = ui.allocate_exact_size(Vec2::new(content_w, row_h), sense);
                    let p = ui.painter();
                    // The line under the pointer that a click would select.
                    let target = if !selectable {
                        None
                    } else if self.split {
                        match parsed.split[n] {
                            SplitRow::Full(i) => changed(Some(i)),
                            SplitRow::Pair(l, r) => {
                                let left = response.hover_pos().is_some_and(|pos| pos.x < rect.center().x);
                                changed(if left { l } else { r })
                            }
                        }
                    } else {
                        changed(Some(n))
                    };
                    if target.is_some() {
                        response.clone().on_hover_cursor(CursorIcon::PointingHand);
                        if response.clicked() {
                            clicked = target;
                        }
                    }
                    if self.split {
                        match parsed.split[n] {
                            SplitRow::Full(i) => paint_header(p, rect, &parsed.lines[i], &pc.font, &t),
                            SplitRow::Pair(l, r) => {
                                let half = rect.width() / 2.0;
                                let left = Rect::from_min_size(rect.min, Vec2::new(half, row_h));
                                let right = Rect::from_min_size(rect.min + Vec2::new(half, 0.0), Vec2::new(half, row_h));
                                for (side, idx, is_old) in [(left, l, true), (right, r, false)] {
                                    match idx {
                                        Some(i) => {
                                            let line = &parsed.lines[i];
                                            let number = if is_old { line.old } else { line.new };
                                            let (first, h) = line_hits(i);
                                            paint_line(p, side, line, &[number], &pc, is_selected(i), h, first, current);
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
                            Kind::File | Kind::Hunk => paint_header(p, rect, line, &pc.font, &t),
                            _ => {
                                let (first, h) = line_hits(n);
                                paint_line(p, rect, line, &[line.old, line.new], &pc, is_selected(n), h, first, current)
                            }
                        }
                    }
                }
            });
        ui.spacing_mut().item_spacing.y = saved_spacing;

        let mut selection_changed = false;
        if let (Some(i), Some(selection)) = (clicked, selection) {
            let anchor_id = self.id.with("anchor");
            let shift = ui.input(|inp| inp.modifiers.shift);
            let line = &parsed.lines[i];
            let changed_in = |from: usize, to: usize| {
                parsed.lines[from..to].iter().filter(|l| matches!(l.kind, Kind::Add | Kind::Del)).map(|l| l.raw).collect::<Vec<_>>()
            };
            if line.kind == Kind::Hunk {
                // The whole hunk: select it, or clear it when it is fully selected.
                let end = (i + 1..parsed.lines.len()).find(|&j| matches!(parsed.lines[j].kind, Kind::Hunk | Kind::File)).unwrap_or(parsed.lines.len());
                let lines = changed_in(i + 1, end);
                if lines.iter().all(|r| selection.contains(r)) {
                    for r in &lines {
                        selection.remove(r);
                    }
                } else {
                    selection.extend(lines);
                }
            } else {
                let anchor = ui.data(|d| d.get_temp::<usize>(anchor_id));
                let anchor = anchor.and_then(|a| parsed.lines.iter().position(|l| l.raw == a));
                match anchor {
                    Some(a) if shift => selection.extend(changed_in(a.min(i), a.max(i) + 1)),
                    _ => {
                        if !selection.remove(&line.raw) {
                            selection.insert(line.raw);
                        }
                    }
                }
                ui.data_mut(|d| d.insert_temp(anchor_id, line.raw));
            }
            selection_changed = true;
        }
        let response = ui.interact(out.inner_rect, self.id.with("area"), Sense::hover());
        DiffViewResponse {
            response,
            matches: hits.hits.len(),
            selection_changed,
            additions: parsed.additions,
            deletions: parsed.deletions,
        }
    }
}

struct Colors {
    add_bg: Color32,
    del_bg: Color32,
    add_emph: Color32,
    del_emph: Color32,
    hit: Color32,
    hit_current: Color32,
    green: Color32,
    red: Color32,
}

impl Colors {
    /// Background of a selected added or deleted line.
    fn selected(&self, kind: Kind) -> Color32 {
        if kind == Kind::Add { self.add_emph } else { self.del_emph }
    }
}

struct PaintCtx<'a> {
    num_w: f32,
    char_w: f32,
    font: FontId,
    t: &'a Theme,
    c: &'a Colors,
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
    pc: &PaintCtx<'_>,
    selected: bool,
    hits: &[(usize, usize, usize)],
    first_hit: usize,
    current: Option<usize>,
) {
    let (t, c) = (pc.t, pc.c);
    let (bg, emph, sign, sign_color) = match line.kind {
        Kind::Add => (c.add_bg, c.add_emph, "+", c.green),
        Kind::Del => (c.del_bg, c.del_emph, "-", c.red),
        _ => (Color32::TRANSPARENT, Color32::TRANSPARENT, " ", t.muted_foreground),
    };
    if bg != Color32::TRANSPARENT {
        p.rect_filled(rect, 0.0, if selected { c.selected(line.kind) } else { bg });
    }
    let p = p.with_clip_rect(rect.intersect(p.clip_rect()));
    let mut x = rect.left();
    for n in numbers {
        if let Some(n) = n {
            p.text(Pos2::new(x + pc.num_w - 8.0, rect.center().y), Align2::RIGHT_CENTER, n.to_string(), pc.font.clone(), t.muted_foreground);
        }
        x += pc.num_w;
    }
    let text_x = x + 20.0;
    let span = |from: usize, to: usize| {
        Rect::from_x_y_ranges(text_x + from as f32 * pc.char_w..=text_x + to as f32 * pc.char_w, rect.top() + 2.0..=rect.bottom() - 2.0)
    };
    for &(from, to) in &line.emph {
        p.rect_filled(span(from, to), 2.0, emph);
    }
    for (k, &(_, from, to)) in hits.iter().enumerate() {
        let fill = if current == Some(first_hit + k) { c.hit_current } else { c.hit };
        p.rect_filled(span(from, to), 2.0, fill);
    }
    if selected {
        p.rect_filled(Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())), 0.0, sign_color);
    }
    p.text(Pos2::new(x + 4.0, rect.center().y), Align2::LEFT_CENTER, sign, pc.font.clone(), sign_color);
    p.text(Pos2::new(text_x, rect.center().y), Align2::LEFT_CENTER, &line.text, pc.font.clone(), t.foreground);
}

/// Builds a patch that holds only the `selected` lines of `diff` (line indices
/// of the diff text, as [`DiffView::selection`] collects them), for staging or
/// unstaging part of a file:
///
/// - `reverse = false`: apply with `git apply --cached` to stage the selected
///   lines of an unstaged diff (`git diff`).
/// - `reverse = true`: apply with `git apply --cached --reverse` to unstage the
///   selected lines of a staged diff (`git diff --cached`).
///
/// Hunks without selected lines are left out, and so are files without any;
/// the result is empty when nothing is selected.
pub fn partial_patch(diff: &str, selected: &BTreeSet<usize>, reverse: bool) -> String {
    let lines: Vec<&str> = diff.lines().map(|l| l.trim_end_matches('\r')).collect();
    let mut out = String::new();
    let mut header: Vec<&str> = Vec::new(); // the current file's header, until a hunk is written
    let mut header_written = false;
    let mut shift: i64 = 0; // new count - old count of the hunks written for this file
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if !line.starts_with("@@ ") {
            if line.starts_with("diff --git ") || (header_written && line.starts_with("--- ")) {
                header.clear();
                header_written = false;
                shift = 0;
            }
            if !header_written {
                header.push(line);
            }
            i += 1;
            continue;
        }
        // A hunk: rewrite its body, then its header.
        let (old_start, new_start, tail) = parse_hunk_header(line);
        let mut body = Vec::new();
        let (mut old_n, mut new_n) = (0i64, 0i64);
        let mut has_change = false;
        let mut last_kept = false;
        let mut j = i + 1;
        while j < lines.len() {
            let l = lines[j];
            let sel = selected.contains(&j);
            match l.as_bytes().first() {
                Some(b' ') | None => {
                    body.push(format!(" {}", l.get(1..).unwrap_or("")));
                    old_n += 1;
                    new_n += 1;
                    last_kept = true;
                }
                Some(b'-') if sel => {
                    body.push(l.to_owned());
                    old_n += 1;
                    has_change = true;
                    last_kept = true;
                }
                Some(b'+') if sel => {
                    body.push(l.to_owned());
                    new_n += 1;
                    has_change = true;
                    last_kept = true;
                }
                // Unselected lines stay as they are on the side the patch is applied to.
                Some(b'-') if !reverse => {
                    body.push(format!(" {}", &l[1..]));
                    old_n += 1;
                    new_n += 1;
                    last_kept = true;
                }
                Some(b'+') if reverse => {
                    body.push(format!(" {}", &l[1..]));
                    old_n += 1;
                    new_n += 1;
                    last_kept = true;
                }
                Some(b'-') | Some(b'+') => last_kept = false,
                Some(b'\\') => {
                    if last_kept {
                        body.push(l.to_owned());
                    }
                }
                _ => break,
            }
            j += 1;
        }
        i = j;
        if !has_change {
            continue;
        }
        if !header_written {
            for h in &header {
                out.push_str(h);
                out.push('\n');
            }
            header_written = true;
        }
        // Applying forward matches the old side, which is unchanged; applying in
        // reverse matches the new side. The other side shifts by earlier hunks.
        let start = |count: i64, fixed_start: i64, fixed_count: i64, sign: i64| {
            let mut s = fixed_start + sign * shift;
            if fixed_count == 0 && count > 0 {
                s += 1;
            } else if count == 0 && fixed_count > 0 {
                s -= 1;
            }
            s.max(0)
        };
        let (o, n) = if reverse {
            (start(old_n, new_start, new_n, -1), new_start)
        } else {
            (old_start, start(new_n, old_start, old_n, 1))
        };
        out.push_str(&format!("@@ -{o},{old_n} +{n},{new_n} @@{tail}\n"));
        for b in body {
            out.push_str(&b);
            out.push('\n');
        }
        shift += new_n - old_n;
    }
    out
}

/// (old start, new start, text after the closing `@@`).
fn parse_hunk_header(line: &str) -> (i64, i64, &str) {
    let rest = &line[3..];
    let (ranges, tail) = rest.split_once(" @@").unwrap_or((rest, ""));
    let mut nums = ranges.split_whitespace();
    let start = |s: Option<&str>| s.and_then(|s| s.get(1..)?.split(',').next()?.parse::<i64>().ok()).unwrap_or(1);
    (start(nums.next()), start(nums.next()), tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Line continuations would eat the leading spaces of context lines.
    const DIFF: &str = concat!(
        "diff --git a/f.txt b/f.txt\n",
        "index 1..2 100644\n",
        "--- a/f.txt\n",
        "+++ b/f.txt\n",
        "@@ -1,4 +1,4 @@ fn main\n",
        " one\n",
        "-two\n",
        "+TWO\n",
        " three\n",
        "-four\n",
        "+FOUR\n",
        "@@ -10,2 +10,3 @@\n",
        " ten\n",
        "+ten and a half\n",
        " eleven\n",
    );

    #[test]
    fn word_diff_marks_changed_words() {
        let (a, b) = word_diff("let x = foo(1);", "let x = bar(1);").unwrap();
        assert_eq!(a, vec![(8, 11)]);
        assert_eq!(b, vec![(8, 11)]);
        assert!(word_diff("completely different", "nothing alike here").is_none());
    }

    #[test]
    fn parse_pairs_get_emphasis() {
        let p = parse(DIFF, 0);
        let two = p.lines.iter().find(|l| l.text == "two").unwrap();
        assert!(two.emph.is_empty(), "one-word lines have nothing in common");
        let p = parse("@@ -1 +1 @@\n-let a = 1;\n+let a = 2;\n", 0);
        assert_eq!(p.lines[1].emph, vec![(8, 9)]);
        assert_eq!(p.lines[2].emph, vec![(8, 9)]);
    }

    #[test]
    fn stage_one_line() {
        // Line 7 is "+TWO"; line 6 ("-two") stays as context.
        let sel = BTreeSet::from([7]);
        let patch = partial_patch(DIFF, &sel, false);
        assert_eq!(
            patch,
            "diff --git a/f.txt b/f.txt\nindex 1..2 100644\n--- a/f.txt\n+++ b/f.txt\n\
             @@ -1,4 +1,5 @@ fn main\n one\n two\n+TWO\n three\n four\n"
        );
    }

    #[test]
    fn later_hunks_shift() {
        // Delete "two" (line 6) and add "ten and a half" (line 13).
        let sel = BTreeSet::from([6, 13]);
        let patch = partial_patch(DIFF, &sel, false);
        assert!(patch.contains("@@ -1,4 +1,3 @@ fn main\n one\n-two\n three\n four\n"), "{patch}");
        assert!(patch.contains("@@ -10,2 +9,3 @@\n ten\n+ten and a half\n eleven\n"), "{patch}");
    }

    #[test]
    fn unstage_keeps_unselected_additions() {
        let sel = BTreeSet::from([10]); // "+FOUR"
        let patch = partial_patch(DIFF, &sel, true);
        assert!(patch.contains("@@ -1,3 +1,4 @@ fn main\n one\n TWO\n three\n+FOUR\n"), "{patch}");
        assert!(partial_patch(DIFF, &BTreeSet::new(), true).is_empty());
    }

    #[test]
    fn search_hits() {
        let p = parse(DIFF, 0);
        let hits = find_hits(&p, "Ten");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1].1, 0);
    }
}
