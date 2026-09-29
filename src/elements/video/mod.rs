pub mod player;
pub mod source;

pub use player::{VideoPlayer, VideoState};
pub use source::VideoSource;

use vello::peniko::Color;

/// Transport bar height in logical px.
pub const VIDEO_BAR_H: f32 = 48.0;
/// Transport button box in logical px.
pub const VIDEO_BTN: f32 = 32.0;
/// Gaps in the transport row in logical px.
pub const VIDEO_GAP: f32 = 8.0;
/// Time label size in logical px.
pub const VIDEO_TIME_SIZE: f32 = 12.0;
/// Time label slot width in logical px.
pub const VIDEO_TIME_W: f32 = 110.0;
/// Corner radius of the video frame in logical px.
pub const VIDEO_RADIUS: f32 = 12.0;
/// Drop shadow color under the frame.
pub const VIDEO_SHADOW: Color = Color::from_rgba8(0, 0, 0, 60);
/// Drop shadow blur in logical px.
pub const VIDEO_SHADOW_BLUR: f32 = 12.0;
/// Drop shadow y offset in logical px.
pub const VIDEO_SHADOW_DY: f32 = 4.0;
