pub mod action;
pub mod basic;
pub mod confirm;

pub use action::{ActionAlert, AlertEvent};
pub use basic::{AlertAction, AlertButton, BasicAlert};
pub use confirm::ConfirmationDialog;

use vello::peniko::Color;

use crate::animation::{Easing, Tween};

/// Card width in logical px (middle of compact 210 and original 420).
pub const ALERT_WIDTH: f32 = 315.0;
/// Card corner radius in logical px.
pub const ALERT_RADIUS: f32 = 18.0;
/// Inner padding in logical px.
pub const ALERT_PAD: f32 = 18.0;
/// Title size in logical px (semibold).
pub const ALERT_TITLE_SIZE: f32 = 12.75;
/// Message size in logical px (regular).
pub const ALERT_MESSAGE_SIZE: f32 = 11.25;
/// Gap between title and message in logical px.
pub const ALERT_TITLE_GAP: f32 = 6.0;
/// Gap between message and buttons in logical px.
pub const ALERT_MESSAGE_GAP: f32 = 15.0;
/// Action button height in logical px.
pub const ALERT_BUTTON_H: f32 = 33.0;
/// Gap between two action buttons in logical px.
pub const ALERT_BUTTON_GAP: f32 = 9.0;
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
/// Cancel button tint (system gray, translucent style).
pub const ALERT_CANCEL: Color = Color::from_rgb8(0x8e, 0x8e, 0x93);

/// Entrance fade sample at `elapsed` seconds (engine tween, shared
/// by both alert variants so their timing always matches).
pub(crate) fn fade_sample(elapsed: f32) -> f32 {
    Tween::new(0.0_f32, 1.0, ALERT_FADE_SECONDS)
        .easing(Easing::CubicOut)
        .sample(elapsed)
        .0
}
