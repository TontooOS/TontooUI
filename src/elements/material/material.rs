use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::MATERIAL_RADIUS;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::desaturate;

/// Material thickness: five translucent steps from barely-there to
/// nearly opaque (like the reference bars).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaterialKind {
    /// Faint veil over the background.
    UltraThin,
    /// Light veil.
    Thin,
    /// Standard veil.
    #[default]
    Regular,
    /// Heavy veil, obscuring more.
    Thick,
    /// Heaviest veil, almost opaque.
    UltraThick,
}

impl MaterialKind {
    /// Overlay alpha in 0..1 (white in dark mode, black in light
    /// mode — see `Material::overlay`).
    pub fn alpha(&self) -> f32 {
        match self {
            Self::UltraThin => 0.18,
            Self::Thin => 0.30,
            Self::Regular => 0.45,
            Self::Thick => 0.60,
            Self::UltraThick => 0.75,
        }
    }

    /// Display name ("Ultra Thin", ...).
    pub fn name(&self) -> &'static str {
        match self {
            Self::UltraThin => "Ultra Thin",
            Self::Thin => "Thin",
            Self::Regular => "Regular",
            Self::Thick => "Thick",
            Self::UltraThick => "Ultra Thick",
        }
    }
}

/// All kinds in display order.
pub const ALL_MATERIALS: [MaterialKind; 5] = [
    MaterialKind::UltraThin,
    MaterialKind::Thin,
    MaterialKind::Regular,
    MaterialKind::Thick,
    MaterialKind::UltraThick,
];

/// Simple material container: any child view over a plain
/// translucent rounded fill in one of five thicknesses. Deliberately
/// plain transparency — no zoom, no backdrop sampling, nothing
/// glass-complex. Presses forward to the child through the `View`
/// protocol, so bars, buttons and rows stay interactive inside.
pub struct Material<V> {
    child: V,
    kind: MaterialKind,
    radius: f32,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl<V: View> Material<V> {
    pub fn new(child: V, kind: MaterialKind) -> Self {
        Self {
            child,
            kind,
            radius: MATERIAL_RADIUS,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Barely-there veil.
    pub fn ultra_thin(child: V) -> Self {
        Self::new(child, MaterialKind::UltraThin)
    }

    /// Light veil.
    pub fn thin(child: V) -> Self {
        Self::new(child, MaterialKind::Thin)
    }

    /// Standard veil.
    pub fn regular(child: V) -> Self {
        Self::new(child, MaterialKind::Regular)
    }

    /// Heavy veil.
    pub fn thick(child: V) -> Self {
        Self::new(child, MaterialKind::Thick)
    }

    /// Almost opaque veil.
    pub fn ultra_thick(child: V) -> Self {
        Self::new(child, MaterialKind::UltraThick)
    }

    /// Thickness step.
    pub fn kind(mut self, kind: MaterialKind) -> Self {
        self.kind = kind;
        self
    }

    /// Corner radius in logical px (clamped to >= 0).
    pub fn radius(mut self, px: f32) -> Self {
        self.radius = px.max(0.0);
        self
    }

    /// Wrapped content for state updates.
    pub fn child_mut(&mut self) -> &mut V {
        &mut self.child
    }

    /// Live theme: white veil in dark mode, black veil in light mode.
    pub fn set_theme(&mut self, dark: bool) {
        self.dark = dark;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_kind(&mut self, kind: MaterialKind) {
        self.kind = kind;
    }

    pub fn set_radius(&mut self, px: f32) {
        self.radius = px.max(0.0);
    }

    pub fn kind_value(&self) -> MaterialKind {
        self.kind
    }

    /// Overlay fill for the current mode and focus.
    pub fn overlay(&self) -> Color {
        let alpha = (self.kind.alpha() * 255.0).round() as u8;
        let base = if self.dark {
            Color::from_rgba8(255, 255, 255, alpha)
        } else {
            Color::from_rgba8(0, 0, 0, alpha)
        };
        if self.focused {
            base
        } else {
            desaturate(base)
        }
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }
}

impl<V: View + 'static> View for Material<V> {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.child.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        self.child.place(fonts, x, y, w, h);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let radius = self
            .radius
            .min(self.placed_w / 2.0)
            .min(self.placed_h / 2.0)
            .max(0.0);
        let body = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
            px(radius),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.overlay()),
            None,
            &body,
        );
        self.child.draw(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.child.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.child.mouse_up(x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.child.set_hover(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    #[test]
    fn alphas_rise_with_thickness() {
        let mut last = 0.0;
        for kind in ALL_MATERIALS {
            assert!(kind.alpha() > last, "{kind:?}");
            last = kind.alpha();
        }
        assert_eq!(ALL_MATERIALS.len(), 5);
    }

    #[test]
    fn constructors_pick_kinds() {
        use crate::elements::BasicText;

        assert_eq!(
            Material::ultra_thin(BasicText::new("x")).kind_value(),
            MaterialKind::UltraThin
        );
        assert_eq!(
            Material::ultra_thick(BasicText::new("x")).kind_value(),
            MaterialKind::UltraThick
        );
    }

    #[test]
    fn overlay_follows_mode_and_focus() {
        use crate::elements::BasicText;

        let mut bar = Material::regular(BasicText::new("x"));
        bar.set_theme(true);
        assert_eq!(
            bar.overlay(),
            Color::from_rgba8(255, 255, 255, (0.45_f32 * 255.0).round() as u8)
        );
        bar.set_theme(false);
        assert_eq!(
            bar.overlay(),
            Color::from_rgba8(0, 0, 0, (0.45_f32 * 255.0).round() as u8)
        );
        bar.set_theme(true);
        bar.set_focused(false);
        let c = bar.overlay().to_rgba8();
        assert_eq!(c.r, c.g);
        assert_eq!(c.g, c.b);
    }

    #[test]
    fn presses_reach_nested_buttons() {
        use std::cell::RefCell;
        use std::rc::Rc;

        use crate::elements::buttons::Button;

        let fired: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let flag = fired.clone();
        let mut bar = Material::thin(
            Button::new("Tap").on_press(move || {
                *flag.borrow_mut() = true;
            }),
        );
        let mut fonts = FontSystem::new();
        let (w, h) = bar.measure(&mut fonts);
        bar.place(&mut fonts, 0.0, 0.0, w, h);
        bar.mouse_down((w / 2.0) as f64, (h / 2.0) as f64);
        bar.mouse_up((w / 2.0) as f64, (h / 2.0) as f64);
        assert!(*fired.borrow());
    }
}
