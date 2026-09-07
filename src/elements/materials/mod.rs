//! Material — frosted glass / vibrancy category.
//!
//! The `Material` category groups frosted glass materials. SwiftUI's
//! `Material` (ultraThinMaterial … ultraThickMaterial, bar) is ported here.
//! Rendering is directly on the window background (#1d1d1d dark / #ececec light)
//! with SF Pro context and no extra card.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`material`] | [`Materials`] + [`Material`] | `type` | All materials (ultraThin … ultraThick, bar) |

pub mod material;

pub use material::{Material, Materials};
