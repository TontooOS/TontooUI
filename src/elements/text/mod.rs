pub mod font_preview;
pub mod foreground;
pub mod formatted;
pub mod labeled;
pub mod span;
pub mod style;
pub mod text;

pub use font_preview::{
    FontPreview, FONT_PREVIEW_DEFAULT_SAMPLE, FONT_PREVIEW_GAP, FONT_PREVIEW_SAMPLE_SIZE,
    FONT_PREVIEW_SAMPLE_WEIGHT, FONT_PREVIEW_TITLE_SIZE, FONT_PREVIEW_TITLE_WEIGHT,
};

pub use foreground::{ResolvedForeground, TextForeground, TEXT_TERTIARY_DARK, TEXT_TERTIARY_LIGHT};
pub use formatted::FormattedText;
pub use labeled::{LABELED_GAP, LabeledText};
pub use span::{Span, parse_markdown};
pub use style::TextStyle;
pub use text::{BasicText, TextAlignment};
