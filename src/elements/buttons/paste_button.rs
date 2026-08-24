//! PasteButton — SwiftUI system pasteboard button.
//!
//! A prominent blue button with a clipboard icon labeled `Paste`. On click it
//! reads the system clipboard asynchronously and delivers the text to the
//! handler, mirroring `SwiftUI.PasteButton`.

use std::sync::Arc;

use uikit::app::ColorScheme;
use uikit::style::{Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use gtk::prelude::*;
use gtk::{Button as GtkButton, Image as GtkImage, Label as GtkLabel, Orientation};

use super::common::{BLUE_DARK, BLUE_LIGHT, resolve_colors, sf_icon_path, ButtonStyle};

/// SwiftUI `PasteButton` — a system button that reads text from the
/// clipboard (pasteboard) and delivers it to the given handler.
pub struct PasteButton {
    id: WidgetId,
    color_scheme: Option<ColorScheme>,
    on_paste: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl PasteButton {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            color_scheme: None,
            on_paste: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn color_scheme(mut self, c: ColorScheme) -> Self {
        self.color_scheme = Some(c);
        self
    }

    /// Set the handler receiving the clipboard text (None when the
    /// pasteboard holds no text).
    pub fn on_paste(mut self, handler: impl Fn(Option<String>) + Send + Sync + 'static) -> Self {
        self.on_paste = Some(Arc::new(handler));
        self
    }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 90.0, 32.0)
    }
}

impl Default for PasteButton {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewContent for PasteButton {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let dark = crate::elements::resolve_scheme(self.color_scheme) == ColorScheme::Dark;
        let tint = if dark { BLUE_DARK } else { BLUE_LIGHT };
        let colors = resolve_colors(ButtonStyle::BorderedProminent, tint, dark);

        let btn = GtkButton::new();
        let content = gtk::Box::new(Orientation::Horizontal, 6);
        content.set_halign(gtk::Align::Center);
        content.set_valign(gtk::Align::Center);

        #[cfg(feature = "coreicon")]
        if let Some(path) = sf_icon_path("doc.on.doc.fill", (255, 255, 255)) {
            let img = GtkImage::from_file(&path);
            img.set_pixel_size(14);
            content.append(&img);
        }
        content.append(&GtkLabel::new(Some("Paste")));
        btn.set_child(Some(&content));

        let css = format!(
            "button {{
                background: {bg};
                border: none;
                border-radius: 7px;
                color: #FFFFFF;
                font-family: 'SF Pro Text';
                font-size: 14px;
                font-weight: 500;
                padding: 5px 12px;
                min-height: 26px;
            }}
            button:hover {{ filter: brightness(1.15); }}
            button:active {{ filter: brightness(0.85); }}
            button label {{ color: #FFFFFF; }}",
            bg = colors.background
        );
        uikit::widget::apply_css(&btn, &css);

        if let Some(handler) = &self.on_paste {
            let handler = handler.clone();
            btn.connect_clicked(move |b| {
                let handler = handler.clone();
                let clipboard = b.clipboard();
                clipboard.read_text_async(None::<&gtk::gio::Cancellable>, move |res| {
                    handler(res.ok().flatten().map(|s| s.to_string()));
                });
            });
        }

        btn.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(90.0, 32.0)
    }
}

impl Widget for PasteButton {
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
