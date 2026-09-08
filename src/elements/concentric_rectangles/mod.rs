//! ConcentricRectangle — category for concentric rectangle shapes.
//!
//! Category `ConcentricRectangle` groups concentric rectangle shapes. All
//! elements render directly on the window background (#1d1d1d dark / #ececec
//! light), no extra card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`uniform_concentric_rectangle`] | [`UniformConcentricRectangle`] | `initializer` | Create a rectangle with the same corner style set on four corners. |
//! | [`concentric_rectangle`] | [`ConcentricRectangle`] | `initializer` | A concentric rectangle whose corner radii are defined from the same circle. |

pub mod uniform_concentric_rectangle;
pub mod concentric_rectangle;

pub use uniform_concentric_rectangle::UniformConcentricRectangle;
pub use concentric_rectangle::ConcentricRectangle;
