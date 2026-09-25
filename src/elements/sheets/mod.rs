pub mod basic;

pub use basic::{BasicSheet, SheetSize};

use vello::peniko::Color;

/// Card corner radius in logical px.
pub const SHEET_RADIUS: f32 = 20.0;
/// Fade in/out time in seconds (engine tween).
pub const SHEET_FADE_SECONDS: f32 = 0.25;
/// Dim alpha over the app behind the sheet (0..255).
pub const SHEET_DIM_ALPHA: u8 = 77;
/// Default card fill (dark mode).
pub const SHEET_BG_DARK: Color = Color::from_rgb8(0x2c, 0x2c, 0x2e);
/// Default card fill (light mode).
pub const SHEET_BG_LIGHT: Color = Color::from_rgb8(0xff, 0xff, 0xff);
/// Bordered button fill on a dark card: darker than the card so
/// buttons never melt into the sheet.
pub const SHEET_BUTTON_BG_DARK: Color = Color::from_rgb8(0x1e, 0x20, 0x22);
/// Card border (dark mode).
pub const SHEET_BORDER_DARK: Color = Color::from_rgba8(255, 255, 255, 36);
/// Card border (light mode).
pub const SHEET_BORDER_LIGHT: Color = Color::from_rgba8(0, 0, 0, 31);
/// Drop shadow color under the card.
pub const SHEET_SHADOW: Color = Color::from_rgba8(0, 0, 0, 64);
/// Drop shadow blur in logical px.
pub const SHEET_SHADOW_BLUR: f32 = 16.0;
/// Drop shadow y offset in logical px.
pub const SHEET_SHADOW_DY: f32 = 4.0;
