pub mod basic;

pub use basic::{AlertAction, AlertButton, BasicAlert};

use vello::peniko::Color;

/// Card width in logical px.
pub const ALERT_WIDTH: f32 = 420.0;
/// Card corner radius in logical px.
pub const ALERT_RADIUS: f32 = 24.0;
/// Inner padding in logical px.
pub const ALERT_PAD: f32 = 24.0;
/// Title size in logical px (semibold, centered).
pub const ALERT_TITLE_SIZE: f32 = 17.0;
/// Message size in logical px (regular, centered).
pub const ALERT_MESSAGE_SIZE: f32 = 15.0;
/// Gap between title and message in logical px.
pub const ALERT_TITLE_GAP: f32 = 8.0;
/// Gap between message and buttons in logical px.
pub const ALERT_MESSAGE_GAP: f32 = 20.0;
/// Action button height in logical px.
pub const ALERT_BUTTON_H: f32 = 44.0;
/// Gap between two action buttons in logical px.
pub const ALERT_BUTTON_GAP: f32 = 12.0;
/// Fade in/out time in seconds (engine tween).
pub const ALERT_FADE_SECONDS: f32 = 0.25;
/// Dim alpha over the app behind the alert (0..255).
pub const ALERT_DIM_ALPHA: u8 = 77;
/// Title text on dark frost.
pub const ALERT_TITLE_DARK: Color = Color::WHITE;
/// Title text on light frost.
pub const ALERT_TITLE_LIGHT: Color = Color::from_rgb8(0x27, 0x27, 0x27);
/// Message text on dark frost.
pub const ALERT_MESSAGE_DARK: Color = Color::from_rgba8(255, 255, 255, 220);
/// Message text on light frost.
pub const ALERT_MESSAGE_LIGHT: Color = Color::from_rgba8(0x27, 0x27, 0x27, 220);
/// OK button fill (system blue accent).
pub const ALERT_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
