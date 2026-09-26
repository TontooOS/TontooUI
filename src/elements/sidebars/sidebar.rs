use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

use vello::Scene;
use vello::kurbo::{Affine, Circle, Line, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::groupbox::{GROUP_BG_DARK, GROUP_BG_LIGHT};
use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::super::titlebar::{
    TRAFFIC_CLOSE, TRAFFIC_GAP, TRAFFIC_INACTIVE, TRAFFIC_LEFT, TRAFFIC_MAXIMIZE,
    TRAFFIC_MINIMIZE, TRAFFIC_SIZE, TITLEBAR_DIVIDER_DARK, TITLEBAR_DIVIDER_LIGHT,
    TrafficAction,
};
use super::super::toolbar::{
    BasicToolbar, ToolbarItem, TOOLBAR_GAP, TOOLBAR_HEIGHT, TOOLBAR_HIT, TOOLBAR_PAD_X,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::renderer::window::Key;
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Toolbar action behind a pill icon: the single toggle pill or
/// the raw left-pill cell index (roles resolved at drain time, so
/// later visibility flips cannot misroute a click).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingAction {
    Toggle,
    Cell(usize),
}

/// Sidebar width in logical px.
pub const SIDEBAR_W: f32 = 240.0;
/// Minimum sidebar width in logical px.
pub const SIDEBAR_MIN_W: f32 = 220.0;
/// Item row height in logical px.
pub const SIDEBAR_ROW_H: f32 = 46.0;
/// Sidebar inset in logical px.
pub const SIDEBAR_PAD: f32 = 16.0;
/// Row icon box in logical px.
pub const SIDEBAR_ICON_SIZE: f32 = 22.0;
/// Gap between icon and label in logical px.
pub const SIDEBAR_ICON_GAP: f32 = 12.0;
/// Item label size in logical px.
pub const SIDEBAR_LABEL_SIZE: f32 = 17.0;
/// Content toolbar title size in logical px.
pub const SIDEBAR_TITLE_SIZE: f32 = 19.0;
/// Traffic lights top edge in logical px.
pub const SIDEBAR_TRAFFIC_TOP: f32 = 22.0;
/// Toolbar pills row top edge in logical px (expanded sidebar):
/// vertically centered on the traffic lights row.
pub const SIDEBAR_BAR_TOP: f32 = SIDEBAR_TRAFFIC_TOP + TRAFFIC_SIZE / 2.0 - TOOLBAR_HEIGHT / 2.0;
/// Items top edge in logical px.
pub const SIDEBAR_ITEMS_TOP: f32 = 100.0;
/// Content toolbar height in logical px.
pub const SIDEBAR_TOOLBAR_H: f32 = 64.0;
/// Selected row fill in dark mode.
const SIDEBAR_SEL_DARK: Color = Color::from_rgba8(255, 255, 255, 28);
/// Selected row fill in light mode.
const SIDEBAR_SEL_LIGHT: Color = Color::from_rgba8(0, 0, 0, 18);

/// One sidebar entry: tinted SF Symbol plus label. The tint
/// defaults to the theme accent.
#[derive(Clone, Debug)]
pub struct SidebarItem {
    label: String,
    icon: String,
    tint: Option<Color>,
}

impl SidebarItem {
    pub fn new(label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: icon.into(),
            tint: None,
        }
    }

    /// Manual icon tint: wins over the theme accent until cleared.
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }
}

/// App sidebar: full-height navigation column with embedded traffic
/// lights (replacing the titlebar decoration), icon items and a
/// content toolbar with back, title, dev buttons and a single far-
/// right toggle. Selecting an item switches the right-side page,
/// which the sidebar owns (one page per item; missing pages stay
/// empty). Collapsing hides the column and lets the content fill
/// the width; the toggle button reopens it.
pub struct Sidebar {
    items: Vec<SidebarItem>,
    pages: Vec<Box<dyn View>>,
    selected: usize,
    on_select: Option<Box<dyn FnMut(usize)>>,
    title: Option<String>,
    show_back: bool,
    on_back: Option<Box<dyn FnMut()>>,
    collapsed: bool,
    on_collapse: Option<Box<dyn FnMut(bool)>>,
    show_toggle: bool,
    collapsible: bool,
    left_bar: BasicToolbar,
    right_bar: BasicToolbar,
    dev_icons: Vec<String>,
    dev_actions: Vec<Box<dyn FnMut()>>,
    pending: Rc<Cell<Option<PendingAction>>>,
    width_setting: f32,
    accent: Color,
    dark: bool,
    glass_mode: ThemeMode,
    glass_amount: GlassAmount,
    focused: bool,
    traffic_hover: Option<usize>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Sidebar {
    pub fn new(items: Vec<SidebarItem>) -> Self {
        let pending: Rc<Cell<Option<PendingAction>>> = Rc::new(Cell::new(None));
        let mut bar = Self {
            items,
            pages: Vec::new(),
            selected: 0,
            on_select: None,
            title: None,
            show_back: true,
            show_toggle: true,
            collapsible: true,
            on_back: None,
            collapsed: false,
            on_collapse: None,
            left_bar: BasicToolbar::new(),
            right_bar: BasicToolbar::new(),
            dev_icons: Vec::new(),
            dev_actions: Vec::new(),
            pending,
            width_setting: SIDEBAR_W,
            accent: Color::from_rgb8(0x00, 0x7a, 0xff),
            dark: true,
            glass_mode: ThemeMode::Dark,
            glass_amount: GlassAmount::Glass,
            focused: true,
            traffic_hover: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        bar.sync_bars();
        bar
    }

    /// Rebuild the pills: left holds back (optional) plus the dev
    /// icons, right holds the toggle alone (optional, far right).
    /// Actions report through shared pending state (applied on the
    /// next mouse-up or draw).
    fn sync_bars(&mut self) {
        let mut left = Vec::new();
        if self.show_back {
            left.push(ToolbarItem::icon("chevron.left"));
        }
        for name in &self.dev_icons {
            left.push(ToolbarItem::icon(name.clone()));
        }
        let pending = self.pending.clone();
        self.left_bar = BasicToolbar::from_items(left).on_action(move |index| {
            pending.set(Some(PendingAction::Cell(index)));
        });
        self.left_bar.set_theme(self.glass_mode, self.glass_amount);
        self.left_bar.set_focused(self.focused);
        let pending = self.pending.clone();
        let mut right = BasicToolbar::from_items(if self.show_toggle {
            vec![ToolbarItem::icon("sidebar.left")]
        } else {
            Vec::new()
        })
        .on_action(move |_| {
            pending.set(Some(PendingAction::Toggle));
        });
        right.set_disabled(!self.collapsible);
        right.set_theme(self.glass_mode, self.glass_amount);
        right.set_focused(self.focused);
        self.right_bar = right;
    }

    /// Apply one pending pill action. Left-pill cells resolve to
    /// back, then dev callbacks in pill order.
    fn drain_pending(&mut self) {
        match self.pending.take() {
            Some(PendingAction::Toggle) => {
                self.toggle_sidebar()
            }
            Some(PendingAction::Cell(index)) => {
                let mut k = index;
                if self.show_back {
                    if k == 0 {
                        if let Some(callback) = self.on_back.as_mut() {
                            callback();
                        }
                        return;
                    }
                    k -= 1;
                }
                if let Some(action) = self.dev_actions.get_mut(k) {
                    action();
                }
            }
            None => {}
        }
    }

    /// Content page for an item index (order matches `items`;
    /// missing pages stay empty).
    pub fn page(mut self, page: impl View + 'static) -> Self {
        self.pages.push(Box::new(page));
        self
    }

    /// Sidebar width in logical px (clamped to the minimum, ignored
    /// while collapsed).
    pub fn width(mut self, px: f32) -> Self {
        self.width_setting = px.max(SIDEBAR_MIN_W);
        self
    }

    /// Back button in the left pill (default `true`).
    pub fn back_button(mut self, show: bool) -> Self {
        self.show_back = show;
        self.sync_bars();
        self
    }

    pub fn set_back_button(&mut self, show: bool) {
        self.show_back = show;
        self.sync_bars();
    }

    /// Single toggle button, far right (default `true`).
    pub fn toggle_button(mut self, show: bool) -> Self {
        self.show_toggle = show;
        self.sync_bars();
        self
    }

    pub fn set_toggle_button(&mut self, show: bool) {
        self.show_toggle = show;
        self.sync_bars();
    }

    /// Collapsible (default `true`). Disabled, the single toggle
    /// stays visible but gray and ignores clicks.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self.sync_bars();
        self
    }

    pub fn set_collapsible(&mut self, collapsible: bool) {
        self.collapsible = collapsible;
        self.sync_bars();
    }

    /// Fires with the newly selected item index on change.
    pub fn on_select(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Fires when the back button is pressed (history is the app's
    /// job).
    pub fn on_back(mut self, callback: impl FnMut() + 'static) -> Self {
        self.on_back = Some(Box::new(callback));
        self
    }

    /// Fires with the collapsed state on every user collapse flip.
    pub fn on_collapse(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_collapse = Some(Box::new(callback));
        self
    }

    /// Extra icon in the left pill, after back, with its own
    /// press callback.
    pub fn toolbar_button(mut self, icon: impl Into<String>, on_press: impl FnMut() + 'static) -> Self {
        self.dev_icons.push(icon.into());
        self.dev_actions.push(Box::new(on_press));
        self.sync_bars();
        self
    }

    /// Add a toolbar icon after construction (for callbacks that
    /// need a shared handle to the sidebar itself).
    pub fn add_toolbar_button(&mut self, icon: impl Into<String>, on_press: impl FnMut() + 'static) {
        self.dev_icons.push(icon.into());
        self.dev_actions.push(Box::new(on_press));
        self.sync_bars();
    }

    /// Fixed toolbar title (follows the selected item label when
    /// never set).
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    pub fn clear_title(&mut self) {
        self.title = None;
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Select programmatically (fires `on_select` on change).
    /// Returns false for out-of-range indices.
    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.items.len() {
            return false;
        }
        if index != self.selected {
            self.selected = index;
            if let Some(callback) = self.on_select.as_mut() {
                callback(index);
            }
        }
        true
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Collapse programmatically (no callback; use
    /// `toggle_sidebar` for the user path).
    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
        self.sync_bars();
    }

    /// User collapse flip: toggles and fires `on_collapse`.
    /// Ignored while `collapsible(false)` (the collapse pill is
    /// gone; the toggle cell is the only UI path).
    pub fn toggle_sidebar(&mut self) {
        if !self.collapsible {
            return;
        }
        self.collapsed = !self.collapsed;
        self.sync_bars();
        if let Some(callback) = self.on_collapse.as_mut() {
            callback(self.collapsed);
        }
    }

    /// Mutable page access for downcasting (concrete event
    /// forwarding beyond the `View` protocol).
    pub fn page_mut(&mut self, index: usize) -> Option<&mut (dyn View + '_)> {
        let page: &mut Box<dyn View> = self.pages.get_mut(index)?;
        Some(page.as_mut())
    }

    pub fn active_page_mut(&mut self) -> Option<&mut dyn View> {
        self.page_mut(self.selected)
    }

    /// Live theme forwarded to item icons and labels (toolbar
    /// pills keep their own icon grays per mode).
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        self.dark = dark;
    }

    /// Glass stage for the toolbar pills (Lens finish).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.glass_mode = mode;
        self.glass_amount = amount;
        self.left_bar.set_theme(mode, amount);
        self.right_bar.set_theme(mode, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.left_bar.set_focused(focused);
        self.right_bar.set_focused(focused);
    }

    /// True while the Lens toolbar pills are on screen: return it
    /// from `App::wants_backdrop` for the blur pass (same rule as
    /// the toolbar demo).
    pub fn wants_backdrop(&self) -> bool {
        true
    }

    fn bar_w(&self) -> f32 {
        if self.collapsed {
            0.0
        } else {
            self.width_setting
        }
    }

    fn sidebar_bg(&self) -> Color {
        self.eff(if self.dark {
            GROUP_BG_DARK
        } else {
            GROUP_BG_LIGHT
        })
    }

    fn divider_color(&self) -> Color {
        self.eff(if self.dark {
            TITLEBAR_DIVIDER_DARK
        } else {
            TITLEBAR_DIVIDER_LIGHT
        })
    }

    fn text_color(&self) -> Color {
        if self.dark {
            Color::from_rgb8(0xd8, 0xd9, 0xd9)
        } else {
            Color::from_rgb8(0x27, 0x27, 0x27)
        }
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }

    fn content_x(&self) -> f32 {
        self.x + self.bar_w()
    }

    fn traffic_cy(&self) -> f32 {
        self.y + SIDEBAR_TRAFFIC_TOP + TRAFFIC_SIZE / 2.0
    }

    fn traffic_center(&self, index: usize) -> (f32, f32) {
        let cx = self.x + TRAFFIC_LEFT + TRAFFIC_SIZE / 2.0
            + index as f32 * (TRAFFIC_SIZE + TRAFFIC_GAP);
        (cx, self.traffic_cy())
    }

    fn traffic_index(&self, x: f32, y: f32) -> Option<usize> {
        // Traffic lives in the sidebar when expanded, top-left of
        // the content when collapsed: same geometry either way.
        for index in 0..3 {
            let (cx, cy) = self.traffic_center(index);
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= (TRAFFIC_SIZE / 2.0 + 3.0).powi(2) {
                return Some(index);
            }
        }
        None
    }

    fn traffic_at(&self, x: f32, y: f32) -> Option<TrafficAction> {
        match self.traffic_index(x, y) {
            Some(0) => Some(TrafficAction::Close),
            Some(1) => Some(TrafficAction::Minimize),
            Some(2) => Some(TrafficAction::Maximize),
            _ => None,
        }
    }

    /// Click handling for the embedded traffic lights. Returns the
    /// action when a light was hit, otherwise `None`. The app maps
    /// it to a `WindowCommand` like with a titlebar and must not
    /// forward the press to `mouse_down` then.
    pub fn press(&mut self, x: f64, y: f64) -> Option<TrafficAction> {
        self.traffic_at(x as f32, y as f32)
    }

    /// Logical hit rect for window dragging, traffic cluster cut
    /// out so light clicks never drag. Expanded it spans the
    /// sidebar traffic band; collapsed the content band up to the
    /// far-right pills.
    pub fn drag_rect(&self) -> (f32, f32, f32, f32) {
        let cut = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0;
        if self.collapsed {
            let left_w = self.left_bar_w();
            let pills_w = SIDEBAR_PAD
                + self.right_bar_w()
                + if left_w > 0.0 {
                    left_w + TOOLBAR_GAP
                } else {
                    0.0
                };
            let end = self.x + self.width - pills_w;
            (
                self.x + cut,
                self.y,
                (end - self.x - cut).max(0.0),
                SIDEBAR_TOOLBAR_H,
            )
        } else {
            (
                self.x + cut,
                self.y,
                (self.bar_w() - cut).max(0.0),
                SIDEBAR_TRAFFIC_TOP + TRAFFIC_SIZE + 6.0,
            )
        }
    }

    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.traffic_hover = self.traffic_index(x, y);
        self.left_bar.set_hover(x, y);
        self.right_bar.set_hover(x, y);
        if let Some(page) = self.active_page_mut() {
            page.set_hover(x, y);
        }
    }

    fn toolbar_cy(&self) -> f32 {
        if self.collapsed {
            self.y + (SIDEBAR_TOOLBAR_H - TOOLBAR_HEIGHT) / 2.0
        } else {
            self.y + SIDEBAR_BAR_TOP
        }
    }

    /// Left pill icon count (back when shown, then devs).
    fn left_count(&self) -> usize {
        (if self.show_back { 1 } else { 0 }) + self.dev_icons.len()
    }

    /// Left pill width in logical px, zero when empty.
    fn left_bar_w(&self) -> f32 {
        let n = self.left_count() as f32;
        if n <= 0.0 {
            0.0
        } else {
            TOOLBAR_PAD_X * 2.0 + n * TOOLBAR_HIT + (n - 1.0) * TOOLBAR_GAP
        }
    }

    /// Right pill width in logical px (toggle alone, zero hidden).
    fn right_bar_w(&self) -> f32 {
        if self.show_toggle {
            TOOLBAR_PAD_X * 2.0 + TOOLBAR_HIT
        } else {
            0.0
        }
    }

    fn title_x(&self) -> f32 {
        if self.collapsed {
            self.x + TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0 + SIDEBAR_PAD
        } else {
            self.content_x() + SIDEBAR_PAD
        }
    }

    fn place_bars(&mut self, fonts: &mut FontSystem) {
        let cy = self.toolbar_cy();
        if self.collapsed {
            // Far-right group in the content: left pill, gap, toggle.
            if self.right_bar_w() > 0.0 {
                let right_x = self.x + self.width - SIDEBAR_PAD - self.right_bar_w();
                self.right_bar.place(fonts, right_x, cy, self.right_bar_w(), TOOLBAR_HEIGHT);
                if self.left_bar_w() > 0.0 {
                    self.left_bar.place(
                        fonts,
                        right_x - TOOLBAR_GAP - self.left_bar_w(),
                        cy,
                        self.left_bar_w(),
                        TOOLBAR_HEIGHT,
                    );
                }
            } else if self.left_bar_w() > 0.0 {
                let bar_x = self.x + self.width - SIDEBAR_PAD - self.left_bar_w();
                self.left_bar.place(fonts, bar_x, cy, self.left_bar_w(), TOOLBAR_HEIGHT);
            }
        } else {
            // Sidebar top row: pill left, single toggle right.
            if self.left_bar_w() > 0.0 {
                self.left_bar.place(
                    fonts,
                    self.x + SIDEBAR_PAD,
                    cy,
                    self.left_bar_w(),
                    TOOLBAR_HEIGHT,
                );
            }
            if self.right_bar_w() > 0.0 {
                self.right_bar.place(
                    fonts,
                    self.x + self.bar_w() - SIDEBAR_PAD - self.right_bar_w(),
                    cy,
                    self.right_bar_w(),
                    TOOLBAR_HEIGHT,
                );
            }
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let (x32, y32) = (x as f32, y as f32);
        self.left_bar.mouse_down(x, y);
        self.right_bar.mouse_down(x, y);
        // Item rows (sidebar visible only).
        if !self.collapsed && x32 >= self.x && x32 <= self.x + self.bar_w() {
            let top = self.y + SIDEBAR_ITEMS_TOP;
            if y32 >= top {
                let index = ((y32 - top) / SIDEBAR_ROW_H).floor() as usize;
                if index < self.items.len() {
                    self.select(index);
                }
            }
        }
        if let Some(page) = self.active_page_mut() {
            page.mouse_down(x, y);
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.left_bar.mouse_up(x, y);
        self.right_bar.mouse_up(x, y);
        // Pill clicks land in shared pending state (applied here, so
        // unit tests never need a draw in between).
        self.drain_pending();
        if let Some(page) = self.active_page_mut() {
            page.mouse_up(x, y);
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.left_bar.mouse_move(x as f32, y as f32);
        self.right_bar.mouse_move(x as f32, y as f32);
    }

    /// Scroll wheel delta in logical px: forwarded to the active page.
    pub fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        if let Some(page) = self.active_page_mut() {
            page.mouse_wheel(dx, dy);
        }
    }

    /// Printable text for the active page (the app forwards its
    /// `text` here).
    pub fn page_text(&mut self, text: &str) {
        if let Some(page) = self.active_page_mut() {
            page.text(text);
        }
    }

    /// Key handling for the active page. Returns true when consumed.
    pub fn page_key(&mut self, key: Key) -> bool {
        if let Some(page) = self.active_page_mut() {
            page.key(key)
        } else {
            false
        }
    }

    fn page_rect(&self) -> (f32, f32, f32, f32) {
        let cx = self.content_x();
        (
            cx,
            self.y + SIDEBAR_TOOLBAR_H,
            (self.x + self.width - cx).max(0.0),
            (self.height - SIDEBAR_TOOLBAR_H).max(0.0),
        )
    }

    fn effective_title(&self) -> String {
        if let Some(title) = self.title.clone() {
            return title;
        }
        self.items
            .get(self.selected)
            .map(|item| item.label.clone())
            .unwrap_or_default()
    }

    fn render(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        if self.width <= 0.0 || self.height <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let bar_w = self.bar_w();
        // Sidebar body (full height, square: the shell rounds the window).
        if bar_w > 0.0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.sidebar_bg()),
                None,
                &vello::kurbo::Rect::new(px(self.x), px(self.y), px(self.x + bar_w), px(self.y + self.height)),
            );
            // Divider between sidebar and content.
            let dx = self.x + bar_w;
            scene.stroke(
                &Stroke::new(1.0 * scale),
                Affine::IDENTITY,
                &Brush::Solid(self.divider_color()),
                None,
                &Line::new((px(dx), px(self.y)), (px(dx), px(self.y + self.height))),
            );
            self.render_items(scene, fonts, images);
        }
        // Traffic on top of everything (sidebar top when expanded,
        // content top-left when collapsed).
        self.render_traffic(scene, fonts);
        self.render_toolbar(scene, fonts, images);
        // Active page below the toolbar.
        let (px0, py0, pw, ph) = self.page_rect();
        if pw > 0.0 && ph > 0.0 {
            if let Some(page) = self.active_page_mut() {
                page.place(fonts, px0, py0, pw, ph);
                page.draw(scene, fonts, images);
            }
        }
    }

    fn render_traffic(&self, scene: &mut Scene, fonts: &FontSystem) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let colors = if self.focused {
            [TRAFFIC_CLOSE, TRAFFIC_MINIMIZE, TRAFFIC_MAXIMIZE]
        } else {
            [TRAFFIC_INACTIVE; 3]
        };
        for (index, color) in colors.iter().enumerate() {
            let (cx, cy) = self.traffic_center(index);
            let circle = Circle::new((px(cx), px(cy)), (TRAFFIC_SIZE / 2.0 * fonts.scale) as f64);
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(*color), None, &circle);
            // Hovered light gets a subtle white ring (glyphs stay a
            // titlebar-only detail for now).
            if self.traffic_hover == Some(index) && self.focused {
                let ring = Circle::new((px(cx), px(cy)), (TRAFFIC_SIZE / 2.0 + 2.0) as f64 * scale);
                scene.stroke(
                    &Stroke::new(1.5 * scale),
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(255, 255, 255, 120)),
                    None,
                    &ring,
                );
            }
        }
    }

    fn render_items(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        let scale = fonts.scale as f64;
        let text = self.eff(self.text_color());
        for (i, item) in self.items.iter().enumerate() {
            let ry = self.y + SIDEBAR_ITEMS_TOP + i as f32 * SIDEBAR_ROW_H;
            if i == self.selected {
                let wash = RoundedRect::new(
                    (self.x + 8.0) as f64 * scale,
                    ry as f64 * scale,
                    (self.x + self.bar_w() - 8.0) as f64 * scale,
                    (ry + SIDEBAR_ROW_H) as f64 * scale,
                    10.0 * scale,
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(if self.dark {
                        SIDEBAR_SEL_DARK
                    } else {
                        SIDEBAR_SEL_LIGHT
                    })),
                    None,
                    &wash,
                );
            }
            let tint = self.eff(item.tint.unwrap_or(self.accent));
            let mut icon = SFSymbolImage::new(item.icon.clone())
                .size(SIDEBAR_ICON_SIZE)
                .color(tint);
            icon.place(
                fonts,
                self.x + SIDEBAR_PAD,
                ry + (SIDEBAR_ROW_H - SIDEBAR_ICON_SIZE) / 2.0,
                SIDEBAR_ICON_SIZE,
                SIDEBAR_ICON_SIZE,
            );
            icon.draw(scene, fonts, images);
            let layout = fonts.layout_text(&item.label, SIDEBAR_LABEL_SIZE, text, None);
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.x + SIDEBAR_PAD + SIDEBAR_ICON_SIZE + SIDEBAR_ICON_GAP,
                ry + (SIDEBAR_ROW_H - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }
    }

    fn render_toolbar(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        // Pending pill actions apply here too (clicks handled
        // between frames still land before the next paint).
        self.drain_pending();
        let title = self.effective_title();
        let layout = fonts.layout_text_weighted(&title, SIDEBAR_TITLE_SIZE, self.eff(self.text_color()), 600.0, None);
        let (_, th) = FontSystem::layout_size(&layout);
        draw_layout(
            scene,
            &layout,
            self.title_x(),
            self.y + (SIDEBAR_TOOLBAR_H - th / fonts.scale) / 2.0,
            fonts.scale,
        );
        if self.left_bar_w() > 0.0 {
            self.left_bar.draw(scene, fonts, images);
        }
        if self.right_bar_w() > 0.0 {
            self.right_bar.draw(scene, fonts, images);
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl View for Sidebar {
    /// Intrinsic size: sidebar plus a 480 px content minimum with
    /// the full placed height. Apps usually place it full-bleed
    /// over the viewport instead (no titlebar needed: traffic
    /// lights live in the sidebar).
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.bar_w() + 480.0, self.height.max(SIDEBAR_TOOLBAR_H))
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(0.0);
        self.height = h.max(0.0);
        self.place_bars(fonts);
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.mouse_up(x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.set_hover(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar() -> Sidebar {
        let mut bar = Sidebar::new(vec![
            SidebarItem::new("General", "gear"),
            SidebarItem::new("Security", "lock.fill"),
        ])
        .page(crate::elements::BasicText::new("General page"))
        .page(crate::elements::BasicText::new("Security page"));
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        bar
    }

    #[test]
    fn select_switches_and_fires_on_change() {
        use std::cell::Cell;
        use std::rc::Rc;

        let seen: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
        let capture = seen.clone();
        let mut bar = bar().on_select(move |i| capture.set(Some(i)));
        assert_eq!(bar.selected_index(), 0);
        assert!(bar.select(1));
        assert_eq!(bar.selected_index(), 1);
        assert_eq!(seen.get(), Some(1));
        assert!(!bar.select(9));
        assert_eq!(bar.selected_index(), 1);
    }

    #[test]
    fn item_click_selects() {
        let mut bar = bar();
        bar.mouse_down(100.0, (SIDEBAR_ITEMS_TOP + SIDEBAR_ROW_H + 10.0) as f64);
        assert_eq!(bar.selected_index(), 1);
    }

    /// Center of a pill cell at `(bar_x, bar_y)`: leading layout,
    /// pad plus hit boxes with gaps, vertically centered.
    fn pill_cell(bar_x: f32, bar_y: f32, index: usize) -> (f64, f64) {
        let cx = bar_x + TOOLBAR_PAD_X + index as f32 * (TOOLBAR_HIT + TOOLBAR_GAP)
            + TOOLBAR_HIT / 2.0;
        let cy = bar_y + (TOOLBAR_HEIGHT - TOOLBAR_HIT) / 2.0 + TOOLBAR_HIT / 2.0;
        (cx as f64, cy as f64)
    }

    /// Toggle cell in the expanded sidebar (single pill far right).
    fn toggle_cell() -> (f64, f64) {
        let bar_w = TOOLBAR_PAD_X * 2.0 + TOOLBAR_HIT;
        pill_cell(SIDEBAR_W - SIDEBAR_PAD - bar_w, SIDEBAR_BAR_TOP, 0)
    }

    /// Back cell in the expanded sidebar (left pill first icon).
    fn back_cell() -> (f64, f64) {
        pill_cell(SIDEBAR_PAD, SIDEBAR_BAR_TOP, 0)
    }

    /// Toggle cell when collapsed (single pill at the far right edge).
    fn collapsed_toggle_cell(bar: &Sidebar) -> (f64, f64) {
        let _ = bar;
        let bar_w = TOOLBAR_PAD_X * 2.0 + TOOLBAR_HIT;
        let bar_x = 900.0 - SIDEBAR_PAD - bar_w;
        let bar_y = (SIDEBAR_TOOLBAR_H - TOOLBAR_HEIGHT) / 2.0;
        pill_cell(bar_x, bar_y, 0)
    }

    #[test]
    fn toggle_button_collapses_and_reopens() {
        let mut bar = bar();
        assert!(!bar.is_collapsed());
        let (tx, ty) = toggle_cell();
        bar.mouse_down(tx, ty);
        bar.mouse_up(tx, ty);
        assert!(bar.is_collapsed());
        // Reopen through the collapsed far-right pill (frames
        // re-place every draw, like real apps do).
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        let (rx, ry) = collapsed_toggle_cell(&bar);
        bar.mouse_down(rx, ry);
        bar.mouse_up(rx, ry);
        assert!(!bar.is_collapsed());
    }

    #[test]
    fn disabled_collapse_ignores_toggle() {
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")]).collapsible(false);
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        let (cx, cy) = toggle_cell();
        bar.mouse_down(cx, cy);
        bar.mouse_up(cx, cy);
        assert!(!bar.is_collapsed());
    }

    #[test]
    fn hidden_toggle_moves_back_first() {
        use std::cell::Cell;
        use std::rc::Rc;

        let count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let taps = count.clone();
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .toggle_button(false)
            .on_back(move || taps.set(taps.get() + 1));
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        // Back is the first left-pill icon (toggle lives far right).
        let (bx, by) = pill_cell(SIDEBAR_PAD, SIDEBAR_BAR_TOP, 0);
        bar.mouse_down(bx, by);
        bar.mouse_up(bx, by);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn empty_left_pill_is_skipped() {
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .toggle_button(false)
            .back_button(false);
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        assert_eq!(bar.left_bar_w(), 0.0);
        // Click where the pill would sit: nothing happens.
        bar.mouse_down(38.0, SIDEBAR_BAR_TOP as f64 + 18.0);
        bar.mouse_up(38.0, SIDEBAR_BAR_TOP as f64 + 18.0);
        assert!(!bar.is_collapsed());
        assert_eq!(bar.selected_index(), 0);
    }

    #[test]
    fn back_button_fires_when_shown() {
        use std::cell::Cell;
        use std::rc::Rc;

        let count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let taps = count.clone();
        let mut bar = bar().on_back(move || taps.set(taps.get() + 1));
        let (bx, by) = back_cell();
        bar.mouse_down(bx, by);
        bar.mouse_up(bx, by);
        assert_eq!(count.get(), 1);
        // Hidden back never fires (left pill stays empty there).
        let mut hidden = Sidebar::new(vec![SidebarItem::new("G", "gear")]).back_button(false);
        hidden.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        hidden.mouse_down(bx, by);
        hidden.mouse_up(bx, by);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn dev_icon_fires_its_callback() {
        use std::cell::Cell;
        use std::rc::Rc;

        let count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let taps = count.clone();
        let mut bar = bar();
        bar.add_toolbar_button("magnifyingglass", move || taps.set(taps.get() + 1));
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        // Dev icons follow back in the left pill.
        let (bx, by) = pill_cell(SIDEBAR_PAD, SIDEBAR_BAR_TOP, 1);
        bar.mouse_down(bx, by);
        bar.mouse_up(bx, by);
        assert_eq!(count.get(), 1);
        assert!(!bar.is_collapsed());
    }

    #[test]
    fn traffic_press_maps_lights() {
        let mut bar = bar();
        let (cx, cy) = (TRAFFIC_LEFT + TRAFFIC_SIZE / 2.0, SIDEBAR_TRAFFIC_TOP + TRAFFIC_SIZE / 2.0);
        assert_eq!(bar.press(cx as f64, cy as f64), Some(TrafficAction::Close));
        let (mx, my) = (cx + TRAFFIC_SIZE + TRAFFIC_GAP, cy);
        assert_eq!(bar.press(mx as f64, my as f64), Some(TrafficAction::Minimize));
        let (xx, xy) = (mx + TRAFFIC_SIZE + TRAFFIC_GAP, cy);
        assert_eq!(bar.press(xx as f64, xy as f64), Some(TrafficAction::Maximize));
        assert_eq!(bar.press(500.0, 500.0), None);
    }

    #[test]
    fn drag_rect_excludes_traffic_cluster() {
        let bar = bar();
        let (dx, dy, dw, dh) = bar.drag_rect();
        let cluster_end = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0;
        assert_eq!(dx, cluster_end);
        assert_eq!(dw, SIDEBAR_W - cluster_end);
        assert_eq!((dy, dh), (0.0, SIDEBAR_TRAFFIC_TOP + TRAFFIC_SIZE + 6.0));
    }

    #[test]
    fn collapsed_drag_reaches_pills_edge() {
        let mut bar = bar();
        bar.set_collapsed(true);
        let (dx, _, dw, _) = bar.drag_rect();
        let cluster_end = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0;
        assert_eq!(dx, cluster_end);
        // Drag ends where the far-right group begins.
        let left_w = bar.left_bar_w();
        let pills_w = SIDEBAR_PAD
            + bar.right_bar_w()
            + if left_w > 0.0 {
                left_w + TOOLBAR_GAP
            } else {
                0.0
            };
        assert_eq!(dw, 900.0 - pills_w - cluster_end);
    }

    #[test]
    fn collapsed_content_fills_width() {
        let mut bar = bar();
        bar.set_collapsed(true);
        assert_eq!(bar.bar_w(), 0.0);
        let (px, _, pw, _) = bar.page_rect();
        assert_eq!((px, pw), (0.0, 900.0));
    }

    #[test]
    fn missing_page_stays_empty() {
        let mut bar = Sidebar::new(vec![SidebarItem::new("Solo", "gear")]);
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        assert!(bar.active_page_mut().is_none());
        assert!(bar.select(0));
    }
}
