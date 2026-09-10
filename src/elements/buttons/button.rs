//! Button — the core SwiftUI-style push button.
//!
//! Recreates the SwiftUI `Button` API surface: roles, styles, tint, border
//! shapes and sizing. Roles without a label resolve to their system default
//! labels (Cancel / Close / Done / Delete).
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let ok = Button::new("Tap Me")
//!     .style(ButtonStyle::BorderedProminent)
//!     .tint(Color::from_rgb(0, 122, 255))
//!     .on_click(|| println!("tapped"));
//!
//! let del = Button::new("").role(ButtonRole::Destructive);
//! ```

use std::sync::Arc;

use uikit::app::ColorScheme;
use uikit::style::{Color, Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use super::common::{
    ButtonBorderShape, ButtonRole, ButtonSizing, ButtonStyle, effective_tint_hex, render_button_widget,
    resolve_colors,
};

/// SwiftUI-style push button.
pub struct Button {
    id: WidgetId,
    label: String,
    role: Option<ButtonRole>,
    style: ButtonStyle,
    tint: Option<Color>,
    border_shape: ButtonBorderShape,
    sizing: ButtonSizing,
    icon: Option<String>,
    icon_size: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
    color_scheme: Option<ColorScheme>,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Button {
    /// Create a button with the given label (empty label + role resolves to
    /// the role's system default label).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            label: label.into(),
            role: None,
            style: ButtonStyle::Automatic,
            tint: None,
            border_shape: ButtonBorderShape::Automatic,
            sizing: ButtonSizing::Automatic,
            icon: None,
            icon_size: None,
            width: None,
            height: None,
            color_scheme: None,
            on_click: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Set the button role (affects tint and default label).
    pub fn role(mut self, role: ButtonRole) -> Self {
        self.role = Some(role);
        self
    }

    /// Set the visual style.
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Override the tint color (defaults to the role/system blue).
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    /// Set the border shape.
    pub fn border_shape(mut self, shape: ButtonBorderShape) -> Self {
        self.border_shape = shape;
        self
    }

    /// Set the sizing behavior.
    pub fn sizing(mut self, sizing: ButtonSizing) -> Self {
        self.sizing = sizing;
        self
    }

    /// Show an SF Symbol-style icon (from CoreIcon assets) before the label.
    /// An empty label with an icon renders an icon-only button.
    pub fn icon(mut self, symbol: impl Into<String>) -> Self {
        self.icon = Some(symbol.into());
        self
    }

    /// Glyph size in pixels (default 15).
    pub fn icon_size(mut self, px: f32) -> Self {
        self.icon_size = Some(px);
        self
    }

    /// Force a fixed size.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
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

    /// The effective label: the configured label or the role default.
    pub fn effective_label(&self) -> String {
        if self.label.trim().is_empty() {
            self.role
                .map(|r| r.default_label().to_string())
                .unwrap_or_default()
        } else {
            self.label.clone()
        }
    }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 120.0, 34.0)
    }
}

impl ViewContent for Button {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        let scheme = crate::elements::resolve_scheme(self.color_scheme);
        let dark = scheme == ColorScheme::Dark;
        let tint = effective_tint_hex(self.role, self.tint.as_ref(), dark);
        let colors = resolve_colors(self.style, &tint, dark);
        let label = self.effective_label();
        render_button_widget(
            &label,
            &self.icon,
            self.icon_size.unwrap_or(15.0),
            &colors,
            self.border_shape,
            self.sizing,
            self.width,
            self.height,
            &self.on_click,
        )
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        let label = self.effective_label();
        let text_w = label.chars().count() as f32 * 8.5;
        let icon_w = if self.icon.is_some() { 21.0 } else { 0.0 };
        let w = self
            .width
            .unwrap_or((text_w + icon_w + 28.0).max(34.0));
        let circle = super::common::effective_shape(self.border_shape, !label.is_empty())
            == super::common::ButtonBorderShape::Circle;
        let h = self.height.unwrap_or(if circle {
            w
        } else {
            32.0
        });
        Size::new(w, h)
    }
}

impl Widget for Button {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_label_falls_back_to_role() {
        assert_eq!(
            Button::new("").role(ButtonRole::Cancel).effective_label(),
            "Cancel"
        );
        assert_eq!(
            Button::new("Tap").role(ButtonRole::Cancel).effective_label(),
            "Tap"
        );
    }

    #[test]
    fn builder_fields() {
        let b = Button::new("Tap Me")
            .style(ButtonStyle::BorderedProminent)
            .tint(Color::from_rgb(48, 209, 88))
            .border_shape(ButtonBorderShape::RoundedRectangle(8.0))
            .sizing(ButtonSizing::Flexible)
            .icon("checkmark");
        assert_eq!(b.style, ButtonStyle::BorderedProminent);
        assert_eq!(b.border_shape, ButtonBorderShape::RoundedRectangle(8.0));
        assert_eq!(b.sizing, ButtonSizing::Flexible);
        assert_eq!(b.icon.as_deref(), Some("checkmark"));
        assert_eq!(b.tint, Some(Color::from_rgb(48, 209, 88)));
    }

    #[test]
    fn icon_size_builder() {
        assert_eq!(Button::new("").icon_size, None);
        let b = Button::new("").icon("plus").icon_size(40.0);
        assert_eq!(b.icon_size, Some(40.0));
    }

    #[test]
    fn destructive_role_maps_to_red() {
        let b = Button::new("Delete").role(ButtonRole::Destructive);
        assert_eq!(
            effective_tint_hex(b.role, b.tint.as_ref(), true),
            super::super::common::RED_DARK
        );
        let c = Button::new("Tap");
        assert_eq!(
            effective_tint_hex(c.role, c.tint.as_ref(), true),
            super::super::common::BLUE_DARK
        );
    }
}
