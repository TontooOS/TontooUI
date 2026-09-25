use vello::peniko::Color;

/// System color palette: name a color instead of spelling a hex or
/// RGB triplet (`SystemColor::Teal.color()`). Values are the Apple
/// system colors, matching the theme accents where they overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemColor {
    Red,
    Orange,
    Yellow,
    Green,
    Mint,
    Teal,
    Cyan,
    Blue,
    Indigo,
    Purple,
    Pink,
    Brown,
    Gray,
}

/// Every system color in display order.
pub const ALL_SYSTEM_COLORS: [SystemColor; 13] = [
    SystemColor::Red,
    SystemColor::Orange,
    SystemColor::Yellow,
    SystemColor::Green,
    SystemColor::Mint,
    SystemColor::Teal,
    SystemColor::Cyan,
    SystemColor::Blue,
    SystemColor::Indigo,
    SystemColor::Purple,
    SystemColor::Pink,
    SystemColor::Brown,
    SystemColor::Gray,
];

impl SystemColor {
    /// Display name ("Red", "Orange", ...).
    pub fn name(&self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Orange => "Orange",
            Self::Yellow => "Yellow",
            Self::Green => "Green",
            Self::Mint => "Mint",
            Self::Teal => "Teal",
            Self::Cyan => "Cyan",
            Self::Blue => "Blue",
            Self::Indigo => "Indigo",
            Self::Purple => "Purple",
            Self::Pink => "Pink",
            Self::Brown => "Brown",
            Self::Gray => "Gray",
        }
    }

    /// Parse a display name (case-insensitive). `None` for unknown
    /// names.
    pub fn from_str(raw: &str) -> Option<Self> {
        ALL_SYSTEM_COLORS
            .iter()
            .find(|c| c.name().eq_ignore_ascii_case(raw))
            .copied()
    }

    /// The color value.
    pub fn color(&self) -> Color {
        match self {
            Self::Red => Color::from_rgb8(0xff, 0x3b, 0x30),
            Self::Orange => Color::from_rgb8(0xff, 0x95, 0x00),
            Self::Yellow => Color::from_rgb8(0xff, 0xcc, 0x00),
            Self::Green => Color::from_rgb8(0x34, 0xc7, 0x59),
            Self::Mint => Color::from_rgb8(0x00, 0xc7, 0xbe),
            Self::Teal => Color::from_rgb8(0x30, 0xb0, 0xc7),
            Self::Cyan => Color::from_rgb8(0x32, 0xad, 0xe6),
            Self::Blue => Color::from_rgb8(0x00, 0x7a, 0xff),
            Self::Indigo => Color::from_rgb8(0x58, 0x56, 0xd6),
            Self::Purple => Color::from_rgb8(0xaf, 0x52, 0xde),
            Self::Pink => Color::from_rgb8(0xff, 0x2d, 0x55),
            Self::Brown => Color::from_rgb8(0xa2, 0x84, 0x5e),
            Self::Gray => Color::from_rgb8(0x8e, 0x8e, 0x93),
        }
    }
}

impl From<SystemColor> for Color {
    fn from(color: SystemColor) -> Self {
        color.color()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_names_round_trip() {
        assert_eq!(ALL_SYSTEM_COLORS.len(), 13);
        for color in ALL_SYSTEM_COLORS {
            assert_eq!(SystemColor::from_str(color.name()), Some(color));
            assert_eq!(SystemColor::from_str(&color.name().to_lowercase()), Some(color));
        }
        assert_eq!(SystemColor::from_str("chartreuse"), None);
        assert_eq!(SystemColor::from_str(""), None);
    }

    #[test]
    fn values_match_system_palette() {
        assert_eq!(SystemColor::Red.color(), Color::from_rgb8(0xff, 0x3b, 0x30));
        assert_eq!(SystemColor::Teal.color(), Color::from_rgb8(0x30, 0xb0, 0xc7));
        assert_eq!(SystemColor::Brown.color(), Color::from_rgb8(0xa2, 0x84, 0x5e));
        let gray: Color = SystemColor::Gray.into();
        assert_eq!(gray, Color::from_rgb8(0x8e, 0x8e, 0x93));
    }
}
