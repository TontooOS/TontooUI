pub mod basic;

pub use basic::{BasicLink, open_url};

use vello::peniko::Color;

/// Link blue (both modes, like the reference rows).
pub const LINK_BLUE: Color = Color::from_rgb8(0x0a, 0x84, 0xff);
/// Pressed link blue (dimmed while held).
pub const LINK_BLUE_PRESSED: Color = Color::from_rgba8(0x0a, 0x84, 0xff, 140);
/// Gap between icon and label in logical px.
pub const LINK_ICON_GAP: f32 = 8.0;
/// Leading SF icon box in logical px.
pub const LINK_ICON_SIZE: f32 = 20.0;
