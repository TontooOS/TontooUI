use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};

use super::super::dividers::{DIVIDER_DARK, DIVIDER_FILL, DIVIDER_LIGHT, DIVIDER_THIN};
use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Row height in logical px.
pub const LIST_ROW_H: f32 = 28.0;
/// Row text size in logical px.
pub const LIST_FONT_SIZE: f32 = 13.0;
/// Horizontal text inset in logical px.
pub const LIST_PAD_X: f32 = 12.0;
/// Divider thickness between rows in logical px.
pub const LIST_DIVIDER_H: f32 = DIVIDER_THIN;
/// Extra space above a section title row in logical px (none before
/// the first row).
pub const LIST_SECTION_GAP: f32 = 20.0;

/// One list row: a plain item with an optional right-aligned badge,
/// or a dim section title that groups the items below it. Sections
/// and badges are display-only, like the rest of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListRow {
    Item { text: String, badge: Option<String> },
    Section(String),
}

impl ListRow {
    /// Plain item row without a badge.
    pub fn item(text: impl Into<String>) -> Self {
        Self::Item {
            text: text.into(),
            badge: None,
        }
    }

    /// Dim section title row (e.g. `"Grouped"`).
    pub fn section(title: impl Into<String>) -> Self {
        Self::Section(title.into())
    }

    /// Right-aligned badge text (e.g. `"5"`, `"12"`). Only affects
    /// item rows; a no-op on section titles.
    pub fn badge(self, badge: impl Into<String>) -> Self {
        match self {
            Self::Item { text, .. } => Self::Item {
                text,
                badge: Some(badge.into()),
            },
            section => section,
        }
    }

    /// Main row text (item text or section title).
    pub fn text(&self) -> &str {
        match self {
            Self::Item { text, .. } => text,
            Self::Section(title) => title,
        }
    }

    /// Badge text, if the row is an item with a badge.
    pub fn badge_text(&self) -> Option<&str> {
        match self {
            Self::Item { badge, .. } => badge.as_deref(),
            Self::Section(_) => None,
        }
    }

    /// True for section title rows.
    pub fn is_section(&self) -> bool {
        matches!(self, Self::Section(_))
    }
}

/// Static text list: plain rows separated by full-bleed hairlines,
/// like the reference (Row 1..Row 10 with dividers between rows and
/// no divider after the last row). Items can carry a right-aligned
/// badge (`Inbox — 5`) and section titles (`Grouped`) group the
/// items below them. Display-only, no selection, no hover, no mouse
/// handling.
///
/// The list is full bleed: the intrinsic width is the divider fill
/// extent, so stacks hand it the whole parent width with no edge
/// gap. Row text follows the theme (white/black), badges and
/// section titles use the dim theme text, and dividers follow the
/// theme divider color unless the dev sets manual colors.
pub struct BasicList {
    rows: Vec<ListRow>,
    row_h: f32,
    font_size: f32,
    pad_x: f32,
    show_dividers: bool,
    text_color: Color,
    text_manual: bool,
    dim_color: Color,
    dim_manual: bool,
    divider_color: Color,
    divider_manual: bool,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl BasicList {
    pub fn new(rows: Vec<String>) -> Self {
        Self::from_rows(rows.into_iter().map(ListRow::item).collect())
    }

    /// List from fully specified rows (items, badges, sections).
    pub fn from_rows(rows: Vec<ListRow>) -> Self {
        Self {
            rows,
            row_h: LIST_ROW_H,
            font_size: LIST_FONT_SIZE,
            pad_x: LIST_PAD_X,
            show_dividers: true,
            text_color: Color::WHITE,
            text_manual: false,
            dim_color: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            dim_manual: false,
            divider_color: DIVIDER_DARK,
            divider_manual: false,
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Convenience constructor from a slice (e.g. `["Row 1", "Row 2"]`).
    pub fn from_slice(rows: &[&str]) -> Self {
        Self::new(rows.iter().map(|s| s.to_string()).collect())
    }

    /// Row height in logical px (clamped to >= 0).
    pub fn row_height(mut self, px: f32) -> Self {
        self.row_h = px.max(0.0);
        self
    }

    /// Row text size in logical px (clamped to >= 1).
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_size = px.max(1.0);
        self
    }

    /// Horizontal text inset in logical px (clamped to >= 0).
    pub fn padding(mut self, px: f32) -> Self {
        self.pad_x = px.max(0.0);
        self
    }

    /// Hairlines between rows (shown by default).
    pub fn show_dividers(mut self, show: bool) -> Self {
        self.show_dividers = show;
        self
    }

    /// Manual row text color: wins over the theme until cleared via
    /// `set_theme` on a non-manual list (see `apply_theme_preset`
    /// flow: rebuild or call `clear_manual_colors`).
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self.text_manual = true;
        self
    }

    /// Manual divider color: wins over the theme divider color.
    pub fn divider_color(mut self, color: Color) -> Self {
        self.divider_color = color;
        self.divider_manual = true;
        self
    }

    /// Manual dim color for badges and section titles: wins over the
    /// theme dim text.
    pub fn dim_color(mut self, color: Color) -> Self {
        self.dim_color = color;
        self.dim_manual = true;
        self
    }

    /// Clear manual colors so the list follows the theme again.
    pub fn clear_manual_colors(mut self) -> Self {
        self.text_manual = false;
        self.dim_manual = false;
        self.divider_manual = false;
        self.text_color = if self.dark {
            Color::WHITE
        } else {
            Color::BLACK
        };
        self.dim_color = if self.dark {
            Color::from_rgb8(0x9a, 0x9a, 0x9e)
        } else {
            Color::from_rgb8(0x6e, 0x6e, 0x72)
        };
        self.divider_color = if self.dark {
            DIVIDER_DARK
        } else {
            DIVIDER_LIGHT
        };
        self
    }

    /// Live theme: row text plus mode colors, dim text (badges,
    /// section titles) and the divider line. A manually set
    /// text/dim/divider color wins over the system one.
    pub fn set_theme(&mut self, divider: Color, dark: bool) {
        self.dark = dark;
        if !self.text_manual {
            self.text_color = if dark {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }
        if !self.dim_manual {
            self.dim_color = if dark {
                Color::from_rgb8(0x9a, 0x9a, 0x9e)
            } else {
                Color::from_rgb8(0x6e, 0x6e, 0x72)
            };
        }
        if !self.divider_manual {
            self.divider_color = divider;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Replace all rows with plain items.
    pub fn set_rows(&mut self, rows: Vec<String>) {
        self.rows = rows.into_iter().map(ListRow::item).collect();
    }

    /// Replace all rows with plain items from a slice.
    pub fn set_rows_slice(&mut self, rows: &[&str]) {
        self.rows = rows.iter().map(|s| ListRow::item(*s)).collect();
    }

    /// Replace all rows with fully specified rows (items, badges,
    /// sections).
    pub fn set_list_rows(&mut self, rows: Vec<ListRow>) {
        self.rows = rows;
    }

    /// Push one plain item row at the end.
    pub fn push_row(&mut self, row: impl Into<String>) {
        self.rows.push(ListRow::item(row));
    }

    /// Push one item row at the end.
    pub fn push_item(&mut self, text: impl Into<String>) {
        self.rows.push(ListRow::item(text));
    }

    /// Push one section title row at the end.
    pub fn push_section(&mut self, title: impl Into<String>) {
        self.rows.push(ListRow::section(title));
    }

    /// Remove all rows.
    pub fn clear(&mut self) {
        self.rows.clear();
    }

    pub fn rows(&self) -> &[ListRow] {
        &self.rows
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Row height in logical px.
    pub fn set_row_height(&mut self, px: f32) {
        self.row_h = px.max(0.0);
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    /// Total content height for the current rows in logical px:
    /// rows times row height, one hairline per inner gap, plus the
    /// section gap above every section title except a leading one.
    pub fn content_height(&self) -> f32 {
        if self.rows.is_empty() {
            return 0.0;
        }
        let dividers = if self.show_dividers {
            (self.rows.len() - 1) as f32 * LIST_DIVIDER_H
        } else {
            0.0
        };
        let sections = self.rows.iter().skip(1).filter(|r| r.is_section()).count();
        self.rows.len() as f32 * self.row_h
            + dividers
            + sections as f32 * LIST_SECTION_GAP
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }
}

impl Default for BasicList {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl View for BasicList {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (DIVIDER_FILL, self.content_height())
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
    ) {
        if self.rows.is_empty() || self.width <= 0.0 || self.row_h <= 0.0 {
            return;
        }
        let scale = fonts.scale;
        let rows = std::mem::take(&mut self.rows);
        let mut ry = self.y;
        for (i, row) in rows.iter().enumerate() {
            // Section titles breathe: extra gap above, except first.
            if i > 0 && row.is_section() {
                ry += LIST_SECTION_GAP;
            }
            let color = if row.is_section() {
                self.eff(self.dim_color)
            } else {
                self.eff(self.text_color)
            };
            let badge = row.badge_text();
            let max_text = if badge.is_some() {
                (self.width - self.pad_x * 2.0).max(0.0)
            } else {
                (self.width - self.pad_x).max(0.0)
            };
            let layout = fonts.layout_text(row.text(), self.font_size, color, Some(max_text));
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.x + self.pad_x,
                ry + (self.row_h - th / scale) / 2.0,
                scale,
            );
            // Right-aligned dim badge (e.g. Inbox — 5).
            if let Some(badge) = badge {
                let badge_layout = fonts.layout_text(
                    badge,
                    self.font_size,
                    self.eff(self.dim_color),
                    Some(max_text),
                );
                let (bw, bh) = FontSystem::layout_size(&badge_layout);
                draw_layout(
                    scene,
                    &badge_layout,
                    self.x + self.width - self.pad_x - bw / scale,
                    ry + (self.row_h - bh / scale) / 2.0,
                    scale,
                );
            }
            ry += self.row_h;
            // Hairline between rows only, never after the last row.
            if self.show_dividers && i + 1 < rows.len() {
                let line = Rect::new(
                    self.x as f64 * scale as f64,
                    ry as f64 * scale as f64,
                    (self.x + self.width) as f64 * scale as f64,
                    (ry + LIST_DIVIDER_H) as f64 * scale as f64,
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.divider_color)),
                    None,
                    &line,
                );
                ry += LIST_DIVIDER_H;
            }
        }
        self.rows = rows;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> BasicList {
        BasicList::from_slice(&["Row 1", "Row 2", "Row 3"])
    }

    #[test]
    fn content_height_counts_rows_and_inner_dividers() {
        let list = BasicList::from_slice(&["Row 1", "Row 2", "Row 3"]);
        assert_eq!(
            list.content_height(),
            3.0 * LIST_ROW_H + 2.0 * LIST_DIVIDER_H
        );
        let single = BasicList::from_slice(&["Row 1"]);
        assert_eq!(single.content_height(), LIST_ROW_H);
        let empty = BasicList::new(Vec::new());
        assert_eq!(empty.content_height(), 0.0);
    }

    #[test]
    fn hidden_dividers_take_no_space() {
        let list = BasicList::from_slice(&["Row 1", "Row 2"]).show_dividers(false);
        assert_eq!(list.content_height(), 2.0 * LIST_ROW_H);
    }

    #[test]
    fn fills_parent_width() {
        let mut fonts = FontSystem::new();
        let mut list = list();
        let (w, h) = list.measure(&mut fonts);
        assert!(w >= DIVIDER_FILL);
        assert_eq!(h, list.content_height());
        list.place(&mut fonts, 0.0, 0.0, 800.0, h);
        assert_eq!(list.rect().2, 800.0);
    }

    #[test]
    fn from_slice_collects_rows() {
        let list = BasicList::from_slice(&["Row 1", "Row 2"]);
        assert_eq!(list.row_count(), 2);
        assert_eq!(list.rows()[0].text(), "Row 1");
        assert_eq!(list.rows()[0].badge_text(), None);
        assert!(!list.rows()[0].is_section());
    }

    #[test]
    fn section_gap_counts_once_per_inner_section() {
        let list = BasicList::from_rows(vec![
            ListRow::item("Item 1"),
            ListRow::item("Item 2"),
            ListRow::section("Grouped"),
            ListRow::item("Item 3"),
            ListRow::item("Item 4"),
        ]);
        assert_eq!(
            list.content_height(),
            5.0 * LIST_ROW_H + 4.0 * LIST_DIVIDER_H + LIST_SECTION_GAP
        );
        // Leading section: no gap above the first row.
        let leading = BasicList::from_rows(vec![
            ListRow::section("Grouped"),
            ListRow::item("Item 1"),
        ]);
        assert_eq!(
            leading.content_height(),
            2.0 * LIST_ROW_H + LIST_DIVIDER_H
        );
    }

    #[test]
    fn badges_attach_to_items_only() {
        let item = ListRow::item("Inbox").badge("5");
        assert_eq!(item.badge_text(), Some("5"));
        let section = ListRow::section("Grouped").badge("5");
        assert_eq!(section.badge_text(), None);
        assert!(section.is_section());
    }

    #[test]
    fn push_section_and_set_list_rows() {
        let mut list = BasicList::new(Vec::new());
        list.push_item("Item 1");
        list.push_section("Grouped");
        assert_eq!(list.row_count(), 2);
        assert!(list.rows()[1].is_section());
        list.set_list_rows(vec![ListRow::item("Inbox").badge("100")]);
        assert_eq!(list.rows()[0].badge_text(), Some("100"));
    }

    #[test]
    fn push_and_clear_rows() {
        let mut list = BasicList::new(Vec::new());
        list.push_row("Row 1");
        list.push_row("Row 2");
        assert_eq!(list.row_count(), 2);
        list.clear();
        assert_eq!(list.row_count(), 0);
        assert_eq!(list.content_height(), 0.0);
    }

    #[test]
    fn manual_colors_win_over_theme() {
        let mut list =
            BasicList::from_slice(&["Row 1"]).text_color(Color::from_rgb8(
                0xff, 0x2d, 0x55,
            ));
        list.set_theme(DIVIDER_LIGHT, false);
        assert_eq!(
            list.text_color,
            Color::from_rgb8(0xff, 0x2d, 0x55)
        );
        let mut plain = BasicList::from_slice(&["Row 1"]);
        plain.set_theme(DIVIDER_LIGHT, false);
        assert_eq!(plain.text_color, Color::BLACK);
        assert_eq!(plain.divider_color, DIVIDER_LIGHT);
    }

    #[test]
    fn clamps_geometry_builders() {
        let list = BasicList::new(Vec::new())
            .row_height(-4.0)
            .font_size(-2.0)
            .padding(-8.0);
        assert_eq!(list.row_h, 0.0);
        assert_eq!(list.font_size, 1.0);
        assert_eq!(list.pad_x, 0.0);
    }
}
