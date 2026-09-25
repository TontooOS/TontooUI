use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Fill};

use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::super::text::BasicText;
use super::super::toggles::{Toggle, ToggleStyle};
use super::{GROUP_BG_DARK, GROUP_BG_LIGHT, GROUP_PAD, GROUP_RADIUS};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::{ThemeMode, desaturate};

/// Gap between the leading element and the label in logical px.
pub const GROUP_ROW_GAP: f32 = 12.0;
/// Gap between rows in logical px.
pub const GROUP_ROW_SPACING: f32 = 14.0;
/// Leading SF icon box in logical px.
pub const GROUP_SYMBOL_SIZE: f32 = 24.0;

/// One styled row: a leading element (toggle, icon, toolbar, ...)
/// plus a label.
struct GroupRow {
    leading: Box<dyn View>,
    label: BasicText,
    text: String,
}

/// Styled group box: rows with a leading element and a label in one
/// rounded box (like the reference settings rows: checked blue box
/// plus "Notifications", gray box plus "Dark Mode"). The leading
/// slot takes any view — toggles, SF icons, toolbars — and stays
/// interactive: presses and hovers forward through the `View`
/// protocol. Theme and focus of leading elements stay with the app
/// (see `row_leading_mut`); labels follow `set_theme` themselves.
pub struct StyledGroupBox {
    rows: Vec<GroupRow>,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl StyledGroupBox {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Custom row: any leading view plus a label.
    pub fn row(mut self, leading: impl View + 'static, label: impl Into<String>) -> Self {
        self.push_row(Box::new(leading), label.into());
        self
    }

    /// Switch toggle row, bound to `on`.
    pub fn toggle_row(mut self, label: impl Into<String>, on: bool) -> Self {
        self.push_row(
            Box::new(Toggle::new("").style(ToggleStyle::Switch).on(on)),
            label.into(),
        );
        self
    }

    /// Checkbox row, bound to `checked` (like the reference rows).
    pub fn check_row(mut self, label: impl Into<String>, checked: bool) -> Self {
        self.push_row(
            Box::new(Toggle::new("").style(ToggleStyle::Checkbox).on(checked)),
            label.into(),
        );
        self
    }

    /// SF Symbol row with a fixed-size leading icon.
    pub fn symbol_row(mut self, symbol: impl Into<String>, label: impl Into<String>) -> Self {
        self.push_row(
            Box::new(SFSymbolImage::new(symbol.into()).size(GROUP_SYMBOL_SIZE)),
            label.into(),
        );
        self
    }

    fn push_row(&mut self, leading: Box<dyn View>, label: String) {
        self.rows.push(GroupRow {
            leading,
            label: BasicText::new(label.clone()),
            text: label,
        });
        // Labels follow the box theme right away.
        if let Some(row) = self.rows.last_mut() {
            row.label.set_theme(if self.dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            });
            row.label.set_focused(self.focused);
        }
    }

    /// Number of rows.
    pub fn row_len(&self) -> usize {
        self.rows.len()
    }

    /// Leading element of a row for state and theme updates
    /// (toggles, icons, ...). `None` for bad indices or types.
    pub fn row_leading_mut<T: View + 'static>(&mut self, index: usize) -> Option<&mut T> {
        self.rows
            .get_mut(index)?
            .leading
            .as_any_mut()
            .downcast_mut::<T>()
    }

    /// Row label text.
    pub fn row_label(&self, index: usize) -> Option<&str> {
        self.rows.get(index).map(|row| row.text.as_str())
    }

    pub fn set_row_label(&mut self, index: usize, label: impl Into<String>) {
        if let Some(row) = self.rows.get_mut(index) {
            let label = label.into();
            if label != row.text {
                row.text = label.clone();
                row.label.set_text(label);
            }
        }
    }

    /// Live theme for the box fill and the labels. Leading elements
    /// keep their own theme: wire them through `row_leading_mut`
    /// (see the demo).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.dark = mode == ThemeMode::Dark;
        for row in &mut self.rows {
            row.label.set_theme(mode);
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        for row in &mut self.rows {
            row.label.set_focused(focused);
        }
    }

    /// Forward hover to the leading elements (toggles highlight).
    pub fn set_hover(&mut self, x: f32, y: f32) {
        for row in &mut self.rows {
            row.leading.set_hover(x, y);
        }
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn fill(&self) -> vello::peniko::Color {
        let base = if self.dark {
            GROUP_BG_DARK
        } else {
            GROUP_BG_LIGHT
        };
        if self.focused {
            base
        } else {
            desaturate(base)
        }
    }

    fn row_sizes(&mut self, fonts: &mut FontSystem) -> Vec<(f32, f32, f32, f32)> {
        // (leading_w, leading_h, label_w, label_h) per row.
        let mut out = Vec::with_capacity(self.rows.len());
        for row in self.rows.iter_mut() {
            let (lw, lh) = row.leading.measure(fonts);
            let (tw, th) = row.label.measure(fonts);
            out.push((lw, lh, tw, th));
        }
        out
    }

    fn layout_rows(&mut self, fonts: &mut FontSystem) {
        let sizes = self.row_sizes(fonts);
        let mut y = self.y + GROUP_PAD;
        for (row, &(lw, lh, tw, th)) in self.rows.iter_mut().zip(sizes.iter()) {
            let row_h = lh.max(th);
            row.leading.place(fonts, self.x + GROUP_PAD, y + (row_h - lh) / 2.0, lw, lh);
            row.label.place(
                fonts,
                self.x + GROUP_PAD + lw + GROUP_ROW_GAP,
                y + (row_h - th) / 2.0,
                tw,
                th,
            );
            y += row_h + GROUP_ROW_SPACING;
        }
    }
}

impl Default for StyledGroupBox {
    fn default() -> Self {
        Self::new()
    }
}

impl View for StyledGroupBox {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let sizes = self.row_sizes(fonts);
        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        for (index, (lw, lh, tw, th)) in sizes.iter().enumerate() {
            w = w.max(lw + GROUP_ROW_GAP + tw);
            h += lh.max(*th);
            if index + 1 < sizes.len() {
                h += GROUP_ROW_SPACING;
            }
        }
        (w + GROUP_PAD * 2.0, h + GROUP_PAD * 2.0)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
        self.layout_rows(fonts);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let body = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
            GROUP_RADIUS as f64 * scale,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.fill()),
            None,
            &body,
        );
        for row in &mut self.rows {
            row.leading.draw(scene, fonts, images);
            row.label.draw(scene, fonts, images);
        }
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        for row in &mut self.rows {
            row.leading.mouse_down(x, y);
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        for row in &mut self.rows {
            row.leading.mouse_up(x, y);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::buttons::Button;
    use crate::renderer::text::FontSystem;

    #[test]
    fn checkbox_rows_match_reference() {
        let group = StyledGroupBox::new()
            .check_row("Notifications", true)
            .check_row("Dark Mode", false)
            .check_row("Location Services", true);
        assert_eq!(group.row_len(), 3);
        assert_eq!(group.row_label(0), Some("Notifications"));
        assert_eq!(group.row_label(5), None);
    }

    #[test]
    fn checkbox_click_flips_state() {
        let mut group = StyledGroupBox::new().check_row("Dark Mode", false);
        let mut fonts = FontSystem::new();
        let (w, h) = group.measure(&mut fonts);
        group.place(&mut fonts, 0.0, 0.0, w, h);
        // Click into the leading checkbox (left padded corner).
        group.mouse_down((GROUP_PAD + 4.0) as f64, (GROUP_PAD + 10.0) as f64);
        group.mouse_up((GROUP_PAD + 4.0) as f64, (GROUP_PAD + 10.0) as f64);
        let toggle = group.row_leading_mut::<Toggle>(0).expect("toggle");
        assert!(toggle.is_on());
    }

    #[test]
    fn toggle_and_symbol_rows_store_state() {
        let mut group = StyledGroupBox::new()
            .toggle_row("Wi-Fi", true)
            .symbol_row("wifi", "Network");
        assert_eq!(group.row_len(), 2);
        assert!(group.row_leading_mut::<Toggle>(0).expect("toggle").is_on());
        assert!(group
            .row_leading_mut::<SFSymbolImage>(1)
            .expect("symbol")
            .name_value()
            == "wifi");
        // Wrong type misses.
        assert!(group.row_leading_mut::<Button>(0).is_none());
    }

    #[test]
    fn set_row_label_updates_text() {
        let mut group = StyledGroupBox::new().check_row("Old", false);
        group.set_row_label(0, "New");
        assert_eq!(group.row_label(0), Some("New"));
        group.set_row_label(9, "Miss");
        assert_eq!(group.row_len(), 1);
    }

    #[test]
    fn measure_grows_with_rows() {
        let mut fonts = FontSystem::new();
        let mut one = StyledGroupBox::new().check_row("A", false);
        let mut two = StyledGroupBox::new()
            .check_row("A", false)
            .check_row("B", false);
        assert!(two.measure(&mut fonts).1 > one.measure(&mut fonts).1);
    }
}
