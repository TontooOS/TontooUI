use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Line, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{ThemeMode, desaturate};

/// Panel corner radius in logical px.
pub const BARMENU_RADIUS: f32 = 12.0;
/// Row height in logical px.
pub const BARMENU_ROW_H: f32 = 26.0;
/// Row label size in logical px.
pub const BARMENU_FONT_SIZE: f32 = 13.0;
/// Shortcut label size in logical px.
pub const BARMENU_SHORTCUT_SIZE: f32 = 12.0;
/// Icon box in logical px.
pub const BARMENU_ICON_SIZE: f32 = 16.0;
/// Horizontal padding inside the panel in logical px.
pub const BARMENU_PAD_X: f32 = 6.0;
/// Vertical padding inside the panel in logical px.
pub const BARMENU_PAD_Y: f32 = 6.0;
/// Gap between icon and label in logical px.
pub const BARMENU_TEXT_GAP: f32 = 8.0;
/// Divider row height in logical px.
pub const BARMENU_DIV_H: f32 = 9.0;
/// Gap between a panel and its submenu in logical px.
pub const BARMENU_SUB_GAP: f32 = 4.0;
/// Minimum panel width in logical px.
pub const BARMENU_MIN_W: f32 = 200.0;

/// Panel fill, dark mode (translucent, no blur).
pub const BARMENU_BG_DARK: Color = Color::from_rgba8(36, 36, 38, 225);
/// Panel fill, light mode (translucent, no blur).
pub const BARMENU_BG_LIGHT: Color = Color::from_rgba8(236, 236, 236, 240);

/// One row of a bar menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BarItem {
    /// Clickable action row, fires `on_action` with its path.
    Action(BarAction),
    /// Thin divider line.
    Divider,
}

/// Action row content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarAction {
    pub label: String,
    /// SF Symbol name for the leading 16px icon, or `None`.
    pub icon: Option<String>,
    /// Right-aligned shortcut text (e.g. `"⌥⌘⎋"`), or `None`.
    pub shortcut: Option<String>,
    /// Dimmed and non-clickable when false (default true).
    pub enabled: bool,
    /// Child rows, opened beside the parent on hover. Empty means none.
    pub children: Vec<BarItem>,
}

impl Default for BarAction {
    fn default() -> Self {
        Self {
            label: String::new(),
            icon: None,
            shortcut: None,
            enabled: true,
            children: Vec::new(),
        }
    }
}

impl BarItem {
    pub fn action(label: impl Into<String>) -> BarActionBuilder {
        BarActionBuilder(BarAction {
            label: label.into(),
            ..Default::default()
        })
    }

    pub fn divider() -> Self {
        BarItem::Divider
    }
}

/// Builder for action rows (submenu arrow appears automatically when
/// children are set).
pub struct BarActionBuilder(BarAction);

impl BarActionBuilder {
    pub fn icon(mut self, symbol: impl Into<String>) -> Self {
        self.0.icon = Some(symbol.into());
        self
    }

    pub fn shortcut(mut self, text: impl Into<String>) -> Self {
        self.0.shortcut = Some(text.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.0.enabled = false;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.0.enabled = enabled;
        self
    }

    pub fn submenu(mut self, items: Vec<BarItem>) -> Self {
        self.0.children = items;
        self
    }

    pub fn build(self) -> BarItem {
        BarItem::Action(self.0)
    }
}

impl BarAction {
    fn has_submenu(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Transparency-only dropdown panel for shell bars (menubar).
///
/// Unlike [`super::NestedMenu`] this element never uses glass: the panel
/// is a plain translucent fill, so it renders in a single pass with no
/// backdrop blur (which crashes on layer surfaces). Rows carry an SF
/// Symbol icon, a label, an optional shortcut and an optional submenu
/// (hover opens the child panel beside the parent, right preferred).
///
/// Closed by default; [`BarMenu::open_at`] shows it anchored at a point
/// (clamped into the viewport from [`BarMenu::set_viewport`]). A press
/// outside the panels closes it; an action press fires `on_action` with
/// the row path (e.g. `[0, 2]`) and closes.
pub struct BarMenu {
    items: Vec<BarItem>,
    open: bool,
    anchor: (f32, f32),
    /// Open submenu chain: row index per level.
    path: Vec<usize>,
    hovered: Vec<Option<usize>>,
    armed: Option<(usize, usize)>,
    dark: bool,
    focused: bool,
    text_color: Color,
    dim_color: Color,
    divider_color: Color,
    hover_color: Color,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    panels: Vec<(f32, f32, f32, f32)>,
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
    on_action: Option<Box<dyn FnMut(Vec<usize>)>>,
}

impl BarMenu {
    pub fn new(items: Vec<BarItem>) -> Self {
        Self {
            items,
            open: false,
            anchor: (0.0, 0.0),
            path: Vec::new(),
            hovered: vec![None],
            armed: None,
            dark: true,
            focused: true,
            text_color: Color::from_rgb8(0xf5, 0xf5, 0xf7),
            dim_color: Color::from_rgba8(245, 245, 247, 140),
            divider_color: Color::from_rgba8(255, 255, 255, 31),
            hover_color: Color::from_rgba8(255, 255, 255, 31),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            panels: Vec::new(),
            vp_x: 0.0,
            vp_y: 0.0,
            vp_w: 0.0,
            vp_h: 0.0,
            on_action: None,
        }
    }

    pub fn on_action(mut self, callback: impl FnMut(Vec<usize>) + 'static) -> Self {
        self.on_action = Some(Box::new(callback));
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Show the panel anchored at (`x`, `y`) in logical px.
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.anchor = (x, y);
        self.open = true;
        self.path.clear();
        self.hovered = vec![None];
        self.armed = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.path.clear();
        self.armed = None;
    }

    /// Clamp panels into this rect (call every frame, like `NestedMenu`).
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vp_x = x;
        self.vp_y = y;
        self.vp_w = w;
        self.vp_h = h;
    }

    /// Live theme. Fixed `Color` rows are unaffected; text follows mode.
    pub fn set_theme(&mut self, mode: ThemeMode) {
        let dark = mode == ThemeMode::Dark;
        if dark != self.dark {
            self.dark = dark;
            if dark {
                self.text_color = Color::from_rgb8(0xf5, 0xf5, 0xf7);
                self.dim_color = Color::from_rgba8(245, 245, 247, 140);
                self.divider_color = Color::from_rgba8(255, 255, 255, 31);
                self.hover_color = Color::from_rgba8(255, 255, 255, 31);
            } else {
                self.text_color = Color::from_rgb8(0x1e, 0x1e, 0x1e);
                self.dim_color = Color::from_rgba8(30, 30, 30, 140);
                self.divider_color = Color::from_rgba8(0, 0, 0, 31);
                self.hover_color = Color::from_rgba8(0, 0, 0, 20);
            }
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn eff_text(&self) -> Color {
        if self.focused {
            self.text_color
        } else {
            desaturate(self.text_color)
        }
    }

    fn eff_dim(&self) -> Color {
        if self.focused {
            self.dim_color
        } else {
            desaturate(self.dim_color)
        }
    }

    /// Rows of the panel at `level` (0 is the root).
    fn level_items(&self, level: usize) -> &[BarItem] {
        let mut items = self.items.as_slice();
        for depth in 0..level {
            let Some(&row) = self.path.get(depth) else {
                return &[];
            };
            match items.get(row) {
                Some(BarItem::Action(action)) => items = action.children.as_slice(),
                _ => return &[],
            }
        }
        items
    }

    /// Height of one row in logical px.
    fn row_height(item: &BarItem) -> f32 {
        match item {
            BarItem::Divider => BARMENU_DIV_H,
            BarItem::Action(_) => BARMENU_ROW_H,
        }
    }

    /// Panel size for `items` (content + padding, at least the minimum).
    fn panel_size(items: &[BarItem]) -> (f32, f32) {
        let mut h = BARMENU_PAD_Y * 2.0;
        for item in items {
            h += Self::row_height(item);
        }
        (BARMENU_MIN_W, h)
    }

    /// Recompute panel rects from the anchor, clamped into the viewport.
    /// Level 0 sits at the anchor; submenus open right of their row,
    /// falling back to the left.
    fn layout_panels(&mut self) {
        self.panels.clear();
        if !self.open {
            return;
        }
        let (w0, h0) = Self::panel_size(&self.items);
        let x0 = self.anchor.0.min((self.vp_x + self.vp_w - w0).max(self.vp_x));
        let y0 = self.anchor.1.min((self.vp_y + self.vp_h - h0).max(self.vp_y));
        self.panels.push((x0, y0, w0, h0));
        // Open submenu panel beside its row (right preferred, else left).
        // Single submenu depth is all shell menus need; deeper paths
        // still report correctly.
        let open: Option<(f32, Vec<BarItem>)> = self.path.first().copied().and_then(|depth| {
            let mut y_offset = BARMENU_PAD_Y;
            for (index, item) in self.items.iter().enumerate() {
                if index == depth {
                    let items = self.level_items(1).to_vec();
                    return Some((y0 + y_offset, items));
                }
                y_offset += Self::row_height(item);
            }
            None
        });
        if let Some((row_y, items)) = open {
            let (w1, h1) = Self::panel_size(&items);
            let mut x1 = x0 + w0 + BARMENU_SUB_GAP;
            if x1 + w1 > self.vp_x + self.vp_w {
                x1 = (x0 - w1 - BARMENU_SUB_GAP).max(self.vp_x);
            }
            let y1 = row_y.min((self.vp_y + self.vp_h - h1).max(self.vp_y));
            self.panels.push((x1, y1, w1, h1));
        }
    }

    fn hit_panel(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        for (level, (px, py, pw, ph)) in self.panels.iter().enumerate() {
            if x < *px || x > px + pw || y < *py || y > py + ph {
                continue;
            }
            let mut cy = py + BARMENU_PAD_Y;
            for (row, item) in self.level_items(level).iter().enumerate() {
                let rh = Self::row_height(item);
                if y >= cy && y < cy + rh {
                    return Some((level, row));
                }
                cy += rh;
            }
            return None;
        }
        None
    }

    fn fire(&mut self, path: Vec<usize>) {
        if let Some(callback) = self.on_action.as_mut() {
            callback(path);
        }
        self.close();
    }

    fn press_inside(&mut self, level: usize, row: usize) {
        let fire = matches!(
            self.level_items(level).get(row),
            Some(BarItem::Action(action)) if action.enabled && action.children.is_empty()
        );
        if fire {
            let mut path = self.path.clone();
            path.truncate(level);
            path.push(row);
            self.fire(path);
        }
    }

    /// Draw one row; returns nothing. Caller clips to the panel.
    #[allow(clippy::too_many_arguments)]
    fn draw_row(
        &self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        item: &BarItem,
        x: f32,
        y: f32,
        w: f32,
        hovered: bool,
    ) {
        match item {
            BarItem::Divider => {
                let scale = fonts.scale as f64;
                let line = Line::new(
                    vello::kurbo::Point::new((x + 8.0) as f64 * scale, (y + BARMENU_DIV_H / 2.0) as f64 * scale),
                    vello::kurbo::Point::new((x + w - 8.0) as f64 * scale, (y + BARMENU_DIV_H / 2.0) as f64 * scale),
                );
                scene.stroke(
                    &vello::kurbo::Stroke::new(scale),
                    Affine::IDENTITY,
                    &Brush::Solid(self.divider_color),
                    None,
                    &line,
                );
            }
            BarItem::Action(action) => {
                let scale = fonts.scale as f64;
                if hovered && action.enabled {
                    let bg = RoundedRect::new(
                        x as f64 * scale,
                        y as f64 * scale,
                        (x + w) as f64 * scale,
                        (y + BARMENU_ROW_H) as f64 * scale,
                        6.0 * scale,
                    );
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(self.hover_color),
                        None,
                        &bg,
                    );
                }
                let text = if action.enabled {
                    self.eff_text()
                } else {
                    self.eff_dim()
                };
                let mut cx = x + BARMENU_PAD_X;
                if let Some(symbol) = action.icon.as_deref() {
                    let target = (BARMENU_ICON_SIZE * fonts.scale * 2.0).ceil().max(1.0) as u32;
                    if let Some((image, iw, ih)) = images.get(symbol, text, target) {
                        let s = (BARMENU_ICON_SIZE / iw as f32)
                            .min(BARMENU_ICON_SIZE / ih as f32);
                        let ix = cx + (BARMENU_ICON_SIZE - iw as f32 * s) / 2.0;
                        let iy = y + (BARMENU_ROW_H - ih as f32 * s) / 2.0;
                        let transform = Affine::translate((
                            ix as f64 * scale,
                            iy as f64 * scale,
                        )) * Affine::scale(s as f64 * scale);
                        scene.draw_image(&image, transform);
                    }
                    cx += BARMENU_ICON_SIZE + BARMENU_TEXT_GAP;
                } else {
                    cx += BARMENU_ICON_SIZE + BARMENU_TEXT_GAP;
                }
                let layout = fonts.layout_text(
                    &action.label,
                    BARMENU_FONT_SIZE,
                    text,
                    None,
                );
                let (_, th) = FontSystem::layout_size(&layout);
                draw_layout(scene, &layout, cx, y + (BARMENU_ROW_H - th / fonts.scale) / 2.0, fonts.scale);
                // Shortcut or submenu chevron, right-aligned.
                let right = action
                    .shortcut
                    .as_deref()
                    .or(if action.has_submenu() { Some("›") } else { None });
                if let Some(right) = right {
                    let layout = fonts.layout_text(
                        right,
                        BARMENU_SHORTCUT_SIZE,
                        self.eff_dim(),
                        None,
                    );
                    let (tw, th) = FontSystem::layout_size(&layout);
                    draw_layout(
                        scene,
                        &layout,
                        x + w - BARMENU_PAD_X - tw / fonts.scale,
                        y + (BARMENU_ROW_H - th / fonts.scale) / 2.0,
                        fonts.scale,
                    );
                }
            }
        }
    }
}

impl View for BarMenu {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let _ = fonts;
        if !self.open {
            return (0.0, 0.0);
        }
        self.layout_panels();
        let (w, h) = self.panels.first().copied().map(|(_, _, w, h)| (w, h)).unwrap_or((0.0, 0.0));
        (w, h)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        let _ = fonts;
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        self.layout_panels();
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if !self.open {
            return;
        }
        self.layout_panels();
        let scale = fonts.scale as f64;
        let bg = if self.dark {
            BARMENU_BG_DARK
        } else {
            BARMENU_BG_LIGHT
        };
        let border = self.divider_color;
        for level in 0..self.panels.len() {
            let (px, py, pw, ph) = self.panels[level];
            let frame = RoundedRect::new(
                px as f64 * scale,
                py as f64 * scale,
                (px + pw) as f64 * scale,
                (py + ph) as f64 * scale,
                BARMENU_RADIUS as f64 * scale,
            );
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(bg), None, &frame);
            scene.stroke(
                &vello::kurbo::Stroke::new(scale),
                Affine::IDENTITY,
                &Brush::Solid(border),
                None,
                &frame,
            );
            let mut cy = py + BARMENU_PAD_Y;
            for (row, item) in self.level_items(level).iter().enumerate() {
                let rh = Self::row_height(item);
                let hovered = self.hovered.get(level).copied().flatten() == Some(row);
                self.draw_row(scene, fonts, images, item, px + BARMENU_PAD_X, cy, pw - BARMENU_PAD_X * 2.0, hovered);
                cy += rh;
            }
        }
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        if !self.open {
            return;
        }
        match self.hit_panel(x as f32, y as f32) {
            Some((level, row)) => {
                self.armed = Some((level, row));
            }
            None => self.close(),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if !self.open {
            return;
        }
        let hit = self.hit_panel(x as f32, y as f32);
        if self.armed.is_some() && hit == self.armed {
            if let Some((level, row)) = hit {
                self.press_inside(level, row);
            }
        }
        self.armed = None;
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        if !self.open {
            return;
        }
        match self.hit_panel(x, y) {
            Some((0, row)) => {
                // Hovering a submenu row opens its child panel.
                let opens_submenu = matches!(
                    self.level_items(0).get(row),
                    Some(BarItem::Action(action))
                        if action.enabled && !action.children.is_empty()
                );
                self.hovered = vec![Some(row)];
                if opens_submenu {
                    self.path = vec![row];
                    self.hovered.push(None);
                } else {
                    self.path.clear();
                }
            }
            Some((level, row)) if level > 0 => {
                if self.hovered.len() <= level {
                    self.hovered.resize(level + 1, None);
                }
                self.hovered[level] = Some(row);
            }
            _ => {
                self.hovered = vec![None];
            }
        }
        self.layout_panels();
    }

    fn set_focused(&mut self, focused: bool) {
        self.set_focused(focused);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    fn menu() -> BarMenu {
        BarMenu::new(vec![
            BarItem::action("About").build(),
            BarItem::divider(),
            BarItem::action("Quit").shortcut("⌘Q").build(),
            BarItem::action("More")
                .submenu(vec![BarItem::action("Sub").build()])
                .build(),
        ])
        .on_action(|_| {})
    }

    /// Open the menu like the shell does: viewport, anchor, measure, place.
    fn opened(menu: &mut BarMenu, fonts: &mut FontSystem) {
        menu.set_viewport(0.0, 0.0, 800.0, 600.0);
        menu.open_at(10.0, 30.0);
        let (w, h) = menu.measure(fonts);
        menu.place(fonts, 10.0, 30.0, w, h);
    }

    #[test]
    fn closed_measures_empty() {
        let mut fonts = FontSystem::new();
        let mut menu = menu();
        assert!(!menu.is_open());
        assert_eq!(menu.measure(&mut fonts), (0.0, 0.0));
    }

    #[test]
    fn open_panel_holds_all_rows() {
        let mut fonts = FontSystem::new();
        let mut menu = menu();
        opened(&mut menu, &mut fonts);
        assert!(menu.is_open());
        let (w, h) = menu.measure(&mut fonts);
        assert!(w >= BARMENU_MIN_W);
        // 3 action rows + 1 divider + padding.
        assert_eq!(h, BARMENU_PAD_Y * 2.0 + BARMENU_ROW_H * 3.0 + BARMENU_DIV_H);
    }

    #[test]
    fn outside_press_closes() {
        let mut fonts = FontSystem::new();
        let mut menu = menu();
        opened(&mut menu, &mut fonts);
        menu.mouse_down(700.0, 500.0);
        assert!(!menu.is_open());
    }

    #[test]
    fn action_press_fires_path_and_closes() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let mut fonts = FontSystem::new();
        let fired: Rc<RefCell<Vec<Vec<usize>>>> = Rc::new(RefCell::new(Vec::new()));
        let flag = fired.clone();
        let mut menu = BarMenu::new(vec![BarItem::action("Quit").build()])
            .on_action(move |path| flag.borrow_mut().push(path));
        opened(&mut menu, &mut fonts);
        // First action row starts at pad_y inside the panel.
        menu.mouse_down(20.0, (30.0 + BARMENU_PAD_Y + 4.0) as f64);
        menu.mouse_up(20.0, (30.0 + BARMENU_PAD_Y + 4.0) as f64);
        assert_eq!(*fired.borrow(), vec![vec![0]]);
        assert!(!menu.is_open());
    }

    #[test]
    fn submenu_opens_on_hover() {
        let mut fonts = FontSystem::new();
        let mut menu = menu();
        opened(&mut menu, &mut fonts);
        // Hover the submenu row (index 3: pad + 2 rows + divider).
        let y = 30.0 + BARMENU_PAD_Y + BARMENU_ROW_H * 2.0 + BARMENU_DIV_H + 4.0;
        menu.set_hover(20.0, y);
        assert_eq!(menu.path, vec![3]);
    }

    #[test]
    fn disabled_row_never_fires() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let mut fonts = FontSystem::new();
        let fired: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let flag = fired.clone();
        let mut menu = BarMenu::new(vec![BarItem::action("Quit").disabled().build()])
            .on_action(move |_| *flag.borrow_mut() = true);
        opened(&mut menu, &mut fonts);
        menu.mouse_down(20.0, (30.0 + BARMENU_PAD_Y + 4.0) as f64);
        menu.mouse_up(20.0, (30.0 + BARMENU_PAD_Y + 4.0) as f64);
        assert!(!*fired.borrow());
        assert!(menu.is_open());
    }
}
