//! CustomPhasesAsyncImage — Loads a modifiable image with custom phase handling.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Async load phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AsyncImagePhase {
    #[default]
    Empty,
    Success,
    Failure,
}

/// CustomPhasesAsyncImage — initializer — Loads and displays a modifiable
/// image from the specified URL load request with custom phases.
pub struct CustomPhasesAsyncImage {
    id: WidgetId,
    url: String,
    phase: AsyncImagePhase,
    position_mode: PositionMode,
    position: Position,
}

impl CustomPhasesAsyncImage {
    pub fn new() -> Self {
        Self { id: next_widget_id(), url: "https://example.com/cats.png".to_string(), phase: AsyncImagePhase::Success, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn url(mut self, v: impl Into<String>) -> Self { self.url = v.into(); self }
    pub fn phase(mut self, v: AsyncImagePhase) -> Self { self.phase = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for CustomPhasesAsyncImage { fn default() -> Self { Self::new() } }

impl ViewContent for CustomPhasesAsyncImage {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("ai-phs-phone");
        uikit::widget::apply_css(&phone, ".ai-phs-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        // Top phase band: red for failure/empty, hidden tint for success
        let band = gtk::Box::new(gtk::Orientation::Vertical, 0);
        band.set_size_request(180, 44);
        band.set_halign(gtk::Align::Center);
        band.add_css_class("ai-phs-band");
        let band_col = match self.phase {
            AsyncImagePhase::Success => "#e5484d",
            AsyncImagePhase::Failure => "#5a1a1d",
            AsyncImagePhase::Empty => "#2c2c2e",
        };
        uikit::widget::apply_css(&band, &format!(".ai-phs-band {{ background: {}; border-radius: 12px 12px 0 0; min-width: 180px; min-height: 44px; }}", band_col));
        phone.append(&band);
        // Bottom small cats row (success content)
        let img = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        img.set_halign(gtk::Align::Center);
        img.set_size_request(180, 44);
        for (i, col) in ["#d8b48a", "#c9a86a", "#e8c9a0"].iter().enumerate() {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(24, 30);
            cat.set_valign(gtk::Align::Center);
            cat.add_css_class(&format!("ai-phs-cat{}", i));
            uikit::widget::apply_css(&cat, &format!(".ai-phs-cat{} {{ background: {}; border-radius: 6px; min-width: 24px; min-height: 30px; margin-top: 7px; }}", i, col));
            img.append(&cat);
        }
        phone.append(&img);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CustomPhasesAsyncImage {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 180.0, 110.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phases_exists() { let _ = CustomPhasesAsyncImage::new().phase(AsyncImagePhase::Empty); }
}
