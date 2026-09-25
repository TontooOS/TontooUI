use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Toolbar height in logical px. Kept small on purpose.
pub const TOOLBAR_HEIGHT: f32 = 36.0;
/// Capsule corner radius in logical px (half the height).
pub const TOOLBAR_RADIUS: f32 = 18.0;
/// Icon box in logical px (aspect kept).
pub const TOOLBAR_ICON_SIZE: f32 = 18.0;
/// Horizontal padding inside the pill in logical px.
pub const TOOLBAR_PAD_X: f32 = 8.0;
/// Gap between icon hit targets in logical px.
pub const TOOLBAR_GAP: f32 = 4.0;
/// Square hit target per icon in logical px.
pub const TOOLBAR_HIT: f32 = 28.0;
/// Icon color in dark mode.
pub const TOOLBAR_ICON_DARK: Color = Color::from_rgb8(0xf5, 0xf5, 0xf7);
/// Icon color in light mode.
pub const TOOLBAR_ICON_LIGHT: Color = Color::from_rgb8(0x1e, 0x1e, 0x1e);

/// Placement of the icons inside the toolbar rect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarPlacement {
    /// Icons start at the leading edge.
    #[default]
    Leading,
    /// Icons are centered.
    Center,
    /// Icons end at the trailing edge.
    Trailing,
}

/// Basic toolbar: a small capsule in the clear (`Lens`) glass finish
/// holding icon buttons (SF Symbols from CoreIcon). Each icon is a
/// normal button action: hover tints the icon cell, pressing fires
/// `on_action` with the icon index.
///
/// The toolbar never uses the `Frosted` finish: the body is always
/// `GlassType::Lens` (clear minified center, blur only on the rim).
/// Hover lightens the cell in dark mode and darkens it in light mode;
/// pressing deepens the same tint.
pub struct BasicToolbar {
    icons: Vec<String>,
    placement: ToolbarPlacement,
    dark: bool,
    focused: bool,
    disabled: bool,
    icon_color: Color,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    cells: Vec<(f32, f32, f32, f32)>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    armed: bool,
    glass: GlassContainer,
    on_action: Option<Box<dyn FnMut(usize)>>,
}

impl BasicToolbar {
    pub fn new() -> Self {
        let glass = GlassContainer::new().glass_type(GlassType::Lens);
        Self {
            icons: Vec::new(),
            placement: ToolbarPlacement::Leading,
            dark: true,
            focused: true,
            disabled: false,
            icon_color: TOOLBAR_ICON_DARK,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: TOOLBAR_HEIGHT,
            cells: Vec::new(),
            hovered: None,
            pressed: None,
            armed: false,
            glass,
            on_action: None,
        }
    }

    pub fn from_icons(icons: Vec<String>) -> Self {
        Self::new().icons(icons)
    }

    pub fn icons(mut self, icons: Vec<String>) -> Self {
        self.icons = icons;
        self
    }

    /// Push one SF Symbol icon (e.g. `"chevron.left"`, `"heart"`).
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icons.push(name.into());
        self
    }

    pub fn set_icons(&mut self, icons: Vec<String>) {
        self.icons = icons;
    }

    /// Icon placement inside the toolbar rect.
    pub fn placement(mut self, placement: ToolbarPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn set_placement(&mut self, placement: ToolbarPlacement) {
        self.placement = placement;
    }

    /// Normal button action: fires with the icon index on click.
    pub fn on_action(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_action = Some(Box::new(callback));
        self
    }

    pub fn set_on_action(&mut self, callback: impl FnMut(usize) + 'static) {
        self.on_action = Some(Box::new(callback));
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Live theme: dark flag drives the icon color plus the hover/press
    /// tint direction; the glass amount is forwarded to the `Lens` body.
    /// The finish stays `Lens` and never switches to `Frosted`.
    pub fn set_theme(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.icon_color = if self.dark {
            TOOLBAR_ICON_DARK
        } else {
            TOOLBAR_ICON_LIGHT
        };
        self.glass.set_glass_type(GlassType::Lens);
        self.glass.set_theme(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
    }

    fn content_width(&self) -> f32 {
        if self.icons.is_empty() {
            return TOOLBAR_PAD_X * 2.0;
        }
        TOOLBAR_PAD_X * 2.0
            + self.icons.len() as f32 * TOOLBAR_HIT
            + (self.icons.len() as f32 - 1.0) * TOOLBAR_GAP
    }

    fn layout_cells(&mut self) {
        self.cells.clear();
        let content = self.content_width() - TOOLBAR_PAD_X * 2.0;
        let start_x = match self.placement {
            ToolbarPlacement::Leading => self.x + TOOLBAR_PAD_X,
            ToolbarPlacement::Center => {
                self.x + (self.width - content) / 2.0
            }
            ToolbarPlacement::Trailing => {
                self.x + self.width - TOOLBAR_PAD_X - content
            }
        };
        let cy = self.y + (self.height - TOOLBAR_HIT) / 2.0;
        let mut cx = start_x;
        for _ in self.icons.iter() {
            self.cells.push((cx, cy, TOOLBAR_HIT, TOOLBAR_HIT));
            cx += TOOLBAR_HIT + TOOLBAR_GAP;
        }
    }

    fn cell_at(&self, x: f32, y: f32) -> Option<usize> {
        self.cells
            .iter()
            .enumerate()
            .find(|(_, (cx, cy, cw, ch))| {
                x >= *cx && x <= *cx + *cw && y >= *cy && y <= *cy + *ch
            })
            .map(|(index, _)| index)
    }

    /// Update hover from logical cursor position.
    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.hovered = self.cell_at(x, y);
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.set_hover(x, y);
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        if let Some(index) = self.cell_at(x as f32, y as f32) {
            self.pressed = Some(index);
            self.hovered = Some(index);
            self.armed = true;
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn finish_press(&mut self, x: f64, y: f64) {
        let armed = self.armed;
        let pressed = self.pressed;
        self.armed = false;
        self.pressed = None;
        if armed && !self.disabled {
            if let Some(index) = pressed {
                if self.cell_at(x as f32, y as f32) == Some(index) {
                    if let Some(callback) = self.on_action.as_mut() {
                        callback(index);
                    }
                }
            }
        }
    }

    /// Click handling. Returns the icon index when hit, otherwise `None`.
    pub fn press(&mut self, x: f32, y: f32) -> Option<usize> {
        if self.disabled {
            return None;
        }
        let hit = self.cell_at(x, y);
        if let Some(index) = hit {
            if let Some(callback) = self.on_action.as_mut() {
                callback(index);
            }
        }
        hit
    }

    fn state_tint(&self, hovered: bool, pressed: bool) -> Option<Color> {
        if self.disabled {
            return None;
        }
        if pressed {
            // Press deepens the hover tint in both modes.
            Some(if self.dark {
                Color::from_rgba8(255, 255, 255, 40)
            } else {
                Color::from_rgba8(0, 0, 0, 40)
            })
        } else if hovered {
            Some(if self.dark {
                Color::from_rgba8(255, 255, 255, 20)
            } else {
                Color::from_rgba8(0, 0, 0, 15)
            })
        } else {
            None
        }
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        // Glass body first (Lens finish; skips itself on capture pass).
        // Keep the finish locked to Lens even if a caller touched it.
        self.glass.set_glass_type(GlassType::Lens);
        self.glass.set_bounds(self.x, self.y, self.width, self.height);
        self.glass.set_radius(TOOLBAR_RADIUS);
        self.glass.draw(scene, fonts, images);
        if images.is_capture_pass() {
            return;
        }

        let scale = fonts.scale as f64;
        let mut icon_color = self.icon_color;
        if !self.focused {
            icon_color = desaturate(icon_color);
        }
        if self.disabled {
            let c = icon_color.to_rgba8();
            icon_color =
                Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * 0.4).round() as u8);
        }

        for (index, (cx, cy, cw, ch)) in self.cells.clone().iter().enumerate() {
            let hovered = self.hovered == Some(index);
            let pressed = self.pressed == Some(index) && self.armed;
            if let Some(tint) = self.state_tint(hovered, pressed) {
                let cell = RoundedRect::new(
                    (*cx as f64) * scale,
                    (*cy as f64) * scale,
                    ((cx + cw) as f64) * scale,
                    ((cy + ch) as f64) * scale,
                    (TOOLBAR_HIT / 2.0 as f32) as f64 * scale,
                );
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(tint), None, &cell);
            }
            if let Some(name) = self.icons.get(index).cloned() {
                let target =
                    (TOOLBAR_ICON_SIZE * fonts.scale * 2.0).ceil().max(1.0) as u32;
                if let Some((image, iw, ih)) = images.get(&name, icon_color, target) {
                    let s = (TOOLBAR_ICON_SIZE / iw as f32)
                        .min(TOOLBAR_ICON_SIZE / ih as f32);
                    let ix = cx + (cw - iw as f32 * s) / 2.0;
                    let iy = cy + (ch - ih as f32 * s) / 2.0;
                    let transform =
                        Affine::translate((ix as f64 * scale, iy as f64 * scale))
                            * Affine::scale(s as f64 * scale);
                    scene.draw_image(&image, transform);
                }
            }
        }
    }
}

impl Default for BasicToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for BasicToolbar {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.content_width(), TOOLBAR_HEIGHT)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, _h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(self.content_width());
        self.height = TOOLBAR_HEIGHT;
        self.layout_cells();
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_fits_icons() {
        let mut bar = BasicToolbar::from_icons(vec![
            "chevron.left".to_string(),
            "line.3.horizontal".to_string(),
        ]);
        let mut fonts = FontSystem::new();
        let (w, h) = bar.measure(&mut fonts);
        assert_eq!(h, TOOLBAR_HEIGHT);
        assert_eq!(
            w,
            TOOLBAR_PAD_X * 2.0 + 2.0 * TOOLBAR_HIT + TOOLBAR_GAP
        );
    }

    #[test]
    fn placement_moves_cells() {
        let icons = vec!["a".to_string(), "b".to_string()];
        let mut fonts = FontSystem::new();
        let mut leading =
            BasicToolbar::from_icons(icons.clone()).placement(ToolbarPlacement::Leading);
        let mut trailing =
            BasicToolbar::from_icons(icons).placement(ToolbarPlacement::Trailing);
        leading.place(&mut fonts, 0.0, 0.0, 300.0, TOOLBAR_HEIGHT);
        trailing.place(&mut fonts, 0.0, 0.0, 300.0, TOOLBAR_HEIGHT);
        assert!(leading.cells[0].0 < trailing.cells[0].0);
    }

    #[test]
    fn press_fires_action() {
        let mut fonts = FontSystem::new();
        let mut bar = BasicToolbar::from_icons(vec!["heart".to_string()])
            .on_action(|_| {});
        bar.place(&mut fonts, 0.0, 0.0, 200.0, TOOLBAR_HEIGHT);
        let (cx, cy, cw, ch) = bar.cells[0];
        assert_eq!(bar.press(cx + cw / 2.0, cy + ch / 2.0), Some(0));
        assert_eq!(bar.press(199.0, 1.0), None);
    }

    #[test]
    fn press_down_up_fires_once() {
        use std::cell::Cell;
        use std::rc::Rc;
        let fired = Rc::new(Cell::new(0));
        let moved = fired.clone();
        let mut bar = BasicToolbar::from_icons(vec!["heart".to_string()])
            .on_action(move |_| {
                moved.set(moved.get() + 1);
            });
        let mut fonts = FontSystem::new();
        bar.place(&mut fonts, 0.0, 0.0, 200.0, TOOLBAR_HEIGHT);
        let (cx, cy, cw, ch) = bar.cells[0];
        bar.mouse_down((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        bar.mouse_up((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        assert_eq!(fired.get(), 1);
    }
}
