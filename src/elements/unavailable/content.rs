use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use vello::Scene;
use vello::peniko::Color;

use super::super::buttons::{Button, ButtonStyle};
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

/// Empty-state placeholder: a large SF Symbol on top, a semibold
/// title, a gray message and an optional prominent refresh button
/// (like the reference rows: tray icon, "No Data", "There is no data
/// to display yet. Pull down to refresh.", blue Refresh). Composed
/// from the stock views (symbol, texts, button in a centered stack).
/// The app polls `take_refreshed` per frame; more placeholder
/// variants plug in the same way.
pub struct ContentUnavailable {
    icon: String,
    icon_color: Option<Color>,
    title: String,
    message: String,
    refresh_label: Option<String>,
    accent: Color,
    dark: bool,
    focused: bool,
    fired: Rc<RefCell<bool>>,
    refreshing: bool,
    stack: VStack,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl ContentUnavailable {
    pub fn new(
        icon: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let mut view = Self {
            icon: icon.into(),
            icon_color: None,
            title: title.into(),
            message: message.into(),
            refresh_label: Some("Refresh".to_string()),
            accent: Color::from_rgb8(0x00, 0x7a, 0xff),
            dark: true,
            focused: true,
            fired: Rc::new(RefCell::new(false)),
            refreshing: false,
            stack: VStack::new(),
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        };
        view.rebuild();
        view
    }

    /// SF Symbol name (CoreIcon lookup, like everywhere else).
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = name.into();
        self.rebuild();
        self
    }

    /// Optional icon tint. Without it the glyph stays
    /// `UNAVAILABLE_ICON_GRAY`.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self.rebuild();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self.rebuild();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self.rebuild();
        self
    }

    /// Refresh button on/off. On by default with the "Refresh" label
    /// (use `refresh_label` to rename); off removes the button.
    pub fn refresh(mut self, enabled: bool) -> Self {
        self.refresh_label = enabled.then(|| "Refresh".to_string());
        self.rebuild();
        self
    }

    /// Refresh button label (implies on).
    pub fn refresh_label(mut self, label: impl Into<String>) -> Self {
        self.refresh_label = Some(label.into());
        self.rebuild();
        self
    }

    /// Refresh button accent (system blue by default).
    pub fn accent(mut self, color: Color) -> Self {
        self.accent = color;
        self.rebuild();
        self
    }

    /// Live theme for texts and the refresh button.
    pub fn set_theme(&mut self, mode: ThemeMode, accent: Color) {
        self.dark = mode == ThemeMode::Dark;
        self.accent = accent;
        self.rebuild();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.apply_state();
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.rebuild();
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.rebuild();
    }

    pub fn set_refresh(&mut self, enabled: bool) {
        self.refresh_label = enabled.then(|| "Refresh".to_string());
        self.rebuild();
    }

    /// True while a refresh is in flight (set by `take_refreshed`,
    /// cleared by `finish_refresh`).
    pub fn is_refreshing(&self) -> bool {
        self.refreshing
    }

    /// Poll a refresh press (true once per click). Marks the refresh
    /// in flight; the app reloads and calls `finish_refresh` when
    /// done.
    pub fn take_refreshed(&mut self) -> bool {
        if *self.fired.borrow() {
            *self.fired.borrow_mut() = false;
            self.refreshing = true;
            return true;
        }
        false
    }

    /// End the in-flight refresh (content reloaded).
    pub fn finish_refresh(&mut self) {
        self.refreshing = false;
    }

    /// Forward hover to the refresh button (nothing when off).
    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.each_button(|button| button.set_hover(x, y));
    }

    pub fn icon_value(&self) -> &str {
        &self.icon
    }

    pub fn refresh_visible(&self) -> bool {
        self.refresh_label.is_some()
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn rebuild(&mut self) {
        let icon_tint = self.icon_color.unwrap_or(UNAVAILABLE_ICON_GRAY);
        let mut stack = VStack::new()
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
            );
        if let Some(label) = self.refresh_label.clone() {
            let fired = self.fired.clone();
            stack = stack
                .child(Gap::fixed(UNAVAILABLE_BUTTON_GAP))
                .child(
                    Button::new(label)
                        .style(ButtonStyle::BorderedProminent)
                        .on_press(move || {
                            *fired.borrow_mut() = true;
                        }),
                );
        }
        self.stack = stack;
        self.apply_state();
    }

    fn apply_state(&mut self) {
        // Theme/focus reach the parts through typed access; the
        // stack order is fixed by `rebuild`.
        for index in 0..self.stack.len() {
            if let Some(symbol) = self.stack.child_mut::<SFSymbolImage>(index) {
                symbol.set_focused(self.focused);
            } else if let Some(text) = self.stack.child_mut::<BasicText>(index) {
                text.set_focused(self.focused);
            } else if let Some(button) = self.stack.child_mut::<Button>(index) {
                button.set_theme(self.accent, self.dark);
                button.set_focused(self.focused);
            }
        }
    }

    fn each_button(&mut self, mut f: impl FnMut(&mut Button)) {
        for index in 0..self.stack.len() {
            if let Some(button) = self.stack.child_mut::<Button>(index) {
                f(button);
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

impl View for ContentUnavailable {
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

    fn view() -> ContentUnavailable {
        ContentUnavailable::new(
            "tray",
            "No Data",
            "There is no data to display yet. Pull down to refresh.",
        )
    }

    #[test]
    fn refresh_on_by_default() {
        assert!(view().refresh_visible());
        let off = view().refresh(false);
        assert!(!off.refresh_visible());
    }

    #[test]
    fn icon_and_color_builders() {
        let view = view()
            .icon("star.fill")
            .icon_color(Color::from_rgb8(0xff, 0x9f, 0x0a));
        assert_eq!(view.icon_value(), "star.fill");
    }

    #[test]
    fn refresh_press_reports_once() {
        let mut view = view();
        let mut fonts = FontSystem::new();
        let (w, h) = view.measure(&mut fonts);
        view.place(&mut fonts, 0.0, 0.0, w.max(400.0), h);
        // Last stack child is the refresh button: click its center.
        let mut bx = 0.0;
        let mut by = 0.0;
        let mut bw = 0.0;
        let mut bh = 0.0;
        for index in 0..view.stack.len() {
            if let Some(button) = view.stack.child_mut::<Button>(index) {
                (bx, by, bw, bh) = button.rect();
            }
        }
        assert!(bw > 0.0);
        view.mouse_down((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        view.mouse_up((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        assert!(view.take_refreshed());
        assert!(view.is_refreshing());
        // Second poll is quiet.
        assert!(!view.take_refreshed());
        view.finish_refresh();
        assert!(!view.is_refreshing());
    }

    #[test]
    fn refresh_off_has_no_button() {
        let mut view = view().refresh(false);
        let mut fonts = FontSystem::new();
        let (w, h) = view.measure(&mut fonts);
        view.place(&mut fonts, 0.0, 0.0, w.max(400.0), h);
        let mut found = false;
        for index in 0..view.stack.len() {
            if view.stack.child_mut::<Button>(index).is_some() {
                found = true;
            }
        }
        assert!(!found);
        view.mouse_down(200.0, 300.0);
        view.mouse_up(200.0, 300.0);
        assert!(!view.take_refreshed());
    }
}
