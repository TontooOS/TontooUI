//! Text Format — initializer for non-string formatted Text.
//!
//! SwiftUI `Text` has initializers that take a format style:
//! `Text(1_234.56, format: .currency(code: "USD"))`,
//! `Text(0.874, format: .percent)`, `Text(Measurement(...))`, `Text(Date(...))`, etc.
//! This file ports that to TontooUI. Rendering is directly on the window
//! background (#1d1d1d dark / #ececec light) with SF Pro, no extra card.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

// ── helpers for formatting ───────────────────────────────────────────────

fn format_currency_usd(value: f64) -> String {
    // $1,234.56
    // simple grouping for demo; not a full locale engine
    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    let s = format!("{:.2}", abs);
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let frac = parts.get(1).unwrap_or(&"00");
    let grouped = group_int(int_part);
    format!("{}${}.{}", sign, grouped, frac)
}

fn format_percent(value: f64, decimals: usize) -> String {
    format!("{:.*}%", decimals, value * 100.0)
}

fn format_number(value: f64, decimals: usize) -> String {
    format!("{:.*}", decimals, value)
}

fn group_int(s: &str) -> String {
    let mut out = String::new();
    let mut count = 0;
    for ch in s.chars().rev() {
        if count == 3 {
            out.push(',');
            count = 0;
        }
        out.push(ch);
        count += 1;
    }
    out.chars().rev().collect()
}

fn format_integer_grouped(value: i64) -> String {
    let s = value.abs().to_string();
    let g = group_int(&s);
    if value < 0 { format!("-{}", g) } else { g }
}

fn format_scientific(value: f64) -> String {
    format!("{:.3E}", value)
}

fn format_measurement(value: f64, unit: &str, spaced: bool) -> String {
    if spaced {
        format!("{} {}", strip_trailing_zeros(value), unit)
    } else {
        format!("{}{}", strip_trailing_zeros(value), unit)
    }
}

fn strip_trailing_zeros(v: f64) -> String {
    let s = format!("{:.6}", v);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() { "0".into() } else { trimmed.into() }
}

fn format_temperature_c(value: f64) -> String {
    // 25.5°C
    format!("{}°C", strip_trailing_zeros(value))
}

fn format_temperature_words(value: f64) -> String {
    format!("{} degrees Celsius", strip_trailing_zeros(value))
}

fn format_time(h: u8, m: u8, s: u8, pm: bool) -> String {
    format!("{h}:{m:02}:{s:02} {}", if pm { "PM" } else { "AM" })
}

fn format_date_month_day_year(month: u8, day: u8, year: i32) -> String {
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let mon = months[(month as usize).saturating_sub(1).min(11)];
    format!("{mon} {day}, {year}")
}

fn format_date_numeric(m: u8, d: u8) -> String {
    format!("{m}/{d:02}")
}

fn format_duration(min: u32, sec: u32) -> String {
    format!("{min}:{sec:02}")
}

// ── Format enum for programmatic use ─────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TextFormatKind {
    CurrencyUsd(f64),
    Percent(f64),
    Number(f64),
    IntegerGrouped(i64),
    Scientific(f64),
    Measurement { value: f64, unit: String },
    WeightKg(f64),
    TemperatureC(f64),
    TemperatureWords(f64),
    Time { h: u8, m: u8, s: u8, pm: bool },
    Date { year: i32, month: u8, day: u8 },
    DateNumeric { month: u8, day: u8 },
    Duration { minutes: u32, seconds: u32 },
    Custom(String),
}

impl TextFormatKind {
    pub fn formatted(&self) -> String {
        match self {
            Self::CurrencyUsd(v) => format_currency_usd(*v),
            Self::Percent(v) => format_percent(*v, 1),
            Self::Number(v) => format_number(*v, 2),
            Self::IntegerGrouped(v) => format_integer_grouped(*v),
            Self::Scientific(v) => format_scientific(*v),
            Self::Measurement { value, unit } => format_measurement(*value, unit, true),
            Self::WeightKg(v) => format_measurement(*v, "kg", false),
            Self::TemperatureC(v) => format_temperature_c(*v),
            Self::TemperatureWords(v) => format_temperature_words(*v),
            Self::Time { h, m, s, pm } => format_time(*h, *m, *s, *pm),
            Self::Date { year, month, day } => format_date_month_day_year(*month, *day, *year),
            Self::DateNumeric { month, day } => format_date_numeric(*month, *day),
            Self::Duration { minutes, seconds } => format_duration(*minutes, *seconds),
            Self::Custom(s) => s.clone(),
        }
    }
}

// ── TextFormat widget — single value + palette preview ────────────────────

/// Text Format — creates a text view that displays the formatted representation
/// of a non-string type (currency, percent, measurement, date, etc.).
///
/// The screenshot shows 11 formatted values across 3 rows:
/// `$1,234.56  87.4%  3.14  8,976,543  6.022E23 / 123 meters  70kg  25.5°C  -30.1 degrees Celsius / 9:25:39 PM  Jul 21, 2025  1/03  1:31`
///
/// `TextFormat` can render a single formatted value (`TextFormat::currency(1234.56)`),
/// or as a palette (`TextFormat::demo()`) that shows all values directly on the
/// window background with no extra card — SF Pro, adaptive colors.
pub struct TextFormat {
    id: WidgetId,
    kind: TextFormatKind,
    font_size: f32,
    is_palette: bool,
    position_mode: PositionMode,
    position: Position,
}

impl TextFormat {
    pub fn new(kind: TextFormatKind) -> Self {
        Self {
            id: next_widget_id(),
            kind,
            font_size: 11.0,
            is_palette: false,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn custom(text: impl Into<String>) -> Self {
        Self::new(TextFormatKind::Custom(text.into()))
    }

    // ── convenience ctors (initializer-style) ──
    pub fn currency(value: f64) -> Self { Self::new(TextFormatKind::CurrencyUsd(value)) }
    pub fn percent(value: f64) -> Self { Self::new(TextFormatKind::Percent(value)) }
    pub fn number(value: f64) -> Self { Self::new(TextFormatKind::Number(value)) }
    pub fn integer_grouped(value: i64) -> Self { Self::new(TextFormatKind::IntegerGrouped(value)) }
    pub fn scientific(value: f64) -> Self { Self::new(TextFormatKind::Scientific(value)) }
    pub fn measurement(value: f64, unit: impl Into<String>) -> Self {
        Self::new(TextFormatKind::Measurement { value, unit: unit.into() })
    }
    pub fn weight_kg(value: f64) -> Self { Self::new(TextFormatKind::WeightKg(value)) }
    pub fn temperature_c(value: f64) -> Self { Self::new(TextFormatKind::TemperatureC(value)) }
    pub fn temperature_words(value: f64) -> Self { Self::new(TextFormatKind::TemperatureWords(value)) }
    pub fn time(h: u8, m: u8, s: u8, pm: bool) -> Self { Self::new(TextFormatKind::Time { h, m, s, pm }) }
    pub fn date(year: i32, month: u8, day: u8) -> Self { Self::new(TextFormatKind::Date { year, month, day }) }
    pub fn date_numeric(month: u8, day: u8) -> Self { Self::new(TextFormatKind::DateNumeric { month, day }) }
    pub fn duration(minutes: u32, seconds: u32) -> Self { Self::new(TextFormatKind::Duration { minutes, seconds }) }

    /// Palette preview that shows all 11 formatted values in 3 rows directly on window.
    pub fn demo() -> Self {
        let mut s = Self::new(TextFormatKind::Custom(String::new()));
        s.is_palette = true;
        s
    }

    pub fn font_size(mut self, size: f32) -> Self { self.font_size = size; self }
    pub fn formatted(&self) -> String { self.kind.formatted() }
    pub fn kind(&self) -> &TextFormatKind { &self.kind }

    pub fn to_view(self) -> View {
        if self.is_palette {
            View::new(self).with_frame(0.0, 0.0, 280.0, 74.0)
        } else {
            let w = (self.formatted().len() as f32 * self.font_size * 0.6 + 12.0).max(40.0);
            View::new(self).with_frame(0.0, 0.0, w, 18.0)
        }
    }
}

impl Default for TextFormat {
    fn default() -> Self { Self::currency(1234.56) }
}

impl ViewContent for TextFormat {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let text_color = if is_dark { Color::from_rgb(255,255,255) } else { Color::from_hex("#1d1d1d").unwrap() };
        let text_css = format!("rgba({},{},{},1.0)", (text_color.r*255.0) as u8, (text_color.g*255.0) as u8, (text_color.b*255.0) as u8);

        if self.is_palette {
            // Directly on window — no extra background card
            let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
            outer.set_halign(gtk::Align::Center);
            outer.set_valign(gtk::Align::Center);
            if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 74); }

            let rows: Vec<Vec<String>> = vec![
                vec![
                    format_currency_usd(1234.56),
                    format_percent(0.874, 1),
                    format_number(3.14, 2),
                    format_integer_grouped(8_976_543),
                    format_scientific(6.022e23),
                ],
                vec![
                    format_measurement(123.0, "meters", true),
                    format_measurement(70.0, "kg", false),
                    format_temperature_c(25.5),
                    format_temperature_words(-30.1),
                ],
                vec![
                    format_time(9, 25, 39, true),
                    format_date_month_day_year(7, 21, 2025),
                    format_date_numeric(1, 3),
                    format_duration(1, 31),
                ],
            ];

            for row_vals in rows {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                row.set_halign(gtk::Align::Center);
                for val in row_vals {
                    let lbl = gtk::Label::new(Some(&val));
                    lbl.add_css_class("tf-lbl");
                    // no background, SF Pro, adaptive color
                    uikit::widget::apply_css(&lbl, &format!(
                        ".tf-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}",
                        text_css
                    ));
                    lbl.set_halign(gtk::Align::Center);
                    row.append(&lbl);
                }
                outer.append(&row);
            }
            return outer.upcast();
        }

        // Single value
        let label = gtk::Label::new(Some(&self.formatted()));
        label.set_halign(gtk::Align::Start);
        label.set_valign(gtk::Align::Center);
        label.add_css_class("tf-single");
        uikit::widget::apply_css(&label, &format!(
            ".tf-single {{ color: {}; font-family: 'SF Pro Display'; font-size: {}px; }}",
            text_css, self.font_size as i32
        ));
        if frame.width > 0.0 { label.set_width_request(frame.width as i32); }
        label.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        if self.is_palette { Size::new(280.0, 74.0) }
        else {
            let w = self.formatted().len() as f32 * self.font_size * 0.6 + 12.0;
            Size::new(w, 18.0)
        }
    }
}

impl Widget for TextFormat {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 200.0, 18.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn currency_format() {
        assert_eq!(format_currency_usd(1234.56), "$1,234.56");
        assert_eq!(TextFormat::currency(1234.56).formatted(), "$1,234.56");
    }
    #[test]
    fn percent_and_number() {
        assert_eq!(format_percent(0.874, 1), "87.4%");
        assert_eq!(format_integer_grouped(8976543), "8,976,543");
        assert!(format_scientific(6.022e23).contains('E'));
    }
    #[test]
    fn measurement_and_dates() {
        assert_eq!(format_measurement(123.0, "meters", true), "123 meters");
        assert_eq!(format_temperature_c(25.5), "25.5°C");
        assert_eq!(format_time(9,25,39,true), "9:25:39 PM");
        assert_eq!(format_date_month_day_year(7,21,2025), "Jul 21, 2025");
    }
    #[test]
    fn palette_font() {
        let tf = TextFormat::demo();
        assert!(tf.is_palette);
        assert_eq!(tf.kind, TextFormatKind::Custom(String::new()));
    }
}
