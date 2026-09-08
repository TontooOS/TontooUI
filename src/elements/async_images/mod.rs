//! AsyncImage — category for asynchronous URL images.
//!
//! Category `AsyncImage` groups async image views. All elements render
//! directly on the window background (#1d1d1d dark / #ececec light), no extra
//! card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`custom_placeholder`] | [`CustomPlaceholderAsyncImage`] | `initializer` | Loads and displays a modifiable image from the specified URL load request with a custom placeholder. |
//! | [`custom_phases`] | [`CustomPhasesAsyncImage`] | `initializer` | Loads and displays a modifiable image from the specified URL load request with custom phases. |
//! | [`async_url_image`] | [`AsyncURLImage`] | `initializer` | Loads and displays an image from the specified URL load request. |
//! | [`custom_session`] | [`CustomSessionAsyncImage`] | `modifier` | A modifier that adds a URL session for asynchronous images contained in the view. |

pub mod custom_placeholder;
pub mod custom_phases;
pub mod async_url_image;
pub mod custom_session;

pub use custom_placeholder::CustomPlaceholderAsyncImage;
pub use custom_phases::{CustomPhasesAsyncImage, AsyncImagePhase};
pub use async_url_image::AsyncURLImage;
pub use custom_session::CustomSessionAsyncImage;
