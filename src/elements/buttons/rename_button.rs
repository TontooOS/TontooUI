//! RenameButton — SwiftUI system rename button.
//!
//! A plain tinted button with a pencil icon labeled `Rename` that triggers a
//! standard rename action, mirroring `SwiftUI.RenameButton`.

use std::sync::Arc;

use uikit::app::ColorScheme;
use uikit::style::{Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use super::button::Button;
use super::common::ButtonStyle;

/// SwiftUI `RenameButton` — a plain tinted button with a pencil icon that
/// triggers a standard rename action.
pub struct RenameButton {
    id: WidgetId,
    color_scheme: Option<ColorScheme>,
    on_rename: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl RenameButton {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            color_scheme: None,
            on_rename: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn color_scheme(mut self, c: ColorScheme) -> Self {
        self.color_scheme = Some(c);
        self
    }

    pub fn on_rename(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_rename = Some(Arc::new(handler));
        self
    }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 90.0, 30.0)
    }
}

impl Default for RenameButton {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for RenameButton {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let mut b = Button::new("Rename")
            .style(ButtonStyle::Plain)
            .icon("pencil");
        if let Some(s) = self.color_scheme {
            b = b.color_scheme(s);
        }
        if let Some(h) = &self.on_rename {
            let h = h.clone();
            b = b.on_click(move || h());
        }
        b.to_gtk()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(90.0, 30.0)
    }
}

impl Widget for RenameButton {
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
        self.render(Rect::new(0.0, 0.0, 0.0, 0.0))
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}
