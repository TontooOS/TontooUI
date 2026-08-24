//! ToolbarSpacer — a standard space item in toolbars.
//!
//! Mirrors `SwiftUI.ToolbarSpacer { sizing: SpacerSizing, placement }` with
//! `SpacerSizing.Kind` of `fixed` / `flexible`. A fixed spacer takes up the
//! standard gap between item groups (closing the shared glass capsule), a
//! flexible spacer expands to fill the remaining bar width.

use gtk::prelude::*;

use uikit::style::{Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use super::common::ToolbarSpacerSizing;

/// SwiftUI `ToolbarSpacer` — space between toolbar item groups.
#[derive(Clone)]
pub struct ToolbarSpacer {
    id: WidgetId,
    sizing: ToolbarSpacerSizing,
}

impl ToolbarSpacer {
    /// A spacer with the standard group gap (closes the glass capsule).
    pub fn fixed() -> Self {
        Self {
            id: next_widget_id(),
            sizing: ToolbarSpacerSizing::Fixed,
        }
    }

    /// A spacer that expands to fill the remaining bar width.
    pub fn flexible() -> Self {
        Self {
            id: next_widget_id(),
            sizing: ToolbarSpacerSizing::Flexible,
        }
    }

    /// The configured sizing.
    pub fn sizing(&self) -> ToolbarSpacerSizing {
        self.sizing
    }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 8.0, 32.0)
    }
}

impl Default for ToolbarSpacer {
    fn default() -> Self {
        Self::fixed()
    }
}

impl ViewContent for ToolbarSpacer {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        if self.sizing == ToolbarSpacerSizing::Flexible {
            box_.set_hexpand(true);
        } else {
            box_.set_width_request(6);
        }
        box_.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        false
    }

    fn size_that_fits(&self, available: Size) -> Size {
        match self.sizing {
            ToolbarSpacerSizing::Fixed => Size::new(6.0, 32.0),
            ToolbarSpacerSizing::Flexible => Size::new(available.width, 32.0),
        }
    }
}

impl Widget for ToolbarSpacer {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn position_mode(&self) -> PositionMode {
        PositionMode::Auto
    }

    fn position(&self) -> Position {
        Position::new()
    }

    fn to_gtk(&self) -> gtk::Widget {
        self.render(Rect::new(0.0, 0.0, 0.0, 0.0))
    }

    fn is_interactive(&self) -> bool {
        false
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}
