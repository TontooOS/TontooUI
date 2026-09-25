pub mod horizontal;
pub mod vertical;

pub use horizontal::HorizontalDivider;
pub use vertical::VerticalDivider;

use vello::peniko::Color;

/// Thin default line thickness in logical px.
pub const DIVIDER_THIN: f32 = 1.0;
/// Thick line thickness in logical px.
pub const DIVIDER_THICK: f32 = 5.0;
/// Intrinsic fill extent in logical px. Stacks cap children at the stack
/// size (`min`), so this huge intrinsic size means full bleed: the
/// divider always spans the whole parent with no edge gap.
pub const DIVIDER_FILL: f32 = 32768.0;
/// Default divider color for dark mode (matches the theme palette).
pub const DIVIDER_DARK: Color = Color::from_rgba8(255, 255, 255, 36);
/// Default divider color for light mode (matches the theme palette).
pub const DIVIDER_LIGHT: Color = Color::from_rgba8(0, 0, 0, 31);
/// Preset red line color (system red accent).
pub const DIVIDER_RED: Color = Color::from_rgb8(0xff, 0x3b, 0x30);
/// Preset blue line color (system blue accent).
pub const DIVIDER_BLUE: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
/// Default symmetric inset for the padded style in logical px.
pub const DIVIDER_PADDED_INSET: f32 = 24.0;

/// Named divider looks, mirroring the reference rows: default, red,
/// thick, blue thick and padded (inset) variants. Apply with
/// `HorizontalDivider::styled` / `VerticalDivider::styled`, then
/// fine-tune with the builder methods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerStyle {
    #[default]
    Default,
    Red,
    Thick,
    BlueThick,
    Padded,
}

impl DividerStyle {
    /// Line thickness of the preset in logical px.
    pub fn thickness(&self) -> f32 {
        match self {
            Self::Default | Self::Red | Self::Padded => DIVIDER_THIN,
            Self::Thick | Self::BlueThick => DIVIDER_THICK,
        }
    }

    /// Symmetric inset of the preset in logical px (horizontal inset
    /// for `HorizontalDivider`, vertical inset for `VerticalDivider`).
    pub fn inset(&self) -> f32 {
        match self {
            Self::Padded => DIVIDER_PADDED_INSET,
            _ => 0.0,
        }
    }

    /// Line color of the preset. `None` means the theme default
    /// (follows `set_theme`); otherwise a manual color that wins.
    pub fn color(&self) -> Option<Color> {
        match self {
            Self::Default | Self::Thick | Self::Padded => None,
            Self::Red => Some(DIVIDER_RED),
            Self::BlueThick => Some(DIVIDER_BLUE),
        }
    }
}
