use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Circle, Line, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::{Button, ButtonShape};
use super::super::groupbox::{GROUP_BG_DARK, GROUP_BG_LIGHT};
use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::super::titlebar::{
    TRAFFIC_CLOSE, TRAFFIC_GAP, TRAFFIC_INACTIVE, TRAFFIC_LEFT, TRAFFIC_MAXIMIZE,
    TRAFFIC_MINIMIZE, TRAFFIC_SIZE, TITLEBAR_DIVIDER_DARK, TITLEBAR_DIVIDER_LIGHT,
    TrafficAction,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::renderer::window::Key;
use crate::theme::desaturate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SidebarButton {
    Toggle,
    Back,
    Collapse,
    Dev(usize),
}

/// Sidebar width in logical px.
pub const SIDEBAR_W: f32 = 300.0;
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
/// Items top edge in logical px.
pub const SIDEBAR_ITEMS_TOP: f32 = 84.0;
/// Content toolbar height in logical px.
pub const SIDEBAR_TOOLBAR_H: f32 = 64.0;
/// Toolbar circle button diameter in logical px.
pub const SIDEBAR_TOOLBAR_BTN: f32 = 40.0;
/// Gap between toolbar buttons in logical px.
pub const SIDEBAR_TOOLBAR_GAP: f32 = 12.0;
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
/// content toolbar with toggle, back, title, dev buttons and
/// collapse. Selecting an item switches the right-side page, which
/// the sidebar owns (one page per item; missing pages stay empty).
/// Collapsing hides the column and lets the content fill the width;
/// the toggle button reopens it.
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
    dev_buttons: Vec<Button>,
    toggle_btn: Button,
    back_btn: Button,
    collapse_btn: Button,
    width_setting: f32,
    accent: Color,
    dark: bool,
    focused: bool,
    traffic_hover: Option<usize>,
    armed: Option<SidebarButton>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Sidebar {
    pub fn new(items: Vec<SidebarItem>) -> Self {
        Self {
            items,
            pages: Vec::new(),
            selected: 0,
            on_select: None,
            title: None,
            show_back: true,
            on_back: None,
            collapsed: false,
            on_collapse: None,
            dev_buttons: Vec::new(),
            toggle_btn: Button::new("")
                .shape(ButtonShape::Circle)
                .icon("sidebar.left")
                .icon_size(20.0),
            back_btn: Button::new("")
                .shape(ButtonShape::Circle)
                .icon("chevron.left")
                .icon_size(20.0),
            collapse_btn: Button::new("")
                .shape(ButtonShape::Circle)
                .icon("chevron.left.to.line")
                .icon_size(20.0),
            width_setting: SIDEBAR_W,
            accent: Color::from_rgb8(0x00, 0x7a, 0xff),
            dark: true,
            focused: true,
            traffic_hover: None,
            armed: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
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

    /// Back button in the toolbar (default `true`).
    pub fn back_button(mut self, show: bool) -> Self {
        self.show_back = show;
        self
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

    /// Extra toolbar buttons at the right, before collapse (app-built
    /// circle icon buttons with press callbacks).
    pub fn toolbar_button(mut self, button: Button) -> Self {
        self.dev_buttons.push(button);
        self
    }

    /// Add a toolbar button after construction (for callbacks that
    /// need a shared handle to the sidebar itself).
    pub fn add_toolbar_button(&mut self, button: Button) {
        self.dev_buttons.push(button);
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
    }

    /// User collapse flip: toggles and fires `on_collapse`.
    pub fn toggle_sidebar(&mut self) {
        self.collapsed = !self.collapsed;
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

    /// Live theme forwarded to buttons, icons and labels.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        self.dark = dark;
        for button in self.all_buttons_mut() {
            button.set_theme(accent, dark);
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        for button in self.all_buttons_mut() {
            button.set_focused(focused);
        }
    }

    fn all_buttons_mut(&mut self) -> Vec<&mut Button> {
        let mut out = Vec::new();
        out.push(&mut self.toggle_btn);
        out.push(&mut self.back_btn);
        out.push(&mut self.collapse_btn);
        out.extend(self.dev_buttons.iter_mut());
        out
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
        if self.collapsed {
            return None;
        }
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

    /// Logical hit rect for window dragging: the sidebar top strip
    /// minus the traffic cluster, so light clicks never drag.
    pub fn drag_rect(&self) -> (f32, f32, f32, f32) {
        let cut = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0;
        (
            self.x + cut,
            self.y,
            (self.bar_w() - cut).max(0.0),
            SIDEBAR_ITEMS_TOP,
        )
    }

    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.traffic_hover = self.traffic_index(x, y);
        self.toggle_btn.set_hover(x, y);
        self.back_btn.set_hover(x, y);
        self.collapse_btn.set_hover(x, y);
        for button in &mut self.dev_buttons {
            button.set_hover(x, y);
        }
        if let Some(page) = self.active_page_mut() {
            page.set_hover(x, y);
        }
    }

    fn toolbar_cy(&self) -> f32 {
        self.y + (SIDEBAR_TOOLBAR_H - SIDEBAR_TOOLBAR_BTN) / 2.0
    }

    fn toggle_rect(&self) -> (f32, f32, f32, f32) {
        (
            self.content_x() + SIDEBAR_PAD,
            self.toolbar_cy(),
            SIDEBAR_TOOLBAR_BTN,
            SIDEBAR_TOOLBAR_BTN,
        )
    }

    fn back_rect(&self) -> Option<(f32, f32, f32, f32)> {
        if !self.show_back {
            return None;
        }
        let (tx, ty, _, _) = self.toggle_rect();
        Some((
            tx + SIDEBAR_TOOLBAR_BTN + SIDEBAR_TOOLBAR_GAP,
            ty,
            SIDEBAR_TOOLBAR_BTN,
            SIDEBAR_TOOLBAR_BTN,
        ))
    }

    fn title_x(&self) -> f32 {
        match self.back_rect() {
            Some((bx, _, bw, _)) => bx + bw + SIDEBAR_TOOLBAR_GAP,
            None => {
                let (tx, _, bw, _) = self.toggle_rect();
                tx + bw + SIDEBAR_TOOLBAR_GAP
            }
        }
    }

    fn collapse_rect(&self) -> (f32, f32, f32, f32) {
        (
            self.x + self.width - SIDEBAR_PAD - SIDEBAR_TOOLBAR_BTN,
            self.toolbar_cy(),
            SIDEBAR_TOOLBAR_BTN,
            SIDEBAR_TOOLBAR_BTN,
        )
    }

    fn dev_rect(&self, index: usize) -> Option<(f32, f32, f32, f32)> {
        if index >= self.dev_buttons.len() {
            return None;
        }
        // Dev buttons line up left of collapse, in order.
        let from_right = self.dev_buttons.len() - index;
        let (cx, cy, _, _) = self.collapse_rect();
        Some((
            cx - from_right as f32 * (SIDEBAR_TOOLBAR_BTN + SIDEBAR_TOOLBAR_GAP),
            cy,
            SIDEBAR_TOOLBAR_BTN,
            SIDEBAR_TOOLBAR_BTN,
        ))
    }

    fn place_buttons(&mut self, fonts: &mut FontSystem) {
        let (tx, ty, tw, th) = self.toggle_rect();
        self.toggle_btn.place(fonts, tx, ty, tw, th);
        if let Some((bx, by, bw, bh)) = self.back_rect() {
            self.back_btn.place(fonts, bx, by, bw, bh);
        }
        for i in 0..self.dev_buttons.len() {
            if let Some((dx, dy, dw, dh)) = self.dev_rect(i) {
                if let Some(button) = self.dev_buttons.get_mut(i) {
                    button.place(fonts, dx, dy, dw, dh);
                }
            }
        }
        let (cx, cy, cw, ch) = self.collapse_rect();
        self.collapse_btn.place(fonts, cx, cy, cw, ch);
    }

    fn button_at(&self, x: f32, y: f32) -> Option<SidebarButton> {
        let in_rect = |(bx, by, bw, bh): (f32, f32, f32, f32)| {
            x >= bx && x <= bx + bw && y >= by && y <= by + bh
        };
        if in_rect(self.toggle_rect()) {
            return Some(SidebarButton::Toggle);
        }
        if let Some(rect) = self.back_rect() {
            if in_rect(rect) {
                return Some(SidebarButton::Back);
            }
        }
        if in_rect(self.collapse_rect()) {
            return Some(SidebarButton::Collapse);
        }
        for i in 0..self.dev_buttons.len() {
            if let Some(rect) = self.dev_rect(i) {
                if in_rect(rect) {
                    return Some(SidebarButton::Dev(i));
                }
            }
        }
        None
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let (x32, y32) = (x as f32, y as f32);
        // Arm one toolbar button: release on the same button fires.
        // The toggle/back/collapse buttons carry no press callback of
        // their own (the sidebar fires below); dev buttons fire
        // theirs through `Button` itself.
        self.armed = self.button_at(x32, y32);
        match self.armed {
            Some(SidebarButton::Toggle) => self.toggle_btn.mouse_down(x, y),
            Some(SidebarButton::Back) => self.back_btn.mouse_down(x, y),
            Some(SidebarButton::Collapse) => self.collapse_btn.mouse_down(x, y),
            Some(SidebarButton::Dev(i)) => {
                if let Some(button) = self.dev_buttons.get_mut(i) {
                    button.mouse_down(x, y);
                }
            }
            None => {}
        }
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
        let armed = self.armed.take();
        let released = self.button_at(x as f32, y as f32);
        self.toggle_btn.mouse_up(x, y);
        self.back_btn.mouse_up(x, y);
        self.collapse_btn.mouse_up(x, y);
        for button in &mut self.dev_buttons {
            button.mouse_up(x, y);
        }
        // Fire only for press-plus-release on the same button, so a
        // drag across the toolbar never flips state.
        if armed.is_some() && armed == released {
            match armed {
                Some(SidebarButton::Toggle) | Some(SidebarButton::Collapse) => {
                    self.toggle_sidebar()
                }
                Some(SidebarButton::Back) => {
                    if let Some(callback) = self.on_back.as_mut() {
                        callback();
                    }
                }
                _ => {}
            }
        }
        if let Some(page) = self.active_page_mut() {
            page.mouse_up(x, y);
        }
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
            self.render_traffic(scene, fonts);
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
        self.toggle_btn.draw(scene, fonts, images);
        if self.show_back {
            self.back_btn.draw(scene, fonts, images);
        }
        for button in self.dev_buttons.iter_mut() {
            button.draw(scene, fonts, images);
        }
        self.collapse_btn.draw(scene, fonts, images);
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
        self.place_buttons(fonts);
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

    #[test]
    fn toggle_button_collapses_and_reopens() {
        // Toggle sits first in the content toolbar.
        let mut bar = bar();
        let tx = SIDEBAR_W + SIDEBAR_PAD + SIDEBAR_TOOLBAR_BTN / 2.0;
        let ty = (SIDEBAR_TOOLBAR_H - SIDEBAR_TOOLBAR_BTN) / 2.0 + SIDEBAR_TOOLBAR_BTN / 2.0;
        assert!(!bar.is_collapsed());
        bar.mouse_down(tx as f64, ty as f64);
        bar.mouse_up(tx as f64, ty as f64);
        assert!(bar.is_collapsed());
        // Reopen: collapsed, the toolbar (with toggle) starts at x.
        let rx = SIDEBAR_PAD + SIDEBAR_TOOLBAR_BTN / 2.0;
        bar.mouse_down(rx as f64, ty as f64);
        bar.mouse_up(rx as f64, ty as f64);
        assert!(!bar.is_collapsed());
    }

    #[test]
    fn collapse_button_collapses() {
        let mut bar = bar();
        let cx = 900.0 - SIDEBAR_PAD - SIDEBAR_TOOLBAR_BTN / 2.0;
        let cy = SIDEBAR_TOOLBAR_H / 2.0;
        bar.mouse_down(cx as f64, cy as f64);
        bar.mouse_up(cx as f64, cy as f64);
        assert!(bar.is_collapsed());
    }

    #[test]
    fn back_button_fires_when_shown() {
        use std::cell::Cell;
        use std::rc::Rc;

        let count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let taps = count.clone();
        let mut bar = bar().on_back(move || taps.set(taps.get() + 1));
        let bx = SIDEBAR_W + SIDEBAR_PAD + SIDEBAR_TOOLBAR_BTN + SIDEBAR_TOOLBAR_GAP
            + SIDEBAR_TOOLBAR_BTN / 2.0;
        let by = SIDEBAR_TOOLBAR_H / 2.0;
        bar.mouse_down(bx as f64, by as f64);
        bar.mouse_up(bx as f64, by as f64);
        assert_eq!(count.get(), 1);
        // Hidden back never fires.
        let mut hidden = Sidebar::new(vec![SidebarItem::new("G", "gear")]).back_button(false);
        hidden.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        hidden.mouse_down(bx as f64, by as f64);
        hidden.mouse_up(bx as f64, by as f64);
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
        assert!(dw > 0.0);
        assert_eq!((dy, dh), (0.0, SIDEBAR_ITEMS_TOP));
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
