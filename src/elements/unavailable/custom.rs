use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::buttons::Button;
use super::super::images::SFSymbolImage;
use super::super::layout::{Align, View, VStack};
use super::super::text::{BasicText, TextAlignment, TextForeground, TextStyle};
use super::{
    UNAVAILABLE_BUTTON_GAP, UNAVAILABLE_ICON_GAP, UNAVAILABLE_ICON_GRAY,
    UNAVAILABLE_ICON_SIZE, UNAVAILABLE_TITLE_GAP, UNAVAILABLE_WRAP_WIDTH,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::ThemeMode;

/// Custom empty state: a large SF Symbol on top, a semibold title, a
/// gray message and any custom view below (badge, button, progress,
/// like the reference "Coming Soon" rows). Same icon/text styling as
/// the siblings; presses and hovers reach custom buttons through the
/// `View` protocol.
pub struct CustomContentUnavailable<V> {
    icon: String,
    icon_color: Option<Color>,
    title: String,
    message: String,
    dark: bool,
    focused: bool,
    stack: VStack,
    custom: std::marker::PhantomData<V>,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl<V: View + 'static> CustomContentUnavailable<V> {
    pub fn new(
        icon: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        custom: V,
    ) -> Self {
        let mut view = Self {
            icon: icon.into(),
            icon_color: None,
            title: title.into(),
            message: message.into(),
            dark: true,
            focused: true,
            stack: VStack::new(),
            custom: std::marker::PhantomData,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        };
        view.rebuild(custom);
        view
    }

    /// SF Symbol name (CoreIcon lookup, like everywhere else).
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.set_icon(name);
        self
    }

    /// Optional icon tint. Without it the glyph stays
    /// `UNAVAILABLE_ICON_GRAY`.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.set_icon_color(Some(color));
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.set_title(title);
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.set_message(message);
        self
    }

    /// Live theme for the texts (custom content themes itself via
    /// `child_mut`).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark == self.dark {
            return;
        }
        self.dark = dark;
        self.apply_state();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.apply_state();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if title == self.title {
            return;
        }
        self.title = title.clone();
        if let Some(text) = self.nth_text(0) {
            text.set_text(title);
        }
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        let message = message.into();
        if message == self.message {
            return;
        }
        self.message = message.clone();
        if let Some(text) = self.nth_text(1) {
            text.set_text(message);
        }
    }

    pub fn set_icon(&mut self, name: impl Into<String>) {
        let name = name.into();
        if name == self.icon {
            return;
        }
        self.icon = name.clone();
        if let Some(symbol) = self.symbol_mut() {
            symbol.set_symbol(name);
        }
    }

    pub fn set_icon_color(&mut self, color: Option<Color>) {
        if color == self.icon_color {
            return;
        }
        self.icon_color = color;
        if let Some(symbol) = self.symbol_mut() {
            symbol.set_color(color);
        }
    }

    /// Custom content for state updates (labels, toggles, ...). The
    /// custom view is always the last stack child.
    pub fn child_mut(&mut self) -> Option<&mut V> {
        let last = self.stack.len().checked_sub(1)?;
        self.stack.child_mut::<V>(last)
    }

    pub fn icon_value(&self) -> &str {
        &self.icon
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn rebuild(&mut self, custom: V) {
        let icon_tint = self.icon_color.unwrap_or(UNAVAILABLE_ICON_GRAY);
        self.stack = VStack::new()
            .align(Align::Center)
            .spacing(0.0)
            .child(
                SFSymbolImage::new(self.icon.clone())
                    .size(UNAVAILABLE_ICON_SIZE)
                    .color(icon_tint),
            )
            .child(Gap::fixed(UNAVAILABLE_ICON_GAP))
            .child(
                BasicText::new(self.title.clone())
                    .style(TextStyle::Headline)
                    .alignment(TextAlignment::Center),
            )
            .child(Gap::fixed(UNAVAILABLE_TITLE_GAP))
            .child(
                BasicText::new(self.message.clone())
                    .foreground(TextForeground::Secondary)
                    .alignment(TextAlignment::Center)
                    .width(UNAVAILABLE_WRAP_WIDTH),
            )
            .child(Gap::fixed(UNAVAILABLE_BUTTON_GAP))
            .child(custom);
        self.apply_state();
    }

    /// Nth `BasicText` in stack order (0 title, 1 message). The
    /// custom view may itself be text, so position beats type here.
    fn nth_text(&mut self, nth: usize) -> Option<&mut BasicText> {
        let mut seen = 0;
        for index in 0..self.stack.len() {
            if self.stack.child_mut::<BasicText>(index).is_some() {
                if seen == nth {
                    return self.stack.child_mut::<BasicText>(index);
                }
                seen += 1;
            }
        }
        None
    }

    fn symbol_mut(&mut self) -> Option<&mut SFSymbolImage> {
        self.stack.child_mut::<SFSymbolImage>(0)
    }

    fn apply_state(&mut self) {
        for index in 0..self.stack.len() {
            if let Some(symbol) = self.stack.child_mut::<SFSymbolImage>(index) {
                symbol.set_focused(self.focused);
            } else if let Some(text) = self.stack.child_mut::<BasicText>(index) {
                text.set_focused(self.focused);
            } else if let Some(button) = self.stack.child_mut::<Button>(index) {
                button.set_focused(self.focused);
            }
        }
    }

    /// Forward hover to custom buttons (nothing for static content).
    pub fn set_hover(&mut self, x: f32, y: f32) {
        for index in 0..self.stack.len() {
            if let Some(button) = self.stack.child_mut::<Button>(index) {
                button.set_hover(x, y);
            }
        }
    }
}

/// Fixed-size spacer for stack gaps.
struct Gap {
    px: f32,
}

impl Gap {
    fn fixed(px: f32) -> Self {
        Self { px: px.max(0.0) }
    }
}

impl View for Gap {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.px, self.px)
    }

    fn place(&mut self, _fonts: &mut FontSystem, _x: f32, _y: f32, _w: f32, _h: f32) {}

    fn draw(
        &mut self,
        _scene: &mut Scene,
        _fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
    ) {
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<V: View + 'static> View for CustomContentUnavailable<V> {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.stack.measure(fonts)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        self.stack.place(fonts, x, y, w, h);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.stack.draw(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.stack.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.stack.mouse_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    fn view() -> CustomContentUnavailable<Button> {
        CustomContentUnavailable::new(
            "sparkles",
            "Coming Soon",
            "This feature is under development. Check back later!",
            Button::new("Notify Me"),
        )
    }

    #[test]
    fn custom_child_is_last_and_clickable() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let fired: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let flag = fired.clone();
        let mut view = CustomContentUnavailable::new(
            "sparkles",
            "Coming Soon",
            "Later!",
            Button::new("Notify Me").on_press(move || {
                *flag.borrow_mut() = true;
            }),
        );
        assert_eq!(view.icon_value(), "sparkles");
        let mut fonts = FontSystem::new();
        let (w, h) = view.measure(&mut fonts);
        view.place(&mut fonts, 0.0, 0.0, w.max(400.0), h);
        assert!(view.child_mut().is_some());
        let (bx, by, bw, bh) = view
            .child_mut()
            .map(|button| button.rect())
            .expect("custom button");
        view.mouse_down((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        view.mouse_up((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        assert!(*fired.borrow());
    }

    #[test]
    fn setters_update_parts_without_rebuild() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let fired: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let flag = fired.clone();
        let mut view = CustomContentUnavailable::new(
            "sparkles",
            "Coming Soon",
            "Later!",
            Button::new("Notify Me").on_press(move || {
                *flag.borrow_mut() = true;
            }),
        );
        let mut fonts = FontSystem::new();
        let (w, h) = view.measure(&mut fonts);
        view.place(&mut fonts, 0.0, 0.0, w.max(400.0), h);
        // Live updates must not rebuild the stack: a running press
        // on the custom button survives them.
        let (bx, by, bw, bh) = view
            .child_mut()
            .map(|button| button.rect())
            .expect("custom button");
        view.mouse_down((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        view.set_title("Later Still");
        view.set_message("Almost!");
        view.set_icon("star.fill");
        view.set_icon_color(Some(Color::from_rgb8(0xff, 0x9f, 0x0a)));
        view.set_theme(ThemeMode::Dark);
        view.mouse_up((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        assert!(*fired.borrow());
        assert_eq!(view.icon_value(), "star.fill");
    }
}
