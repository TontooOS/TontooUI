//! Shapes — category for shape elements.
//!
//! Category `Shapes` groups shape views. All elements render directly on the
//! window background (#1d1d1d dark / #ececec light), no extra card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`circle`] | [`Circle`] | `modifier` | A circle shape centered in its frame. |
//! | [`ellipse`] | [`Ellipse`] | `modifier` | An elliptical shape filling its frame. |
//! | [`capsule`] | [`Capsule`] | `modifier` | A capsule (stadium) shape filling its frame. |
//! | [`rectangle`] | [`RectangleShape`] | `modifier` | A rectangular shape filling its frame. |
//! | [`rounded_rectangle`] | [`RoundedRectangle`] | `modifier` | A rectangle with rounded corners. |
//! | [`uneven_rounded_rectangle`] | [`UnevenRoundedRectangle`] | `modifier` | A rectangle with uneven corner radii. |
//! | [`container_relative_shape`] | [`ContainerRelativeShape`] | `modifier` | A shape that is replaced by an inset version of the current container shape. |

pub mod circle;
pub mod ellipse;
pub mod capsule;
pub mod rectangle;
pub mod rounded_rectangle;
pub mod uneven_rounded_rectangle;
pub mod container_relative_shape;

pub use circle::Circle;
pub use ellipse::Ellipse;
pub use capsule::Capsule;
pub use rectangle::RectangleShape;
pub use rounded_rectangle::RoundedRectangle;
pub use uneven_rounded_rectangle::UnevenRoundedRectangle;
pub use container_relative_shape::ContainerRelativeShape;
