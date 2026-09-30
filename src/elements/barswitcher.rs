use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::glass::{GlassContainer, GlassType};
use super::layout::View;
use super::toolbar::{
    TOOLBAR_GAP, TOOLBAR_HEIGHT, TOOLBAR_HIT, TOOLBAR_ICON_SIZE, TOOLBAR_PAD_X,
    TOOLBAR_RADIUS, ToolbarPlacement,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Label size in logical px for text cells.
pub const BARSWITCHER_FONT_SIZE: f32 = 13.0;
/// Gap between icon and label inside a combined cell in logical px.
pub const BARSWITCHER_ICON_TEXT_GAP: f32 = 6.0;
/// Extra side padding for cells holding a label in logical px.
pub const BARSWITCHER_TEXT_PAD_X: f32 = 10.0;
/// Minimum cell width in logical px once a cell holds a label, so
/// short labels still read as pills.
pub const BARSWITCHER_MIN_W: f32 = 44.0;
/// Selected label weight (semibold); unselected labels use regular.
pub const BARSWITCHER_SELECTED_WEIGHT: f32 = 600.0;
/// Unselected label weight.
pub const BARSWITCHER_WEIGHT: f32 = 400.0;
/// Icon/label color in dark mode.
pub const BARSWITCHER_COLOR_DARK: Color = Color::from_rgb8(0xd8, 0xd9, 0xd9);
/// Icon/label color in light mode.
pub const BARSWITCHER_COLOR_LIGHT: Color = Color::from_rgb8(0x27, 0x27, 0x27);

/// One switcher cell: an SF Symbol icon, a text label, or both side by
/// side. At least one side should be set; fully empty items take no
/// space and never hit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BarSwitcherItem {
    pub icon: Option<String>,
    pub label: Option<String>,
}

impl BarSwitcherItem {
    /// Icon-only cell (e.g. `"list.bullet"`).
    pub fn icon(name: impl Into<String>) -> Self {
        Self {
            icon: Some(name.into()),
            label: None,
        }
    }

    /// Text-only cell (e.g. `"Settings"`).
    pub fn text(label: impl Into<String>) -> Self {
        Self {
            icon: None,
            label: Some(label.into()),
        }
    }

    /// Combined cell: icon left, label right.
    pub fn both(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            icon: Some(name.into()),
            label: Some(label.into()),
        }
    }

    fn is_empty(&self) -> bool {
        self.icon.is_none() && self.label.is_none()
    }
}

/// Bar switcher: the `BasicToolbar` capsule in the clear (`Lens`) glass
/// finish, but each cell is a single-select option holding an icon, a
/// text label, or both. The selected cell keeps the gray hover-style
/// highlight; clicking another cell moves the selection there and
/// fires `on_select` with the new index so the app can swap views.
/// Clicking the already-selected cell fires nothing.
///
/// Selection changes on release inside the armed cell (`mouse_down`
/// arms, `View::mouse_up` selects); the shell forwards both. Like the
/// toolbar the finish stays `Lens` and never switches to `Frosted`.
pub struct BarSwitcher {
    items: Vec<BarSwitcherItem>,
    selected: usize,
    placement: ToolbarPlacement,
    dark: bool,
    focused: bool,
    disabled: bool,
    color: Color,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    cells: Vec<(f32, f32, f32, f32)>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    armed: bool,
    glass: GlassContainer,
    on_select: Option<Box<dyn FnMut(usize)>>,
}

impl BarSwitcher {
    pub fn new() -> Self {
        let glass = GlassContainer::new().glass_type(GlassType::Lens);
        Self {
            items: Vec::new(),
            selected: 0,
            placement: ToolbarPlacement::Center,
            dark: true,
            focused: true,
            disabled: false,
            color: BARSWITCHER_COLOR_DARK,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: TOOLBAR_HEIGHT,
            cells: Vec::new(),
            hovered: None,
            pressed: None,
            armed: false,
            glass,
            on_select: None,
        }
    }

    pub fn from_items(items: Vec<BarSwitcherItem>) -> Self {
        Self::new().items(items)
    }

    pub fn items(mut self, items: Vec<BarSwitcherItem>) -> Self {
        self.items = items;
        self.selected = self.clamp_index(self.selected);
        self
    }

    /// Push any item (icon, text or both).
    pub fn item(mut self, item: BarSwitcherItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn set_items(&mut self, items: Vec<BarSwitcherItem>) {
        self.items = items;
        self.selected = self.clamp_index(self.selected);
    }

    /// Initial selection without firing `on_select`.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = self.clamp_index(index);
        self
    }

    /// Cell placement inside the toolbar rect.
    pub fn placement(mut self, placement: ToolbarPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn set_placement(&mut self, placement: ToolbarPlacement) {
        self.placement = placement;
    }

    /// Selection event: fires with the new index whenever the
    /// selection changes (click, `press`, `select`, `set_selected`).
    /// Clicking the already-selected cell fires nothing.
    pub fn on_select(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn set_on_select(&mut self, callback: impl FnMut(usize) + 'static) {
        self.on_select = Some(Box::new(callback));
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Live theme: dark flag drives the icon/label color plus the
    /// hover/press tint direction; the glass amount is forwarded to
    /// the `Lens` body. The finish stays `Lens`.
    pub fn set_theme(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.color = if self.dark {
            BARSWITCHER_COLOR_DARK
        } else {
            BARSWITCHER_COLOR_LIGHT
        };
        self.glass.set_glass_type(GlassType::Lens);
        self.glass.set_theme(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_item(&self) -> Option<&BarSwitcherItem> {
        self.items.get(self.selected)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn clamp_index(&self, index: usize) -> usize {
        if self.items.is_empty() {
            0
        } else {
            index.min(self.items.len() - 1)
        }
    }

    /// Select immediately. Fires `on_select` when the selection changed.
    pub fn select(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.notify();
        }
    }

    /// Alias for `select`: no animation exists, both apply instantly.
    pub fn set_selected(&mut self, index: usize) {
        self.select(index);
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_select.as_mut() {
            callback(self.selected);
        }
    }

    fn text_w(&self, fonts: &mut FontSystem, label: &str) -> f32 {
        let layout = fonts.layout_text(label, BARSWITCHER_FONT_SIZE, Color::WHITE, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    fn cell_content_w(&self, fonts: &mut FontSystem, item: &BarSwitcherItem) -> f32 {
        let icon_w = if item.icon.is_some() {
            TOOLBAR_ICON_SIZE
        } else {
            0.0
        };
        let label_w = item
            .label
            .as_deref()
            .map(|label| self.text_w(fonts, label))
            .unwrap_or(0.0);
        let gap = if item.icon.is_some() && item.label.is_some() {
            BARSWITCHER_ICON_TEXT_GAP
        } else {
            0.0
        };
        icon_w + gap + label_w
    }

    fn cell_w(&self, fonts: &mut FontSystem, item: &BarSwitcherItem) -> f32 {
        if item.is_empty() {
            return 0.0;
        }
        if item.label.is_none() {
            return TOOLBAR_HIT;
        }
        (self.cell_content_w(fonts, item) + BARSWITCHER_TEXT_PAD_X * 2.0)
            .max(BARSWITCHER_MIN_W)
    }

    fn content_width(&self, fonts: &mut FontSystem) -> f32 {
        if self.items.is_empty() {
            return TOOLBAR_PAD_X * 2.0;
        }
        let widths: Vec<f32> = self
            .items
            .iter()
            .map(|item| self.cell_w(fonts, item))
            .collect();
        self.content_width_cached(&widths)
    }

    fn layout_cells(&mut self, fonts: &mut FontSystem) {
        self.cells.clear();
        let widths: Vec<f32> = self
            .items
            .iter()
            .map(|item| self.cell_w(fonts, item))
            .collect();
        let content = self.content_width_cached(&widths);
        let start_x = match self.placement {
            ToolbarPlacement::Leading => self.x + TOOLBAR_PAD_X,
            ToolbarPlacement::Center => {
                self.x + TOOLBAR_PAD_X + (self.width - content) / 2.0
            }
            ToolbarPlacement::Trailing => self.x + self.width - TOOLBAR_PAD_X - content,
        };
        let cell_y = self.y + (self.height - TOOLBAR_HIT) / 2.0;
        let mut cx = start_x;
        for (item, w) in self.items.iter().zip(widths.iter()) {
            if *w <= 0.0 || item.is_empty() {
                self.cells.push((0.0, 0.0, 0.0, 0.0));
                continue;
            }
            self.cells.push((cx, cell_y, *w, TOOLBAR_HIT));
            cx += *w + TOOLBAR_GAP;
        }
    }

    fn content_width_cached(&self, widths: &[f32]) -> f32 {
        let cells: f32 = widths.iter().sum();
        let gaps = widths.iter().filter(|w| **w > 0.0).count().max(1) as f32 - 1.0;
        TOOLBAR_PAD_X * 2.0 + cells + gaps * TOOLBAR_GAP
    }

    /// Hit index of the cell at `(x, y)`. Empty items never hit.
    fn cell_at(&self, x: f32, y: f32) -> Option<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(index, (_, _, w, h))| {
                *w > 0.0
                    && *h > 0.0
                    && !self.items.get(*index).is_none_or(BarSwitcherItem::is_empty)
            })
            .find(|(_, (cx, cy, cw, ch))| {
                x >= *cx && x <= *cx + *cw && y >= *cy && y <= *cy + *ch
            })
            .map(|(index, _)| index)
    }

    /// Update hover from logical cursor position.
    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.hovered = self.cell_at(x, y);
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.set_hover(x, y);
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        if let Some(index) = self.cell_at(x as f32, y as f32) {
            self.pressed = Some(index);
            self.hovered = Some(index);
            self.armed = true;
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn finish_press(&mut self, x: f64, y: f64) {
        let armed = self.armed;
        let pressed = self.pressed;
        self.armed = false;
        self.pressed = None;
        if armed && !self.disabled {
            if let Some(index) = pressed {
                if self.cell_at(x as f32, y as f32) == Some(index) {
                    self.select(index);
                }
            }
        }
    }

    /// Immediate selection. Selects the hit cell (firing `on_select`
    /// on change) and returns its index, otherwise `None`.
    pub fn press(&mut self, x: f32, y: f32) -> Option<usize> {
        if self.disabled {
            return None;
        }
        let hit = self.cell_at(x, y);
        if let Some(index) = hit {
            self.select(index);
        }
        hit
    }

    /// Gray cell tint: pressed deepens it, hover shows it, and the
    /// selected cell keeps it so the choice stays visible.
    /// Dark mode lightens (white overlay), light mode darkens it.
    fn state_tint(&self, hovered: bool, pressed: bool, selected: bool) -> Option<Color> {
        if self.disabled {
            return None;
        }
        if pressed {
            Some(if self.dark {
                Color::from_rgba8(255, 255, 255, 40)
            } else {
                Color::from_rgba8(0, 0, 0, 40)
            })
        } else if hovered || selected {
            Some(if self.dark {
                Color::from_rgba8(255, 255, 255, 20)
            } else {
                Color::from_rgba8(0, 0, 0, 15)
            })
        } else {
            None
        }
    }

    fn eff(&self, color: Color) -> Color {
        let mut color = if self.focused {
            color
        } else {
            desaturate(color)
        };
        if self.disabled {
            let c = color.to_rgba8();
            color = Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * 0.4).round() as u8);
        }
        color
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        // Glass body first (Lens finish; skips itself on capture pass).
        // Keep the finish locked to Lens even if a caller touched it.
        self.glass.set_glass_type(GlassType::Lens);
        self.glass.set_bounds(self.x, self.y, self.width, self.height);
        self.glass.set_radius(TOOLBAR_RADIUS);
        self.glass.draw(scene, fonts, images);
        if images.is_capture_pass() {
            return;
        }

        let scale = fonts.scale as f64;
        let color = self.eff(self.color);

        for (index, item) in self.items.clone().iter().enumerate() {
            let (cx, cy, cw, ch) = self.cells[index];
            if cw <= 0.0 || ch <= 0.0 {
                continue;
            }
            let hovered = self.hovered == Some(index);
            let pressed = self.pressed == Some(index) && self.armed;
            let selected = self.selected == index;
            if let Some(tint) = self.state_tint(hovered, pressed, selected) {
                let cell = RoundedRect::new(
                    (cx as f64) * scale,
                    (cy as f64) * scale,
                    ((cx + cw) as f64) * scale,
                    ((cy + ch) as f64) * scale,
                    (TOOLBAR_HIT / 2.0) as f64 * scale,
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(tint),
                    None,
                    &cell,
                );
            }

            // Icon and label centered as one group inside the cell.
            let label_w = item
                .label
                .as_deref()
                .map(|label| self.text_w(fonts, label))
                .unwrap_or(0.0);
            let icon_w = if item.icon.is_some() {
                TOOLBAR_ICON_SIZE
            } else {
                0.0
            };
            let gap = if item.icon.is_some() && item.label.is_some() {
                BARSWITCHER_ICON_TEXT_GAP
            } else {
                0.0
            };
            let mut tx = cx + (cw - (icon_w + gap + label_w)) / 2.0;

            if let Some(name) = item.icon.clone() {
                let target =
                    (TOOLBAR_ICON_SIZE * fonts.scale * 2.0).ceil().max(1.0) as u32;
                if let Some((image, iw, ih)) = images.get(&name, color, target) {
                    let s = (TOOLBAR_ICON_SIZE / iw as f32)
                        .min(TOOLBAR_ICON_SIZE / ih as f32);
                    let ix = tx + (icon_w - iw as f32 * s) / 2.0;
                    let iy = cy + (ch - ih as f32 * s) / 2.0;
                    let transform =
                        Affine::translate((ix as f64 * scale, iy as f64 * scale))
                            * Affine::scale(s as f64 * scale);
                    scene.draw_image(&image, transform);
                }
                tx += icon_w + gap;
            }

            if let Some(label) = item.label.clone() {
                let weight = if selected {
                    BARSWITCHER_SELECTED_WEIGHT
                } else {
                    BARSWITCHER_WEIGHT
                };
                let layout =
                    fonts.layout_text_weighted(&label, BARSWITCHER_FONT_SIZE, color, weight, None);
                let (_, th) = FontSystem::layout_size(&layout);
                let th = th / fonts.scale;
                let ty = cy + (ch - th) / 2.0;
                draw_layout(scene, &layout, tx, ty, fonts.scale);
            }
        }
    }
}

impl Default for BarSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl View for BarSwitcher {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        (self.content_width(fonts), TOOLBAR_HEIGHT)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, _h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(self.content_width(fonts));
        self.height = TOOLBAR_HEIGHT;
        self.layout_cells(fonts);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_press(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn switcher() -> BarSwitcher {
        BarSwitcher::from_items(vec![
            BarSwitcherItem::both("square.grid.2x2", "Grid"),
            BarSwitcherItem::icon("list.bullet"),
            BarSwitcherItem::text("Settings"),
        ])
        .on_select(|_| {})
    }

    #[test]
    fn item_constructors() {
        assert_eq!(
            BarSwitcherItem::icon("a"),
            BarSwitcherItem {
                icon: Some("a".to_string()),
                label: None,
            }
        );
        assert_eq!(
            BarSwitcherItem::text("A"),
            BarSwitcherItem {
                icon: None,
                label: Some("A".to_string()),
            }
        );
        assert!(!BarSwitcherItem::both("a", "A").is_empty());
        assert!(BarSwitcherItem::default().is_empty());
    }

    #[test]
    fn text_cells_are_wider_than_icon_cells() {
        let mut bar = switcher();
        let mut fonts = FontSystem::new();
        let (w, h) = bar.measure(&mut fonts);
        assert_eq!(h, TOOLBAR_HEIGHT);
        bar.place(&mut fonts, 0.0, 0.0, w, h);
        // Icon-only cell keeps the square hit target.
        assert_eq!(bar.cells[1].2, TOOLBAR_HIT);
        // Labeled cells grow past the square target.
        assert!(bar.cells[0].2 > TOOLBAR_HIT);
        assert!(bar.cells[2].2 > TOOLBAR_HIT);
    }

    #[test]
    fn click_selects_and_fires_once_on_change() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut bar = BarSwitcher::from_items(vec![
            BarSwitcherItem::text("One"),
            BarSwitcherItem::text("Two"),
        ])
        .on_select(move |_| count.set(count.get() + 1));
        let mut fonts = FontSystem::new();
        let (w, h) = bar.measure(&mut fonts);
        bar.place(&mut fonts, 0.0, 0.0, w, h);
        assert_eq!(bar.selected_index(), 0);

        // Release without press does nothing.
        bar.mouse_up(1000.0, 1000.0);
        assert_eq!(bar.selected_index(), 0);
        assert_eq!(fires.get(), 0);

        // Press and release on the second cell selects it once.
        let (cx, cy, cw, ch) = bar.cells[1];
        bar.mouse_down((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        bar.mouse_up((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        assert_eq!(bar.selected_index(), 1);
        assert_eq!(fires.get(), 1);

        // Clicking the selected cell again fires nothing.
        bar.mouse_down((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        bar.mouse_up((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        assert_eq!(fires.get(), 1);

        // Press inside, release outside keeps the selection.
        bar.mouse_down((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        bar.mouse_up(5000.0, 5000.0);
        assert_eq!(bar.selected_index(), 1);
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn press_selects_immediately() {
        let mut bar = switcher();
        let mut fonts = FontSystem::new();
        let (w, h) = bar.measure(&mut fonts);
        bar.place(&mut fonts, 0.0, 0.0, w, h);
        let (cx, cy, cw, ch) = bar.cells[2];
        assert_eq!(bar.press(cx + cw / 2.0, cy + ch / 2.0), Some(2));
        assert_eq!(bar.selected_index(), 2);
        assert_eq!(bar.selected_item(), Some(&BarSwitcherItem::text("Settings")));
        assert_eq!(bar.press(1999.0, 1.0), None);
    }

    #[test]
    fn set_selected_clamps_and_builder_is_silent() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut bar = BarSwitcher::from_items(vec![BarSwitcherItem::text("One")])
            .selected(0)
            .on_select(move |_| count.set(count.get() + 1));
        assert_eq!(fires.get(), 0);
        bar.set_selected(99);
        assert_eq!(bar.selected_index(), 0);
        assert_eq!(fires.get(), 0);
        assert_eq!(bar.len(), 1);
        assert!(!bar.is_empty());
    }

    #[test]
    fn disabled_never_selects() {
        let mut bar = BarSwitcher::from_items(vec![
            BarSwitcherItem::text("One"),
            BarSwitcherItem::text("Two"),
        ])
        .disabled(true)
        .on_select(|_| panic!("must not fire while disabled"));
        let mut fonts = FontSystem::new();
        let (w, h) = bar.measure(&mut fonts);
        bar.place(&mut fonts, 0.0, 0.0, w, h);
        let (cx, cy, cw, ch) = bar.cells[1];
        bar.mouse_down((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        bar.mouse_up((cx + cw / 2.0) as f64, (cy + ch / 2.0) as f64);
        assert_eq!(bar.selected_index(), 0);
        assert_eq!(bar.press(cx + cw / 2.0, cy + ch / 2.0), None);
    }

    #[test]
    fn empty_items_take_no_space_and_never_hit() {
        let mut bar = BarSwitcher::from_items(vec![
            BarSwitcherItem::text("One"),
            BarSwitcherItem::default(),
            BarSwitcherItem::text("Two"),
        ])
        .on_select(|_| {});
        let mut fonts = FontSystem::new();
        let (w, h) = bar.measure(&mut fonts);
        bar.place(&mut fonts, 0.0, 0.0, w, h);
        assert_eq!(bar.cells[1], (0.0, 0.0, 0.0, 0.0));
        // An empty cell never hits, even at its zero rect origin.
        assert_eq!(bar.press(0.0, bar.cells[0].1 + 1.0), None);
    }
}
