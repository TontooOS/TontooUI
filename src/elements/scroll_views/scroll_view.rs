//! ScrollView — SwiftUI-style scrollable container with edge effects.
//! Own category/folder. Light/Dark adaptive, SF Pro for content.

use uikit::app::ColorScheme;
use uikit::style::{Rect, Size, Padding};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollEdgeEffect { Soft, Hard }
impl Default for ScrollEdgeEffect { fn default() -> Self { Self::Soft } }

pub struct ScrollView {
    id: WidgetId,
    content: Option<Box<dyn Widget>>,
    horizontal: bool,
    vertical: bool,
    edge_effect: ScrollEdgeEffect,
    color_scheme: Option<ColorScheme>,
    position_mode: PositionMode,
    position: Position,
}

impl ScrollView {
    pub fn new() -> Self {
        Self { id: next_widget_id(), content: None, horizontal: false, vertical: true, edge_effect: ScrollEdgeEffect::Soft, color_scheme: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn content(mut self, w: impl Widget + 'static) -> Self { self.content = Some(Box::new(w)); self }
    pub fn view_content(mut self, v: View) -> Self {
        struct ViewWidget(View);
        impl Widget for ViewWidget {
            fn id(&self) -> WidgetId { self.0.id() }
            fn to_gtk(&self) -> gtk::Widget { self.0.to_gtk() }
            fn padding(&self) -> Padding { Padding::ZERO }
        }
        self.content = Some(Box::new(ViewWidget(v)));
        self
    }
    pub fn horizontal(mut self, h: bool) -> Self { self.horizontal = h; self }
    pub fn vertical(mut self, v: bool) -> Self { self.vertical = v; self }
    pub fn edge_effect(mut self, e: ScrollEdgeEffect) -> Self { self.edge_effect = e; self }
    /// Hard edge — SwiftUI `ScrollView` with hard cutoff dividing line (as in screenshot).
    pub fn hard_edge(mut self) -> Self { self.edge_effect = ScrollEdgeEffect::Hard; self }
    pub fn color_scheme(mut self, c: ColorScheme) -> Self { self.color_scheme = Some(c); self }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0, 0.0, 360.0, 220.0) }
}
impl Default for ScrollView {
    fn default() -> Self { Self::new() }
}

impl ViewContent for ScrollView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(self.color_scheme) == ColorScheme::Dark;
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_hscrollbar_policy(if self.horizontal { gtk::PolicyType::Automatic } else { gtk::PolicyType::Never });
        scrolled.set_vscrollbar_policy(if self.vertical { gtk::PolicyType::Automatic } else { gtk::PolicyType::Never });
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        if frame.width > 0.0 { scrolled.set_width_request(frame.width as i32); }
        if frame.height > 0.0 { scrolled.set_height_request(frame.height as i32); }

        // Edge effect styling — Hard shows a dividing line at the top edge (hard cutoff)
        if self.edge_effect == ScrollEdgeEffect::Hard {
            let line_col = if is_dark { "rgba(255,255,255,0.08)" } else { "rgba(0,0,0,0.08)" };
            let bg = if is_dark { "#1d1d1d" } else { "#ececec" };
            // ScrolledWindow styling: add top border line for hard effect
            let css = format!("scrolledwindow {{ background: {bg}; }} scrolledwindow > viewport {{ background: {bg}; }} .hard-edge {{ border-top: 1px solid {line_col}; }}");
            uikit::widget::apply_css(&scrolled, &css);
        }

        if let Some(ref content) = self.content {
            let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
            // For Hard edge, add a hard dividing line at the top of the scroll content
            if self.edge_effect == ScrollEdgeEffect::Hard {
                let top_line = gtk::Separator::new(gtk::Orientation::Horizontal);
                let col = if is_dark { "rgba(255,255,255,0.10)" } else { "rgba(0,0,0,0.10)" };
                uikit::widget::apply_css(&top_line, &format!("separator {{ background: {col}; min-height: 1px; }}"));
                outer.append(&top_line);
            }
            let child = content.to_gtk();
            child.set_hexpand(true);
            child.set_vexpand(true);
            outer.append(&child);
            scrolled.set_child(Some(&outer));
        }

        scrolled.upcast()
    }
    fn size_that_fits(&self, available: Size) -> Size { available }
}

impl Widget for ScrollView {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget {
        self.render(Rect::new(0.0, 0.0, 360.0, 220.0))
    }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> Padding { Padding::ZERO }
    /// Forward into the scrolled content so views that hide the window bar
    /// (e.g. `Sidebar`) are found even when nested in a `ScrollView`.
    fn children(&self) -> Vec<&dyn Widget> {
        self.content.iter().map(|c| c.as_ref()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scroll_edge_hard() {
        let s = ScrollView::new().hard_edge();
        assert_eq!(s.edge_effect, ScrollEdgeEffect::Hard);
    }

    #[test]
    fn scroll_view_forwards_children() {
        struct Leaf;
        impl Widget for Leaf {
            fn id(&self) -> WidgetId {
                0
            }
            fn to_gtk(&self) -> gtk::Widget {
                unimplemented!()
            }
            fn hides_window_bar(&self) -> bool {
                true
            }
        }
        assert!(ScrollView::new().children().is_empty());
        let s = ScrollView::new().content(Leaf);
        assert_eq!(s.children().len(), 1);
        assert!(s.hides_window_bar_recursive());
    }
}
