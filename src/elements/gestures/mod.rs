pub mod area;

pub use area::GestureArea;

/// Hold time for a long press in seconds (mirrors context menus).
pub const GESTURE_LONG_PRESS_SECONDS: f64 = 0.6;
/// Wander allowance during press tracking in logical px: moving
/// further cancels tap and long press and starts a drag.
pub const GESTURE_MOVE_SLOP: f32 = 10.0;
/// Zoom factor per wheel notch (logical px delta scaled by this).
pub const GESTURE_MAGNIFY_STEP: f32 = 0.005;
/// Minimum zoom scale.
pub const GESTURE_MAGNIFY_MIN: f32 = 0.25;
/// Maximum zoom scale.
pub const GESTURE_MAGNIFY_MAX: f32 = 4.0;
