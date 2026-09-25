pub mod content;
pub mod search;

pub use content::ContentUnavailable;
pub use search::SearchEmpty;

use vello::peniko::Color;

/// SF icon box in logical px.
pub const UNAVAILABLE_ICON_SIZE: f32 = 64.0;
/// Default icon gray (both modes, like the reference).
pub const UNAVAILABLE_ICON_GRAY: Color = Color::from_rgb8(0x8e, 0x8e, 0x93);
/// Gap between icon and title in logical px.
pub const UNAVAILABLE_ICON_GAP: f32 = 16.0;
/// Gap between title and message in logical px.
pub const UNAVAILABLE_TITLE_GAP: f32 = 8.0;
/// Gap between message and refresh button in logical px.
pub const UNAVAILABLE_BUTTON_GAP: f32 = 16.0;
/// Message wrap width in logical px.
pub const UNAVAILABLE_WRAP_WIDTH: f32 = 320.0;
