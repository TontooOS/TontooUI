pub mod foreground;
pub mod formatted;
pub mod labeled;
pub mod span;
pub mod style;
pub mod text;

pub use foreground::{ResolvedForeground, TextForeground, TEXT_TERTIARY_DARK, TEXT_TERTIARY_LIGHT};
pub use formatted::FormattedText;
pub use labeled::{LABELED_GAP, LabeledText};
pub use span::{Span, parse_markdown};
pub use style::TextStyle;
pub use text::{BasicText, TextAlignment};
