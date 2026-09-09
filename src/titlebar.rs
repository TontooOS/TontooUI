//! TitleBar — macOS-style decoration bar with traffic lights and app content.
//!
//! A standalone element so TontooUI apps can build windows with a top bar:
//! traffic lights stay reserved on the left, the app fills the rest via
//! [`content`](TitleBar::content) (replaces the title zone), and the title
//! text can be hidden. This element is unrelated to [`Sidebar`]: the
//! sidebar keeps its own fixed layout and accepts no title-bar content.
//!
//! ```rust,ignore
//! use tontooui::prelude::*;
//!
//! let bar = TitleBar::new()
//!     .title("Finder")
//!     .content(HStack::new()
//!         .spacing(8.0)
//!         .child(Button::new("Share")))
//!     .without_maximize();
//!
//! let view = View::new(bar);
//! ```

use std::rc::Rc;
use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Default bar height in px when no explicit height is set (matches the
/// UIKit standard title bar: 17px lights + 14px padding).
pub const DEFAULT_BAR_HEIGHT: f32 = 31.0;

// ═══════════════════════════════════════════════════════════════
// TitleBar
// ═══════════════════════════════════════════════════════════════

/// macOS-style decoration bar: traffic lights on the left (always
/// reserved), app content filling the rest, optional centered title.
pub struct TitleBar {
    id: WidgetId,
    title: String,
    show_title: bool,
    custom: Option<Rc<dyn Widget>>,
    show_maximize: bool,
    minimize_enabled: bool,
    height: f32,
    position_mode: PositionMode,
    position: Position,
}

impl TitleBar {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            title: String::new(),
            show_title: true,
            custom: None,
            show_maximize: true,
            minimize_enabled: true,
            height: 0.0,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Centered title text (empty = no title). Only renders when no
    /// custom content is set.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Show or hide the title text (default: shown).
    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    /// Hide the title text, keeping only lights (and custom content).
    pub fn without_title(self) -> Self {
        self.show_title(false)
    }

    /// Custom content filling the bar after the reserved traffic lights,
    /// from the left to the right edge. Replaces the title zone; the
    /// content lays out its own alignment.
    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.custom = Some(Rc::new(widget));
        self
    }

    /// Show or hide the green maximize button (default: shown).
    pub fn show_maximize(mut self, show: bool) -> Self {
        self.show_maximize = show;
        self
    }

    /// Hide the green maximize button, keeping only close and minimize.
    pub fn without_maximize(self) -> Self {
        self.show_maximize(false)
    }

    /// Enable or disable the minimize (middle) button (default: enabled).
    /// Disabled stays gray and ignores clicks.
    pub fn minimize_enabled(mut self, enabled: bool) -> Self {
        self.minimize_enabled = enabled;
        self
    }

    /// Explicit bar height in px (`0.0` = auto, default).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Whether custom content is set.
    pub fn has_content(&self) -> bool {
        self.custom.is_some()
    }

    /// Effective bar height in px (explicit height or the default).
    pub fn bar_height(&self) -> f32 {
        if self.height > 0.0 {
            self.height
        } else {
            DEFAULT_BAR_HEIGHT
        }
    }

    pub fn to_view(self) -> View {
        View::new(self)
    }
}

impl Default for TitleBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for TitleBar {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let w = if frame.width > 0.0 {
            frame.width
        } else {
            320.0
        };
        let mut lights = uikit::widgets::TrafficLights::new()
            .with_title(self.title.clone())
            .show_title(self.show_title)
            .show_maximize(self.show_maximize)
            .minimize_enabled(self.minimize_enabled);
        if self.height > 0.0 {
            lights = lights.bar_height(self.height);
        }
        if let Some(custom) = self.custom.clone() {
            lights = lights.with_custom_shared(custom);
        }
        let bar = lights.to_gtk();
        bar.set_hexpand(true);
        bar.set_size_request(w as i32, self.bar_height() as i32);
        bar.add_css_class("uikit-titlebar");
        bar
    }

    fn can_become_first_responder(&self) -> bool {
        false
    }

    fn size_that_fits(&self, available: Size) -> Size {
        Size::new(available.width, self.bar_height())
    }
}

impl Widget for TitleBar {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }
    fn position(&self) -> Position {
        self.position
    }
    fn to_gtk(&self) -> gtk::Widget {
        let w = self.render(Rect::new(0.0, 0.0, 0.0, 0.0));
        w.set_hexpand(true);
        w
    }
    fn is_interactive(&self) -> bool {
        true
    }
    fn fill_width(&self) -> bool {
        true
    }
    fn padding(&self) -> uikit::style::Padding {
        uikit::style::Padding::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titlebar_defaults() {
        let bar = TitleBar::new();
        assert!(bar.title.is_empty());
        assert!(bar.show_title);
        assert!(!bar.has_content());
        assert!(bar.show_maximize);
        assert!(bar.minimize_enabled);
        assert_eq!(bar.bar_height(), DEFAULT_BAR_HEIGHT);
    }

    #[test]
    fn titlebar_builder() {
        let bar = TitleBar::new()
            .title("Finder")
            .without_title()
            .without_maximize()
            .minimize_enabled(false)
            .height(44.0);
        assert_eq!(bar.title, "Finder");
        assert!(!bar.show_title);
        assert!(!bar.show_maximize);
        assert!(!bar.minimize_enabled);
        assert_eq!(bar.bar_height(), 44.0);
    }

    #[test]
    fn titlebar_custom_content() {
        let bar = TitleBar::new().content(uikit::widgets::Text::new("tools"));
        assert!(bar.has_content());
    }

    #[test]
    fn titlebar_default_height() {
        assert_eq!(TitleBar::new().height(0.0).bar_height(), DEFAULT_BAR_HEIGHT);
    }
}
