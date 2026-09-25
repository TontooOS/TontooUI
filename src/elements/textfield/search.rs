use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::glass::{GlassContainer, GlassType};
use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::{
    FieldCore, TEXTFIELD_PLACEHOLDER_DARK, TEXTFIELD_PLACEHOLDER_LIGHT,
    caret_blink,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::renderer::window::Key;
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Search text size in logical px.
pub const SEARCH_FONT_SIZE: f32 = 14.0;
/// Search icon box in logical px.
pub const SEARCH_ICON_SIZE: f32 = 16.0;
/// Search horizontal padding in logical px.
pub const SEARCH_PAD_X: f32 = 14.0;
/// Search icon-text gap in logical px.
pub const SEARCH_GAP: f32 = 8.0;

/// Toolbar-like search field: frosted glass capsule with a magnifier
/// icon and a single-line text input on top (like the reference
/// pill). Click inside to select (caret to end); ESC or a click
/// outside deselects. Typing, Backspace and caret keys arrive
/// through `type_text` and `key`. The frost needs the shell blur
/// pass: the app opts in with `wants_backdrop` while visible (like
/// alerts and the date picker).
pub struct SearchField {
    core: FieldCore,
    icon: SFSymbolImage,
    glass: GlassContainer,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl SearchField {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            core: FieldCore::new(placeholder.into()),
            icon: SFSymbolImage::new("magnifyingglass").size(SEARCH_ICON_SIZE),
            glass: GlassContainer::new().glass_type(GlassType::Frosted),
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Live theme: frost amount, dark mode and the caret accent.
    pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.core.accent = accent;
        self.core.dark = self.dark;
        self.core.dirty = true;
        self.glass.set_theme(mode, glass);
        self.icon.set_theme(
            if self.dark {
                TEXTFIELD_PLACEHOLDER_DARK
            } else {
                TEXTFIELD_PLACEHOLDER_LIGHT
            },
            self.dark,
        );
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.core.focused = focused;
        self.core.dirty = true;
        self.glass.set_focused(focused);
        self.icon.set_focused(focused);
    }

    /// Fires with the full text on every user edit (live search).
    pub fn on_change(mut self, callback: impl FnMut(&str) + 'static) -> Self {
        self.core.on_change = Some(Box::new(callback));
        self
    }

    /// Programmatic text (caret to end, no `on_change`).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.core.set_text(text.into());
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        let placeholder = placeholder.into();
        if placeholder != self.core.placeholder {
            self.core.placeholder = placeholder;
            self.core.dirty = true;
        }
    }

    pub fn text_value(&self) -> &str {
        &self.core.text
    }

    pub fn is_selected(&self) -> bool {
        self.core.selected
    }

    /// Type printable text at the caret (the app forwards its
    /// `text` here while selected).
    pub fn type_text(&mut self, content: &str) {
        if self.core.selected {
            self.core.insert(content);
        }
    }

    /// Key handling while selected: Backspace deletes, Left/Right
    /// move the caret, ESC deselects. Returns true when consumed.
    /// The app forwards its `key` here.
    pub fn key(&mut self, key: Key) -> bool {
        if !self.core.selected {
            return false;
        }
        match key {
            Key::Backspace => self.core.backspace(),
            Key::Left => self.core.move_left(),
            Key::Right => self.core.move_right(),
            Key::Escape => self.core.deselect(),
            _ => return false,
        }
        true
    }

    /// Press handling: click inside selects (caret to end), anywhere
    /// else deselects. The app forwards every press here.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.hit(x as f32, y as f32) {
            self.core.select();
        } else {
            self.core.deselect();
        }
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.placed_w && y >= self.y && y <= self.y + self.placed_h
    }

    fn text_color(&self) -> Color {
        let base = if self.dark {
            Color::from_rgb8(0xd8, 0xd9, 0xd9)
        } else {
            Color::from_rgb8(0x27, 0x27, 0x27)
        };
        if self.focused {
            base
        } else {
            desaturate(base)
        }
    }
}

impl View for SearchField {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (layout, _) = self.core.ensure_layout(
            fonts,
            SEARCH_FONT_SIZE,
            self.text_color(),
            self.text_color(),
            None,
        );
        let (_, th) = FontSystem::layout_size(layout);
        (
            160.0 + SEARCH_PAD_X * 2.0 + SEARCH_ICON_SIZE + SEARCH_GAP,
            th / fonts.scale + 16.0,
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        self.glass.set_bounds(x, y, w, h);
        self.glass.set_radius((w.min(h) / 2.0).max(0.0));
        let (iw, ih) = self.icon.measure(fonts);
        self.icon.place(fonts, x + SEARCH_PAD_X, y + (h - ih) / 2.0, iw, ih);
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
        if images.is_capture_pass() {
            // Backdrop capture: the glass skips itself; icon, text
            // and caret skip too so the blur stays clean.
            return;
        }
        self.glass.draw(scene, fonts, images);
        self.icon.draw(scene, fonts, images);
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let text = self.text_color();
        let placeholder = if self.dark {
            TEXTFIELD_PLACEHOLDER_DARK
        } else {
            TEXTFIELD_PLACEHOLDER_LIGHT
        };
        let ix = self.x + SEARCH_PAD_X + SEARCH_ICON_SIZE + SEARCH_GAP;
        let iw = (self.placed_w - (ix - self.x) - SEARCH_PAD_X).max(0.0);
        let clip = RoundedRect::new(
            px(ix),
            px(self.y),
            px(ix + iw),
            px(self.y + self.placed_h),
            px(4.0),
        );
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
        let th = {
            let (layout, _) =
                self.core
                    .ensure_layout(fonts, SEARCH_FONT_SIZE, text, placeholder, None);
            FontSystem::layout_size(layout).1
        };
        let ty = self.y + (self.placed_h - th / fonts.scale) / 2.0;
        let caret_x = self.core.caret_x(fonts, SEARCH_FONT_SIZE, text);
        self.core.track_caret(caret_x, iw);
        let scroll = self.core.scroll;
        {
            let (layout, _) =
                self.core
                    .ensure_layout(fonts, SEARCH_FONT_SIZE, text, placeholder, None);
            draw_layout(scene, layout, ix - scroll, ty, fonts.scale);
        }
        if self.core.selected && caret_blink() {
            let cx = ix - scroll + caret_x;
            let caret = if self.core.focused {
                self.core.accent
            } else {
                desaturate(self.core.accent)
            };
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(caret),
                None,
                &vello::kurbo::Rect::new(
                    px(cx),
                    px(ty),
                    px(cx + super::TEXTFIELD_CARET_W),
                    px(ty + th / fonts.scale),
                ),
            );
        }
        scene.pop_layer();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    fn field() -> SearchField {
        SearchField::new("Search items…")
    }

    #[test]
    fn typing_selects_and_edits() {
        let mut field = field();
        let mut fonts = FontSystem::new();
        let (_, h) = field.measure(&mut fonts);
        field.place(&mut fonts, 0.0, 0.0, 400.0, h);
        assert!(!field.is_selected());
        field.mouse_down(200.0, (h / 2.0) as f64);
        assert!(field.is_selected());
        field.type_text("app");
        assert_eq!(field.text_value(), "app");
        field.key(Key::Backspace);
        assert_eq!(field.text_value(), "ap");
    }

    #[test]
    fn escape_and_outside_click_deselect() {
        let mut field = field();
        let mut fonts = FontSystem::new();
        let (_, h) = field.measure(&mut fonts);
        field.place(&mut fonts, 0.0, 0.0, 400.0, h);
        field.mouse_down(200.0, (h / 2.0) as f64);
        assert!(field.is_selected());
        assert!(field.key(Key::Escape));
        assert!(!field.is_selected());
    }

    #[test]
    fn accent_follows_theme_accent() {
        let mut field = field();
        let pink = Color::from_rgb8(0xff, 0x2d, 0x55);
        field.set_theme(ThemeMode::Dark, pink, crate::theme::GlassAmount::Glass);
        assert_eq!(field.core.accent, pink);
    }
}
