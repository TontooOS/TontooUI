//! EditButton — SwiftUI system edit-mode toggle button.
//!
//! Toggles between the `Edit` and `Done` labels on every click, mirroring
//! `SwiftUI.EditButton`. The label is updated in place on the rendered
//! `GtkButton` (via a weak reference), so repeated toggling does not rebuild
//! the widget tree.

use std::cell::RefCell;
use std::rc::Rc;

use uikit::app::ColorScheme;
use uikit::style::{Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use gtk::prelude::*;
use gtk::Button as GtkButton;

use super::common::{BLUE_DARK, BLUE_LIGHT};

/// SwiftUI `EditButton` — toggles between the "Edit" and "Done" states.
pub struct EditButton {
    id: WidgetId,
    editing: Rc<RefCell<bool>>,
    color_scheme: Option<ColorScheme>,
    live: Rc<RefCell<Option<glib::WeakRef<GtkButton>>>>,
    position_mode: PositionMode,
    position: Position,
}

impl EditButton {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            editing: Rc::new(RefCell::new(false)),
            color_scheme: None,
            live: Rc::new(RefCell::new(None)),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn color_scheme(mut self, c: ColorScheme) -> Self {
        self.color_scheme = Some(c);
        self
    }

    /// Whether the button currently shows the "Done" (editing) state.
    pub fn is_editing(&self) -> bool {
        *self.editing.borrow()
    }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 60.0, 30.0)
    }
}

impl Default for EditButton {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for EditButton {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let dark = crate::elements::resolve_scheme(self.color_scheme) == ColorScheme::Dark;
        let tint = if dark { BLUE_DARK } else { BLUE_LIGHT };

        let btn = GtkButton::with_label(if *self.editing.borrow() {
            "Done"
        } else {
            "Edit"
        });
        let css = format!(
            "button {{
                background: rgba(0,0,0,0);
                border: none;
                color: {tint};
                font-family: 'SF Pro Text';
                font-size: 15px;
                font-weight: 500;
                padding: 6px 10px;
                min-height: 24px;
            }}
            button:hover {{ filter: brightness(1.2); }}
            button label {{ color: {tint}; }}",
            tint = tint
        );
        uikit::widget::apply_css(&btn, &css);

        let weak = glib::WeakRef::new();
        weak.set(Some(&btn));

        let editing = self.editing.clone();
        btn.connect_clicked(move |b| {
            let now = !*editing.borrow();
            *editing.borrow_mut() = now;
            b.set_label(if now { "Done" } else { "Edit" });
        });
        *self.live.borrow_mut() = Some(weak);

        btn.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(60.0, 30.0)
    }
}

impl Widget for EditButton {
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
