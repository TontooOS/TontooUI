pub mod basic;
pub mod image;
pub mod styled;

pub use basic::{BasicLink, is_openable, open_url};
pub use image::{LinkWithImage, LINK_IMAGE_GAP};
pub use styled::{LinkStyle, StyledLink, LINK_PILL_BG, LINK_PILL_BORDER, LINK_PILL_PAD_X, LINK_PILL_PAD_Y, LINK_PILL_RADIUS};

use vello::peniko::Color;

/// Link blue (both modes, like the reference rows).
pub const LINK_BLUE: Color = Color::from_rgb8(0x0a, 0x84, 0xff);
/// Pressed link blue (dimmed while held).
pub const LINK_BLUE_PRESSED: Color = Color::from_rgba8(0x0a, 0x84, 0xff, 140);
/// Gap between icon and label in logical px.
pub const LINK_ICON_GAP: f32 = 8.0;
/// Leading SF icon box in logical px.
pub const LINK_ICON_SIZE: f32 = 20.0;
