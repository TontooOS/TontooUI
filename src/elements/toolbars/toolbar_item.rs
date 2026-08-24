//! ToolbarItem — a single item inside a [`Toolbar`].
//!
//! Mirrors the SwiftUI toolbar item behavior: flat glyph button that shares
//! the glass capsule background with its group (`sharedBackgroundVisibility`),
//! placeable in the title area (`ToolbarItemPlacement::Principal`) and
//! hideable (`ToolbarItemHiddenModifier`).

use std::sync::Arc;

use uikit::app::ColorScheme;
use uikit::style::{Padding, Rect, Size};
use uikit::view::ViewContent;
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use gtk::prelude::*;
use gtk::{Button as GtkButton, Image as GtkImage, Label as GtkLabel};

use super::common::{ToolbarItemPlacement, toolbar_glyph_color};

/// One item in a [`Toolbar`](super::Toolbar).
pub struct ToolbarItem {
    id: WidgetId,
    label: String,
    icon: Option<String>,
    content: Option<Box<dyn Widget>>,
    placement: ToolbarItemPlacement,
    shared_background: bool,
    hidden: bool,
    color_scheme: Option<ColorScheme>,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ToolbarItem {
    /// Create an item with an SF Symbol-style icon (CoreIcon asset).
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            label: String::new(),
            icon: Some(icon.into()),
            content: None,
            placement: ToolbarItemPlacement::Automatic,
            shared_background: true,
            hidden: false,
            color_scheme: None,
            on_click: None,
        }
    }

    /// Optional text label next to the icon (or alone with an empty icon).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Replace the icon/label with custom widget content (e.g. for
    /// `ToolbarItemPlacement::Principal` title-area content).
    pub fn content(mut self, widget: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(widget));
        self
    }

    /// Set the item placement (title area via [`ToolbarItemPlacement::Principal`]).
    pub fn placement(mut self, placement: ToolbarItemPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Control the shared glass background visibility for this item
    /// (`false` renders the bare glyph without glass, mirroring
    /// `sharedBackgroundVisibility(.hidden)`).
    pub fn shared_background(mut self, visible: bool) -> Self {
        self.shared_background = visible;
        self
    }

    /// Hide the item entirely (mirrors `toolbarItemHidden(_:)`).
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Force a color scheme (defaults to detecting the system scheme).
    pub fn color_scheme(mut self, c: ColorScheme) -> Self {
        self.color_scheme = Some(c);
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }

    pub(crate) fn placement_role(&self) -> ToolbarItemPlacement {
        self.placement
    }

    pub(crate) fn shares_background(&self) -> bool {
        self.shared_background
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.hidden
    }

    fn render_item(&self) -> gtk::Widget {
        if let Some(content) = &self.content {
            return content.to_gtk();
        }

        let dark = crate::elements::resolve_scheme(self.color_scheme) == ColorScheme::Dark;
        let glyph = toolbar_glyph_color(dark);

        let btn = GtkButton::new();
        let inner = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        inner.set_halign(gtk::Align::Center);
        inner.set_valign(gtk::Align::Center);

        #[cfg(feature = "coreicon")]
        if let Some(symbol) = &self.icon {
            if let Some(path) =
                crate::elements::buttons::common::sf_icon_path(symbol, glyph)
            {
                let img = GtkImage::from_file(&path);
                img.set_pixel_size(17);
                inner.append(&img);
            }
        }
        if !self.label.is_empty() {
            inner.append(&GtkLabel::new(Some(&self.label)));
        }
        if inner.first_child().is_some() {
            btn.set_child(Some(&inner));
        }

        let fg = format!("rgb({},{},{})", glyph.0, glyph.1, glyph.2);
        let css = format!(
            "button {{
                background: rgba(0,0,0,0);
                border: none;
                border-radius: 9999px;
                padding: 5px;
                min-width: 28px;
                min-height: 28px;
            }}
            button:hover {{ background: rgba(255,255,255,0.10); }}
            button:active {{ background: rgba(255,255,255,0.18); }}
            button label {{
                color: {fg};
                font-family: 'SF Pro Text';
                font-size: 14px;
            }}",
            fg = fg
        );
        uikit::widget::apply_css(&btn, &css);

        if let Some(handler) = &self.on_click {
            let handler = handler.clone();
            btn.connect_clicked(move |_| handler());
        }

        btn.upcast()
    }
}

impl ViewContent for ToolbarItem {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        self.render_item()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        if self.content.is_some() {
            return Size::new(40.0, 32.0);
        }
        let text_w = self.label.chars().count() as f32 * 8.0;
        Size::new(38.0 + text_w, 32.0)
    }
}

impl Widget for ToolbarItem {
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
        self.render_item()
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}
