//! Material — SwiftUI `Material` / frosted glass.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Single material thickness — mirrors `SwiftUI.Material`.
///
/// SwiftUI provides `ultraThinMaterial`, `thinMaterial`, `regularMaterial`,
/// `thickMaterial`, `ultraThickMaterial` plus `bar`. Each maps to a
/// frosted translucency. Alpha values are chosen to match the screenshot
/// (5 teal/blue swatches left-to-right with decreasing vibrancy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    UltraThin,
    Thin,
    Regular,
    Thick,
    UltraThick,
    Bar,
}

impl Material {
    pub fn all() -> [Self; 6] {
        [
            Self::UltraThin,
            Self::Thin,
            Self::Regular,
            Self::Thick,
            Self::UltraThick,
            Self::Bar,
        ]
    }

    /// Palette subset shown in the reference card — the 5 main thicknesses.
    pub fn palette() -> [Self; 5] {
        [
            Self::UltraThin,
            Self::Thin,
            Self::Regular,
            Self::Thick,
            Self::UltraThick,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::UltraThin => "ultraThinMaterial",
            Self::Thin => "thinMaterial",
            Self::Regular => "regularMaterial",
            Self::Thick => "thickMaterial",
            Self::UltraThick => "ultraThickMaterial",
            Self::Bar => "bar",
        }
    }

    /// Opacity used for the frosted fill. Values are tuned to be visible
    /// directly on the TontooOS window background (#1d1d1d dark / #ececec light)
    /// with SF Pro context and no extra card.
    pub fn alpha(self, is_dark: bool) -> f32 {
        match (self, is_dark) {
            (Self::UltraThin, true) => 0.08,
            (Self::UltraThin, false) => 0.05,
            (Self::Thin, true) => 0.14,
            (Self::Thin, false) => 0.09,
            (Self::Regular, true) => 0.22,
            (Self::Regular, false) => 0.14,
            (Self::Thick, true) => 0.36,
            (Self::Thick, false) => 0.22,
            (Self::UltraThick, true) => 0.54,
            (Self::UltraThick, false) => 0.32,
            (Self::Bar, true) => 0.72,
            (Self::Bar, false) => 0.58,
        }
    }

    /// Resolve to a `Color` (white translucent in dark, dark translucent in light
    /// is not needed — materials are always light frosted, bordered adaptively).
    pub fn color(self, is_dark: bool) -> Color {
        let a = self.alpha(is_dark);
        if is_dark {
            Color::new(1.0, 1.0, 1.0, a)
        } else {
            // Light mode: also white frosted but on #ececec it still reads as frosted;
            // slightly cooler white to hint vibrancy.
            Color::new(0.99, 0.99, 1.0, a)
        }
    }

    /// Border for the material swatch — hairline that ensures contrast directly
    /// on window without a card.
    pub fn border_css(self, is_dark: bool) -> String {
        if is_dark {
            "1px solid rgba(255,255,255,0.14)".into()
        } else {
            "1px solid rgba(0,0,0,0.08)".into()
        }
    }
}

/// Palette widget — `Materials` — "All materials".
///
/// Shows the 5 main thickness swatches in a single row directly on the
/// window background, no extra card. Each swatch is a `44×44` rounded
/// frosted rectangle (simulated via semi-transparent fill + hairline border).
/// A second row with a blurred image placeholder mimics the screenshot's
/// bottom row (3 gray/brown frosted cards over an image) but is rendered
/// as simple translucent boxes directly on window to keep "no extra background".
pub struct Materials {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl Materials {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 260.0, 96.0)
    }
}

impl Default for Materials {
    fn default() -> Self { Self::new() }
}

impl ViewContent for Materials {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;

        // Outer — directly on window, no background card
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 10);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 {
            outer.set_size_request(frame.width as i32, 96);
        }

        // Row 1: 5 main materials (teal tint hint in screenshot — we add a very subtle teal/cyan tint
        // to the first swatches to match the blue-cyan top bar).
        let row1 = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row1.set_halign(gtk::Align::Center);

        let tints: [Option<Color>; 5] = [
            Some(Color::from_rgb(68, 180, 255)), // subtle blue-cyan tint for ultraThin
            Some(Color::from_rgb(88, 190, 255)),
            Some(Color::from_rgb(108, 200, 255)),
            None,
            None,
        ];

        for (idx, variant) in Material::palette().iter().enumerate() {
            let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
            sw.set_size_request(44, 44);
            sw.add_css_class("mat-swatch");

            // Mix base material color with optional tint at low weight
            let base = variant.color(is_dark);
            let bg_css = if let Some(tint) = tints[idx] {
                // blend 12% tint into white frosted
                let r = base.r * 0.88 + tint.r * 0.12;
                let g = base.g * 0.88 + tint.g * 0.12;
                let b = base.b * 0.88 + tint.b * 0.12;
                Color::new(r, g, b, base.a).to_css()
            } else {
                base.to_css()
            };
            let border = variant.border_css(is_dark);
            // frosted hint via box-shadow blur simulation + border
            uikit::widget::apply_css(
                &sw,
                &format!(
                    ".mat-swatch {{ background: {}; border: {}; border-radius: 10px; min-width: 44px; min-height: 44px; box-shadow: 0 1px 8px rgba(0,0,0,0.12); }}",
                    bg_css, border
                ),
            );
            row1.append(&sw);
        }
        outer.append(&row1);

        // Row 2: 3 swatches with more muted tone to mimic the bottom row
        // where materials are shown over a photographic background.
        // Render directly on window as 3 muted frosted boxes (no image, but
        // translucency shows window background through).
        let row2 = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row2.set_halign(gtk::Align::Center);
        let muted = [
            Material::Thin,
            Material::Regular,
            Material::Thick,
        ];
        for variant in muted {
            let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
            sw.set_size_request(44, 28);
            sw.add_css_class("mat-swatch2");
            let mut col = variant.color(is_dark);
            // slightly desaturate for muted row
            col = Color::new(col.r * 0.96, col.g * 0.96, col.b * 0.98, col.a);
            uikit::widget::apply_css(
                &sw,
                &format!(
                    ".mat-swatch2 {{ background: {}; border: {}; border-radius: 8px; min-width: 44px; min-height: 28px; }}",
                    col.to_css(),
                    variant.border_css(is_dark)
                ),
            );
            row2.append(&sw);
        }
        outer.append(&row2);

        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(260.0, 96.0) }
}

impl Widget for Materials {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 260.0, 96.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn material_palette_len() {
        assert_eq!(Material::palette().len(), 5);
        assert_eq!(Material::all().len(), 6);
    }
    #[test]
    fn material_alpha_order() {
        // alpha increases with thickness
        let is_dark = true;
        assert!(Material::UltraThin.alpha(is_dark) < Material::Thin.alpha(is_dark));
        assert!(Material::Thin.alpha(is_dark) < Material::Regular.alpha(is_dark));
        assert!(Material::Regular.alpha(is_dark) < Material::Thick.alpha(is_dark));
        assert!(Material::Thick.alpha(is_dark) < Material::UltraThick.alpha(is_dark));
    }
    #[test]
    fn material_color_adaptive() {
        let c_dark = Material::Regular.color(true);
        let c_light = Material::Regular.color(false);
        assert!((c_dark.a - 0.22).abs() < 0.01);
        assert!((c_light.a - 0.14).abs() < 0.01);
    }
}
