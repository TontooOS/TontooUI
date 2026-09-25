use std::any::Any;
use std::time::{SystemTime, UNIX_EPOCH};

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Circle, Join, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
use super::menu::{MENU_CHECK_H, MENU_CHECK_W};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::renderer::window::Key;
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Calendar cell width in logical px.
pub const DATE_CELL_W: f32 = 32.0;
/// Calendar cell height in logical px.
pub const DATE_CELL_H: f32 = 28.0;
/// Header bar height in logical px.
pub const DATE_HEADER_H: f32 = 32.0;
/// Weekday row height in logical px.
pub const DATE_WEEK_H: f32 = 20.0;
/// Panel padding on every side in logical px.
pub const DATE_PAD: f32 = 8.0;
/// Panel corner radius in logical px.
pub const DATE_RADIUS: f32 = 9.0;
/// Header title size in logical px.
pub const DATE_TITLE_SIZE: f32 = 14.0;
/// Day number size in logical px.
pub const DATE_DAY_SIZE: f32 = 14.0;
/// Weekday header size in logical px.
pub const DATE_WEEK_SIZE: f32 = 10.0;
/// Popup list row text size in logical px.
pub const DATE_LIST_SIZE: f32 = 13.0;
/// Selected-day circle radius in logical px.
pub const DATE_SEL_R: f32 = 12.0;
/// Month nav button hit width in logical px.
pub const DATE_NAV_W: f32 = 28.0;
/// Gap between month and year zones in logical px.
pub const DATE_TITLE_GAP: f32 = 8.0;
/// Popup list row height in logical px.
pub const DATE_LIST_ROW_H: f32 = 26.0;
/// Popup list row gap in logical px.
pub const DATE_LIST_SPACING: f32 = 2.0;
/// Popup list padding in logical px.
pub const DATE_LIST_PAD: f32 = 6.0;
/// Popup list corner radius in logical px.
pub const DATE_LIST_RADIUS: f32 = 9.0;
/// Visible popup rows before scrolling in logical px.
pub const DATE_LIST_VISIBLE: usize = 8;
/// Checkmark column width in popup rows in logical px.
pub const DATE_LIST_CHECK_COL: f32 = 20.0;
/// Gap between check column and row text in logical px.
pub const DATE_LIST_TEXT_GAP: f32 = 6.0;
/// Scrollbar width in logical px.
pub const DATE_SCROLL_W: f32 = 6.0;
/// Gap between header and popup panel in logical px.
pub const DATE_POP_GAP: f32 = 4.0;
/// First selectable year.
pub const DATE_YEAR_MIN: i32 = 1;
/// Last selectable year.
pub const DATE_YEAR_MAX: i32 = 3000;
/// Default selection fill (theme accent blue).
pub const DATE_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
/// Calendar edge shadow blur in logical px (heavy, like the menu).
pub const DATE_SHADOW_BLUR: f32 = 24.0;

/// Full English month names, January first.
pub const DATE_MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Monday-first weekday headers, like the reference.
pub const DATE_WEEKDAYS: [&str; 7] = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"];

/// Days since 1970-01-01 for a civil date (Hinnant algorithm).
fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (month as i32 + 9).rem_euclid(12);
    let doy = (153 * mp + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Civil date for days since 1970-01-01.
fn civil_from_days(days: i32) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if month <= 2 { y + 1 } else { y }, month, day)
}

/// Today as (year, month, day) from the system clock.
fn today() -> (i32, u32, u32) {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0) as i32;
    civil_from_days(days)
}

/// Days in a month (1-12), Gregorian leap rule.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Monday-first weekday index (0-6) of the first of the month.
pub fn first_weekday(year: i32, month: u32) -> u32 {
    // 1970-01-01 was a Thursday (index 3).
    (days_from_civil(year, month, 1) + 3).rem_euclid(7) as u32
}

/// Open header popup list, if any.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Popup {
    #[default]
    None,
    Months,
    Years,
}

/// Clickable header zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeaderZone {
    Month,
    Year,
}

/// Press arming while the calendar itself is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Armed {
    Day(u32),
    Nav(i32),
    Header(HeaderZone),
}

/// Date picker: frosted glass calendar with a header bar. The header
/// shows the month and the year as two menus plus `<`/`>` month
/// steppers. Clicking the month opens all twelve months; clicking
/// the year opens a scrollable year list (`DATE_YEAR_MIN` to
/// `DATE_YEAR_MAX`, current year visible on open). Clicking a day
/// selects it with a filled circle, like the reference.
///
/// Panels are always clamped into the `set_viewport` bounds so the
/// glass never samples outside the window. Apps must call
/// `set_viewport` every frame and forward `mouse_wheel` (see
/// `examples/date.rs`).
pub struct DatePicker {
    view_year: i32,
    view_month: u32,
    selected: (i32, u32, u32),
    popup: Popup,
    list_offset: f32,
    hovered_row: Option<usize>,
    hovered_day: Option<u32>,
    hovered_zone: Option<HeaderZone>,
    hovered_nav: Option<i32>,
    armed_row: Option<usize>,
    selection: Color,
    selection_manual: bool,
    hover: Color,
    hover_manual: bool,
    dark: bool,
    text_color: Color,
    text_dim: Color,
    glass: GlassContainer,
    glass_pop: GlassContainer,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    panel_x: f32,
    panel_y: f32,
    panel_w: f32,
    panel_h: f32,
    grid_x: f32,
    grid_y: f32,
    title_x: f32,
    title_y: f32,
    title_w: f32,
    month_x: f32,
    month_w: f32,
    year_x: f32,
    year_w: f32,
    pop_x: f32,
    pop_y: f32,
    pop_w: f32,
    pop_h: f32,
    pop_month_w: f32,
    pop_year_w: f32,
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
    armed: Option<Armed>,
    disabled: bool,
    focused: bool,
    on_select: Option<Box<dyn FnMut(i32, u32, u32)>>,
}

impl DatePicker {
    pub fn new() -> Self {
        let (y, m, d) = today();
        let mut glass = GlassContainer::new();
        glass.set_glass_type(GlassType::Frosted);
        let mut glass_pop = GlassContainer::new();
        glass_pop.set_glass_type(GlassType::Frosted);
        Self {
            view_year: y,
            view_month: m,
            selected: (y, m, d),
            popup: Popup::None,
            list_offset: 0.0,
            hovered_row: None,
            hovered_day: None,
            hovered_zone: None,
            hovered_nav: None,
            armed_row: None,
            selection: Color::WHITE,
            hover: DATE_ACCENT,
            hover_manual: false,
            selection_manual: false,
            dark: true,
            text_color: Color::WHITE,
            text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            glass,
            glass_pop,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            panel_x: 0.0,
            panel_y: 0.0,
            panel_w: 0.0,
            panel_h: 0.0,
            grid_x: 0.0,
            grid_y: 0.0,
            title_x: 0.0,
            title_y: 0.0,
            title_w: 0.0,
            month_x: 0.0,
            month_w: 0.0,
            year_x: 0.0,
            year_w: 0.0,
            pop_x: 0.0,
            pop_y: 0.0,
            pop_w: 0.0,
            pop_h: 0.0,
            pop_month_w: 0.0,
            pop_year_w: 0.0,
            vp_x: 0.0,
            vp_y: 0.0,
            vp_w: f32::MAX,
            vp_h: f32::MAX,
            armed: None,
            disabled: false,
            focused: true,
            on_select: None,
        }
    }

    /// Initial selection without firing `on_select`. The viewed month
    /// follows the selection. The day clamps to the month length.
    pub fn selected(mut self, year: i32, month: u32, day: u32) -> Self {
        let month = month.clamp(1, 12);
        let day = day.clamp(1, days_in_month(year, month));
        self.selected = (year, month, day);
        self.view_year = year;
        self.view_month = month;
        self
    }

    /// Manual selection-circle fill: wins over the default (label
    /// color, black in light mode like the reference).
    pub fn accent(mut self, color: Color) -> Self {
        self.selection = color;
        self.selection_manual = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, callback: impl FnMut(i32, u32, u32) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Manual row-hover fill: wins over the system accent until
    /// cleared. The popup hover follows the system accent unless set
    /// by hand.
    pub fn hover_fill(mut self, color: Color) -> Self {
        self.hover = color;
        self.hover_manual = true;
        self
    }

    /// Live theme: label colors and mode. The selection circle keeps
    /// the label color (black in light mode, white in dark) unless
    /// set manually with `accent`; the popup hover follows the
    /// accent unless set manually with `hover_fill`.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.hover_manual {
            self.hover = accent;
        }
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.text_dim = Color::from_rgb8(0x9a, 0x9a, 0x9e);
            if !self.selection_manual {
                self.selection = Color::WHITE;
            }
        } else {
            self.text_color = Color::BLACK;
            self.text_dim = Color::from_rgb8(0x6e, 0x6e, 0x72);
            if !self.selection_manual {
                self.selection = Color::BLACK;
            }
        }
    }

    /// Glass stage for the calendar and popup panels (frost tint per
    /// setting).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.glass.set_theme(mode, amount);
        self.glass_pop.set_theme(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
        self.glass_pop.set_focused(focused);
    }

    /// Window bounds the panel is clamped into. Apps must call this
    /// every frame with the current viewport.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vp_x = x;
        self.vp_y = y;
        self.vp_w = w.max(0.0);
        self.vp_h = h.max(0.0);
    }

    pub fn selected_date(&self) -> (i32, u32, u32) {
        self.selected
    }

    pub fn viewed(&self) -> (i32, u32) {
        (self.view_year, self.view_month)
    }

    /// Set the selection (clamped to the month). Fires `on_select`
    /// when the date changed.
    pub fn set_selected(&mut self, year: i32, month: u32, day: u32) {
        let month = month.clamp(1, 12);
        let day = day.clamp(1, days_in_month(year, month));
        if (year, month, day) != self.selected {
            self.selected = (year, month, day);
            self.view_year = year;
            self.view_month = month;
            self.notify();
        }
    }

    /// Step the viewed month by `delta` (wraps years).
    pub fn step_month(&mut self, delta: i32) {
        let total = self.view_year * 12 + (self.view_month as i32 - 1) + delta;
        self.view_year = total.div_euclid(12);
        self.view_month = (total.rem_euclid(12) + 1) as u32;
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_select.as_mut() {
            callback(self.selected.0, self.selected.1, self.selected.2);
        }
    }

    fn panel_content_w(&self) -> f32 {
        DATE_CELL_W * 7.0
    }

    fn panel_content_h(&self) -> f32 {
        DATE_HEADER_H + DATE_WEEK_H + DATE_CELL_H * 6.0
    }

    fn eff(&self, color: Color) -> Color {
        let mut color = if self.focused {
            color
        } else {
            desaturate(color)
        };
        if self.disabled {
            let c = color.to_rgba8();
            color = Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * 0.4).round() as u8);
        }
        color
    }

    fn layout_panel(&mut self, fonts: &mut FontSystem) {
        let w = (self.panel_content_w() + DATE_PAD * 2.0).min(self.vp_w).max(0.0);
        let h = (self.panel_content_h() + DATE_PAD * 2.0).min(self.vp_h).max(0.0);
        // Keep the panel inside the placed rect when it fits, else
        // clamp into the viewport.
        let mut x = self.x;
        let mut y = self.y;
        if x + w > self.vp_x + self.vp_w {
            x = self.vp_x + self.vp_w - w;
        }
        if y + h > self.vp_y + self.vp_h {
            y = self.vp_y + self.vp_h - h;
        }
        if x < self.vp_x {
            x = self.vp_x;
        }
        if y < self.vp_y {
            y = self.vp_y;
        }
        self.panel_x = x;
        self.panel_y = y;
        self.panel_w = w;
        self.panel_h = h;
        self.glass.set_bounds(x, y, w, h);
        self.glass.set_radius(DATE_RADIUS);
        self.grid_x = x + DATE_PAD;
        self.grid_y = y + DATE_PAD + DATE_HEADER_H + DATE_WEEK_H;
        self.title_x = x + DATE_PAD;
        self.title_y = y + DATE_PAD;
        self.title_w = (w - DATE_PAD * 2.0 - DATE_NAV_W * 2.0).max(0.0);
        // Month zone fits the longest name, year zone a 4-digit year.
        let mut month_w: f32 = 0.0;
        for name in DATE_MONTHS {
            month_w = month_w.max(self.text_w(fonts, name, DATE_TITLE_SIZE));
        }
        self.month_x = self.title_x;
        self.month_w = month_w;
        self.year_x = self.month_x + month_w + DATE_TITLE_GAP;
        self.year_w = (self.title_w - month_w - DATE_TITLE_GAP).max(0.0);
        // Cache popup content widths (fonts available here) so the
        // popup can lay out on clicks without fonts.
        let mut months: f32 = 0.0;
        for name in DATE_MONTHS {
            months = months.max(self.text_w(fonts, name, DATE_LIST_SIZE));
        }
        self.pop_month_w = months + DATE_LIST_CHECK_COL + DATE_LIST_TEXT_GAP + DATE_SCROLL_W;
        self.pop_year_w = self.text_w(fonts, "3000", DATE_LIST_SIZE)
            + DATE_LIST_CHECK_COL
            + DATE_LIST_TEXT_GAP
            + DATE_SCROLL_W;
        self.layout_popup();
    }

    fn text_w(&self, fonts: &mut FontSystem, text: &str, size: f32) -> f32 {
        let layout = fonts.layout_text(text, size, Color::WHITE, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    fn popup_rows(&self) -> usize {
        match self.popup {
            Popup::None => 0,
            Popup::Months => 12,
            Popup::Years => (DATE_YEAR_MAX - DATE_YEAR_MIN + 1) as usize,
        }
    }

    fn popup_row_label(&self, row: usize) -> String {
        match self.popup {
            Popup::None => String::new(),
            Popup::Months => DATE_MONTHS[row.min(11)].to_string(),
            Popup::Years => (DATE_YEAR_MIN + row as i32).to_string(),
        }
    }

    fn popup_current_row(&self) -> usize {
        match self.popup {
            Popup::None => 0,
            Popup::Months => (self.view_month - 1) as usize,
            Popup::Years => (self.view_year - DATE_YEAR_MIN).max(0) as usize,
        }
    }

    fn list_stride(&self) -> f32 {
        DATE_LIST_ROW_H + DATE_LIST_SPACING
    }

    fn list_total_h(&self) -> f32 {
        let n = self.popup_rows() as f32;
        if n <= 0.0 {
            return 0.0;
        }
        n * DATE_LIST_ROW_H + (n - 1.0) * DATE_LIST_SPACING
    }

    fn list_visible(&self) -> usize {
        self.popup_rows().min(DATE_LIST_VISIBLE)
    }

    fn list_visible_h(&self) -> f32 {
        let n = self.list_visible() as f32;
        if n <= 0.0 {
            return 0.0;
        }
        n * DATE_LIST_ROW_H + (n - 1.0) * DATE_LIST_SPACING
    }

    fn max_offset(&self) -> f32 {
        (self.list_total_h() - self.list_visible_h()).max(0.0)
    }

    /// Popup panel rect under the active header zone, clamped into the
    /// viewport so the glass never samples outside the window.
    fn layout_popup(&mut self) {
        if self.popup == Popup::None || self.popup_rows() == 0 {
            self.pop_w = 0.0;
            self.pop_h = 0.0;
            return;
        }
        let content = match self.popup {
            Popup::Months => self.pop_month_w,
            Popup::Years => self.pop_year_w,
            Popup::None => 0.0,
        };
        let anchor = match self.popup {
            Popup::Months => (self.month_x, self.month_w),
            Popup::Years => (self.year_x, self.year_w),
            Popup::None => (self.title_x, self.title_w),
        };
        let mut w = (content + DATE_LIST_PAD * 2.0).max(anchor.1 + DATE_LIST_PAD);
        let mut h = self.list_visible_h() + DATE_LIST_PAD * 2.0;
        w = w.min(self.vp_w).max(0.0);
        h = h.min(self.vp_h).max(0.0);
        let mut x = anchor.0 - DATE_LIST_PAD / 2.0;
        if x + w > self.vp_x + self.vp_w {
            x = self.vp_x + self.vp_w - w;
        }
        if x < self.vp_x {
            x = self.vp_x;
        }
        let below = self.title_y + DATE_HEADER_H + DATE_POP_GAP;
        let mut y = below;
        if y + h > self.vp_y + self.vp_h {
            y = self.title_y - DATE_POP_GAP - h;
        }
        if y < self.vp_y {
            y = self.vp_y;
        }
        self.pop_x = x;
        self.pop_y = y;
        self.pop_w = w;
        self.pop_h = h;
        self.list_offset = self.list_offset.clamp(0.0, self.max_offset());
        self.glass_pop.set_bounds(x, y, w, h);
        self.glass_pop.set_radius(DATE_LIST_RADIUS);
    }

    fn open_months(&mut self) {
        self.popup = Popup::Months;
        self.list_offset = 0.0;
        self.hovered_row = None;
        self.armed_row = None;
        self.layout_popup();
    }

    fn open_years(&mut self) {
        self.popup = Popup::Years;
        // Current year visible on open, roughly centered.
        let stride = self.list_stride();
        self.list_offset = ((self.view_year - DATE_YEAR_MIN) as f32 * stride
            - (DATE_LIST_VISIBLE as f32 - 1.0) / 2.0 * stride)
            .clamp(0.0, self.max_offset());
        self.hovered_row = None;
        self.armed_row = None;
        self.layout_popup();
    }

    fn close_popup(&mut self) {
        self.popup = Popup::None;
        self.hovered_row = None;
        self.hovered_day = None;
        self.hovered_zone = None;
        self.hovered_nav = None;
        self.armed_row = None;
    }

    fn pop_row_at(&self, x: f32, y: f32) -> Option<usize> {
        if self.popup == Popup::None {
            return None;
        }
        let list_y = self.pop_y + DATE_LIST_PAD;
        if x < self.pop_x
            || x > self.pop_x + self.pop_w
            || y < list_y
            || y > list_y + self.list_visible_h()
        {
            return None;
        }
        let rel = y - list_y + self.list_offset;
        if rel < 0.0 {
            return None;
        }
        let row = (rel / self.list_stride()).floor() as usize;
        if row >= self.popup_rows() {
            return None;
        }
        let within = rel - row as f32 * self.list_stride();
        if within > DATE_LIST_ROW_H {
            return None;
        }
        Some(row)
    }

    fn cell_at(&self, x: f32, y: f32) -> Option<u32> {
        if x < self.grid_x
            || x > self.grid_x + DATE_CELL_W * 7.0
            || y < self.grid_y
            || y > self.grid_y + DATE_CELL_H * 6.0
        {
            return None;
        }
        let col = ((x - self.grid_x) / DATE_CELL_W).floor() as u32;
        let row = ((y - self.grid_y) / DATE_CELL_H).floor() as u32;
        let first = first_weekday(self.view_year, self.view_month);
        let days = days_in_month(self.view_year, self.view_month);
        let index = row * 7 + col;
        if index < first || index >= first + days {
            return None;
        }
        Some(index - first + 1)
    }

    fn nav_hit(&self, x: f32, y: f32) -> Option<i32> {
        if y < self.title_y || y > self.title_y + DATE_HEADER_H {
            return None;
        }
        let right = self.panel_x + self.panel_w - DATE_PAD;
        if x >= right - DATE_NAV_W * 2.0 && x < right - DATE_NAV_W {
            Some(-1)
        } else if x >= right - DATE_NAV_W && x <= right {
            Some(1)
        } else {
            None
        }
    }

    fn zone_at(&self, x: f32, y: f32) -> Option<HeaderZone> {
        if y < self.title_y || y > self.title_y + DATE_HEADER_H {
            return None;
        }
        if x >= self.month_x && x <= self.month_x + self.month_w {
            Some(HeaderZone::Month)
        } else if x >= self.year_x && x <= self.year_x + self.year_w {
            Some(HeaderZone::Year)
        } else {
            None
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if self.popup != Popup::None {
            // Open popup: arm the row under the pointer, if any.
            self.armed = None;
            self.armed_row = self.pop_row_at(x, y);
            return;
        }
        if let Some(day) = self.cell_at(x, y) {
            self.armed = Some(Armed::Day(day));
        } else if let Some(delta) = self.nav_hit(x, y) {
            self.armed = Some(Armed::Nav(delta));
        } else if let Some(zone) = self.zone_at(x, y) {
            self.armed = Some(Armed::Header(zone));
        } else {
            self.armed = None;
        }
    }

    /// Hover in accent color: popup rows when open, otherwise the
    /// day cell, header zone and nav chevron under the pointer.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if self.popup != Popup::None {
            self.hovered_row = self.pop_row_at(x, y);
            self.hovered_day = None;
            self.hovered_zone = None;
            self.hovered_nav = None;
        } else {
            self.hovered_row = None;
            self.hovered_day = self.cell_at(x, y);
            self.hovered_zone = self.zone_at(x, y);
            self.hovered_nav = self.nav_hit(x, y);
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn finish_up(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if self.popup != Popup::None {
            let armed = self.armed_row.take();
            if let Some(row) = armed {
                // Press + release on the same row picks it.
                if self.pop_row_at(x, y) == Some(row) {
                    match self.popup {
                        Popup::Months => {
                            self.view_month = (row + 1) as u32;
                        }
                        Popup::Years => {
                            self.view_year = DATE_YEAR_MIN + row as i32;
                        }
                        Popup::None => {}
                    }
                    self.close_popup();
                    return;
                }
            }
            // Header taps switch lists, everything else closes.
            match self.zone_at(x, y) {
                Some(HeaderZone::Month) => self.open_months(),
                Some(HeaderZone::Year) => self.open_years(),
                None => self.close_popup(),
            }
            return;
        }
        match self.armed.take() {
            Some(Armed::Day(day)) => {
                if self.cell_at(x, y) == Some(day) {
                    self.set_selected(self.view_year, self.view_month, day);
                }
            }
            Some(Armed::Nav(delta)) => {
                if self.nav_hit(x, y) == Some(delta) {
                    self.step_month(delta);
                }
            }
            Some(Armed::Header(zone)) => {
                if self.zone_at(x, y) == Some(zone) {
                    match zone {
                        HeaderZone::Month => self.open_months(),
                        HeaderZone::Year => self.open_years(),
                    }
                }
            }
            None => {}
        }
    }

    /// Scroll the open popup list (`dy` in logical px, down positive).
    pub fn mouse_wheel(&mut self, _dx: f64, dy: f64) {
        if self.popup == Popup::None || self.disabled {
            return;
        }
        self.list_offset = (self.list_offset - dy as f32).clamp(0.0, self.max_offset());
    }

    pub fn key(&mut self, key: Key) {
        if self.disabled {
            return;
        }
        if key == Key::Escape {
            self.close_popup();
        }
    }

    fn cell_center(&self, day: u32) -> Option<(f32, f32)> {
        let first = first_weekday(self.view_year, self.view_month);
        let index = first + day - 1;
        let col = index % 7;
        let row = index / 7;
        Some((
            self.grid_x + col as f32 * DATE_CELL_W + DATE_CELL_W / 2.0,
            self.grid_y + row as f32 * DATE_CELL_H + DATE_CELL_H / 2.0,
        ))
    }

    fn draw_chevron(&self, scene: &mut Scene, cx: f32, cy: f32, flip: bool, scale: f32, color: Color) {
        let px = |v: f32| v as f64 * scale as f64;
        let (w, h) = (5.0, 8.0);
        let x0 = if flip { cx + w / 2.0 } else { cx - w / 2.0 };
        let x1 = if flip { cx - w / 2.0 } else { cx + w / 2.0 };
        let mut path = BezPath::new();
        path.move_to((px(x0), px(cy - h / 2.0)));
        path.line_to((px(x1), px(cy)));
        path.line_to((px(x0), px(cy + h / 2.0)));
        let mut stroke = Stroke::new(2.0 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }
}

impl Default for DatePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl View for DatePicker {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (
            self.panel_content_w() + DATE_PAD * 2.0,
            self.panel_content_h() + DATE_PAD * 2.0,
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        self.layout_panel(fonts);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        if images.is_capture_pass() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        self.layout_panel(fonts);

        // Frosted glass panel (opaque finish, no clear background).
        let rect = vello::kurbo::Rect::new(
            px(self.panel_x),
            px(self.panel_y),
            px(self.panel_x + self.panel_w),
            px(self.panel_y + self.panel_h),
        );
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            rect,
            Color::from_rgba8(0, 0, 0, 70),
            px(DATE_RADIUS),
            DATE_SHADOW_BLUR as f64 * scale,
        );
        self.glass.draw(scene, fonts, images);

        // Header: month and year menus (accent on hover) plus month
        // steppers.
        let month_label = DATE_MONTHS[(self.view_month - 1) as usize].to_string();
        let year_label = self.view_year.to_string();
        let month_color = if self.hovered_zone == Some(HeaderZone::Month) && !self.disabled {
            self.hover
        } else {
            self.text_color
        };
        let year_color = if self.hovered_zone == Some(HeaderZone::Year) && !self.disabled {
            self.hover
        } else {
            self.text_color
        };
        let month_layout = fonts.layout_text_weighted(
            &month_label,
            DATE_TITLE_SIZE,
            self.eff(month_color),
            600.0,
            None,
        );
        let year_layout = fonts.layout_text_weighted(
            &year_label,
            DATE_TITLE_SIZE,
            self.eff(year_color),
            600.0,
            None,
        );
        let (_, month_th) = FontSystem::layout_size(&month_layout);
        let (_, year_th) = FontSystem::layout_size(&year_layout);
        draw_layout(
            scene,
            &month_layout,
            self.month_x,
            self.title_y + (DATE_HEADER_H - month_th / fonts.scale) / 2.0,
            fonts.scale,
        );
        draw_layout(
            scene,
            &year_layout,
            self.year_x,
            self.title_y + (DATE_HEADER_H - year_th / fonts.scale) / 2.0,
            fonts.scale,
        );
        let right = self.panel_x + self.panel_w - DATE_PAD;
        let prev_color = if self.hovered_nav == Some(-1) && !self.disabled {
            self.hover
        } else {
            self.text_color
        };
        let next_color = if self.hovered_nav == Some(1) && !self.disabled {
            self.hover
        } else {
            self.text_color
        };
        self.draw_chevron(
            scene,
            right - DATE_NAV_W * 1.5,
            self.title_y + DATE_HEADER_H / 2.0,
            true,
            fonts.scale,
            self.eff(prev_color),
        );
        self.draw_chevron(
            scene,
            right - DATE_NAV_W * 0.5,
            self.title_y + DATE_HEADER_H / 2.0,
            false,
            fonts.scale,
            self.eff(next_color),
        );

        // Weekday header.
        for (i, name) in DATE_WEEKDAYS.iter().enumerate() {
            let layout = fonts.layout_text_weighted(
                name,
                DATE_WEEK_SIZE,
                self.eff(self.text_dim),
                600.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.grid_x + i as f32 * DATE_CELL_W + (DATE_CELL_W - tw / fonts.scale) / 2.0,
                self.title_y + DATE_HEADER_H + (DATE_WEEK_H - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }

        // Day cells.
        let days = days_in_month(self.view_year, self.view_month);
        for day in 1..=days {
            let Some((cx, cy)) = self.cell_center(day) else {
                continue;
            };
            let is_selected = self.selected == (self.view_year, self.view_month, day);
            let is_hovered = Some(day) == self.hovered_day && !self.disabled;
            if is_selected {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.selection)),
                    None,
                    &Circle::new((px(cx), px(cy)), px(DATE_SEL_R)),
                );
            } else if is_hovered {
                // Hovered day: accent circle, like the popup hover.
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.hover)),
                    None,
                    &Circle::new((px(cx), px(cy)), px(DATE_SEL_R)),
                );
            }
            // Contrast text on filled circles: the default
            // selection is the label color, so dark mode gets black
            // text and light mode white text; hover is always accent
            // with white text.
            let day_color = if is_hovered && !is_selected {
                Color::WHITE
            } else if is_selected {
                if self.selection_manual {
                    Color::WHITE
                } else if self.dark {
                    Color::BLACK
                } else {
                    Color::WHITE
                }
            } else {
                self.text_color
            };
            let layout = fonts.layout_text_weighted(
                &day.to_string(),
                DATE_DAY_SIZE,
                self.eff(day_color),
                400.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                cx - (tw / fonts.scale) / 2.0,
                cy - (th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }

        if self.popup != Popup::None {
            self.draw_popup(scene, fonts, images);
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl DatePicker {
    /// Open frosted popup list with hover rows, current checkmark and
    /// a scrollbar when content exceeds the visible rows.
    fn draw_popup(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let list_y = self.pop_y + DATE_LIST_PAD;
        let rect = vello::kurbo::Rect::new(
            px(self.pop_x),
            px(self.pop_y),
            px(self.pop_x + self.pop_w),
            px(self.pop_y + self.pop_h),
        );
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            rect,
            Color::from_rgba8(0, 0, 0, 70),
            px(DATE_LIST_RADIUS),
            DATE_SHADOW_BLUR as f64 * scale,
        );
        self.glass_pop.draw(scene, fonts, images);

        let rows = self.popup_rows();
        let visible = self.list_visible();
        let current = self.popup_current_row();
        let first_visible = (self.list_offset / self.list_stride()).floor() as usize;
        for i in 0..=visible {
            let row = first_visible + i;
            if row >= rows {
                break;
            }
            let ry = list_y + (row as f32 * self.list_stride() - self.list_offset);
            if ry + DATE_LIST_ROW_H < list_y || ry > list_y + self.list_visible_h() {
                continue;
            }
            let is_hovered = Some(row) == self.hovered_row && !self.disabled;
            if is_hovered {
                let hl = RoundedRect::new(
                    px(self.pop_x + DATE_LIST_PAD / 2.0),
                    px(ry),
                    px(self.pop_x + self.pop_w - DATE_LIST_PAD / 2.0 - DATE_SCROLL_W),
                    px(ry + DATE_LIST_ROW_H),
                    px(6.0),
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.hover)),
                    None,
                    &hl,
                );
            }
            let color = if is_hovered {
                Color::WHITE
            } else {
                self.text_color
            };
            if row == current {
                self.draw_list_check(
                    scene,
                    self.pop_x + DATE_LIST_PAD + (DATE_LIST_CHECK_COL - MENU_CHECK_W) / 2.0,
                    ry + (DATE_LIST_ROW_H - MENU_CHECK_H) / 2.0,
                    fonts.scale,
                    self.eff(color),
                );
            }
            let label = self.popup_row_label(row);
            let layout = fonts.layout_text_weighted(
                &label,
                DATE_LIST_SIZE,
                self.eff(color),
                400.0,
                None,
            );
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.pop_x + DATE_LIST_PAD + DATE_LIST_CHECK_COL + DATE_LIST_TEXT_GAP,
                ry + (DATE_LIST_ROW_H - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }

        // Scrollbar when content exceeds the visible rows.
        if rows > visible {
            let track_y = list_y;
            let track_h = self.list_visible_h();
            let thumb_h = (track_h * visible as f32 / rows as f32).max(12.0);
            let travel = (track_h - thumb_h).max(0.0);
            let thumb_y = track_y
                + if self.max_offset() > 0.0 {
                    self.list_offset / self.max_offset() * travel
                } else {
                    0.0
                };
            let thumb = RoundedRect::new(
                px(self.pop_x + self.pop_w - DATE_LIST_PAD - DATE_SCROLL_W),
                px(thumb_y),
                px(self.pop_x + self.pop_w - DATE_LIST_PAD),
                px(thumb_y + thumb_h),
                px(DATE_SCROLL_W / 2.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(self.text_dim)),
                None,
                &thumb,
            );
        }
    }

    fn draw_list_check(&self, scene: &mut Scene, x: f32, y: f32, scale: f32, color: Color) {
        // Fixed-size checkmark, independent of sizes.
        let px = |v: f32| v as f64 * scale as f64;
        let mut path = BezPath::new();
        path.move_to((px(x), px(y + MENU_CHECK_H * 0.55)));
        path.line_to((px(x + MENU_CHECK_W * 0.38), px(y + MENU_CHECK_H)));
        path.line_to((px(x + MENU_CHECK_W), px(y)));
        let mut stroke = Stroke::new(2.0 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_roundtrip() {
        for (y, m, d) in [(1970, 1, 1), (2000, 2, 29), (2026, 7, 16), (1999, 12, 31)] {
            let days = days_from_civil(y, m, d);
            assert_eq!(civil_from_days(days), (y, m, d));
        }
    }

    #[test]
    fn july_2026_starts_wednesday() {
        // Reference screenshot: the 1st sits under WED (Monday-first).
        assert_eq!(first_weekday(2026, 7), 2);
        assert_eq!(days_in_month(2026, 7), 31);
    }

    #[test]
    fn leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
    }

    #[test]
    fn step_month_wraps_years() {
        let mut p = DatePicker::new().selected(2026, 12, 15);
        p.step_month(1);
        assert_eq!(p.viewed(), (2027, 1));
        p.step_month(-2);
        assert_eq!(p.viewed(), (2026, 11));
    }

    #[test]
    fn click_day_selects_on_release_inside() {
        let mut p = DatePicker::new().selected(2026, 7, 1);
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        // Release without press does nothing.
        p.mouse_up(1000.0, 1000.0);
        assert_eq!(p.selected_date(), (2026, 7, 1));
        // Press + release on the 16th selects it.
        let (cx, cy) = p.cell_center(16).expect("16 visible");
        p.mouse_down(cx as f64, cy as f64);
        p.mouse_up(cx as f64, cy as f64);
        assert_eq!(p.selected_date(), (2026, 7, 16));
        // Press on a day, release outside keeps the selection.
        p.mouse_down(cx as f64, cy as f64);
        p.mouse_up(700.0, 500.0);
        assert_eq!(p.selected_date(), (2026, 7, 16));
    }

    #[test]
    fn blank_cells_never_select() {
        let mut p = DatePicker::new().selected(2026, 7, 1);
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        // Top-left cell is blank for July 2026 (starts Wednesday).
        let x = (p.grid_x + DATE_CELL_W / 2.0) as f64;
        let y = (p.grid_y + DATE_CELL_H / 2.0) as f64;
        p.mouse_down(x, y);
        p.mouse_up(x, y);
        assert_eq!(p.selected_date(), (2026, 7, 1));
    }

    fn placed_picker() -> (DatePicker, FontSystem) {
        let mut p = DatePicker::new().selected(2026, 7, 16);
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        (p, fonts)
    }

    fn click(p: &mut DatePicker, x: f64, y: f64) {
        p.mouse_down(x, y);
        p.mouse_up(x, y);
    }

    fn month_click(p: &mut DatePicker) {
        let x = (p.month_x + 4.0) as f64;
        let y = (p.title_y + DATE_HEADER_H / 2.0) as f64;
        click(p, x, y);
    }

    fn year_click(p: &mut DatePicker) {
        let x = (p.year_x + 4.0) as f64;
        let y = (p.title_y + DATE_HEADER_H / 2.0) as f64;
        click(p, x, y);
    }

    fn pop_row_point(p: &DatePicker, row: usize) -> (f64, f64) {
        let x = (p.pop_x + p.pop_w / 2.0) as f64;
        let y = (p.pop_y + DATE_LIST_PAD + row as f32 * p.list_stride() - p.list_offset
            + DATE_LIST_ROW_H / 2.0) as f64;
        (x, y)
    }

    #[test]
    fn month_menu_opens_and_picks() {
        let (mut p, _) = placed_picker();
        // Click the month zone opens all twelve months.
        month_click(&mut p);
        assert_eq!(p.popup, Popup::Months);
        assert_eq!(p.popup_rows(), 12);
        // Hover highlights, click picks March.
        let (rx, ry) = pop_row_point(&p, 2);
        p.mouse_move(rx, ry);
        assert_eq!(p.hovered_row, Some(2));
        click(&mut p, rx, ry);
        assert_eq!(p.popup, Popup::None);
        assert_eq!(p.viewed(), (2026, 3));
        // Selection untouched by month navigation.
        assert_eq!(p.selected_date(), (2026, 7, 16));
    }

    #[test]
    fn year_menu_shows_current_and_scrolls() {
        let (mut p, _) = placed_picker();
        // Click the year zone opens the 1-3000 list at 2026.
        year_click(&mut p);
        assert_eq!(p.popup, Popup::Years);
        assert_eq!(p.popup_rows(), 3000);
        assert_eq!(p.popup_current_row(), 2025);
        // Current year visible on open.
        let rel = 2025 as f32 * p.list_stride() - p.list_offset;
        assert!(rel >= 0.0 && rel <= p.list_visible_h());
        // Wheel scrolls and clamps at both ends.
        p.mouse_wheel(0.0, -100000.0);
        assert_eq!(p.list_offset, p.max_offset());
        p.mouse_wheel(0.0, 100000.0);
        assert_eq!(p.list_offset, 0.0);
        // Scroll to the top and pick year 1.
        let (rx, ry) = pop_row_point(&p, 0);
        click(&mut p, rx, ry);
        assert_eq!(p.popup, Popup::None);
        assert_eq!(p.viewed(), (1, 7));
    }

    #[test]
    fn popup_outside_click_and_escape_close() {
        let (mut p, _) = placed_picker();
        month_click(&mut p);
        assert_eq!(p.popup, Popup::Months);
        // Outside press + release closes without changing.
        p.mouse_down(700.0, 500.0);
        p.mouse_up(700.0, 500.0);
        assert_eq!(p.popup, Popup::None);
        assert_eq!(p.viewed(), (2026, 7));
        // Escape closes too.
        year_click(&mut p);
        assert_eq!(p.popup, Popup::Years);
        p.key(Key::Escape);
        assert_eq!(p.popup, Popup::None);
    }

    #[test]
    fn header_tap_switches_lists() {
        let (mut p, _) = placed_picker();
        month_click(&mut p);
        assert_eq!(p.popup, Popup::Months);
        // Tapping the year zone while open switches lists.
        year_click(&mut p);
        assert_eq!(p.popup, Popup::Years);
    }

    #[test]
    fn hover_uses_accent_everywhere() {
        let (mut p, _) = placed_picker();
        p.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        // Day, zone and nav hover all resolve to the accent.
        let (cx, cy) = p.cell_center(16).expect("16 visible");
        p.mouse_move(cx as f64, cy as f64);
        assert_eq!(p.hovered_day, Some(16));
        let mx = (p.month_x + 4.0) as f64;
        let my = (p.title_y + DATE_HEADER_H / 2.0) as f64;
        p.mouse_move(mx, my);
        assert_eq!(p.hovered_zone, Some(HeaderZone::Month));
        assert_eq!(p.hover, Color::from_rgb8(0xff, 0x2d, 0x55));
    }

    #[test]
    fn nav_chevrons_step_month() {
        let mut p = DatePicker::new().selected(2026, 7, 16);
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        let right = p.panel_x + p.panel_w - DATE_PAD;
        let prev = (right - DATE_NAV_W * 1.5) as f64;
        let next = (right - DATE_NAV_W * 0.5) as f64;
        let y = (p.title_y + DATE_HEADER_H / 2.0) as f64;
        p.mouse_down(next, y);
        p.mouse_up(next, y);
        assert_eq!(p.viewed(), (2026, 8));
        p.mouse_down(prev, y);
        p.mouse_up(prev, y);
        p.mouse_down(prev, y);
        p.mouse_up(prev, y);
        assert_eq!(p.viewed(), (2026, 6));
    }

    #[test]
    fn select_fires_callback_only_on_change() {
        use std::cell::Cell;
        use std::rc::Rc;
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut p = DatePicker::new()
            .selected(2026, 7, 16)
            .on_select(move |_, _, _| count.set(count.get() + 1));
        p.set_selected(2026, 7, 16);
        assert_eq!(fires.get(), 0);
        p.set_selected(2026, 7, 17);
        assert_eq!(p.selected_date(), (2026, 7, 17));
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn set_selected_clamps_day() {
        let mut p = DatePicker::new();
        p.set_selected(2026, 2, 30);
        assert_eq!(p.selected_date(), (2026, 2, 28));
    }
}
