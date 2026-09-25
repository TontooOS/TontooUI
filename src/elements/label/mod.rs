pub mod basic;
pub mod icon;
pub mod image;
pub mod styled;

pub use basic::BasicLabel;
pub use icon::IconLabel;
pub use image::ImageLabel;
pub use styled::{LabelStyle, StyledLabel};

use vello::peniko::Color;

/// Leading SF icon box in logical px.
pub const LABEL_ICON_SIZE: f32 = 24.0;
/// Gap between icon/image and text in logical px.
pub const LABEL_GAP: f32 = 12.0;
/// Async image box in logical px.
pub const LABEL_IMAGE_SIZE: f32 = 44.0;
/// Status dot diameter in logical px.
pub const LABEL_DOT: f32 = 16.0;
/// Default icon gray (both modes).
pub const LABEL_ICON_GRAY: Color = Color::from_rgba8(255, 255, 255, 220);
