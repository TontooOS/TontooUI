use vello::peniko::Color;

use crate::theme::{Theme, ThemeMode};

/// Tertiary text in dark mode (white at 40% alpha).
pub const TEXT_TERTIARY_DARK: Color = Color::from_rgba8(255, 255, 255, 102);
/// Tertiary text in light mode (black at 30% alpha).
pub const TEXT_TERTIARY_LIGHT: Color = Color::from_rgba8(0, 0, 0, 77);

/// Text foreground (SwiftUI `foregroundStyle`): semantic theme colors,
/// a fixed color or a horizontal gradient across the text bounds.
#[derive(Clone, Debug, PartialEq)]
pub enum TextForeground {
    /// Theme text color (`#F5F5F7` dark, `#1E1E1E` light).
    Primary,
    /// Theme dim text (`text_dim` from the palette).
    Secondary,
    /// Faint theme text (white 40% dark, black 30% light).
    Tertiary,
    /// Fixed color in both modes.
    Color(Color),
    /// Horizontal gradient spread evenly across the text bounds. One
    /// color behaves like `Color`, an empty list falls back to
    /// `Primary`.
    Gradient(Vec<Color>),
}

impl Default for TextForeground {
    fn default() -> Self {
        Self::Primary
    }
}

/// Resolved foreground: either a solid color or gradient stops.
#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedForeground {
    Solid(Color),
    Gradient(Vec<Color>),
}

impl TextForeground {
    /// Resolve semantic colors against the theme. Unfocused windows
    /// desaturate through `focused = false`, like the rest of the
    /// palette.
    pub fn resolve(&self, mode: ThemeMode, focused: bool) -> ResolvedForeground {
        let palette = Theme {
            mode,
            ..Theme::default()
        }
        .palette();
        let solid = |color: Color| {
            if focused {
                color
            } else {
                crate::theme::desaturate(color)
            }
        };
        match self {
            Self::Primary => ResolvedForeground::Solid(solid(palette.text)),
            Self::Secondary => ResolvedForeground::Solid(solid(palette.text_dim)),
            Self::Tertiary => ResolvedForeground::Solid(solid(match mode {
                ThemeMode::Dark => TEXT_TERTIARY_DARK,
                ThemeMode::Light => TEXT_TERTIARY_LIGHT,
            })),
            Self::Color(color) => ResolvedForeground::Solid(solid(*color)),
            Self::Gradient(colors) => match colors.as_slice() {
                [] => ResolvedForeground::Solid(solid(palette.text)),
                [single] => ResolvedForeground::Solid(solid(*single)),
                stops => ResolvedForeground::Gradient(if focused {
                    stops.to_vec()
                } else {
                    stops.iter().map(|c| crate::theme::desaturate(*c)).collect()
                }),
            },
        }
    }
}
