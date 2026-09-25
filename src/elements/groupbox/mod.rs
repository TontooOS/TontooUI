pub mod basic;

pub use basic::BasicGroupBox;

use vello::peniko::Color;

/// Fill in dark mode: slightly lighter than the `#1B2022` background.
pub const GROUP_BG_DARK: Color = Color::from_rgb8(0x27, 0x2d, 0x30);
/// Fill in light mode: slightly darker than the `#FFFFFF` background
/// (white cards need shade to read as raised).
pub const GROUP_BG_LIGHT: Color = Color::from_rgb8(0xf2, 0xf2, 0xf5);
/// Corner radius in logical px.
pub const GROUP_RADIUS: f32 = 12.0;
/// Inner padding in logical px.
pub const GROUP_PAD: f32 = 12.0;
