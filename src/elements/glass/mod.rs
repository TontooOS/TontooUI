//! Glass — liquid glass material and container.
//!
//! [`GlassMaterial`] describes how the glass behaves, [`render_glass`]
//! composites it over any backdrop image, and [`GlassContainer`] keeps an
//! arbitrary content widget while replacing its background with glass.

pub mod container;
pub mod material;

pub use container::{GlassBehind, GlassContainer, GlassStyle};
pub use material::{ClearGlass, GlassMaterial, render_clear_glass, render_glass};
