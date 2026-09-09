//! Tab — a single tab inside a `TabView`: label, icon and detail content.
//!
//! Mirrors SwiftUI's `Tab`:
//!
//! ```rust,ignore
//! use tontooui::prelude::*;
//! use tontooui::{Tab, TabSection, TabView};
//!
//! let view = TabView::new()
//!     .title("ExploreSwiftUISandbox")
//!     .tab(Tab::new(Text::new("0").font_size(28.0)))
//!     .section(TabSection::new().header("Foo").tab(
//!         Tab::new(Text::new("1").font_size(28.0))
//!             .title("1")
//!             .system_image("1.circle"),
//!     ))
//!     .tab(Tab::new(Text::new("2").font_size(28.0))
//!         .title("2")
//!         .image("cats24x24"))
//!     .tab(Tab::new(Text::new("3").font_size(28.0))
//!         .title("3")
//!         .system_image("3.circle"))
//!     .to_view();
//! ```
//!
//! A tab without title or icon (like the first `Tab` above) renders as an
//! empty row that still selects and shows its detail content.

use std::rc::Rc;
use uikit::widget::Widget;

/// Tab — initializer — a single tab with title, image/systemImage and detail content.
pub struct Tab {
    pub(crate) title: String,
    pub(crate) system_image: Option<String>,
    pub(crate) image_path: Option<String>,
    pub(crate) content: Option<Rc<dyn Widget>>,
}

impl Tab {
    /// Create a tab showing `content` in the detail area when selected.
    pub fn new(content: impl Widget + 'static) -> Self {
        Self {
            title: String::new(),
            system_image: None,
            image_path: None,
            content: Some(Rc::new(content)),
        }
    }

    /// Sidebar label of the tab (empty = icon-only/empty row, like
    /// SwiftUI's `Tab { Text("0") }` without a label).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// SF Symbol name for the sidebar icon (rendered as a monochrome glyph,
    /// like Apple's sidebar tabs — not the colorful Settings-style icon).
    pub fn system_image(mut self, name: impl Into<String>) -> Self {
        self.system_image = Some(name.into());
        self
    }

    /// PNG file path for the sidebar icon. Missing files render as an
    /// empty icon slot instead of a broken image.
    pub fn image(mut self, path: impl Into<String>) -> Self {
        self.image_path = Some(path.into());
        self
    }

    /// Sidebar label text.
    pub fn label_text(&self) -> &str {
        &self.title
    }

    /// Whether the tab shows anything in the sidebar (label or icon).
    pub fn has_sidebar_label(&self) -> bool {
        !self.title.is_empty() || self.system_image.is_some() || self.image_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uikit::widgets::Text;

    #[test]
    fn tab_defaults() {
        let t = Tab::new(Text::new("0"));
        assert_eq!(t.label_text(), "");
        assert!(!t.has_sidebar_label());
        assert!(t.content.is_some());
    }

    #[test]
    fn tab_builder() {
        let t = Tab::new(Text::new("1"))
            .title("1")
            .system_image("1.circle");
        assert_eq!(t.label_text(), "1");
        assert_eq!(t.system_image.as_deref(), Some("1.circle"));
        assert!(t.has_sidebar_label());
    }

    #[test]
    fn tab_image() {
        let t = Tab::new(Text::new("2")).title("2").image("cats24x24");
        assert_eq!(t.image_path.as_deref(), Some("cats24x24"));
        assert!(t.has_sidebar_label());
    }
}
