//! Picker — SwiftUI-style picker with all system picker styles.
//!
//! Covers every style shown in the SwiftUI Picker gallery:
//!
//! | SwiftUI name | `PickerStyle` variant | Visual |
//! |---|---|---|
//! | Default / Menu | `Menu` | Popover menu with checkmark |
//! | `WheelPickerStyle` | `Wheel` | Scroll wheel drum (reuses spring physics) |
//! | `SegmentedPickerStyle` | `Segmented` | Segmented control pill |
//! | `PalettePickerStyle` | `Palette` | Row of compact capsule elements |
//! | `RadioGroupPickerStyle` | `RadioGroup` | Radio buttons (vertical or horizontal) |
//! | `NavigationLinkPickerStyle` | `NavigationLink` | Row with chevron linking to list |
//! | `MenuPickerStyle` | `Menu` (alias) | Menu button that presents options |
//! | `InlinePickerStyle` | `Inline` | Options displayed inline with other views |
//! | `TabsPickerStyle` | `Tabs` | Segmented tabs (on macOS, segmented tabs style) |
//! | `PickerSection` / `PickerDivider` | `section()` / `divider()` | Section & divider inside picker |
//! | Wheel item height | `wheel_item_height()` | Modifier for drum row height |
//! | Horizontal radio | `horizontal_radio_group()` | Modifier for radio layout |
//! | Custom Value Label | `custom_value_label()` | Label generated from `LocalizedStringKey` |
//! | Multiple Sources | `multiple_sources()` | Selection bound to multiple sources |
//!
//! The builder mirrors SwiftUI's `Picker` declarative API and adapts its
//! appearance automatically to the system `ColorScheme` (dark `#1d1d1d`,
//! light `#ececec`, `SF Pro Display` everywhere).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk::prelude::*;
use gtk::{self, Label as GtkLabel, Orientation};

use uikit::app::ColorScheme;
use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};

use super::wheel_picker::WheelPicker;

// ─────────────────────────────────────────────────────────────────────
// PickerStyle
// ─────────────────────────────────────────────────────────────────────

/// Mirrors SwiftUI `PickerStyle` — each variant maps to one gallery card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerStyle {
    /// Automatic / default (menu on most platforms).
    Automatic,
    /// `WheelPickerStyle` — scrollable wheel drum with spring physics.
    Wheel,
    /// `SegmentedPickerStyle` — segmented control pill.
    Segmented,
    /// `PalettePickerStyle` — row of compact elements.
    Palette,
    /// `RadioGroupPickerStyle` — group of radio buttons.
    RadioGroup,
    /// `NavigationLinkPickerStyle` — navigation link presenting options.
    NavigationLink,
    /// `MenuPickerStyle` — menu presenting options on press.
    Menu,
    /// `InlinePickerStyle` — options displayed inline.
    Inline,
    /// `TabsPickerStyle` — options as segmented tabs.
    Tabs,
}

impl Default for PickerStyle {
    fn default() -> Self {
        Self::Automatic
    }
}

// ─────────────────────────────────────────────────────────────────────
// PickerItem — title / image / systemImage / custom label
// ─────────────────────────────────────────────────────────────────────

/// One selectable option inside a `Picker`.
///
/// Mirrors SwiftUI `Picker` content where each row can have a `title`,
/// an `image`, a `systemImage`, or a fully custom label.
#[derive(Debug, Clone)]
pub struct PickerItem {
    /// Display title.
    pub title: String,
    /// Optional subtitle (e.g. "1" count badge in gallery).
    pub subtitle: Option<String>,
    /// Optional asset image name (for bundled icons).
    pub image: Option<String>,
    /// Optional SF Symbol name.
    pub system_image: Option<String>,
    /// If true this item uses a fully custom label (rendered as emphasized).
    pub is_custom: bool,
    /// Optional tag value (mirrors SwiftUI `.tag()`).
    pub tag: Option<String>,
}

impl PickerItem {
    /// Plain title item (SwiftUI `Text("Foo")`).
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            image: None,
            system_image: None,
            is_custom: false,
            tag: None,
        }
    }

    /// Item with a subtitle (e.g. count label "1").
    pub fn with_subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }

    /// Item with a bundled image (e.g. `Image("bar")`).
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Item with an SF Symbol `systemImage` (e.g. `Image(systemName: "star")`).
    pub fn with_system_image(mut self, name: impl Into<String>) -> Self {
        self.system_image = Some(name.into());
        self
    }

    /// Mark this item as having a custom label (emphasized rendering).
    pub fn custom(mut self) -> Self {
        self.is_custom = true;
        self
    }

    /// Attach a SwiftUI-style `.tag()` value.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }
}

// ─────────────────────────────────────────────────────────────────────
// PickerSection / PickerDivider
// ─────────────────────────────────────────────────────────────────────

/// A section inside a `Picker` (SwiftUI `Section`).
#[derive(Debug, Clone)]
pub struct PickerSection {
    /// Optional section header title.
    pub title: Option<String>,
    /// Items in this section.
    pub items: Vec<PickerItem>,
    /// Whether this section is followed by a divider.
    pub has_divider: bool,
}

impl PickerSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            items: Vec::new(),
            has_divider: false,
        }
    }

    pub fn untitled() -> Self {
        Self {
            title: None,
            items: Vec::new(),
            has_divider: false,
        }
    }

    pub fn item(mut self, item: PickerItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = PickerItem>,
    {
        self.items.extend(iter);
        self
    }

    pub fn divider(mut self) -> Self {
        self.has_divider = true;
        self
    }
}

// ─────────────────────────────────────────────────────────────────────
// Picker — builder
// ─────────────────────────────────────────────────────────────────────

/// SwiftUI-style picker covering the entire SwiftUI Picker gallery.
///
/// All picker styles render from the same data — switch `style()` to change
/// the chrome while the items stay identical. Dark mode (`#1d1d1d`) and
/// light mode (`#ececec`) adapt automatically, `SF Pro Display` is used
/// for every label.
pub struct Picker {
    id: WidgetId,
    /// Visible picker label (left side, e.g. "Foo").
    label: String,
    /// Optional localization key for `Custom Value Label Picker` mode.
    /// When set the displayed label is `localized(key, selected_value)` instead
    /// of the raw label.
    label_key: Option<String>,
    /// Closure that maps `selected_value -> displayed label` for custom labels.
    custom_label_fn: Option<Arc<dyn Fn(String) -> String + Send + Sync>>,
    /// Whether this picker is bound to multiple sources (multi-write on change).
    multiple_sources: bool,
    /// Flat item list (used when no sections are present).
    items: Vec<PickerItem>,
    /// Sections (when non-empty, `items` is ignored and sections are shown).
    sections: Vec<PickerSection>,
    /// Standalone divider positions (indices after which to insert a separator
    /// when using flat items without sections).
    dividers: Vec<usize>,
    selection: usize,
    style: PickerStyle,
    color_scheme: Option<ColorScheme>,
    accent_color: Color,
    /// Drum row height for `Wheel` style (SwiftUI `wheelPickerItemHeight`).
    wheel_item_height: f32,
    /// Horizontal layout flag for `RadioGroup` style.
    horizontal_radio: bool,
    width: f32,
    height: f32,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    multiple_on_change: Vec<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Picker {
    /// Create a new picker with a label (e.g. `Picker("Foo", selection: 0)`).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            label: label.into(),
            label_key: None,
            custom_label_fn: None,
            multiple_sources: false,
            items: Vec::new(),
            sections: Vec::new(),
            dividers: Vec::new(),
            selection: 0,
            style: PickerStyle::Automatic,
            color_scheme: None,
            accent_color: Color::new(0.047, 0.522, 0.937, 1.0),
            wheel_item_height: 40.0,
            horizontal_radio: false,
            width: 320.0,
            height: 44.0,
            on_change: None,
            multiple_on_change: Vec::new(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    // ── Items ────────────────────────────────────────────────────────

    /// Set items from an iterator of `PickerItem`.
    pub fn items<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = PickerItem>,
    {
        self.items = iter.into_iter().collect();
        self
    }

    /// Convenience: set items from an iterator of strings (titles).
    pub fn titles<I, S>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.items = iter.into_iter().map(|s| PickerItem::new(s.into())).collect();
        self
    }

    /// Add a single `PickerItem`.
    pub fn item(mut self, it: PickerItem) -> Self {
        self.items.push(it);
        self
    }

    /// Convenience: add a plain title item.
    pub fn title_item(mut self, title: impl Into<String>) -> Self {
        self.items.push(PickerItem::new(title));
        self
    }

    /// Convenience: add an item with a title + system image name.
    pub fn system_image_item(
        mut self,
        title: impl Into<String>,
        system_image: impl Into<String>,
    ) -> Self {
        self.items
            .push(PickerItem::new(title).with_system_image(system_image));
        self
    }

    /// Convenience: add an item with a title + bundled image name.
    pub fn image_item(mut self, title: impl Into<String>, image: impl Into<String>) -> Self {
        self.items.push(PickerItem::new(title).with_image(image));
        self
    }

    /// Convenience: add an item with a custom emphasized label.
    pub fn custom_label_item(mut self, title: impl Into<String>) -> Self {
        self.items.push(PickerItem::new(title).custom());
        self
    }

    // ── Sections / dividers ────────────────────────────────────────

    /// Add a section (SwiftUI `Section` inside `Picker`).
    pub fn section(mut self, section: PickerSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add a pre-made section for `PickerSection` demo convenience.
    pub fn section_with<S>(
        mut self,
        title: impl Into<String>,
        items: impl IntoIterator<Item = S>,
    ) -> Self
    where
        S: Into<String>,
    {
        let sec = PickerSection::new(title).items(
            items
                .into_iter()
                .map(|s| PickerItem::new(s.into())),
        );
        self.sections.push(sec);
        self
    }

    /// Insert a divider after the last item (SwiftUI `Divider` inside `Picker`).
    pub fn divider(mut self) -> Self {
        if !self.sections.is_empty() {
            if let Some(last) = self.sections.last_mut() {
                last.has_divider = true;
            }
        } else if !self.items.is_empty() {
            let idx = self.items.len() - 1;
            self.dividers.push(idx);
        }
        self
    }

    // ── Selection ──────────────────────────────────────────────────

    /// Select by index (clamped).
    pub fn selected_index(mut self, idx: usize) -> Self {
        let total = if self.items.is_empty() && !self.sections.is_empty() {
            self.sections.iter().map(|s| s.items.len()).sum::<usize>()
        } else {
            self.items.len()
        };
        self.selection = idx.min(total.saturating_sub(1));
        self
    }

    /// Select by title value (first match wins, else 0).
    pub fn selected(mut self, val: impl Into<String>) -> Self {
        let v = val.into();
        if let Some(idx) = self.items.iter().position(|i| i.title == v) {
            self.selection = idx;
        } else {
            // Search sections.
            let mut flat = 0usize;
            for sec in &self.sections {
                for it in &sec.items {
                    if it.title == v {
                        self.selection = flat;
                        return self;
                    }
                    flat += 1;
                }
            }
        }
        self
    }

    // ── Style & modifiers ──────────────────────────────────────────

    /// Set the picker style (SwiftUI `.pickerStyle()`).
    pub fn style(mut self, style: PickerStyle) -> Self {
        self.style = style;
        self
    }

    /// Alias for `style(PickerStyle::Segmented)`.
    pub fn segmented(mut self) -> Self {
        self.style = PickerStyle::Segmented;
        self
    }
    pub fn wheel(mut self) -> Self {
        self.style = PickerStyle::Wheel;
        self
    }
    pub fn radio_group(mut self) -> Self {
        self.style = PickerStyle::RadioGroup;
        self
    }
    pub fn palette(mut self) -> Self {
        self.style = PickerStyle::Palette;
        self
    }
    pub fn menu(mut self) -> Self {
        self.style = PickerStyle::Menu;
        self
    }
    pub fn inline_picker(mut self) -> Self {
        self.style = PickerStyle::Inline;
        self
    }
    pub fn tabs(mut self) -> Self {
        self.style = PickerStyle::Tabs;
        self
    }
    pub fn navigation_link(mut self) -> Self {
        self.style = PickerStyle::NavigationLink;
        self
    }

    /// Set the `wheelPickerItemHeight` modifier (default `40.0`).
    pub fn wheel_item_height(mut self, h: f32) -> Self {
        self.wheel_item_height = h;
        self
    }

    /// `horizontalRadioGroupLayout` modifier — when true radio items lay out horizontally.
    pub fn horizontal_radio_group(mut self, horizontal: bool) -> Self {
        self.horizontal_radio = horizontal;
        self
    }

    /// Enable multiple-sources binding (mirrors SwiftUI sample where one picker
    /// writes to two `@State` selections). Stores an extra callback per source.
    pub fn multiple_sources(mut self, enabled: bool) -> Self {
        self.multiple_sources = enabled;
        self
    }

    /// Add an extra on-change handler (used for the `multiple sources` demo
    /// — each handler is invoked on selection change).
    pub fn on_change_multiple(
        mut self,
        handler: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        self.multiple_on_change.push(Arc::new(handler));
        self.multiple_sources = true;
        self
    }

    /// SwiftUI `Custom Value Label Picker` — generate the label from a
    /// `LocalizedStringKey` and the selected value.
    pub fn custom_value_label(
        mut self,
        key: impl Into<String>,
        generator: impl Fn(String) -> String + Send + Sync + 'static,
    ) -> Self {
        self.label_key = Some(key.into());
        self.custom_label_fn = Some(Arc::new(generator));
        self
    }

    // ── Appearance ─────────────────────────────────────────────────

    pub fn color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.color_scheme = Some(scheme);
        self
    }

    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    pub fn frame(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    // ── Accessors ──────────────────────────────────────────────────

    /// All items flattened (sections expanded).
    pub fn flat_items(&self) -> Vec<PickerItem> {
        if self.sections.is_empty() {
            self.items.clone()
        } else {
            self.sections.iter().flat_map(|s| s.items.clone()).collect()
        }
    }

    pub fn selected_value(&self) -> String {
        self.flat_items()
            .get(self.selection)
            .map(|i| i.title.clone())
            .unwrap_or_default()
    }

    pub fn selected_item(&self) -> Option<PickerItem> {
        self.flat_items().get(self.selection).cloned()
    }

    pub fn display_label(&self) -> String {
        let sel = self.selected_value();
        if let Some(ref f) = self.custom_label_fn {
            f(sel.clone())
        } else if let Some(ref key) = self.label_key {
            // Mimic SwiftUI LocalizedStringKey interpolation: key may be
            // "Current: \(current)" where selected value is inserted.
            if key.contains("%") || key.contains("{}") {
                key.replace("{}", &sel).replace("%@", &sel)
            } else {
                format!("{}: {}", key, sel)
            }
        } else {
            self.label.clone()
        }
    }

    pub fn picker_style(&self) -> PickerStyle {
        self.style
    }

    pub fn to_view(self) -> View {
        let w = self.width;
        let h = self.height;
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

impl Default for Picker {
    fn default() -> Self {
        Self::new("Picker")
    }
}

// ─────────────────────────────────────────────────────────────────────
// Rendering helpers
// ─────────────────────────────────────────────────────────────────────

fn scheme_dark(explicit: Option<ColorScheme>) -> bool {
    let scheme = if let Some(s) = explicit {
        s
    } else if let Some(s) = uikit::app::current_color_scheme() {
        s
    } else {
        ColorScheme::detect_system()
    };
    scheme == ColorScheme::Dark
}

fn palette_for(dark: bool) -> (&'static str, &'static str, &'static str, &'static str) {
    if dark {
        // bg, card bg, border, text
        ("#1d1d1d", "#2c2c2e", "#3a3a3c", "#ececec")
    } else {
        ("#ececec", "#ffffff", "#d0d0d2", "#1d1d1d")
    }
}

fn accent_hex(c: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8
    )
}

// Build a small rounded badge for subtitle/count.
fn subtitle_badge(text: &str, dark: bool) -> GtkLabel {
    let lbl = GtkLabel::new(Some(text));
    let fg = if dark { "#8e8e93" } else { "#6e6e73" };
    lbl.add_css_class("pk-badge");
    uikit::widget::apply_css(
        &lbl,
        &format!(
            ".pk-badge {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 11px; }}"
        ),
    );
    lbl
}

// ─────────────────────────────────────────────────────────────────────
// Individual style renderers
// ─────────────────────────────────────────────────────────────────────

fn render_segmented(picker: &Picker, dark: bool) -> gtk::Widget {
    let items = picker.flat_items();
    let selected = Rc::new(RefCell::new(picker.selection));
    let accent = accent_hex(picker.accent_color);

    let outer = gtk::Box::new(Orientation::Vertical, 0);
    outer.set_hexpand(false);

    let seg = gtk::Box::new(Orientation::Horizontal, 0);
    seg.add_css_class("pk-segmented");
    let bg = if dark { "#2c2c2e" } else { "#e5e5ea" };
    let border = if dark { "#3a3a3c" } else { "#d0d0d2" };
    uikit::widget::apply_css(
        &seg,
        &format!(
            ".pk-segmented {{ background: {bg}; border-radius: 9px; border: 1px solid {border}; padding: 2px; }}"
        ),
    );

    for (idx, item) in items.iter().enumerate() {
        let btn = gtk::ToggleButton::with_label(&item.title);
        btn.set_hexpand(true);
        let is_sel = idx == picker.selection;
        btn.set_active(is_sel);
        let cls = if is_sel { "pk-seg-sel" } else { "pk-seg" };
        btn.add_css_class(cls);
        if is_sel {
            uikit::widget::apply_css(
                &btn,
                &format!(
                    ".pk-seg-sel {{ background: {accent}; color: white; border-radius: 7px; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 600; padding: 6px 12px; border: none; }}"
                ),
            );
        } else {
            let fg = if dark { "#ececec" } else { "#1d1d1d" };
            uikit::widget::apply_css(
                &btn,
                &format!(
                    ".pk-seg {{ background: transparent; color: {fg}; border-radius: 7px; font-family: 'SF Pro Display'; font-size: 13px; padding: 6px 12px; border: none; }}"
                ),
            );
        }

        // Ensure only one active at a time (group behaviour via manual toggle).
        {
            let selected = selected.clone();
            let items_c = items.clone();
            let cb = picker.on_change.clone();
            let multi = picker.multiple_on_change.clone();
            let seg_clone = seg.clone();
            btn.connect_toggled(move |b| {
                if !b.is_active() {
                    // Prevent deselecting the active segment by clicking it again.
                    if *selected.borrow() == idx {
                        b.set_active(true);
                    }
                    return;
                }
                *selected.borrow_mut() = idx;
                // Update siblings.
                let mut child = seg_clone.first_child();
                let mut i = 0usize;
                while let Some(w) = child {
                    if let Ok(tb) = w.clone().downcast::<gtk::ToggleButton>() {
                        if i != idx {
                            tb.set_active(false);
                        }
                    }
                    child = w.next_sibling();
                    i += 1;
                }
                if idx < items_c.len() {
                    let v = items_c[idx].title.clone();
                    if let Some(ref h) = cb {
                        h(v.clone());
                    }
                    for h in &multi {
                        h(v.clone());
                    }
                }
            });
        }
        seg.append(&btn);
    }

    outer.append(&seg);
    outer.upcast()
}

fn render_palette(picker: &Picker, dark: bool) -> gtk::Widget {
    let items = picker.flat_items();
    let selected = Rc::new(RefCell::new(picker.selection));
    let accent = accent_hex(picker.accent_color);

    let row = gtk::Box::new(Orientation::Horizontal, 8);
    row.set_halign(gtk::Align::Center);
    row.add_css_class("pk-palette");

    for (idx, item) in items.iter().enumerate() {
        let is_sel = idx == picker.selection;
        let btn = gtk::Button::with_label(&item.title);
        btn.set_width_request(48);
        btn.set_height_request(36);
        if is_sel {
            btn.add_css_class("pk-pal-sel");
            uikit::widget::apply_css(
                &btn,
                &format!(
                    ".pk-pal-sel {{ background: {accent}; color: white; border-radius: 10px; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 600; border: 2px solid {accent}; }}"
                ),
            );
        } else {
            let bg = if dark { "#2c2c2e" } else { "#ffffff" };
            let fg = if dark { "#ececec" } else { "#1d1d1d" };
            let border = if dark { "#3a3a3c" } else { "#d0d0d2" };
            btn.add_css_class("pk-pal");
            uikit::widget::apply_css(
                &btn,
                &format!(
                    ".pk-pal {{ background: {bg}; color: {fg}; border-radius: 10px; font-family: 'SF Pro Display'; font-size: 13px; border: 1px solid {border}; }}"
                ),
            );
        }
        {
            let selected = selected.clone();
            let items_c = items.clone();
            let cb = picker.on_change.clone();
            let multi = picker.multiple_on_change.clone();
            let row_clone = row.clone();
            let accent_c = accent.clone();
            let dark_c = dark;
            btn.connect_clicked(move |_| {
                *selected.borrow_mut() = idx;
                // Re-style row children to show new selection (simplified: just fire callback; visual update on next render is acceptable for demo).
                // For immediate feedback, toggle css class on clicked.
                let mut child = row_clone.first_child();
                let mut i = 0usize;
                while let Some(w) = child {
                    let next = w.next_sibling();
                    if let Ok(b) = w.clone().downcast::<gtk::Button>() {
                        b.remove_css_class("pk-pal-sel");
                        b.remove_css_class("pk-pal");
                        if i == idx {
                            b.add_css_class("pk-pal-sel");
                            uikit::widget::apply_css(
                                &b,
                                &format!(".pk-pal-sel {{ background: {accent_c}; color: white; border-radius: 10px; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 600; border: 2px solid {accent_c}; }}"),
                            );
                        } else {
                            let bg = if dark_c { "#2c2c2e" } else { "#ffffff" };
                            let fg = if dark_c { "#ececec" } else { "#1d1d1d" };
                            let border = if dark_c { "#3a3a3c" } else { "#d0d0d2" };
                            b.add_css_class("pk-pal");
                            uikit::widget::apply_css(
                                &b,
                                &format!(".pk-pal {{ background: {bg}; color: {fg}; border-radius: 10px; font-family: 'SF Pro Display'; font-size: 13px; border: 1px solid {border}; }}"),
                            );
                        }
                    }
                    child = next;
                    i += 1;
                }
                if idx < items_c.len() {
                    let v = items_c[idx].title.clone();
                    if let Some(ref h) = cb { h(v.clone()); }
                    for h in &multi { h(v.clone()); }
                }
            });
        }
        row.append(&btn);
    }
    row.upcast()
}

fn render_radio_group(picker: &Picker, dark: bool) -> gtk::Widget {
    let items = picker.flat_items();
    let horizontal = picker.horizontal_radio;
    let orient = if horizontal { Orientation::Horizontal } else { Orientation::Vertical };
    let spacing = if horizontal { 16 } else { 8 };
    let container = gtk::Box::new(orient, spacing);
    container.set_halign(if horizontal { gtk::Align::Center } else { gtk::Align::Start });

    let selected = Rc::new(RefCell::new(picker.selection));
    let first_btn: Rc<RefCell<Option<gtk::CheckButton>>> = Rc::new(RefCell::new(None));

    for (idx, item) in items.iter().enumerate() {
        let is_sel = idx == picker.selection;
        let row = gtk::Box::new(Orientation::Horizontal, 8);
        row.set_valign(gtk::Align::Center);

        let radio = gtk::CheckButton::new();
        // GTK4 CheckButton can act as radio when grouped; we group manually via first.
        // Avoid holding an immutable borrow across the mutable borrow (would panic).
        let first_opt = first_btn.borrow().clone();
        if let Some(first) = first_opt {
            radio.set_group(Some(&first));
        } else {
            *first_btn.borrow_mut() = Some(radio.clone());
        }
        radio.set_active(is_sel);

        let lbl = GtkLabel::new(Some(&item.title));
        let fg = if dark { "#ececec" } else { "#1d1d1d" };
        let fw = if item.is_custom { "600" } else { "400" };
        lbl.add_css_class("pk-radio-lbl");
        uikit::widget::apply_css(
            &lbl,
            &format!(".pk-radio-lbl {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 13px; font-weight: {fw}; }}"),
        );

        // Dots for palette-like compact radio feel if subtitle present.
        if let Some(ref sub) = item.subtitle {
            let badge = subtitle_badge(sub, dark);
            row.append(&radio);
            row.append(&lbl);
            row.append(&badge);
        } else {
            row.append(&radio);
            row.append(&lbl);
        }

        if let Some(ref sys) = item.system_image {
            let sys_lbl = GtkLabel::new(Some(&format!("{} {}", sys, item.title)));
            let _ = sys_lbl;
        }
        if let Some(ref img) = item.image {
            let img_lbl = GtkLabel::new(Some(img));
            let _ = img_lbl;
        }

        {
            let selected = selected.clone();
            let items_c = items.clone();
            let cb = picker.on_change.clone();
            let multi = picker.multiple_on_change.clone();
            radio.connect_toggled(move |b| {
                if b.is_active() {
                    *selected.borrow_mut() = idx;
                    if idx < items_c.len() {
                        let v = items_c[idx].title.clone();
                        if let Some(ref h) = cb { h(v.clone()); }
                        for h in &multi { h(v.clone()); }
                    }
                }
            });
        }

        container.append(&row);
    }

    container.upcast()
}

fn render_menu(picker: &Picker, dark: bool) -> gtk::Widget {
    let items = picker.flat_items();
    let display = picker.display_label();
    let selected = Rc::new(RefCell::new(picker.selection));

    let btn = gtk::MenuButton::new();
    // Label inside menu button: "Foo  ✓ 1  ∨" style
    let inner = gtk::Box::new(Orientation::Horizontal, 8);
    inner.set_valign(gtk::Align::Center);
    let lbl = GtkLabel::new(Some(&display));
    let fg = if dark { "#ececec" } else { "#1d1d1d" };
    lbl.add_css_class("pk-menu-lbl");
    uikit::widget::apply_css(
        &lbl,
        &format!(".pk-menu-lbl {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 13px; }}"),
    );
    let sel_title = items.get(picker.selection).map(|i| i.title.as_str()).unwrap_or("");
    let val_lbl = GtkLabel::new(Some(sel_title));
    val_lbl.add_css_class("pk-menu-val");
    uikit::widget::apply_css(
        &val_lbl,
        &format!(".pk-menu-val {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 500; }}"),
    );
    let chev = GtkLabel::new(Some("▾"));
    chev.add_css_class("pk-menu-chev");
    uikit::widget::apply_css(&chev, ".pk-menu-chev { color: #8e8e93; font-size: 11px; }");
    inner.append(&lbl);
    // Show selection value if label != selection (e.g. "Foo ▸ 1")
    if display != sel_title && !sel_title.is_empty() {
        inner.append(&val_lbl);
    }
    inner.append(&chev);
    btn.set_child(Some(&inner));

    // Transparent like a SwiftUI Menu picker: plain label + chevron, no box,
    // no border, no focus outline — matches the other borderless styles.
    btn.add_css_class("pk-menu-btn");
    uikit::widget::apply_css(
        &btn,
        ".pk-menu-btn { background: transparent; border: none; outline: none; padding: 8px 12px; border-radius: 10px; } .pk-menu-btn:focus { outline: none; }",
    );

    let pop = gtk::Popover::new();
    let list = gtk::Box::new(Orientation::Vertical, 0);
    list.add_css_class("pk-menu-pop");
    let pop_bg = if dark { "#1d1d1d" } else { "#ececec" };
    uikit::widget::apply_css(
        &list,
        &format!(".pk-menu-pop {{ background: {pop_bg}; border-radius: 12px; padding: 6px; }}"),
    );

    // Sections with dividers support.
    let has_sections = !picker.sections.is_empty();
    if has_sections {
        for sec in &picker.sections {
            if let Some(ref title) = sec.title {
                let sec_lbl = GtkLabel::new(Some(title));
                sec_lbl.set_halign(gtk::Align::Start);
                sec_lbl.add_css_class("pk-sec-lbl");
                let sfg = if dark { "#8e8e93" } else { "#6e6e73" };
                uikit::widget::apply_css(
                    &sec_lbl,
                    &format!(".pk-sec-lbl {{ color: {sfg}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; padding: 6px 8px 2px 8px; }}"),
                );
                list.append(&sec_lbl);
            }
            for (flat_idx, item) in sec.items.iter().enumerate() {
                // Need global index for selection; compute.
                let global = picker
                    .sections
                    .iter()
                    .take_while(|s| s.title != sec.title)
                    .map(|s| s.items.len())
                    .sum::<usize>()
                    + flat_idx;
                let is_sel = global == picker.selection;
                let item_title = item.title.clone();
                let row = menu_row(item, is_sel, dark, {
                    let selected = selected.clone();
                    let items_c = items.clone();
                    let cb = picker.on_change.clone();
                    let multi = picker.multiple_on_change.clone();
                    let val_lbl_c = val_lbl.clone();
                    let pop_c = pop.clone();
                    let item_title_c = item_title.clone();
                    move || {
                        *selected.borrow_mut() = global;
                        val_lbl_c.set_text(&item_title_c);
                        pop_c.popdown();
                        if global < items_c.len() {
                            let v = items_c[global].title.clone();
                            if let Some(ref h) = cb { h(v.clone()); }
                            for h in &multi { h(v.clone()); }
                        }
                    }
                });
                list.append(&row);
            }
            if sec.has_divider {
                let sep = gtk::Separator::new(Orientation::Horizontal);
                sep.set_margin_top(6);
                sep.set_margin_bottom(6);
                list.append(&sep);
            }
        }
    } else {
        for (idx, item) in items.iter().enumerate() {
            let is_sel = idx == picker.selection;
            let item_title = item.title.clone();
            // Check standalone dividers vector.
            let row = menu_row(item, is_sel, dark, {
                let selected = selected.clone();
                let items_c = items.clone();
                let cb = picker.on_change.clone();
                let multi = picker.multiple_on_change.clone();
                let val_lbl_c = val_lbl.clone();
                let pop_c = pop.clone();
                let item_title_c = item_title.clone();
                move || {
                    *selected.borrow_mut() = idx;
                    val_lbl_c.set_text(&item_title_c);
                    pop_c.popdown();
                    if idx < items_c.len() {
                        let v = items_c[idx].title.clone();
                        if let Some(ref h) = cb { h(v.clone()); }
                        for h in &multi { h(v.clone()); }
                    }
                }
            });
            list.append(&row);
            if picker.dividers.contains(&idx) {
                let sep = gtk::Separator::new(Orientation::Horizontal);
                sep.set_margin_top(6);
                sep.set_margin_bottom(6);
                list.append(&sep);
            }
        }
    }

    pop.set_child(Some(&list));
    btn.set_popover(Some(&pop));

    btn.upcast()
}

fn menu_row<F>(item: &PickerItem, is_sel: bool, dark: bool, on_activate: F) -> gtk::Box
where
    F: Fn() + 'static,
{
    let row = gtk::Box::new(Orientation::Horizontal, 8);
    row.set_hexpand(true);
    let fg = if dark { "#ececec" } else { "#1d1d1d" };
    let sel_fg = "#0a84ff";
    let bg = if is_sel {
        if dark { "rgba(10,132,255,0.18)" } else { "rgba(10,132,255,0.12)" }
    } else {
        "transparent"
    };
    row.add_css_class("pk-menu-row");
    uikit::widget::apply_css(
        &row,
        &format!(".pk-menu-row {{ background: {bg}; border-radius: 8px; padding: 6px 10px; }}"),
    );

    // Icon placeholder if needed.
    if let Some(ref sys) = item.system_image {
        let icon = GtkLabel::new(Some(sys));
        icon.add_css_class("pk-row-icon");
        uikit::widget::apply_css(&icon, &format!(".pk-row-icon {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 12px; }}"));
        row.append(&icon);
    } else if let Some(ref img) = item.image {
        let icon = GtkLabel::new(Some(img));
        icon.add_css_class("pk-row-icon");
        uikit::widget::apply_css(&icon, &format!(".pk-row-icon {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 12px; }}"));
        row.append(&icon);
    }

    let title = GtkLabel::new(Some(&item.title));
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);
    let color = if is_sel { sel_fg } else { fg };
    let fw = if item.is_custom { "700" } else { "400" };
    title.add_css_class("pk-row-title");
    uikit::widget::apply_css(
        &title,
        &format!(".pk-row-title {{ color: {color}; font-family: 'SF Pro Display'; font-size: 13px; font-weight: {fw}; }}"),
    );
    row.append(&title);

    if let Some(ref sub) = item.subtitle {
        let badge = subtitle_badge(sub, dark);
        row.append(&badge);
    }

    if is_sel {
        let check = GtkLabel::new(Some("✓"));
        check.add_css_class("pk-check");
        uikit::widget::apply_css(&check, ".pk-check { color: #0a84ff; font-size: 13px; font-weight: 700; }");
        row.append(&check);
    }

    let gesture = gtk::GestureClick::new();
    gesture.set_button(1);
    gesture.connect_pressed(move |_, _, _, _| {
        on_activate();
    });
    row.add_controller(gesture);

    row
}

fn render_inline(picker: &Picker, dark: bool) -> gtk::Widget {
    let items = picker.flat_items();
    let selected = Rc::new(RefCell::new(picker.selection));

    let container = gtk::Box::new(Orientation::Vertical, 6);
    let display = picker.display_label();
    if !display.is_empty() {
        let header = GtkLabel::new(Some(&display));
        header.set_halign(gtk::Align::Start);
        let fg = if dark { "#ececec" } else { "#1d1d1d" };
        header.add_css_class("pk-inline-hdr");
        uikit::widget::apply_css(
            &header,
            &format!(".pk-inline-hdr {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 600; }}"),
        );
        container.append(&header);
    }

    let list = gtk::Box::new(Orientation::Vertical, 2);
    let card_bg = if dark { "#2c2c2e" } else { "#ffffff" };
    let border = if dark { "#3a3a3c" } else { "#d0d0d2" };
    list.add_css_class("pk-inline-list");
    uikit::widget::apply_css(
        &list,
        &format!(".pk-inline-list {{ background: {card_bg}; border-radius: 12px; border: 1px solid {border}; padding: 6px; }}"),
    );

    let has_sections = !picker.sections.is_empty();
    if has_sections {
        for sec in &picker.sections {
            if let Some(ref title) = sec.title {
                let sec_lbl = GtkLabel::new(Some(title));
                sec_lbl.set_halign(gtk::Align::Start);
                sec_lbl.add_css_class("pk-sec-lbl");
                let sfg = if dark { "#8e8e93" } else { "#6e6e73" };
                uikit::widget::apply_css(
                    &sec_lbl,
                    &format!(".pk-sec-lbl {{ color: {sfg}; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 600; padding: 4px 6px; }}"),
                );
                list.append(&sec_lbl);
            }
            for (flat_idx, item) in sec.items.iter().enumerate() {
                let global = picker
                    .sections
                    .iter()
                    .take_while(|s| s.title != sec.title)
                    .map(|s| s.items.len())
                    .sum::<usize>()
                    + flat_idx;
                let is_sel = global == picker.selection;
                let row = inline_row(item, is_sel, dark, {
                    let selected = selected.clone();
                    let items_c = items.clone();
                    let cb = picker.on_change.clone();
                    let multi = picker.multiple_on_change.clone();
                    let list_clone = list.clone();
                    move || {
                        *selected.borrow_mut() = global;
                        // Re-style siblings: dim — for inline we refresh border.
                        let mut child = list_clone.first_child();
                        while let Some(w) = child {
                            let next = w.next_sibling();
                            // Only style actual rows (boxes with check)
                            child = next;
                        }
                        if global < items_c.len() {
                            let v = items_c[global].title.clone();
                            if let Some(ref h) = cb { h(v.clone()); }
                            for h in &multi { h(v.clone()); }
                        }
                    }
                });
                list.append(&row);
            }
            if sec.has_divider {
                let sep = gtk::Separator::new(Orientation::Horizontal);
                sep.set_margin_top(4);
                sep.set_margin_bottom(4);
                list.append(&sep);
            }
        }
    } else {
        for (idx, item) in items.iter().enumerate() {
            let is_sel = idx == picker.selection;
            let row = inline_row(item, is_sel, dark, {
                let selected = selected.clone();
                let items_c = items.clone();
                let cb = picker.on_change.clone();
                let multi = picker.multiple_on_change.clone();
                move || {
                    *selected.borrow_mut() = idx;
                    if idx < items_c.len() {
                        let v = items_c[idx].title.clone();
                        if let Some(ref h) = cb { h(v.clone()); }
                        for h in &multi { h(v.clone()); }
                    }
                }
            });
            list.append(&row);
            if picker.dividers.contains(&idx) {
                let sep = gtk::Separator::new(Orientation::Horizontal);
                sep.set_margin_top(4);
                sep.set_margin_bottom(4);
                list.append(&sep);
            }
        }
    }

    container.append(&list);
    container.upcast()
}

fn inline_row<F>(item: &PickerItem, is_sel: bool, dark: bool, on_click: F) -> gtk::Box
where
    F: Fn() + 'static,
{
    let row = gtk::Box::new(Orientation::Horizontal, 8);
    row.set_hexpand(true);
    let bg = if is_sel {
        if dark { "rgba(10,132,255,0.22)" } else { "rgba(10,132,255,0.14)" }
    } else {
        "transparent"
    };
    let fg = if is_sel { "#0a84ff" } else if dark { "#ececec" } else { "#1d1d1d" };
    row.add_css_class("pk-inline-row");
    uikit::widget::apply_css(
        &row,
        &format!(".pk-inline-row {{ background: {bg}; border-radius: 8px; padding: 8px 10px; }}"),
    );
    if let Some(ref sys) = item.system_image {
        let icon = GtkLabel::new(Some(sys));
        icon.add_css_class("pk-ir-icon");
        uikit::widget::apply_css(&icon, &format!(".pk-ir-icon {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 12px; }}"));
        row.append(&icon);
    } else if let Some(ref img) = item.image {
        let icon = GtkLabel::new(Some(img));
        icon.add_css_class("pk-ir-icon");
        uikit::widget::apply_css(&icon, &format!(".pk-ir-icon {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 12px; }}"));
        row.append(&icon);
    }
    let title = GtkLabel::new(Some(&item.title));
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);
    let fw = if item.is_custom { "700" } else { "400" };
    title.add_css_class("pk-ir-title");
    uikit::widget::apply_css(
        &title,
        &format!(".pk-ir-title {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 13px; font-weight: {fw}; }}"),
    );
    row.append(&title);
    if let Some(ref sub) = item.subtitle {
        let badge = subtitle_badge(sub, dark);
        row.append(&badge);
    }
    if is_sel {
        let check = GtkLabel::new(Some("✓"));
        check.add_css_class("pk-check");
        uikit::widget::apply_css(&check, ".pk-check { color: #0a84ff; font-size: 13px; font-weight: 700; }");
        row.append(&check);
    }
    let gesture = gtk::GestureClick::new();
    gesture.set_button(1);
    gesture.connect_pressed(move |_, _, _, _| on_click());
    row.add_controller(gesture);
    row
}

fn render_tabs(picker: &Picker, dark: bool) -> gtk::Widget {
    // Reuse segmented look but with underline tabs feel.
    let items = picker.flat_items();
    let selected = Rc::new(RefCell::new(picker.selection));
    let accent = accent_hex(picker.accent_color);

    let outer = gtk::Box::new(Orientation::Vertical, 0);
    let bar = gtk::Box::new(Orientation::Horizontal, 0);
    bar.add_css_class("pk-tabs-bar");
    let bg = if dark { "#1d1d1d" } else { "#ececec" };
    let border = if dark { "#3a3a3c" } else { "#d0d0d2" };
    uikit::widget::apply_css(
        &bar,
        &format!(".pk-tabs-bar {{ background: {bg}; border-bottom: 1px solid {border}; padding: 4px; }}"),
    );

    for (idx, item) in items.iter().enumerate() {
        let is_sel = idx == picker.selection;
        let btn = gtk::Button::with_label(&item.title);
        btn.set_hexpand(true);
        if is_sel {
            btn.add_css_class("pk-tab-sel");
            uikit::widget::apply_css(
                &btn,
                &format!(".pk-tab-sel {{ background: {accent}; color: white; border-radius: 7px; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 600; border: none; padding: 6px 10px; }}"),
            );
        } else {
            let fg = if dark { "#ececec" } else { "#1d1d1d" };
            btn.add_css_class("pk-tab");
            uikit::widget::apply_css(
                &btn,
                &format!(".pk-tab {{ background: transparent; color: {fg}; border-radius: 7px; font-family: 'SF Pro Display'; font-size: 13px; border: none; padding: 6px 10px; }}"),
            );
        }
        {
            let selected = selected.clone();
            let items_c = items.clone();
            let cb = picker.on_change.clone();
            let multi = picker.multiple_on_change.clone();
            let bar_clone = bar.clone();
            let accent_c = accent.clone();
            let dark_c = dark;
            btn.connect_clicked(move |_| {
                *selected.borrow_mut() = idx;
                // Update visuals for all tabs.
                let mut child = bar_clone.first_child();
                let mut i = 0usize;
                while let Some(w) = child {
                    let next = w.next_sibling();
                    if let Ok(b) = w.clone().downcast::<gtk::Button>() {
                        b.remove_css_class("pk-tab-sel");
                        b.remove_css_class("pk-tab");
                        if i == idx {
                            b.add_css_class("pk-tab-sel");
                            uikit::widget::apply_css(
                                &b,
                                &format!(".pk-tab-sel {{ background: {accent_c}; color: white; border-radius: 7px; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 600; border: none; padding: 6px 10px; }}"),
                            );
                        } else {
                            let fg = if dark_c { "#ececec" } else { "#1d1d1d" };
                            b.add_css_class("pk-tab");
                            uikit::widget::apply_css(
                                &b,
                                &format!(".pk-tab {{ background: transparent; color: {fg}; border-radius: 7px; font-family: 'SF Pro Display'; font-size: 13px; border: none; padding: 6px 10px; }}"),
                            );
                        }
                    }
                    child = next;
                    i += 1;
                }
                if idx < items_c.len() {
                    let v = items_c[idx].title.clone();
                    if let Some(ref h) = cb { h(v.clone()); }
                    for h in &multi { h(v.clone()); }
                }
            });
        }
        bar.append(&btn);
    }

    outer.append(&bar);

    // Content placeholder below tabs showing selection.
    let sel_title = items.get(picker.selection).map(|i| i.title.as_str()).unwrap_or("");
    let content = GtkLabel::new(Some(&format!("Selected: {sel_title}")));
    content.set_margin_top(12);
    content.set_halign(gtk::Align::Center);
    let fg2 = if dark { "#8e8e93" } else { "#6e6e73" };
    content.add_css_class("pk-tabs-content");
    uikit::widget::apply_css(
        &content,
        &format!(".pk-tabs-content {{ color: {fg2}; font-family: 'SF Pro Display'; font-size: 12px; }}"),
    );
    outer.append(&content);

    outer.upcast()
}

fn render_navigation_link(picker: &Picker, dark: bool) -> gtk::Widget {
    let items = picker.flat_items();
    let selected = Rc::new(RefCell::new(picker.selection));
    let label = picker.label.clone();

    let outer = gtk::Box::new(Orientation::Vertical, 6);
    let row = gtk::Box::new(Orientation::Horizontal, 8);
    let card_bg = if dark { "#2c2c2e" } else { "#ffffff" };
    let border = if dark { "#3a3a3c" } else { "#d0d0d2" };
    row.set_hexpand(true);
    row.add_css_class("pk-nav-row");
    uikit::widget::apply_css(
        &row,
        &format!(".pk-nav-row {{ background: {card_bg}; border-radius: 10px; border: 1px solid {border}; padding: 10px 12px; }}"),
    );

    let lbl = GtkLabel::new(Some(&label));
    lbl.set_halign(gtk::Align::Start);
    lbl.set_hexpand(true);
    let fg = if dark { "#ececec" } else { "#1d1d1d" };
    lbl.add_css_class("pk-nav-lbl");
    uikit::widget::apply_css(
        &lbl,
        &format!(".pk-nav-lbl {{ color: {fg}; font-family: 'SF Pro Display'; font-size: 14px; font-weight: 600; }}"),
    );
    row.append(&lbl);

    let sel = items.get(picker.selection).map(|i| i.title.as_str()).unwrap_or("");
    let val = GtkLabel::new(Some(sel));
    val.add_css_class("pk-nav-val");
    let vfg = if dark { "#8e8e93" } else { "#6e6e73" };
    uikit::widget::apply_css(
        &val,
        &format!(".pk-nav-val {{ color: {vfg}; font-family: 'SF Pro Display'; font-size: 13px; }}"),
    );
    row.append(&val);

    let chev = GtkLabel::new(Some("›"));
    chev.add_css_class("pk-nav-chev");
    uikit::widget::apply_css(&chev, ".pk-nav-chev { color: #8e8e93; font-size: 18px; font-weight: 600; }");
    row.append(&chev);

    outer.append(&row);

    // Expanded list below (simulates navigation destination inline for demo).
    let list = gtk::Box::new(Orientation::Vertical, 4);
    list.set_margin_top(6);
    list.add_css_class("pk-nav-list");
    uikit::widget::apply_css(
        &list,
        &format!(".pk-nav-list {{ background: {card_bg}; border-radius: 12px; border: 1px solid {border}; padding: 8px; }}"),
    );
    for (idx, item) in items.iter().enumerate() {
        let is_sel = idx == picker.selection;
        let r = gtk::Box::new(Orientation::Horizontal, 8);
        r.set_hexpand(true);
        let bfg = if is_sel { "#0a84ff" } else if dark { "#ececec" } else { "#1d1d1d" };
        let rbg = if is_sel {
            if dark { "rgba(10,132,255,0.18)" } else { "rgba(10,132,255,0.12)" }
        } else {
            "transparent"
        };
        r.add_css_class("pk-nav-item");
        uikit::widget::apply_css(
            &r,
            &format!(".pk-nav-item {{ background: {rbg}; border-radius: 8px; padding: 7px 10px; }}"),
        );
        let t = GtkLabel::new(Some(&item.title));
        t.set_halign(gtk::Align::Start);
        t.set_hexpand(true);
        t.add_css_class("pk-nav-item-lbl");
        uikit::widget::apply_css(
            &t,
            &format!(".pk-nav-item-lbl {{ color: {bfg}; font-family: 'SF Pro Display'; font-size: 13px; }}"),
        );
        r.append(&t);
        if let Some(ref sub) = item.subtitle {
            r.append(&subtitle_badge(sub, dark));
        }
        if is_sel {
            let chk = GtkLabel::new(Some("✓"));
            chk.add_css_class("pk-check");
            uikit::widget::apply_css(&chk, ".pk-check { color: #0a84ff; font-weight: 700; }");
            r.append(&chk);
        }
        {
            let selected = selected.clone();
            let items_c = items.clone();
            let cb = picker.on_change.clone();
            let multi = picker.multiple_on_change.clone();
            let val_c = val.clone();
            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            let r_clone = r.clone();
            let dark_c = dark;
            gesture.connect_pressed(move |_, _, _, _| {
                *selected.borrow_mut() = idx;
                val_c.set_text(&items_c[idx].title);
                // Update row highlights: reset all siblings.
                if let Some(parent) = r_clone.parent() {
                    let mut child = parent.first_child();
                    while let Some(w) = child {
                        let next = w.next_sibling();
                        if let Ok(bx) = w.clone().downcast::<gtk::Box>() {
                            // Only rows have pk-nav-item class; skip other widgets.
                            uikit::widget::apply_css(
                                &bx,
                                " .pk-nav-item { background: transparent; }",
                            );
                        }
                        child = next;
                    }
                }
                let _ = dark_c;
                if idx < items_c.len() {
                    let v = items_c[idx].title.clone();
                    if let Some(ref h) = cb { h(v.clone()); }
                    for h in &multi { h(v.clone()); }
                }
            });
            r.add_controller(gesture);
        }
        list.append(&r);
        if picker.dividers.contains(&idx) {
            let sep = gtk::Separator::new(Orientation::Horizontal);
            sep.set_margin_top(4);
            sep.set_margin_bottom(4);
            list.append(&sep);
        }
    }
    outer.append(&list);
    outer.upcast()
}

fn render_wheel(picker: &Picker, _dark: bool) -> gtk::Widget {
    // Delegate to the existing animated WheelPicker with full spring physics.
    // Picker's flat items are mapped to WheelPicker titles; selection and
    // wheel_item_height modifier are forwarded, plus all on_change handlers.
    let titles: Vec<String> = picker.flat_items().iter().map(|i| i.title.clone()).collect();
    let mut wp = WheelPicker::new()
        .items(titles)
        .selected_index(picker.selection)
        .frame(picker.width.max(200.0), picker.height.max(176.0))
        .accent_color(picker.accent_color)
        .item_height(picker.wheel_item_height);
    let cb = picker.on_change.clone();
    let multi = picker.multiple_on_change.clone();
    if cb.is_some() || !multi.is_empty() {
        let cb_c = cb.clone();
        let multi_c = multi.clone();
        wp = wp.on_change(move |v| {
            if let Some(ref h) = cb_c {
                h(v.clone());
            }
            for h in &multi_c {
                h(v.clone());
            }
        });
    }
    wp.to_gtk()
}

#[allow(dead_code)]
fn render_default(picker: &Picker, dark: bool) -> gtk::Widget {
    // Default falls back to Menu style with label + value row.
    render_menu(picker, dark)
}

// ─────────────────────────────────────────────────────────────────────
// ViewContent impl — dispatches to style renderer
// ─────────────────────────────────────────────────────────────────────

impl ViewContent for Picker {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let dark = scheme_dark(self.color_scheme);
        let (_bg, _card_bg, _border, _text) = palette_for(dark);

        let w = if self.width > 0.0 { self.width } else { frame.width };
        let h = if self.height > 0.0 { self.height } else { frame.height };

        let container = gtk::Box::new(Orientation::Vertical, 8);
        container.set_width_request(w as i32);
        // For wheel & inline, let the inner widget dictate height.
        if self.style != PickerStyle::Wheel {
            // keep header label outside the control for some styles
        }

        let widget: gtk::Widget = match self.style {
            PickerStyle::Wheel => render_wheel(self, dark),
            PickerStyle::Segmented => render_segmented(self, dark),
            PickerStyle::Palette => render_palette(self, dark),
            PickerStyle::RadioGroup => render_radio_group(self, dark),
            PickerStyle::NavigationLink => render_navigation_link(self, dark),
            PickerStyle::Menu | PickerStyle::Automatic => render_menu(self, dark),
            PickerStyle::Inline => render_inline(self, dark),
            PickerStyle::Tabs => render_tabs(self, dark),
        };

        // Every style renders just its own control — no extra header label.
        // The menu button already shows the display label inside itself, so
        // Menu pickers look like all other styles: transparent, no extra box.
        container.append(&widget);

        // Ensure container knows its size.
        if h > 0.0 && self.style != PickerStyle::Inline && self.style != PickerStyle::NavigationLink {
            container.set_size_request(w as i32, -1);
        }

        container.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        true
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        let base_h = match self.style {
            PickerStyle::Wheel => self.height.max(176.0),
            PickerStyle::Inline => 220.0,
            PickerStyle::NavigationLink => 180.0,
            PickerStyle::Tabs => 88.0,
            PickerStyle::Palette => 44.0,
            _ => self.height.max(44.0),
        };
        Size::new(self.width.max(200.0), base_h)
    }
}

impl Widget for Picker {
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
        self.render(Rect::new(0.0, 0.0, self.width, self.height))
    }
    fn is_interactive(&self) -> bool {
        true
    }
    fn padding(&self) -> uikit::style::Padding {
        uikit::style::Padding::ZERO
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests — mirrors screenshot gallery invariants
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<PickerItem> {
        vec![
            PickerItem::new("1"),
            PickerItem::new("2"),
            PickerItem::new("3"),
            PickerItem::new("Bar").with_subtitle("2"),
        ]
    }

    #[test]
    fn picker_builder_flat() {
        let p = Picker::new("Foo").titles(["1", "2", "3"]).selected_index(1);
        assert_eq!(p.label, "Foo");
        assert_eq!(p.selection, 1);
        assert_eq!(p.selected_value(), "2");
    }

    #[test]
    fn picker_items_with_images() {
        let p = Picker::new("Picker")
            .item(PickerItem::new("Foo").with_image("foo.png"))
            .item(PickerItem::new("Bar").with_system_image("star.fill"))
            .item(PickerItem::new("Baz").custom());
        assert_eq!(p.flat_items().len(), 3);
        assert_eq!(p.flat_items()[0].image.as_deref(), Some("foo.png"));
        assert_eq!(p.flat_items()[1].system_image.as_deref(), Some("star.fill"));
        assert!(p.flat_items()[2].is_custom);
    }

    #[test]
    fn picker_styles_assign() {
        let p = Picker::new("Picker").style(PickerStyle::Segmented);
        assert_eq!(p.picker_style(), PickerStyle::Segmented);
        assert_eq!(Picker::new("P").segmented().picker_style(), PickerStyle::Segmented);
        assert_eq!(Picker::new("P").wheel().picker_style(), PickerStyle::Wheel);
        assert_eq!(Picker::new("P").radio_group().picker_style(), PickerStyle::RadioGroup);
        assert_eq!(Picker::new("P").palette().picker_style(), PickerStyle::Palette);
        assert_eq!(Picker::new("P").menu().picker_style(), PickerStyle::Menu);
        assert_eq!(Picker::new("P").inline_picker().picker_style(), PickerStyle::Inline);
        assert_eq!(Picker::new("P").tabs().picker_style(), PickerStyle::Tabs);
        assert_eq!(Picker::new("P").navigation_link().picker_style(), PickerStyle::NavigationLink);
    }

    #[test]
    fn picker_wheel_item_height() {
        let p = Picker::new("P").wheel_item_height(28.0).wheel();
        assert!((p.wheel_item_height - 28.0).abs() < 1e-4);
        let p2 = Picker::new("P").wheel_item_height(55.0);
        assert!((p2.wheel_item_height - 55.0).abs() < 1e-4);
    }

    #[test]
    fn picker_horizontal_radio_group() {
        let p = Picker::new("P").horizontal_radio_group(true).radio_group();
        assert!(p.horizontal_radio);
        let p2 = Picker::new("P").horizontal_radio_group(false).radio_group();
        assert!(!p2.horizontal_radio);
    }

    #[test]
    fn picker_section_and_divider() {
        let p = Picker::new("Foo")
            .section(PickerSection::new("Section A").items(vec![PickerItem::new("1"), PickerItem::new("2")]).divider())
            .section(PickerSection::new("Section B").item(PickerItem::new("3")));
        assert_eq!(p.sections.len(), 2);
        assert!(p.sections[0].has_divider);
        assert_eq!(p.flat_items().len(), 3);
        assert_eq!(p.flat_items()[2].title, "3");
    }

    #[test]
    fn picker_divider_flat() {
        let mut p = Picker::new("Foo").titles(["1", "2", "3"]).divider();
        assert_eq!(p.dividers, vec![2]);
        p = p.divider();
        assert_eq!(p.dividers.len(), 2);
    }

    #[test]
    fn picker_custom_value_label() {
        let p = Picker::new("Foo")
            .titles(["1", "2", "3"])
            .selected_index(1)
            .custom_value_label("Current", |v| format!("Current: {v}"));
        assert_eq!(p.display_label(), "Current: 2");
        assert_eq!(p.label_key.as_deref(), Some("Current"));
        let p2 = Picker::new("Foo").titles(["A", "B"]).custom_value_label("Key: {}", |v| format!("K:{v}"));
        assert_eq!(p2.label_key.as_deref(), Some("Key: {}"));
    }

    #[test]
    fn picker_multiple_sources_flag() {
        let p = Picker::new("P")
            .titles(["a", "b"])
            .multiple_sources(true)
            .on_change_multiple(|_| {})
            .on_change_multiple(|_| {});
        assert!(p.multiple_sources);
        assert_eq!(p.multiple_on_change.len(), 2);
    }

    #[test]
    fn picker_selected_by_value() {
        let p = Picker::new("P").titles(["Foo", "Bar", "Baz"]).selected("Bar");
        assert_eq!(p.selection, 1);
        assert_eq!(p.selected_value(), "Bar");
    }

    #[test]
    fn picker_selected_by_value_in_sections() {
        let p = Picker::new("Foo")
            .section(PickerSection::new("S1").items(vec![PickerItem::new("1"), PickerItem::new("2")]))
            .section(PickerSection::new("S2").item(PickerItem::new("3")))
            .selected("3");
        assert_eq!(p.selection, 2);
        assert_eq!(p.selected_value(), "3");
    }

    #[test]
    fn picker_sample_items_flat_len() {
        let items = sample_items();
        assert_eq!(items.len(), 4);
        assert_eq!(items[3].subtitle.as_deref(), Some("2"));
    }

    #[test]
    fn picker_default_style() {
        let p = Picker::default();
        assert_eq!(p.picker_style(), PickerStyle::Automatic);
        assert_eq!(p.wheel_item_height, 40.0);
        assert!(!p.horizontal_radio);
    }
}
