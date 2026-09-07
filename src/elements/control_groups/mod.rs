//! ControlGroup — SwiftUI-style control group with styles.
//!
//! Category `ControlGroup` groups controls. Rendering is directly on the
//! window background (#1d1d1d dark / #ececec light), no extra card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`control_group`] | [`ControlGroup`] + [`ControlGroupStyle`] | `initializer` | Creates a new ControlGroup with the specified children |
//! | [`palette`] | [`PaletteControlGroupStyle`] | `style` | A control group style that presents its content as a palette |
//! | [`navigation`] | [`NavigationControlGroupStyle`] | `style` | The navigation control group style |
//! | [`menu`] | [`MenuControlGroupStyle`] | `style` | A control group style that presents its content as a menu when the user interacts |
//! | [`compact_menu`] | [`CompactMenuControlGroupStyle`] | `style` | A control group style that presents its content as a compact menu when the user interacts |

pub mod control_group;
pub mod palette;
pub mod navigation;
pub mod menu;
pub mod compact_menu;

pub use control_group::{ControlGroup, ControlGroupStyle};
pub use palette::PaletteControlGroupStyle;
pub use navigation::NavigationControlGroupStyle;
pub use menu::MenuControlGroupStyle;
pub use compact_menu::CompactMenuControlGroupStyle;
