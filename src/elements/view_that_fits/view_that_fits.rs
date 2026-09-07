//! ViewThatFits — SwiftUI-style adaptive container.
//! Picks the first child that fits the available space, falling back to the last.
//! Dark #1d1d1d / Light #ececec, SF Pro for any Text children (handled by Text itself).

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewThatFitsAxis {
    Horizontal,
    Vertical,
    Both,
}

impl Default for ViewThatFitsAxis {
    fn default() -> Self { Self::Both }
}

pub struct ViewThatFits {
    id: WidgetId,
    axis: ViewThatFitsAxis,
    children: Vec<Box<dyn Widget>>,
    position_mode: PositionMode,
    position: Position,
}

impl ViewThatFits {
    pub fn new() -> Self {
        Self { id: next_widget_id(), axis: ViewThatFitsAxis::Both, children: Vec::new(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn horizontal() -> Self {
        Self { id: next_widget_id(), axis: ViewThatFitsAxis::Horizontal, children: Vec::new(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn vertical() -> Self {
        Self { id: next_widget_id(), axis: ViewThatFitsAxis::Vertical, children: Vec::new(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn axis(mut self, a: ViewThatFitsAxis) -> Self { self.axis = a; self }
    pub fn child(mut self, w: impl Widget + 'static) -> Self { self.children.push(Box::new(w)); self }
    pub fn children(mut self, v: Vec<Box<dyn Widget>>) -> Self { self.children = v; self }
}

impl Default for ViewThatFits {
    fn default() -> Self { Self::new() }
}

// Pick first child that fits available size. Uses Gtk measure for natural size.
fn pick_index(children: &[Box<dyn Widget>], available: Size, axis: ViewThatFitsAxis) -> usize {
    if children.is_empty() { return 0; }
    for (i, c) in children.iter().enumerate() {
        let gtk_w = c.to_gtk();
        // measure natural size
        let (_, nat_w, _, _) = gtk_w.measure(gtk::Orientation::Horizontal, -1);
        let (_, nat_h, _, _) = gtk_w.measure(gtk::Orientation::Vertical, -1);
        let fits = match axis {
            ViewThatFitsAxis::Horizontal => (nat_w as f32) <= available.width || available.width <= 0.0,
            ViewThatFitsAxis::Vertical => (nat_h as f32) <= available.height || available.height <= 0.0,
            ViewThatFitsAxis::Both => {
                let w_ok = available.width <= 0.0 || (nat_w as f32) <= available.width;
                let h_ok = available.height <= 0.0 || (nat_h as f32) <= available.height;
                w_ok && h_ok
            }
        };
        if fits {
            return i;
        }
    }
    // fallback to last
    children.len() - 1
}

impl ViewContent for ViewThatFits {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let available = Size::new(frame.width.max(0.0), frame.height.max(0.0));
        // If no children, return empty box
        if self.children.is_empty() {
            return gtk::Box::new(gtk::Orientation::Vertical, 0).upcast();
        }
        // If frame is zero (Widget path), fallback to first child
        if frame.width <= 0.0 && frame.height <= 0.0 {
            return self.children[0].to_gtk();
        }
        let idx = pick_index(&self.children, available, self.axis);
        self.children[idx].to_gtk()
    }

    fn size_that_fits(&self, available: Size) -> Size {
        if self.children.is_empty() { return Size::ZERO; }
        let idx = pick_index(&self.children, available, self.axis);
        // Return natural size of chosen child approximated via gtk measure
        let gtk_w = self.children[idx].to_gtk();
        let (_, nat_w, _, _) = gtk_w.measure(gtk::Orientation::Horizontal, -1);
        let (_, nat_h, _, _) = gtk_w.measure(gtk::Orientation::Vertical, -1);
        Size::new(nat_w as f32, nat_h as f32)
    }
}

impl Widget for ViewThatFits {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        // Widget path has no frame info — show first child as preview; true fitting happens via ViewContent with frame.
        // For demo on background we want to show the adaptive behavior without a frame,
        // so we wrap in a View that will get a frame, but for direct Widget use we fallback to first.
        if self.children.is_empty() {
            return gtk::Box::new(gtk::Orientation::Vertical, 0).upcast();
        }
        // Return first child's gtk for Widget path; demo uses ViewThatFits via View wrapper with frame to get adaptation.
        // To still demonstrate, we can return a Box that will switch on allocation via size-allocate signal.
        // Simple: return first child now; if used inside a View with frame, ViewContent path will be used.
        self.children[0].to_gtk()
    }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
    fn is_interactive(&self) -> bool { false }
}

impl ViewThatFits {
    pub fn to_view(self) -> View {
        // Keep original children for ViewContent path; estimate size from first child
        let w = 300.0;
        let h = 80.0;
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uikit::widgets::Text;
    #[test]
    fn axis_default() {
        assert_eq!(ViewThatFits::new().axis, ViewThatFitsAxis::Both);
        assert_eq!(ViewThatFits::vertical().axis, ViewThatFitsAxis::Vertical);
    }
    #[test]
    fn child_push() {
        let v = ViewThatFits::new().child(Text::new("A")).child(Text::new("B"));
        assert_eq!(v.children.len(), 2);
    }
}
