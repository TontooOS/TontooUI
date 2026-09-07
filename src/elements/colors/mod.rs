//! Colors — SwiftUI-style Color category for TontooUI.
//!
//! Category `Color` with 10 elements mirroring the SwiftUI / UIKit color
//! system shown in the reference screenshots.
//!
//! | File | Element | Screenshot badge | Description |
//! |---|---|---|--->
//! | [`opacity`] | [`ColorOpacity`] | `style` | The Color opacity modifier |
//! | [`gradient`] | [`ColorGradient`] | `modifier` | The Color gradient modifier |
//! | [`variant`] | [`ColorVariants`] + [`HierarchicalVariant`] | `modifier` | HierarchicalShapeStyle variants |
//! | [`separator`] | [`UIKitSeparatorColors`] + [`UIKitSeparatorColor`] | `type` | UIKit separator colors (separator / opaqueSeparator) |
//! | [`background`] | [`UIKitContentBackgroundColors`] + [`UIKitBackgroundColor`] | `type` | UIKit content background colors (systemBackground etc) |
//! | [`text`] | [`UIKitTextColors`] + [`UIKitTextColor`] | `type` | UIKit text colors (placeholderText) |
//! | [`fill`] | [`UIKitFillColors`] + [`UIKitFillColor`] | `type` | UIKit fill colors (systemFill ...) |
//! | [`label`] | [`UIKitLabelColors`] + [`UIKitLabelColor`] | `type` | UIKit label colors (label / secondaryLabel ...) |
//! | [`semantic`] | [`SemanticColors`] + [`SemanticColor`] | `type` | Semantic colors (primary / secondary) |
//! | [`standard`] | [`StandardColors`] + [`StandardColor`] | `type` | Standard SwiftUI colors |

pub mod opacity;
pub mod gradient;
pub mod variant;
pub mod separator;
pub mod background;
pub mod text;
pub mod fill;
pub mod label;
pub mod semantic;
pub mod standard;

pub use opacity::ColorOpacity;
pub use gradient::ColorGradient;
pub use variant::{ColorVariants, HierarchicalVariant};
pub use separator::{UIKitSeparatorColors, UIKitSeparatorColor};
pub use background::{UIKitContentBackgroundColors, UIKitBackgroundColor};
pub use text::{UIKitTextColors, UIKitTextColor};
pub use fill::{UIKitFillColors, UIKitFillColor};
pub use label::{UIKitLabelColors, UIKitLabelColor};
pub use semantic::{SemanticColors, SemanticColor};
pub use standard::{StandardColors, StandardColor};
