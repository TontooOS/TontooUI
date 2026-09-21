use vello::peniko::Color;

use crate::animation::{Animatable, Easing};

/// Color theme. Mirrors the settings daemon values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn from_str(raw: &str) -> Self {
        match raw {
            "light" => Self::Light,
            _ => Self::Dark,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

/// Accent color. Hex values come from the Settings app palette.
/// `Multicolor` is the default element and renders blue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accent {
    #[default]
    Multicolor,
    Blue,
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Cyan,
    Indigo,
    Purple,
    Purple2,
    Pink,
    Gray,
}

impl Accent {
    pub fn from_str(raw: &str) -> Self {
        match raw {
            "blue" => Self::Blue,
            "red" => Self::Red,
            "orange" => Self::Orange,
            "yellow" => Self::Yellow,
            "green" => Self::Green,
            "teal" => Self::Teal,
            "cyan" => Self::Cyan,
            "indigo" => Self::Indigo,
            "purple" => Self::Purple,
            "purple2" => Self::Purple2,
            "pink" => Self::Pink,
            "gray" => Self::Gray,
            _ => Self::Multicolor,
        }
    }

    pub fn hex(&self) -> &'static str {
        match self {
            Self::Multicolor | Self::Blue => "#007AFF",
            Self::Red => "#FF3B30",
            Self::Orange => "#FF9500",
            Self::Yellow => "#FFCC00",
            Self::Green => "#34C759",
            Self::Teal => "#00C7BE",
            Self::Cyan => "#30B0C7",
            Self::Indigo => "#5856D6",
            Self::Purple => "#AF52DE",
            Self::Purple2 => "#BF5AF2",
            Self::Pink => "#FF2D55",
            Self::Gray => "#8E8E93",
        }
    }

    pub fn color(&self) -> Color {
        hex_color(self.hex())
    }
}

fn hex_channel(hex: &str, at: usize) -> u8 {
    u8::from_str_radix(&hex[at..at + 2], 16).unwrap_or(0)
}

fn hex_color(hex: &'static str) -> Color {
    let bytes = hex.as_bytes();
    if bytes.len() == 7 && bytes[0] == b'#' {
        Color::from_rgb8(
            hex_channel(hex, 1),
            hex_channel(hex, 3),
            hex_channel(hex, 5),
        )
    } else {
        Color::WHITE
    }
}

/// Effective theme: mode plus accent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub accent: Accent,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            accent: Accent::Multicolor,
        }
    }
}

/// Resolved colors for a theme. Dark body `#1d1d1d`, light body `#ececec`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub titlebar_bg: Color,
    pub titlebar_text: Color,
    pub divider: Color,
    pub accent: Color,
}

impl Theme {
    pub fn palette(&self) -> Palette {
        let accent = self.accent.color();
        match self.mode {
            ThemeMode::Dark => Palette {
                bg: Color::from_rgb8(0x1d, 0x1d, 0x1d),
                text: Color::from_rgb8(0xf5, 0xf5, 0xf7),
                text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
                titlebar_bg: Color::from_rgb8(0x2c, 0x2c, 0x2e),
                titlebar_text: Color::from_rgb8(0xf5, 0xf5, 0xf7),
                divider: Color::from_rgba8(255, 255, 255, 36),
                accent,
            },
            ThemeMode::Light => Palette {
                bg: Color::from_rgb8(0xec, 0xec, 0xec),
                text: Color::from_rgb8(0x1e, 0x1e, 0x1e),
                text_dim: Color::from_rgb8(0x6e, 0x6e, 0x72),
                titlebar_bg: Color::from_rgb8(0xde, 0xde, 0xe1),
                titlebar_text: Color::from_rgb8(0x1e, 0x1e, 0x1e),
                divider: Color::from_rgba8(0, 0, 0, 31),
                accent,
            },
        }
    }
}

fn lerp_palette(from: &Palette, to: &Palette, t: f32) -> Palette {
    // Qualified: `Color` also has an inherent 3-argument `lerp`.
    Palette {
        bg: Animatable::lerp(&from.bg, &to.bg, t),
        text: Animatable::lerp(&from.text, &to.text, t),
        text_dim: Animatable::lerp(&from.text_dim, &to.text_dim, t),
        titlebar_bg: Animatable::lerp(&from.titlebar_bg, &to.titlebar_bg, t),
        titlebar_text: Animatable::lerp(&from.titlebar_text, &to.titlebar_text, t),
        divider: Animatable::lerp(&from.divider, &to.divider, t),
        accent: Animatable::lerp(&from.accent, &to.accent, t),
    }
}

/// Seconds for the theme crossfade.
pub const THEME_FADE_SECONDS: f64 = 0.25;
/// Seconds between daemon polls.
pub const THEME_POLL_SECONDS: f64 = 1.0;

/// Watches the settings daemon for theme changes and crossfades the
/// palette. Unix only (daemon socket); elsewhere it serves defaults.
pub struct ThemeWatcher {
    theme: Theme,
    from: Palette,
    fade_start: f64,
    fading: bool,
    last_poll: f64,
    #[cfg(unix)]
    provider: coresettings::SettingsProvider,
    #[cfg(unix)]
    revision: u64,
}

impl ThemeWatcher {
    pub fn new() -> Self {
        let theme = Theme::default();
        Self {
            from: theme.palette(),
            theme,
            fade_start: 0.0,
            fading: false,
            last_poll: f64::NEG_INFINITY,
            #[cfg(unix)]
            provider: coresettings::SettingsProvider::from_env(),
            #[cfg(unix)]
            revision: 0,
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    /// Poll the daemon (throttled). Returns true when the theme changed and
    /// a fade started. Never errors: a missing daemon keeps the theme.
    pub fn poll(&mut self, now_secs: f64) -> bool {
        if now_secs - self.last_poll < THEME_POLL_SECONDS {
            return false;
        }
        self.last_poll = now_secs;
        #[cfg(unix)]
        {
            let Ok(customize) = self.provider.customize() else {
                return false;
            };
            if customize.revision == self.revision {
                return false;
            }
            self.revision = customize.revision;
            let theme = Theme {
                mode: ThemeMode::from_str(match customize.theme {
                    coresettings::ThemeMode::Light => "light",
                    coresettings::ThemeMode::Dark => "dark",
                }),
                accent: Accent::from_str(customize.accent.as_str()),
            };
            if theme != self.theme {
                self.from = self.palette(now_secs);
                self.theme = theme;
                self.fade_start = now_secs;
                self.fading = true;
                return true;
            }
        }
        false
    }

    /// Current palette, mid-fade blended with eased progress.
    pub fn palette(&mut self, now_secs: f64) -> Palette {
        if !self.fading {
            return self.theme.palette();
        }
        let p = ((now_secs - self.fade_start) / THEME_FADE_SECONDS).clamp(0.0, 1.0) as f32;
        if p >= 1.0 {
            self.fading = false;
            // Exact target: perceptual blends land a ulp off the endpoint.
            return self.theme.palette();
        }
        lerp_palette(&self.from, &self.theme.palette(), Easing::CubicOut.apply(p))
    }
}

impl Default for ThemeWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_palette_matches_conventions() {
        let palette = Theme::default().palette();
        assert_eq!(palette.bg, Color::from_rgb8(0x1d, 0x1d, 0x1d));
        assert_eq!(palette.accent, Color::from_rgb8(0x00, 0x7a, 0xff));
    }

    #[test]
    fn multicolor_resolves_blue() {
        assert_eq!(Accent::Multicolor.color(), Accent::Blue.color());
    }

    #[test]
    fn fade_blends_and_finishes() {
        let mut watcher = ThemeWatcher::new();
        // Simulate an external change by driving the fade manually.
        watcher.theme = Theme {
            mode: ThemeMode::Light,
            accent: Accent::Red,
        };
        watcher.from = Theme::default().palette();
        watcher.fade_start = 0.0;
        watcher.fading = true;
        let mid = watcher.palette(0.125);
        assert_ne!(mid.bg, Theme::default().palette().bg);
        assert_ne!(mid.bg, watcher.theme.palette().bg);
        let end = watcher.palette(10.0);
        assert_eq!(end, watcher.theme.palette());
        assert!(!watcher.fading);
    }
}
