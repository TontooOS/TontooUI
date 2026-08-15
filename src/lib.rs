//! # TontooUI
//!
//! A SwiftUI-inspired declarative UI layer for TontooOS.
//!
//! Built on top of [`TontooUIKit`](https://docs.rs/uikit) — provides
//! pre-made elements with a clean, SwiftUI-like builder API.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! fn main() {
//!     let mut app = App::new("My App", 800, 600);
//!     app.set_root(
//!         VStack::new()
//!             .spacing(8.0)
//!             .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
//!             .child(TextInput::new("Enter text...")
//!                 .on_change(|text| println!("Changed: {}", text)))
//!     );
//!     app.run();
//! }
//! ```
//!
//! ## Elements
//!
//! | Element | Description |
//! |---|---|
//! | [`TextInput`] | Single-line text input field |
//! | [`WheelPicker`] | macOS/iOS style scroll wheel picker |
//! | [`Slider`] | Slider with spring physics and squish animation |
//! | [`ProgressView`] | Loading indicator: spinner or progress ring |
//! | [`Sidebar`] | macOS-style sidebar with traffic lights, search, and item list |

pub mod text_input;
pub mod wheel_picker;
pub mod slider;
pub mod progress_view;
pub mod sidebar;

pub use text_input::TextInput;
pub use wheel_picker::WheelPicker;
pub use slider::Slider;
pub use progress_view::ProgressView;
pub use sidebar::{Sidebar, SidebarIcon};

pub const TONTOO_UI_VERSION: (u32, u32, u32) = (0, 1, 0);

pub mod prelude {
    pub use crate::{TextInput, WheelPicker, Slider, ProgressView, Sidebar, SidebarIcon, TONTOO_UI_VERSION};

    // Re-export UIKit types for convenience
    pub use uikit::prelude::*;
}
