//! `<InputOTP>`: one box per character of a one-time code.

use egui::{
    Color32, CornerRadius, CursorIcon, Event, EventFilter, FontId, Key, Pos2, Rect, Response,
    Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetType,
};

use crate::theme::{paint_focus_ring, Theme};

/// `<InputOTP>`: a one-time code entered into separate boxes, for example a
/// 2FA code. Digits only unless [`InputOtp::alphanumeric`]; typing, Backspace
/// and pasting a whole code work. `response.changed()` fires on every edit,
/// and the code is complete when `code.len() == length`.
///
/// ```ignore
/// let r = ui.add(InputOtp::new(&mut code, 6).groups(&[3, 3]));
/// if r.changed() && code.len() == 6 { verify(&code); }
/// ```
#[must_use = "add it with `ui.add(...)`"]
pub struct InputOtp<'t> {
    code: &'t mut String,
    length: usize,
    groups: Vec<usize>,
    alphanumeric: bool,
    invalid: bool,
    label: String,
}

const SLOT: f32 = 36.0;

impl<'t> InputOtp<'t> {
    pub fn new(code: &'t mut String, length: usize) -> Self {
        Self {
            code,
            length: length.max(1),
            groups: vec![length.max(1)],
            alphanumeric: false,
            invalid: false,
            label: "One-time code".into(),
        }
    }
    /// Split the boxes into groups with a dash between them, e.g. `&[3, 3]`.
    /// The sizes should add up to the length.
    pub fn groups(mut self, groups: &[usize]) -> Self {
        let groups: Vec<usize> = groups.iter().copied().filter(|&g| g > 0).collect();
        if groups.iter().sum::<usize>() == self.length {
            self.groups = groups;
        }
        self
    }
    /// Accept letters as well as digits (letters are upper-cased).
    pub fn alphanumeric(mut self) -> Self {
        self.alphanumeric = true;
        self
    }
    /// Red borders, e.g. after a wrong code.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
    /// The accessibility label (default "One-time code").
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    fn accept(&self, c: char) -> Option<char> {
        if c.is_ascii_digit() {
            Some(c)
        } else if self.alphanumeric && c.is_ascii_alphabetic() {
            Some(c.to_ascii_uppercase())
        } else {
            None
        }
    }
}

impl Widget for InputOtp<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let sep_w = 24.0;
        let width = SLOT * self.length as f32 + sep_w * (self.groups.len() - 1) as f32;
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, SLOT), Sense::click());
        response = response.on_hover_cursor(CursorIcon::Text);
        if response.clicked() {
            response.request_focus();
        }

        // Keep the code valid if the caller changed it.
        let cleaned: String = self.code.chars().filter_map(|c| self.accept(c)).take(self.length).collect();
        if cleaned != *self.code {
            *self.code = cleaned;
        }

        let focused = response.has_focus() && ui.is_enabled();
        if focused {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(response.id, EventFilter { horizontal_arrows: true, ..Default::default() })
            });
            let events = ui.input(|i| i.events.clone());
            let mut changed = false;
            for event in events {
                match event {
                    Event::Text(s) | Event::Paste(s) => {
                        let accepted: Vec<char> = s.chars().filter_map(|c| self.accept(c)).collect();
                        for c in accepted {
                            if self.code.chars().count() < self.length {
                                self.code.push(c);
                                changed = true;
                            }
                        }
                    }
                    Event::Key { key: Key::Backspace, pressed: true, modifiers, .. } => {
                        if modifiers.command {
                            changed |= !self.code.is_empty();
                            self.code.clear();
                        } else {
                            changed |= self.code.pop().is_some();
                        }
                    }
                    _ => {}
                }
            }
            if changed {
                response.mark_changed();
            }
        }
        let code = self.code.clone();
        response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, ui.is_enabled(), &self.label));

        if !ui.is_rect_visible(rect) {
            return response;
        }
        let chars: Vec<char> = code.chars().collect();
        let active = chars.len().min(self.length - 1);
        let border = if self.invalid { t.destructive } else { t.input };
        let fill = if t.dark { t.input.gamma_multiply(0.3) } else { Color32::TRANSPARENT };
        let radius = t.radius_md();
        let font = FontId::proportional(14.0);

        let mut x = rect.left();
        let mut slot = 0;
        let mut active_slot = None;
        for (g, &size) in self.groups.iter().enumerate() {
            if g > 0 {
                let c = Pos2::new(x + sep_w / 2.0, rect.center().y);
                ui.painter().hline(c.x - 4.0..=c.x + 4.0, c.y, Stroke::new(1.5, t.muted_foreground));
                x += sep_w;
            }
            let group = Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(SLOT * size as f32, SLOT));
            ui.painter().rect(group, radius, fill, Stroke::new(1.0, border), StrokeKind::Inside);
            for k in 0..size {
                let r = Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::splat(SLOT));
                if k > 0 {
                    ui.painter().vline(r.left(), r.y_range(), Stroke::new(1.0, border));
                }
                if let Some(c) = chars.get(slot) {
                    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, c, font.clone(), t.foreground);
                }
                if slot == active {
                    let corners = CornerRadius {
                        nw: if k == 0 { radius.nw } else { 0 },
                        sw: if k == 0 { radius.sw } else { 0 },
                        ne: if k + 1 == size { radius.ne } else { 0 },
                        se: if k + 1 == size { radius.se } else { 0 },
                    };
                    active_slot = Some((r, corners, chars.get(slot).is_none()));
                }
                x += SLOT;
                slot += 1;
            }
        }
        if let (true, Some((r, corners, empty))) = (focused, active_slot) {
            let mut ring = t.clone();
            if self.invalid {
                ring.ring = t.destructive;
            }
            paint_focus_ring(ui.painter(), r, corners, &ring);
            if empty {
                let time = ui.input(|i| i.time);
                if time % 1.0 < 0.5 {
                    ui.painter().vline(r.center().x, r.center().y - 8.0..=r.center().y + 8.0, Stroke::new(1.0, t.foreground));
                }
                ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(0.5 - time % 0.5));
            }
        }
        response
    }
}
