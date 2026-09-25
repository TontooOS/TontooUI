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

/// Static text list: plain rows separated by full-bleed hairlines,
/// like the reference (Row 1..Row 10 with dividers between rows and
/// no divider after the last row). Display-only, no selection, no
/// hover, no mouse handling.
///
/// The list is full bleed: the intrinsic width is the divider fill
/// extent, so stacks hand it the whole parent width with no edge
/// gap. Row text follows the theme (white/black) and dividers follow
/// the theme divider color unless the dev sets manual colors.
pub struct BasicList {
    rows: Vec<String>,
    row_h: f32,
    font_size: f32,
    pad_x: f32,
    show_dividers: bool,
    text_color: Color,
    text_manual: bool,
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
        Self {
            rows,
            row_h: LIST_ROW_H,
            font_size: LIST_FONT_SIZE,
            pad_x: LIST_PAD_X,
            show_dividers: true,
            text_color: Color::WHITE,
            text_manual: false,
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

    /// Clear manual colors so the list follows the theme again.
    pub fn clear_manual_colors(mut self) -> Self {
        self.text_manual = false;
        self.divider_manual = false;
        self.text_color = if self.dark {
            Color::WHITE
        } else {
            Color::BLACK
        };
        self.divider_color = if self.dark {
            DIVIDER_DARK
        } else {
            DIVIDER_LIGHT
        };
        self
    }

    /// Live theme: row text plus mode colors and the divider line. A
    /// manually set text/divider color wins over the system one.
    pub fn set_theme(&mut self, divider: Color, dark: bool) {
        self.dark = dark;
        if !self.text_manual {
            self.text_color = if dark {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }
        if !self.divider_manual {
            self.divider_color = divider;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Replace all rows.
    pub fn set_rows(&mut self, rows: Vec<String>) {
        self.rows = rows;
    }

    /// Replace all rows from a slice.
    pub fn set_rows_slice(&mut self, rows: &[&str]) {
        self.rows = rows.iter().map(|s| s.to_string()).collect();
    }

    /// Push one row at the end.
    pub fn push_row(&mut self, row: impl Into<String>) {
        self.rows.push(row.into());
    }

    /// Remove all rows.
    pub fn clear(&mut self) {
        self.rows.clear();
    }

    pub fn rows(&self) -> &[String] {
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

    /// Total content height for the current rows in logical px.
    pub fn content_height(&self) -> f32 {
        if self.rows.is_empty() {
            return 0.0;
        }
        let dividers = if self.show_dividers {
            (self.rows.len() - 1) as f32 * LIST_DIVIDER_H
        } else {
            0.0
        };
        self.rows.len() as f32 * self.row_h + dividers
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
            let layout = fonts.layout_text(
                row,
                self.font_size,
                self.eff(self.text_color),
                Some((self.width - self.pad_x).max(0.0)),
            );
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.x + self.pad_x,
                ry + (self.row_h - th / scale) / 2.0,
                scale,
            );
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
        assert_eq!(list.rows()[0], "Row 1");
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
