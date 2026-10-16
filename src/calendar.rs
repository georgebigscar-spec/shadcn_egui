//! Calendar, DatePicker and DateRangePicker, with a small `Date` type so no
//! date crate is needed.

use egui::{Align2, Color32, CursorIcon, FontId, Id, Margin, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetType};

use crate::basic::paint_chevron;
use crate::theme::{paint_focus_ring, Theme};

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];

/// A calendar date (proleptic Gregorian), e.g. `Date::new(2026, 10, 16)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    /// 1–12.
    pub month: u32,
    /// 1–31.
    pub day: u32,
}

impl Date {
    /// `None` if the month or day is out of range.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month)).then_some(Self { year, month, day })
    }

    /// Today in UTC, from the system clock.
    pub fn today() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        Self::from_days(secs.div_euclid(86_400))
    }

    /// Days since 1970-01-01.
    pub fn to_days(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let y = self.year as i64 - (self.month <= 2) as i64;
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` days after 1970-01-01.
    pub fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = (yoe + era * 400 + (month <= 2) as i64) as i32;
        Self { year, month, day }
    }

    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    /// 0 = Sunday … 6 = Saturday.
    pub fn weekday(self) -> u32 {
        (self.to_days() + 4).rem_euclid(7) as u32
    }

    /// `2026-10-16`.
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// `October 16, 2026`, like shadcn's `PPP` format.
    pub fn long(self) -> String {
        format!("{} {}, {}", MONTHS[self.month as usize - 1], self.day, self.year)
    }

    /// `Oct 16, 2026`.
    pub fn short(self) -> String {
        format!("{} {}, {}", &MONTHS[self.month as usize - 1][..3], self.day, self.year)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.iso())
    }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// A from–to selection for [`Calendar::show_range`] and [`DateRangePicker`].
/// While picking, `end` is `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateRange {
    pub start: Option<Date>,
    pub end: Option<Date>,
}

impl DateRange {
    pub fn contains(&self, date: Date) -> bool {
        match (self.start, self.end) {
            (Some(a), Some(b)) => a <= date && date <= b,
            (Some(a), None) => a == date,
            _ => false,
        }
    }
    pub fn is_complete(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }
}

// ---------------------------------------------------------------------------

/// `<Calendar>`: a month grid with previous/next buttons.
///
/// ```ignore
/// Calendar::new("due").show(ui, &mut self.date);              // one date
/// Calendar::new("log").show_range(ui, &mut self.range);       // from–to
/// ```
pub struct Calendar {
    id: Id,
    monday_first: bool,
    max: Option<Date>,
    min: Option<Date>,
    today: Date,
}

/// What a day cell is.
enum Mark {
    Single(Option<Date>),
    Range(DateRange),
}

impl Calendar {
    pub fn new(id_salt: impl egui::AsId) -> Self {
        Self { id: Id::new(id_salt), monday_first: false, max: None, min: None, today: Date::today() }
    }
    /// Weeks start on Monday instead of Sunday.
    pub fn monday_first(mut self) -> Self {
        self.monday_first = true;
        self
    }
    /// Later days are disabled (e.g. no commits in the future).
    pub fn max_date(mut self, date: Date) -> Self {
        self.max = Some(date);
        self
    }
    /// Earlier days are disabled.
    pub fn min_date(mut self, date: Date) -> Self {
        self.min = Some(date);
        self
    }
    /// The day highlighted as today (default: the system date, UTC).
    pub fn today(mut self, date: Date) -> Self {
        self.today = date;
        self
    }

    /// Single date. The response is marked changed when a day is picked.
    pub fn show(self, ui: &mut Ui, selected: &mut Option<Date>) -> Response {
        let (mut response, picked) = self.grid(ui, Mark::Single(*selected), *selected);
        if let Some(d) = picked {
            *selected = Some(d);
            response.mark_changed();
        }
        response
    }

    /// From–to range: the first click sets the start, the second the end
    /// (swapped if it is earlier); a third click starts over.
    pub fn show_range(self, ui: &mut Ui, range: &mut DateRange) -> Response {
        let (mut response, picked) = self.grid(ui, Mark::Range(*range), range.start);
        if let Some(d) = picked {
            *range = match (range.start, range.end) {
                (Some(a), None) if d >= a => DateRange { start: Some(a), end: Some(d) },
                (Some(a), None) => DateRange { start: Some(d), end: Some(a) },
                _ => DateRange { start: Some(d), end: None },
            };
            response.mark_changed();
        }
        response
    }

    fn grid(&self, ui: &mut Ui, mark: Mark, initial: Option<Date>) -> (Response, Option<Date>) {
        let t = Theme::get(ui.ctx());
        let month_id = self.id.with("month");
        let (mut year, mut month) = ui
            .data(|d| d.get_temp::<(i32, u32)>(month_id))
            .unwrap_or_else(|| {
                let d = initial.unwrap_or(self.today);
                (d.year, d.month)
            });
        let cell = 32.0;
        let width = 7.0 * cell;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 32.0 + 8.0 + 24.0 + 6.0 * cell), Sense::hover());
        let p = ui.painter().clone();

        // Header: ‹ Month Year ›
        let header = Rect::from_min_size(rect.min, Vec2::new(width, 32.0));
        p.text(header.center(), Align2::CENTER_CENTER, format!("{} {}", MONTHS[month as usize - 1], year), FontId::proportional(14.0), t.foreground);
        for (dir, center) in [(-1i32, header.left_center() + Vec2::new(14.0, 0.0)), (1, header.right_center() - Vec2::new(14.0, 0.0))] {
            let b = Rect::from_center_size(center, Vec2::splat(28.0));
            let label = if dir < 0 { "Previous month" } else { "Next month" };
            let r = ui.interact(b, self.id.with(label), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            r.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
            p.rect(b, t.radius_md(), if r.hovered() { t.accent } else { Color32::TRANSPARENT }, Stroke::new(1.0, t.input), StrokeKind::Inside);
            let angle = if dir < 0 { std::f32::consts::FRAC_PI_2 } else { -std::f32::consts::FRAC_PI_2 };
            paint_chevron(&p, b.center(), angle, t.foreground);
            if r.clicked() {
                let m = year * 12 + month as i32 - 1 + dir;
                year = m.div_euclid(12);
                month = (m.rem_euclid(12) + 1) as u32;
            }
        }

        // Weekday names.
        let names = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let first_col = if self.monday_first { 1 } else { 0 };
        let top = header.bottom() + 8.0;
        for c in 0..7 {
            let name = names[(c + first_col) % 7];
            p.text(Pos2::new(rect.left() + (c as f32 + 0.5) * cell, top + 12.0), Align2::CENTER_CENTER, name, FontId::proportional(12.0), t.muted_foreground);
        }

        // Six weeks of days, starting on the week that holds the 1st.
        let first = Date { year, month, day: 1 };
        let offset = (first.weekday() as i64 - first_col as i64).rem_euclid(7);
        let start = first.add_days(-offset);
        let mut picked = None;
        for n in 0..42 {
            let date = start.add_days(n);
            let (row, col) = (n / 7, n % 7);
            let r = Rect::from_min_size(Pos2::new(rect.left() + col as f32 * cell, top + 24.0 + row as f32 * cell), Vec2::splat(cell));
            let outside = date.month != month;
            let disabled = self.max.is_some_and(|m| date > m) || self.min.is_some_and(|m| date < m);
            let resp = ui.interact(r, self.id.with(("day", date.year, date.month, date.day)), if disabled { Sense::hover() } else { Sense::click() });
            resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, !disabled, date.long()));
            let (is_selected, in_range) = match &mark {
                Mark::Single(s) => (*s == Some(date), false),
                Mark::Range(range) => (range.start == Some(date) || range.end == Some(date), range.contains(date)),
            };
            let cell_rect = r.shrink(1.0);
            if in_range && !is_selected {
                p.rect_filled(cell_rect, 0.0, t.accent);
            }
            let fg = if is_selected {
                p.rect_filled(cell_rect, t.radius_md(), t.primary);
                t.primary_foreground
            } else if (!disabled && resp.hovered()) || (date == self.today && !in_range) {
                p.rect_filled(cell_rect, t.radius_md(), t.accent);
                t.accent_foreground
            } else if outside || disabled {
                t.muted_foreground
            } else {
                t.foreground
            };
            let fg = if disabled { fg.gamma_multiply(0.5) } else { fg };
            p.text(r.center(), Align2::CENTER_CENTER, date.day.to_string(), FontId::proportional(14.0), fg);
            if resp.clicked() {
                picked = Some(date);
                if outside {
                    year = date.year;
                    month = date.month;
                }
            }
        }
        ui.data_mut(|d| d.insert_temp(month_id, (year, month)));
        (response, picked)
    }
}

// ---------------------------------------------------------------------------

fn paint_trigger(ui: &Ui, rect: Rect, text: &str, placeholder: bool, open: bool, has_focus: bool) {
    let t = Theme::get(ui.ctx());
    let p = ui.painter();
    p.add(t.shadow_sm().as_shape(rect, t.radius_md()));
    p.rect(rect, t.radius_md(), t.background, Stroke::new(1.0, t.input), StrokeKind::Inside);
    // Calendar icon: a rounded box with a header line and two rings.
    let icon = Rect::from_center_size(Pos2::new(rect.left() + 20.0, rect.center().y), Vec2::new(13.0, 12.0));
    let color = t.muted_foreground;
    p.rect_stroke(icon, 2.0, Stroke::new(1.25, color), StrokeKind::Inside);
    p.hline(icon.x_range(), icon.top() + 4.0, Stroke::new(1.25, color));
    for x in [icon.left() + 3.5, icon.right() - 3.5] {
        p.vline(x, (icon.top() - 2.0)..=(icon.top() + 1.5), Stroke::new(1.25, color));
    }
    let fg = if placeholder { t.muted_foreground } else { t.foreground };
    p.text(Pos2::new(rect.left() + 36.0, rect.center().y), Align2::LEFT_CENTER, text, FontId::proportional(14.0), fg);
    if has_focus || open {
        paint_focus_ring(p, rect, t.radius_md(), &t);
    }
}

fn picker_popup<R>(response: &Response, add: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let t = Theme::get(&response.ctx);
    egui::Popup::menu(response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .gap(4.0)
        .frame(t.popover_frame().inner_margin(Margin::same(12)))
        .show(add)
        .map(|r| r.inner)
}

/// `<DatePicker>`: an outline button showing the date that opens a [`Calendar`]
/// in a popover; picking a day closes it.
#[must_use = "add it with `ui.add(...)`"]
pub struct DatePicker<'a> {
    id: Id,
    date: &'a mut Option<Date>,
    placeholder: String,
    width: f32,
    calendar: Box<dyn FnOnce(Calendar) -> Calendar + 'a>,
}

impl<'a> DatePicker<'a> {
    pub fn new(id_salt: impl egui::AsId, date: &'a mut Option<Date>) -> Self {
        Self { id: Id::new(id_salt), date, placeholder: "Pick a date".into(), width: 240.0, calendar: Box::new(|c| c) }
    }
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    /// Configures the calendar, e.g. `.calendar(|c| c.max_date(Date::today()))`.
    pub fn calendar(mut self, f: impl FnOnce(Calendar) -> Calendar + 'a) -> Self {
        self.calendar = Box::new(f);
        self
    }
}

impl Widget for DatePicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(self.width, 36.0), Sense::click());
        let text = self.date.map_or(self.placeholder.clone(), Date::long);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::ComboBox, ui.is_enabled(), &text));
        response = response.on_hover_cursor(CursorIcon::PointingHand);
        let mut changed = false;
        let open = picker_popup(&response, |ui| {
            if (self.calendar)(Calendar::new(self.id)).show(ui, self.date).changed() {
                changed = true;
                ui.close();
            }
        })
        .is_some();
        paint_trigger(ui, rect, &text, self.date.is_none(), open, response.has_focus());
        if changed {
            response.mark_changed();
        }
        response
    }
}

/// A [`DatePicker`] for a from–to range; closes once both ends are picked.
#[must_use = "add it with `ui.add(...)`"]
pub struct DateRangePicker<'a> {
    id: Id,
    range: &'a mut DateRange,
    placeholder: String,
    width: f32,
    calendar: Box<dyn FnOnce(Calendar) -> Calendar + 'a>,
}

impl<'a> DateRangePicker<'a> {
    pub fn new(id_salt: impl egui::AsId, range: &'a mut DateRange) -> Self {
        Self { id: Id::new(id_salt), range, placeholder: "Pick a date range".into(), width: 280.0, calendar: Box::new(|c| c) }
    }
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    /// Configures the calendar, e.g. `.calendar(|c| c.monday_first())`.
    pub fn calendar(mut self, f: impl FnOnce(Calendar) -> Calendar + 'a) -> Self {
        self.calendar = Box::new(f);
        self
    }
}

impl Widget for DateRangePicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(self.width, 36.0), Sense::click());
        let text = match (self.range.start, self.range.end) {
            (Some(a), Some(b)) => format!("{} – {}", a.short(), b.short()),
            (Some(a), None) => format!("{} – …", a.short()),
            _ => self.placeholder.clone(),
        };
        response.widget_info(|| WidgetInfo::labeled(WidgetType::ComboBox, ui.is_enabled(), &text));
        response = response.on_hover_cursor(CursorIcon::PointingHand);
        let mut changed = false;
        let open = picker_popup(&response, |ui| {
            if (self.calendar)(Calendar::new(self.id)).show_range(ui, self.range).changed() {
                changed = true;
                if self.range.is_complete() {
                    ui.close();
                }
            }
        })
        .is_some();
        paint_trigger(ui, rect, &text, self.range.start.is_none(), open, response.has_focus());
        if changed {
            response.mark_changed();
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_round_trip() {
        for days in [-800_000, -1, 0, 1, 59, 60, 11_016, 20_742, 2_932_896] {
            assert_eq!(Date::from_days(days).to_days(), days);
        }
        assert_eq!(Date::from_days(0), Date { year: 1970, month: 1, day: 1 });
        assert_eq!(Date::new(2024, 2, 29).unwrap().add_days(1), Date { year: 2024, month: 3, day: 1 });
        assert!(Date::new(2025, 2, 29).is_none());
    }

    #[test]
    fn weekdays() {
        assert_eq!(Date::new(1970, 1, 1).unwrap().weekday(), 4); // Thursday
        assert_eq!(Date::new(2026, 10, 16).unwrap().weekday(), 5); // Friday
        assert_eq!(Date::new(2000, 2, 29).unwrap().weekday(), 2); // Tuesday
    }
}
