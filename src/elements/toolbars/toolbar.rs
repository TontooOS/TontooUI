//! Toolbar — a SwiftUI-style glass capsule toolbar bar.
//!
//! Mirrors the iOS 26 Liquid Glass toolbar: consecutive items with a shared
//! background merge into one glass capsule, [`ToolbarSpacer`]s separate
//! groups (fixed) or expand (flexible), and items placed with
//! [`ToolbarItemPlacement::Principal`] render in the leading title area.

use uikit::app::ColorScheme;
use uikit::style::{Padding, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use gtk::prelude::*;
use gtk::Orientation;

use super::common::{ToolbarItemPlacement, glass_capsule_css, transparent_capsule_css};
use super::toolbar_item::ToolbarItem;
use super::toolbar_spacer::ToolbarSpacer;

pub(crate) enum ToolbarEntry {
    Item(ToolbarItem),
    Spacer(ToolbarSpacer),
}

/// SwiftUI-style toolbar bar containing items and spacers.
pub struct Toolbar {
    id: WidgetId,
    entries: Vec<ToolbarEntry>,
    color_scheme: Option<ColorScheme>,
    width: Option<f32>,
    transparent: bool,
    position_mode: PositionMode,
    position: Position,
}

impl Toolbar {
    /// Create an empty toolbar.
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            entries: Vec::new(),
            color_scheme: None,
            width: None,
            transparent: false,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    /// Append an item to the bar.
    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.entries.push(ToolbarEntry::Item(item));
        self
    }

    /// Append a spacer to the bar.
    pub fn spacer(mut self, spacer: ToolbarSpacer) -> Self {
        self.entries.push(ToolbarEntry::Spacer(spacer));
        self
    }

    /// Force a color scheme (defaults to detecting the system scheme).
    pub fn color_scheme(mut self, c: ColorScheme) -> Self {
        self.color_scheme = Some(c);
        self
    }

    /// Force a bar width (defaults to the natural content width).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Transparent groups: same pill shape, fully transparent
    /// background, no border, no shadow.
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    /// Whether groups render transparent.
    pub fn is_transparent(&self) -> bool {
        self.transparent
    }

    /// Create a View wrapping this element.
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 120.0, 40.0)
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

/// Seal the current group: style it as a glass capsule (when non-empty) and
/// parent it to the bar.
fn flush_group(group: &gtk::Box, bar: &gtk::Box, dark: bool, transparent: bool) {
    if group.first_child().is_some() {
        group.add_css_class("tb-group");
        if transparent {
            uikit::widget::apply_css(group, &transparent_capsule_css());
        } else {
            uikit::widget::apply_css(group, &glass_capsule_css(dark));
        }
        bar.append(group);
    }
}

impl Toolbar {
    fn render_bar(&self) -> gtk::Widget {
        let dark = crate::elements::resolve_scheme(self.color_scheme) == ColorScheme::Dark;

        let root = gtk::Box::new(Orientation::Horizontal, 0);
        if let Some(w) = self.width {
            root.set_width_request(w as i32);
        }

        let title_area = gtk::Box::new(Orientation::Horizontal, 8);
        title_area.set_valign(gtk::Align::Center);
        let bar = gtk::Box::new(Orientation::Horizontal, 6);
        bar.set_valign(gtk::Align::Center);

        let mut group = gtk::Box::new(Orientation::Horizontal, 0);

        for entry in &self.entries {
            match entry {
                ToolbarEntry::Item(item) => {
                    if item.is_hidden() {
                        continue;
                    }
                    let widget = item.to_gtk();
                    if item.placement_role() == ToolbarItemPlacement::Principal {
                        flush_group(&group, &bar, dark, self.transparent);
                        group = gtk::Box::new(Orientation::Horizontal, 0);
                        title_area.append(&widget);
                    } else if !item.shares_background() {
                        flush_group(&group, &bar, dark, self.transparent);
                        group = gtk::Box::new(Orientation::Horizontal, 0);
                        bar.append(&widget);
                    } else {
                        group.append(&widget);
                    }
                }
                ToolbarEntry::Spacer(spacer) => {
                    flush_group(&group, &bar, dark, self.transparent);
                    group = gtk::Box::new(Orientation::Horizontal, 0);
                    bar.append(&spacer.to_gtk());
                }
            }
        }
        flush_group(&group, &bar, dark, self.transparent);

        root.append(&title_area);
        root.append(&bar);

        root.upcast()
    }
}

impl ViewContent for Toolbar {
    fn render(&self, _frame: Rect) -> gtk::Widget {
        self.render_bar()
    }

    fn can_become_first_responder(&self) -> bool {
        false
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        let items = self
            .entries
            .iter()
            .map(|e| match e {
                ToolbarEntry::Item(i) if !i.is_hidden() => 1.0,
                ToolbarEntry::Spacer(_) => 1.0,
                _ => 0.0,
            })
            .sum::<f32>();
        Size::new(self.width.unwrap_or((items * 38.0).max(40.0)), 40.0)
    }
}

impl Widget for Toolbar {
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
    fn transparent_builder() {
        assert!(!Toolbar::new().is_transparent());
        assert!(Toolbar::new().transparent().is_transparent());
    }
}
