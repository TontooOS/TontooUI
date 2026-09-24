use std::any::Any;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Circle, Join, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
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
/// Editing field text size in logical px.
pub const DATE_EDIT_SIZE: f32 = 13.0;
/// Selected-day circle radius in logical px.
pub const DATE_SEL_R: f32 = 12.0;
/// Month nav button hit width in logical px.
pub const DATE_NAV_W: f32 = 28.0;
/// Gap between title text and edit chevron in logical px.
pub const DATE_CHEV_GAP: f32 = 6.0;
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

/// Parse a month token: full/short English name or 1-12.
fn parse_month(token: &str) -> Option<u32> {
    let lower = token.to_lowercase();
    for (i, name) in DATE_MONTHS.iter().enumerate() {
        if *name.to_lowercase() == lower || name[..3].to_lowercase() == lower {
            return Some(i as u32 + 1);
        }
    }
    token.parse::<u32>().ok().filter(|m| (1..=12).contains(m))
}

/// Parse "July 2026", "jul 2026", "7/2026" or "7 2026".
pub fn parse_month_year(input: &str) -> Option<(i32, u32)> {
    let cleaned = input.trim().replace(['/', '-', '.'], " ");
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    let month = parse_month(parts[0])?;
    let year: i32 = parts[1].parse().ok()?;
    if !(1900..=2100).contains(&year) {
        return None;
    }
    Some((year, month))
}

/// Date picker: frosted glass calendar with a header bar. The header
/// shows "Month Year" plus an edit chevron and `<`/`>` month steppers.
/// Clicking the title turns it into a text input (Enter commits,
/// Escape cancels); the steppers move one month. Clicking a day
/// selects it with a filled circle, like the reference.
///
/// The panel is always clamped into the `set_viewport` bounds so the
/// glass never samples outside the window. Apps must call
/// `set_viewport` every frame and forward `text`/`key` (see
/// `examples/date.rs`).
pub struct DatePicker {
    view_year: i32,
    view_month: u32,
    selected: (i32, u32, u32),
    editing: bool,
    buffer: String,
    cursor: usize,
    selection: Color,
    selection_manual: bool,
    dark: bool,
    text_color: Color,
    text_dim: Color,
    glass: GlassContainer,
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
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
    armed_day: Option<u32>,
    armed_nav: Option<i32>,
    armed_title: bool,
    t0: Instant,
    disabled: bool,
    focused: bool,
    on_select: Option<Box<dyn FnMut(i32, u32, u32)>>,
}

impl DatePicker {
    pub fn new() -> Self {
        let (y, m, d) = today();
        let mut glass = GlassContainer::new();
        glass.set_glass_type(GlassType::Frosted);
        Self {
            view_year: y,
            view_month: m,
            selected: (y, m, d),
            editing: false,
            buffer: String::new(),
            cursor: 0,
            selection: Color::WHITE,
            selection_manual: false,
            dark: true,
            text_color: Color::WHITE,
            text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            glass,
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
            vp_x: 0.0,
            vp_y: 0.0,
            vp_w: f32::MAX,
            vp_h: f32::MAX,
            armed_day: None,
            armed_nav: None,
            armed_title: false,
            t0: Instant::now(),
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

    /// Live theme: label colors and mode. The selection circle keeps
    /// the label color (black in light mode, white in dark) unless
    /// set manually with `accent`.
    pub fn set_theme(&mut self, _accent: Color, dark: bool) {
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

    /// Glass stage for the calendar panel (frost tint per setting).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.glass.set_theme(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
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

    pub fn is_editing(&self) -> bool {
        self.editing
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

    fn title_text(&self) -> String {
        format!("{} {}", DATE_MONTHS[(self.view_month - 1) as usize], self.view_year)
    }

    fn layout_panel(&mut self, _fonts: &mut FontSystem) {
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

    fn field_hit(&self, x: f32, y: f32) -> bool {
        x >= self.title_x
            && x <= self.title_x + self.title_w
            && y >= self.title_y
            && y <= self.title_y + DATE_HEADER_H
    }

    fn title_hit(&self, x: f32, y: f32) -> bool {
        !self.editing && self.field_hit(x, y)
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

    fn start_editing(&mut self) {
        self.editing = true;
        self.buffer = self.title_text();
        self.cursor = self.buffer.len();
    }

    fn commit_edit(&mut self) {
        self.editing = false;
        if let Some((year, month)) = parse_month_year(&self.buffer) {
            self.view_year = year;
            self.view_month = month;
        }
        self.buffer.clear();
        self.cursor = 0;
    }

    fn cancel_edit(&mut self) {
        self.editing = false;
        self.buffer.clear();
        self.cursor = 0;
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        // Clicking anywhere outside the title field commits typing.
        if self.editing && !self.field_hit(x, y) {
            self.commit_edit();
        }
        if let Some(day) = self.cell_at(x, y) {
            self.armed_day = Some(day);
            self.armed_nav = None;
            self.armed_title = false;
        } else if let Some(delta) = self.nav_hit(x, y) {
            self.armed_nav = Some(delta);
            self.armed_day = None;
            self.armed_title = false;
        } else if self.title_hit(x, y) {
            self.armed_title = true;
            self.armed_day = None;
            self.armed_nav = None;
        }
    }

    pub fn mouse_move(&mut self, _x: f64, _y: f64) {}

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn finish_up(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if let Some(day) = self.armed_day.take() {
            if self.cell_at(x, y) == Some(day) {
                self.set_selected(self.view_year, self.view_month, day);
            }
        }
        if let Some(delta) = self.armed_nav.take() {
            if self.nav_hit(x, y) == Some(delta) {
                self.step_month(delta);
            }
        }
        if self.armed_title {
            self.armed_title = false;
            if self.title_hit(x, y) {
                self.start_editing();
            }
        }
    }

    /// Typed text while the title field is open. Caps at 16 chars so
    /// the field never overflows the header.
    pub fn text(&mut self, input: &str) {
        if !self.editing || self.disabled {
            return;
        }
        for ch in input.chars() {
            if self.buffer.len() >= 16 {
                break;
            }
            self.buffer.insert(self.cursor, ch);
            self.cursor += ch.len_utf8();
        }
    }

    pub fn key(&mut self, key: Key) {
        if !self.editing || self.disabled {
            return;
        }
        match key {
            Key::Backspace => {
                if self.cursor > 0 {
                    // Delete the char before the byte cursor.
                    let before = self.buffer[..self.cursor].chars().next_back();
                    if let Some(ch) = before {
                        self.cursor -= ch.len_utf8();
                        self.buffer.remove(self.cursor);
                    }
                }
            }
            Key::Left => {
                if self.cursor > 0 {
                    let before = self.buffer[..self.cursor].chars().next_back();
                    if let Some(ch) = before {
                        self.cursor -= ch.len_utf8();
                    }
                }
            }
            Key::Right => {
                if self.cursor < self.buffer.len() {
                    let next = self.buffer[self.cursor..].chars().next();
                    if let Some(ch) = next {
                        self.cursor += ch.len_utf8();
                    }
                }
            }
            Key::Enter => self.commit_edit(),
            Key::Escape => self.cancel_edit(),
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

        // Header: title (or editing field) plus month steppers.
        if self.editing {
            let field = RoundedRect::new(
                px(self.title_x),
                px(self.title_y + (DATE_HEADER_H - 24.0) / 2.0),
                px(self.title_x + self.title_w),
                px(self.title_y + (DATE_HEADER_H + 24.0) / 2.0),
                px(6.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(if self.dark {
                    Color::from_rgb8(0x3a, 0x3a, 0x3c)
                } else {
                    Color::from_rgb8(0xe5, 0xe5, 0xe5)
                })),
                None,
                &field,
            );
            let layout = fonts.layout_text_weighted(
                &self.buffer,
                DATE_EDIT_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            let (_, th) = FontSystem::layout_size(&layout);
            let tx = self.title_x + 8.0;
            let ty = self.title_y + (DATE_HEADER_H - th / fonts.scale) / 2.0;
            draw_layout(scene, &layout, tx, ty, fonts.scale);
            // Caret, blinking twice a second.
            if self.t0.elapsed().as_millis() % 1000 < 500 {
                let prefix = &self.buffer[..self.cursor.min(self.buffer.len())];
                let caret = fonts.layout_text(prefix, DATE_EDIT_SIZE, Color::WHITE, None);
                let (pw, _) = FontSystem::layout_size(&caret);
                let cx = tx + pw / fonts.scale;
                let line = vello::kurbo::Line::new((px(cx), px(ty)), (px(cx), px(ty + th / fonts.scale)));
                scene.stroke(
                    &Stroke::new(1.5 * scale),
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.text_color)),
                    None,
                    &line,
                );
            }
        } else {
            let layout = fonts.layout_text_weighted(
                &self.title_text(),
                DATE_TITLE_SIZE,
                self.eff(self.text_color),
                600.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            let ty = self.title_y + (DATE_HEADER_H - th / fonts.scale) / 2.0;
            draw_layout(scene, &layout, self.title_x, ty, fonts.scale);
            // Blue edit chevron after the year, like the reference.
            let ccx = self.title_x + tw / fonts.scale + DATE_CHEV_GAP + 2.5;
            let ccy = self.title_y + DATE_HEADER_H / 2.0;
            self.draw_chevron(scene, ccx, ccy, false, fonts.scale, self.eff(Color::from_rgb8(0x00, 0x7a, 0xff)));
        }
        let right = self.panel_x + self.panel_w - DATE_PAD;
        self.draw_chevron(
            scene,
            right - DATE_NAV_W * 1.5,
            self.title_y + DATE_HEADER_H / 2.0,
            true,
            fonts.scale,
            self.eff(self.text_color),
        );
        self.draw_chevron(
            scene,
            right - DATE_NAV_W * 0.5,
            self.title_y + DATE_HEADER_H / 2.0,
            false,
            fonts.scale,
            self.eff(self.text_color),
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
            if is_selected {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.selection)),
                    None,
                    &Circle::new((px(cx), px(cy)), px(DATE_SEL_R)),
                );
            }
            // Contrast text on the filled circle: the default
            // selection is the label color, so dark mode gets black
            // text and light mode white text.
            let day_color = if is_selected {
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
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
    fn parse_month_year_formats() {
        assert_eq!(parse_month_year("July 2026"), Some((2026, 7)));
        assert_eq!(parse_month_year("jul 2026"), Some((2026, 7)));
        assert_eq!(parse_month_year("7/2026"), Some((2026, 7)));
        assert_eq!(parse_month_year("  12 2030 "), Some((2030, 12)));
        assert_eq!(parse_month_year("July"), None);
        assert_eq!(parse_month_year("13/2026"), None);
        assert_eq!(parse_month_year("July 1800"), None);
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

    #[test]
    fn title_click_edits_and_enter_commits() {
        let mut p = DatePicker::new().selected(2026, 7, 16);
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        let tx = (p.title_x + 10.0) as f64;
        let ty = (p.title_y + DATE_HEADER_H / 2.0) as f64;
        p.mouse_down(tx, ty);
        p.mouse_up(tx, ty);
        assert!(p.is_editing());
        // Replace the buffer with March 2027.
        p.buffer.clear();
        p.cursor = 0;
        p.text("March 2027");
        p.key(Key::Enter);
        assert!(!p.is_editing());
        assert_eq!(p.viewed(), (2027, 3));
        // Selection untouched by month navigation.
        assert_eq!(p.selected_date(), (2026, 7, 16));
    }

    #[test]
    fn escape_cancels_editing() {
        let mut p = DatePicker::new().selected(2026, 7, 16);
        let mut fonts = FontSystem::new();
        p.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        let tx = (p.title_x + 10.0) as f64;
        let ty = (p.title_y + DATE_HEADER_H / 2.0) as f64;
        p.mouse_down(tx, ty);
        p.mouse_up(tx, ty);
        p.text("garbage!!!");
        p.key(Key::Escape);
        assert!(!p.is_editing());
        assert_eq!(p.viewed(), (2026, 7));
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
