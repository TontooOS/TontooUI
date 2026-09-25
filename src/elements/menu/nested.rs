use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, Line, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::{BUTTON_BG_DARK, BUTTON_BG_LIGHT};
use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
use super::menu::{
    MENU_BTN_PAD_X, MENU_BUTTON_H, MENU_BUTTON_RADIUS, MENU_CHEV_GAP, MENU_CHEV_W,
    MENU_FONT_SIZE, MENU_GAP, MENU_PAD, MENU_PANEL_GAP, MENU_RADIUS, MENU_ROW_H,
    MENU_ROW_SPACING, MENU_SHADOW_BLUR,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Submenu chevron column width in logical px.
pub const NESTED_CHEV_COL: f32 = 16.0;
/// Gap between row text and submenu chevron in logical px.
pub const NESTED_CHEV_GAP: f32 = 6.0;
/// Divider line vertical space in logical px.
pub const NESTED_DIV_H: f32 = 9.0;
/// Gap between submenu panel and parent panel in logical px.
pub const NESTED_SUB_GAP: f32 = 4.0;
/// Default hover fill (theme accent blue).
pub const NESTED_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// One row of a nested menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuItem {
    /// Clickable action row, fires `on_action` with its path.
    Action(String),
    /// Row opening a child panel on hover, with a `>` chevron.
    Submenu(String, Vec<MenuItem>),
    /// Dimmed non-interactive section title.
    Section(String),
    /// Thin divider line.
    Divider,
}

impl MenuItem {
    pub fn action(label: impl Into<String>) -> Self {
        MenuItem::Action(label.into())
    }

    pub fn submenu(label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        MenuItem::Submenu(label.into(), items)
    }

    pub fn section(title: impl Into<String>) -> Self {
        MenuItem::Section(title.into())
    }

    pub fn divider() -> Self {
        MenuItem::Divider
    }

    fn label(&self) -> Option<&str> {
        match self {
            MenuItem::Action(label) | MenuItem::Submenu(label, _) | MenuItem::Section(label) => {
                Some(label)
            }
            MenuItem::Divider => None,
        }
    }

    fn is_action(&self) -> bool {
        matches!(self, MenuItem::Action(_))
    }

    fn children(&self) -> Option<&[MenuItem]> {
        match self {
            MenuItem::Submenu(_, items) => Some(items),
            _ => None,
        }
    }
}

/// Nested dropdown menu: a button with fixed text plus a down
/// chevron that opens a frosted glass panel. Panels hold action
/// rows, submenu rows (hovering one opens the child panel beside
/// it), dimmed section titles and dividers, like the references.
///
/// Rows are action buttons: every action click fires `on_action`
/// with the row path (e.g. `[0, 2]` for the third row of the first
/// submenu). The closed button has no hover state, like macOS. Each
/// panel uses the `Frosted` glass finish with a heavy edge shadow
/// and clamps into the viewport passed via `set_viewport` (submenu
/// panels prefer the right side, then the left), so glass never
/// samples outside the window. Apps must call `set_viewport` every
/// frame (see `examples/nested.rs`) and return `is_open()` from
/// `App::wants_backdrop` so the shell runs the blur pass.
pub struct NestedMenu {
    label: String,
    button: String,
    items: Vec<MenuItem>,
    open: bool,
    /// Open submenu chain: row index per level.
    path: Vec<usize>,
    hovered: Vec<Option<usize>>,
    last_action: Option<Vec<usize>>,
    hover: Color,
    hover_manual: bool,
    dark: bool,
    text_color: Color,
    text_dim: Color,
    divider: Color,
    glass: GlassContainer,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    btn_x: f32,
    btn_y: f32,
    btn_w: f32,
    label_x: f32,
    label_y: f32,
    panels: Vec<(f32, f32, f32, f32)>,
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
    armed_button: bool,
    armed: Option<(usize, usize)>,
    armed_outside: bool,
    disabled: bool,
    focused: bool,
    on_action: Option<Box<dyn FnMut(Vec<usize>)>>,
}

impl NestedMenu {
    pub fn new(button: impl Into<String>, items: Vec<MenuItem>) -> Self {
        let mut glass = GlassContainer::new();
        glass.set_glass_type(GlassType::Frosted);
        Self {
            label: String::new(),
            button: button.into(),
            items,
            open: false,
            path: Vec::new(),
            hovered: Vec::new(),
            last_action: None,
            hover: NESTED_ACCENT,
            hover_manual: false,
            dark: true,
            text_color: Color::WHITE,
            text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            divider: Color::from_rgba8(255, 255, 255, 40),
            glass,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            btn_x: 0.0,
            btn_y: 0.0,
            btn_w: 0.0,
            label_x: 0.0,
            label_y: 0.0,
            panels: Vec::new(),
            vp_x: 0.0,
            vp_y: 0.0,
            vp_w: f32::MAX,
            vp_h: f32::MAX,
            armed_button: false,
            armed: None,
            armed_outside: false,
            disabled: false,
            focused: true,
            on_action: None,
        }
    }

    /// Optional leading label before the button (empty by default).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Manual hover fill: wins over the system accent until cleared.
    /// The row hover follows the system accent unless set by hand.
    pub fn hover_fill(mut self, color: Color) -> Self {
        self.hover = color;
        self.hover_manual = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Fires on every action click with the row path from the root
    /// (e.g. `[0, 2]`).
    pub fn on_action(mut self, callback: impl FnMut(Vec<usize>) + 'static) -> Self {
        self.on_action = Some(Box::new(callback));
        self
    }

    /// Live theme: hover fill plus mode grays, divider and label
    /// colors. A manually set hover wins over the system accent.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.hover_manual {
            self.hover = accent;
        }
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.text_dim = Color::from_rgb8(0x9a, 0x9a, 0x9e);
            self.divider = Color::from_rgba8(255, 255, 255, 40);
        } else {
            self.text_color = Color::BLACK;
            self.text_dim = Color::from_rgb8(0x6e, 0x6e, 0x72);
            self.divider = Color::from_rgba8(0, 0, 0, 36);
        }
    }

    /// Glass stage for the panels (frost tint per setting).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.glass.set_theme(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Fixed button text (stays put on row clicks).
    pub fn set_button(&mut self, button: impl Into<String>) {
        self.button = button.into();
    }

    pub fn button_text(&self) -> &str {
        &self.button
    }

    /// Window bounds the panels clamp into. Apps must call this
    /// every frame with the current viewport.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vp_x = x;
        self.vp_y = y;
        self.vp_w = w.max(0.0);
        self.vp_h = h.max(0.0);
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        if !self.disabled && !self.items.is_empty() {
            self.open = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.path.clear();
        self.hovered.clear();
        self.armed_button = false;
        self.armed = None;
        self.armed_outside = false;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Last clicked action path, if any (also readable without a
    /// callback).
    pub fn last_action(&self) -> Option<&[usize]> {
        self.last_action.as_deref()
    }

    /// Button rect (x, y, width, height) after layout.
    pub fn button_rect(&self) -> (f32, f32, f32, f32) {
        (self.btn_x, self.btn_y, self.btn_w, MENU_BUTTON_H)
    }

    /// Panel rects from the root level down after layout.
    pub fn panel_rects(&self) -> &[(f32, f32, f32, f32)] {
        &self.panels
    }

    fn items_at(&self, level: usize) -> &[MenuItem] {
        let mut items = self.items.as_slice();
        for row in self.path.iter().take(level) {
            match items.get(*row) {
                Some(MenuItem::Submenu(_, children)) => items = children,
                _ => return &[],
            }
        }
        items
    }

    fn button_bg(&self) -> Color {
        if self.dark {
            BUTTON_BG_DARK
        } else {
            BUTTON_BG_LIGHT
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

    fn label_size(&self, fonts: &mut FontSystem) -> (f32, f32) {
        if self.label.is_empty() {
            return (0.0, 0.0);
        }
        let layout = fonts.layout_text(&self.label, MENU_FONT_SIZE, Color::WHITE, None);
        let (tw, th) = FontSystem::layout_size(&layout);
        (tw / fonts.scale, th / fonts.scale)
    }

    fn text_w(&self, fonts: &mut FontSystem, text: &str) -> f32 {
        let layout = fonts.layout_text(text, MENU_FONT_SIZE, Color::WHITE, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    fn button_w(&self, fonts: &mut FontSystem) -> f32 {
        let mut widest = self.text_w(fonts, &self.button);
        for item in &self.items {
            if let Some(label) = item.label() {
                widest = widest.max(self.text_w(fonts, label));
            }
        }
        widest + MENU_BTN_PAD_X * 2.0 + MENU_CHEV_GAP + MENU_CHEV_W
    }

    fn row_stride(&self) -> f32 {
        MENU_ROW_H + MENU_ROW_SPACING
    }

    /// Height of one item: rows and section titles share the row
    /// height, dividers take their own slot.
    fn item_h(item: &MenuItem) -> f32 {
        match item {
            MenuItem::Divider => NESTED_DIV_H,
            _ => MENU_ROW_H,
        }
    }

    fn panel_content_w(&self, fonts: &mut FontSystem, items: &[MenuItem]) -> f32 {
        let mut widest: f32 = 0.0;
        let mut submenu = false;
        for item in items {
            if let Some(label) = item.label() {
                widest = widest.max(self.text_w(fonts, label));
            }
            if item.children().is_some() {
                submenu = true;
            }
        }
        widest + MENU_PAD * 2.0 + if submenu {
            NESTED_CHEV_GAP + NESTED_CHEV_COL
        } else {
            0.0
        }
    }

    fn panel_content_h(items: &[MenuItem]) -> f32 {
        let mut h = MENU_PAD * 2.0;
        let mut first = true;
        for item in items {
            if !first {
                h += MENU_ROW_SPACING;
            }
            h += Self::item_h(item);
            first = false;
        }
        h
    }

    fn clamp_panel(&self, mut x: f32, mut y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
        let w = w.min(self.vp_w).max(0.0);
        let h = h.min(self.vp_h).max(0.0);
        if x + w > self.vp_x + self.vp_w {
            x = self.vp_x + self.vp_w - w;
        }
        if y + h > self.vp_y + self.vp_h {
            y = self.vp_y + self.vp_h - h;
        }
        if x < self.vp_x {
            x = self.vp_x;
        }
        if y < self.vp_y {
            y = self.vp_y;
        }
        (x, y, w, h)
    }

    /// Panel rects for the root level plus every open submenu level.
    /// The root panel lays out even while closed so hit testing works
    /// on the very next event (tests re-place after state changes,
    /// apps redraw every frame anyway).
    fn layout_panels(&mut self, fonts: &mut FontSystem) {
        self.panels.clear();
        if self.items.is_empty() {
            return;
        }
        // Root panel under (or above) the button.
        let w = self
            .panel_content_w(fonts, &self.items)
            .max(self.btn_w)
            .min(self.vp_w)
            .max(0.0);
        let h = Self::panel_content_h(&self.items);
        let below = self.btn_y + MENU_BUTTON_H + MENU_PANEL_GAP;
        let (x, y, w, h) = if below + h <= self.vp_y + self.vp_h {
            self.clamp_panel(self.btn_x, below, w, h)
        } else {
            let (x, y, w, h) =
                self.clamp_panel(self.btn_x, self.btn_y - MENU_PANEL_GAP - h, w, h);
            (x, y, w, h)
        };
        self.panels.push((x, y, w, h));
        // Submenu levels beside their parent row.
        let mut level = 0;
        while level < self.path.len() {
            let row = self.path[level];
            let Some(children) = self.items_at(level).get(row).and_then(|i| i.children()) else {
                break;
            };
            let (px, py, pw, _) = self.panels[level];
            let ry = self.row_top(level, row);
            let w = self
                .panel_content_w(fonts, children)
                .min(self.vp_w)
                .max(0.0);
            let h = Self::panel_content_h(children);
            // Prefer right, fall back left, then clamp.
            let mut x = px + pw + NESTED_SUB_GAP;
            let mut y = ry - MENU_PAD;
            if x + w > self.vp_x + self.vp_w {
                x = px - NESTED_SUB_GAP - w;
            }
            let (x, y, w, h) = self.clamp_panel(x, y, w, h);
            self.panels.push((x, y, w, h));
            level += 1;
            if self.panels.len() > 8 {
                break;
            }
        }
        while self.hovered.len() < self.panels.len() {
            self.hovered.push(None);
        }
        self.hovered.truncate(self.panels.len());
    }

    /// Top edge of a row (or divider slot) inside a level panel.
    fn row_top(&self, level: usize, row: usize) -> f32 {
        let (x, y, _, _) = self.panels[level];
        let mut top = y + MENU_PAD;
        for item in self.items_at(level).iter().take(row) {
            top += Self::item_h(item) + MENU_ROW_SPACING;
        }
        top
    }

    fn level_row_at(&self, level: usize, x: f32, y: f32) -> Option<usize> {
        let (px, py, pw, ph) = *self.panels.get(level)?;
        if x < px || x > px + pw || y < py || y > py + ph {
            return None;
        }
        let mut top = py + MENU_PAD;
        for (i, item) in self.items_at(level).iter().enumerate() {
            let h = Self::item_h(item);
            if y >= top && y <= top + h {
                return match item {
                    MenuItem::Action(_) | MenuItem::Submenu(_, _) => Some(i),
                    _ => None,
                };
            }
            top += h + MENU_ROW_SPACING;
        }
        None
    }

    /// Deepest level whose panel contains the point, if any.
    fn level_at(&self, x: f32, y: f32) -> Option<usize> {
        for level in (0..self.panels.len()).rev() {
            let (px, py, pw, ph) = self.panels[level];
            if x >= px && x <= px + pw && y >= py && y <= py + ph {
                return Some(level);
            }
        }
        None
    }

    fn button_hit(&self, x: f32, y: f32) -> bool {
        x >= self.btn_x
            && x <= self.btn_x + self.btn_w
            && y >= self.btn_y
            && y <= self.btn_y + MENU_BUTTON_H
    }

    fn action_path(&self, level: usize, row: usize) -> Vec<usize> {
        let mut path: Vec<usize> = self.path.iter().take(level).cloned().collect();
        path.push(row);
        path
    }

    fn item_at_path(&self, path: &[usize]) -> Option<&MenuItem> {
        let mut items = self.items.as_slice();
        let mut found = None;
        for (depth, row) in path.iter().enumerate() {
            found = items.get(*row);
            let last = depth + 1 == path.len();
            match found {
                Some(MenuItem::Submenu(_, children)) if !last => items = children,
                Some(_) if last => {}
                _ => return None,
            }
        }
        found
    }

    fn notify(&mut self, path: Vec<usize>) {
        self.last_action = Some(path.clone());
        if let Some(callback) = self.on_action.as_mut() {
            callback(path);
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if !self.open {
            if self.button_hit(x, y) {
                self.armed_button = true;
            }
            return;
        }
        self.armed_button = false;
        self.armed = None;
        self.armed_outside = false;
        if let Some(level) = self.level_at(x, y) {
            if let Some(row) = self.level_row_at(level, x, y) {
                self.armed = Some((level, row));
                return;
            }
        }
        // Press on panels (gaps, titles) or outside arms a close.
        self.armed_outside = true;
    }

    /// Hover opens submenu rows and highlights action rows; the
    /// closed button never highlights, like macOS.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if !self.open || self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        let Some(level) = self.level_at(x, y) else {
            for hover in self.hovered.iter_mut() {
                *hover = None;
            }
            return;
        };
        // Hovering a level closes deeper levels past it.
        self.path.truncate(level);
        self.hovered.truncate(level + 1);
        while self.hovered.len() <= level {
            self.hovered.push(None);
        }
        let row = self.level_row_at(level, x, y);
        self.hovered[level] = row;
        // Hovering a submenu row opens its child panel.
        if let Some(row) = row {
            let submenu = matches!(
                self.items_at(level).get(row),
                Some(MenuItem::Submenu(_, _))
            );
            if submenu {
                if self.path.len() == level {
                    self.path.push(row);
                } else if self.path[level] != row {
                    self.path[level] = row;
                }
                self.path.truncate(level + 1);
            } else {
                self.path.truncate(level);
            }
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn finish_up(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if !self.open {
            // Click (press + release) on the button opens the menu.
            if self.armed_button && self.button_hit(x, y) {
                self.open();
            }
            self.armed_button = false;
            return;
        }
        if let Some((level, row)) = self.armed.take() {
            // Press + release on the same row: actions fire and
            // close, submenu rows just stay open.
            if self.level_at(x, y) == Some(level) && self.level_row_at(level, x, y) == Some(row) {
                let path = self.action_path(level, row);
                if matches!(self.item_at_path(&path), Some(MenuItem::Action(_))) {
                    self.notify(path);
                    self.close();
                    return;
                }
            }
        }
        self.armed_button = false;
        if self.armed_outside {
            self.armed_outside = false;
            // Release on the button toggles instead of plain closing.
            if self.button_hit(x, y) {
                self.close();
                return;
            }
            self.close();
        }
    }

    /// Scrolling is not needed: panels cap at the window size.
    pub fn mouse_wheel(&mut self, _dx: f64, _dy: f64) {}

    fn draw_sub_chevron(&self, scene: &mut Scene, x: f32, cy: f32, scale: f32, color: Color) {
        // Right chevron marking submenu rows.
        let px = |v: f32| v as f64 * scale as f64;
        let (w, h) = (5.0, 8.0);
        let mut path = BezPath::new();
        path.move_to((px(x), px(cy - h / 2.0)));
        path.line_to((px(x + w), px(cy)));
        path.line_to((px(x), px(cy + h / 2.0)));
        let mut stroke = Stroke::new(2.0 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }

    fn draw_down_chevron(&self, scene: &mut Scene, x: f32, cy: f32, scale: f32, color: Color) {
        let px = |v: f32| v as f64 * scale as f64;
        let w = 7.0;
        let h = 4.5;
        let cx = x + MENU_CHEV_W / 2.0;
        let mut path = BezPath::new();
        path.move_to((px(cx - w / 2.0), px(cy - h / 2.0)));
        path.line_to((px(cx), px(cy + h / 2.0)));
        path.line_to((px(cx + w / 2.0), px(cy - h / 2.0)));
        let mut stroke = Stroke::new(1.5 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }
}

impl View for NestedMenu {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (label_w, label_h) = self.label_size(fonts);
        let label_part = if label_w > 0.0 {
            label_w + MENU_GAP
        } else {
            0.0
        };
        (
            label_part + self.button_w(fonts),
            MENU_BUTTON_H.max(label_h),
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        let (label_w, label_h) = self.label_size(fonts);
        self.label_x = x;
        self.label_y = y + (h - label_h) / 2.0;
        self.btn_x = if label_w > 0.0 { x + label_w + MENU_GAP } else { x };
        self.btn_y = y + (h - MENU_BUTTON_H) / 2.0;
        self.btn_w = self.button_w(fonts).min((w - (self.btn_x - x)).max(0.0));
        self.layout_panels(fonts);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        if images.is_capture_pass() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        if self.items.is_empty() {
            return;
        }
        self.layout_panels(fonts);

        if !self.label.is_empty() {
            let layout = fonts.layout_text_weighted(
                &self.label,
                MENU_FONT_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            draw_layout(scene, &layout, self.label_x, self.label_y, fonts.scale);
        }

        // Button with fixed text (no hover state).
        let button = RoundedRect::new(
            px(self.btn_x),
            px(self.btn_y),
            px(self.btn_x + self.btn_w),
            px(self.btn_y + MENU_BUTTON_H),
            px(MENU_BUTTON_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.button_bg())),
            None,
            &button,
        );
        if self.armed_button && !self.disabled {
            let press = if self.dark {
                Color::from_rgba8(255, 255, 255, 24)
            } else {
                Color::from_rgba8(0, 0, 0, 24)
            };
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(press)),
                None,
                &button,
            );
        }
        let layout = fonts.layout_text_weighted(
            &self.button,
            MENU_FONT_SIZE,
            self.eff(self.text_color),
            400.0,
            None,
        );
        let (_, th) = FontSystem::layout_size(&layout);
        draw_layout(
            scene,
            &layout,
            self.btn_x + MENU_BTN_PAD_X,
            self.btn_y + (MENU_BUTTON_H - th / fonts.scale) / 2.0,
            fonts.scale,
        );
        self.draw_down_chevron(
            scene,
            self.btn_x + self.btn_w - MENU_BTN_PAD_X - MENU_CHEV_W,
            self.btn_y + MENU_BUTTON_H / 2.0,
            fonts.scale,
            self.eff(self.text_dim),
        );

        if !self.open {
            return;
        }

        for level in 0..self.panels.len() {
            let (px0, py0, pw, ph) = self.panels[level];
            let rect = vello::kurbo::Rect::new(
                px(px0),
                px(py0),
                px(px0 + pw),
                px(py0 + ph),
            );
            scene.draw_blurred_rounded_rect(
                Affine::IDENTITY,
                rect,
                Color::from_rgba8(0, 0, 0, 70),
                px(MENU_RADIUS),
                MENU_SHADOW_BLUR as f64 * scale,
            );
            self.glass.set_bounds(px0, py0, pw, ph);
            self.glass.set_radius(MENU_RADIUS);
            self.glass.draw(scene, fonts, images);

            let clip = vello::kurbo::Rect::new(
                px(px0),
                px(py0 + MENU_PAD),
                px(px0 + pw),
                px(py0 + ph - MENU_PAD),
            );
            scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
            let mut top = py0 + MENU_PAD;
            for (i, item) in self.items_at(level).iter().enumerate() {
                match item {
                    MenuItem::Divider => {
                        let line = Line::new(
                            (px(px0 + MENU_PAD), px(top + NESTED_DIV_H / 2.0)),
                            (px(px0 + pw - MENU_PAD), px(top + NESTED_DIV_H / 2.0)),
                        );
                        scene.stroke(
                            &Stroke::new(1.0 * scale),
                            Affine::IDENTITY,
                            &Brush::Solid(self.eff(self.divider)),
                            None,
                            &line,
                        );
                        top += NESTED_DIV_H + MENU_ROW_SPACING;
                    }
                    MenuItem::Section(title) => {
                        let layout = fonts.layout_text_weighted(
                            title,
                            MENU_FONT_SIZE,
                            self.eff(self.text_dim),
                            400.0,
                            None,
                        );
                        let (_, th) = FontSystem::layout_size(&layout);
                        draw_layout(
                            scene,
                            &layout,
                            px0 + MENU_PAD,
                            top + (MENU_ROW_H - th / fonts.scale) / 2.0,
                            fonts.scale,
                        );
                        top += MENU_ROW_H + MENU_ROW_SPACING;
                    }
                    MenuItem::Action(label) | MenuItem::Submenu(label, _) => {
                        let is_sub = matches!(item, MenuItem::Submenu(_, _));
                        let is_hovered =
                            self.hovered.get(level).copied().flatten() == Some(i)
                                && !self.disabled;
                        if is_hovered {
                            let hl = RoundedRect::new(
                                px(px0 + MENU_PAD / 2.0),
                                px(top),
                                px(px0 + pw - MENU_PAD / 2.0),
                                px(top + MENU_ROW_H),
                                px(MENU_BUTTON_RADIUS),
                            );
                            scene.fill(
                                Fill::NonZero,
                                Affine::IDENTITY,
                                &Brush::Solid(self.eff(self.hover)),
                                None,
                                &hl,
                            );
                        }
                        let color = if is_hovered {
                            Color::WHITE
                        } else {
                            self.text_color
                        };
                        let layout = fonts.layout_text_weighted(
                            label,
                            MENU_FONT_SIZE,
                            self.eff(color),
                            400.0,
                            None,
                        );
                        let (_, th) = FontSystem::layout_size(&layout);
                        draw_layout(
                            scene,
                            &layout,
                            px0 + MENU_PAD,
                            top + (MENU_ROW_H - th / fonts.scale) / 2.0,
                            fonts.scale,
                        );
                        if is_sub {
                            self.draw_sub_chevron(
                                scene,
                                px0 + pw - MENU_PAD - NESTED_CHEV_COL,
                                top + MENU_ROW_H / 2.0,
                                fonts.scale,
                                self.eff(color),
                            );
                        }
                        top += MENU_ROW_H + MENU_ROW_SPACING;
                    }
                }
            }
            scene.pop_layer();
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
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

    fn share() -> NestedMenu {
        NestedMenu::new(
            "Share",
            vec![
                MenuItem::section("Choose destination"),
                MenuItem::submenu(
                    "Messages",
                    vec![
                        MenuItem::action("John"),
                        MenuItem::action("Jane"),
                        MenuItem::action("Bob"),
                    ],
                ),
                MenuItem::submenu("Social", vec![MenuItem::action("Post")]),
                MenuItem::divider(),
                MenuItem::action("More..."),
            ],
        )
    }

    fn files() -> NestedMenu {
        NestedMenu::new(
            "File",
            vec![
                MenuItem::section("File Operations"),
                MenuItem::action("New File"),
                MenuItem::action("Open File"),
                MenuItem::action("Save"),
                MenuItem::divider(),
                MenuItem::section("Edit Operations"),
                MenuItem::action("Undo"),
                MenuItem::action("Redo"),
            ],
        )
    }

    fn placed(menu: NestedMenu) -> (NestedMenu, FontSystem) {
        let mut m = menu;
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 0.0, 0.0, w, h);
        (m, fonts)
    }

    /// Re-run layout after a state change, like the next app redraw
    /// (keeps the placed position).
    fn relayout(m: &mut NestedMenu, fonts: &mut FontSystem) {
        let (x, y) = (m.x, m.y);
        let (w, h) = m.measure(fonts);
        m.place(fonts, x, y, w, h);
    }

    fn button_center(m: &NestedMenu) -> (f64, f64) {
        let (x, y, w, _) = m.button_rect();
        ((x + w / 2.0) as f64, (y + MENU_BUTTON_H / 2.0) as f64)
    }

    fn row_point(m: &NestedMenu, level: usize, row: usize) -> (f64, f64) {
        let (x, y, w, _) = m.panels[level];
        ((x + w / 2.0) as f64, (y + MENU_PAD + row as f32 * (MENU_ROW_H + MENU_ROW_SPACING) + MENU_ROW_H / 2.0) as f64)
    }

    #[test]
    fn button_text_stays_fixed() {
        let (mut m, _) = placed(share());
        assert_eq!(m.button_text(), "Share");
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        assert!(m.is_open());
        assert_eq!(m.panel_rects().len(), 1);
    }

    #[test]
    fn hover_submenu_opens_child() {
        let (mut m, mut fonts) = placed(share());
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        // Hovering "Messages" (row 1) opens its child panel.
        let (rx, ry) = row_point(&m, 0, 1);
        m.mouse_move(rx, ry);
        relayout(&mut m, &mut fonts);
        assert_eq!(m.panel_rects().len(), 2);
        assert_eq!(m.path, vec![1]);
        // Child rows are the contacts.
        assert_eq!(m.items_at(1).len(), 3);
    }

    #[test]
    fn submenu_action_fires_path() {
        let fires = Rc::new(Cell::new(Vec::new()));
        let count = fires.clone();
        let mut m = share().on_action(move |path| count.set(path));
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 0.0, 0.0, w, h);
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        let (rx, ry) = row_point(&m, 0, 1);
        m.mouse_move(rx, ry);
        relayout(&mut m, &mut fonts);
        let (jx, jy) = row_point(&m, 1, 2);
        m.mouse_down(jx, jy);
        m.mouse_up(jx, jy);
        assert_eq!(fires.take(), vec![1, 2]);
        assert!(!m.is_open());
    }

    #[test]
    fn sections_and_dividers_are_dead() {
        let (mut m, _) = placed(files());
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        // Section title row 0: hover finds nothing, click closes.
        let (sx, sy) = row_point(&m, 0, 0);
        m.mouse_move(sx, sy);
        m.mouse_down(sx, sy);
        m.mouse_up(sx, sy);
        assert!(!m.is_open());
        assert_eq!(m.last_action(), None);
    }

    #[test]
    fn divider_geometry_counts() {
        // 8 items with own heights plus spacing and padding.
        let h = NestedMenu::panel_content_h(&files().items);
        let expected = MENU_PAD * 2.0 + 7.0 * MENU_ROW_H + NESTED_DIV_H + 7.0 * MENU_ROW_SPACING;
        assert_eq!(h, expected);
    }

    #[test]
    fn outside_click_closes() {
        let (mut m, _) = placed(share());
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        assert!(m.is_open());
        m.mouse_down(700.0, 500.0);
        m.mouse_up(700.0, 500.0);
        assert!(!m.is_open());
    }

    #[test]
    fn submenu_prefers_right_then_left() {
        let mut m = share();
        let mut fonts = FontSystem::new();
        // Button glued to the right edge: the root panel clamps, so
        // the child never fits right and must flip left.
        m.set_viewport(0.0, 0.0, 300.0, 600.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 290.0, 0.0, w, h);
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        let (rx, ry) = row_point(&m, 0, 1);
        m.mouse_move(rx, ry);
        relayout(&mut m, &mut fonts);
        assert_eq!(m.panel_rects().len(), 2);
        let (root_x, _, root_w, _) = m.panel_rects()[0];
        let (sub_x, _, sub_w, _) = m.panel_rects()[1];
        assert!(sub_x + sub_w <= 300.0);
        assert!(sub_x + sub_w <= root_x + 1.0);
    }

    #[test]
    fn empty_never_opens() {
        let mut m = NestedMenu::new("Empty", Vec::new());
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 0.0, 0.0, w.max(1.0), h.max(1.0));
        m.mouse_down(2.0, 2.0);
        m.mouse_up(2.0, 2.0);
        assert!(!m.is_open());
    }
}
