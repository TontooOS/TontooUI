/// Text style (SwiftUI `font`): size plus weight in logical px.
///
/// Sizes follow the SwiftUI type scale; only `Headline` is semibold,
/// everything else is regular.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextStyle {
    /// 34 regular.
    LargeTitle,
    /// 28 regular (SwiftUI Title 1).
    Title,
    /// 22 regular (SwiftUI Title 2).
    Title2,
    /// 20 regular (SwiftUI Title 3).
    Title3,
    /// 17 semibold.
    Headline,
    /// 15 regular.
    Subheadline,
    /// 17 regular (default).
    #[default]
    Body,
    /// 16 regular.
    Callout,
    /// 13 regular.
    Footnote,
    /// 12 regular.
    Caption,
    /// 11 regular.
    Caption2,
}

impl TextStyle {
    /// Label size in logical px.
    pub fn size(self) -> f32 {
        match self {
            Self::LargeTitle => 34.0,
            Self::Title => 28.0,
            Self::Title2 => 22.0,
            Self::Title3 => 20.0,
            Self::Headline => 17.0,
            Self::Subheadline => 15.0,
            Self::Body => 17.0,
            Self::Callout => 16.0,
            Self::Footnote => 13.0,
            Self::Caption => 12.0,
            Self::Caption2 => 11.0,
        }
    }

    /// Font weight (400 regular, 600 semibold).
    pub fn weight(self) -> f32 {
        match self {
            Self::Headline => 600.0,
            _ => 400.0,
        }
    }
}
