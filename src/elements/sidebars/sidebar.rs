use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Circle, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{BlendMode, Brush, Color, Fill};

use super::super::groupbox::{GROUP_BG_DARK, GROUP_BG_LIGHT};
use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::super::titlebar::{
    TRAFFIC_CLOSE, TRAFFIC_GAP, TRAFFIC_INACTIVE, TRAFFIC_LEFT, TRAFFIC_MAXIMIZE,
    TRAFFIC_MINIMIZE, TRAFFIC_SIZE, TITLEBAR_DIVIDER_DARK, TITLEBAR_DIVIDER_LIGHT,
    TrafficAction,
};
use super::super::textfield::SearchField;
use super::super::toolbar::{
    BasicToolbar, ToolbarItem, TOOLBAR_GAP, TOOLBAR_HEIGHT, TOOLBAR_HIT, TOOLBAR_PAD_X,
};
use crate::animation::{Easing, Tween, TweenAnim};
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
/// Absolute minimum sidebar width floor in logical px (the live
/// minimum is larger while pills are shown, see `min_bar_w`).
pub const SIDEBAR_MIN_W: f32 = 120.0;
/// Maximum sidebar width in logical px for edge drags and `width`.
pub const SIDEBAR_MAX_W: f32 = 480.0;
/// Resize grab half-width in logical px around the column edge.
pub const SIDEBAR_RESIZE_HIT: f32 = 6.0;
/// Reopen strip width in logical px at the content left edge while
/// collapsed.
pub const SIDEBAR_REOPEN_HIT: f32 = 8.0;
/// Dragging further than this below the minimum snaps shut.
pub const SIDEBAR_CLOSE_SLOP: f32 = 48.0;
/// Item row height in logical px.
pub const SIDEBAR_ROW_H: f32 = 46.0;
/// Sidebar inset in logical px.
pub const SIDEBAR_PAD: f32 = 16.0;
/// Gap between the traffic cluster and the left pill in logical
/// px (clears the light glow plus glass reflection).
pub const SIDEBAR_PILL_GAP: f32 = 12.0;
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
/// Search row top edge in logical px (below the pills).
pub const SIDEBAR_SEARCH_TOP: f32 = 60.0;
/// Search row height in logical px.
pub const SIDEBAR_SEARCH_H: f32 = 36.0;
/// Items top edge in logical px.
pub const SIDEBAR_ITEMS_TOP: f32 = 108.0;
/// Content toolbar height in logical px.
pub const SIDEBAR_TOOLBAR_H: f32 = 64.0;
/// Collapse/expand slide plus fade in seconds.
pub const SIDEBAR_COLLAPSE_SECONDS: f32 = 0.22;
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

/// One optional left-pill button: icon plus press callback.
/// Slot 0 renders first, slot 1 second; unset slots stay absent.
struct LeftSlot {
    icon: String,
    on_press: Box<dyn FnMut()>,
}

/// App sidebar: full-height navigation column with embedded traffic
/// lights (replacing the titlebar decoration), icon items and a
/// content toolbar with two optional slot buttons, title and a
/// single far-right toggle. Selecting an item switches the
/// right-side page, which the sidebar owns (one page per item;
/// missing pages stay empty). Collapsing hides the column and lets
/// the content fill the width; the toggle button reopens it.
pub struct Sidebar {
    items: Vec<SidebarItem>,
    pages: Vec<Box<dyn View>>,
    selected: usize,
    on_select: Option<Box<dyn FnMut(usize)>>,
    title: Option<String>,
    collapsed: bool,
    on_collapse: Option<Box<dyn FnMut(bool)>>,
    collapse_anim: Option<TweenAnim<f32>>,
    collapse_t0: Instant,
    anim_p: f32,
    resizing: bool,
    resize_hover: bool,
    show_toggle: bool,
    collapsible: bool,
    left_bar: BasicToolbar,
    right_bar: BasicToolbar,
    left_slots: [Option<LeftSlot>; 2],
    slot_map: Vec<usize>,
    search: SearchField,
    show_search: bool,
    on_search: Option<Box<dyn FnMut(&str)>>,
    search_last: String,
    visible: Vec<usize>,
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
        let visible: Vec<usize> = (0..items.len()).collect();
        let mut bar = Self {
            items,
            pages: Vec::new(),
            selected: 0,
            on_select: None,
            title: None,
            show_toggle: true,
            collapsible: true,
            collapsed: false,
            on_collapse: None,
            collapse_anim: None,
            collapse_t0: Instant::now(),
            anim_p: 0.0,
            resizing: false,
            resize_hover: false,
            left_bar: BasicToolbar::new(),
            right_bar: BasicToolbar::new(),
            left_slots: [None, None],
            slot_map: Vec::new(),
            search: SearchField::new("Search"),
            show_search: true,
            on_search: None,
            search_last: String::new(),
            visible,
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

    /// Rebuild the pills: left holds the two optional slots (slot
    /// 0 first, slot 1 second; unset slots stay absent), right
    /// holds the toggle alone (optional, far right). Both pills sit
    /// fixed at the top: traffic row when expanded, toolbar row
    /// when collapsed. Actions report through shared pending state
    /// (applied on the next mouse-up or draw).
    fn sync_bars(&mut self) {
        let icons: Vec<(usize, String)> = self
            .left_slots
            .iter()
            .enumerate()
            .filter_map(|(slot, entry)| {
                entry.as_ref().map(|button| (slot, button.icon.clone()))
            })
            .collect();
        self.slot_map = icons.iter().map(|(slot, _)| *slot).collect();
        let left: Vec<ToolbarItem> = icons
            .into_iter()
            .map(|(_, icon)| ToolbarItem::icon(icon))
            .collect();
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
    /// their slot callback in pill order.
    fn drain_pending(&mut self) {
        match self.pending.take() {
            Some(PendingAction::Toggle) => {
                self.toggle_sidebar()
            }
            Some(PendingAction::Cell(index)) => {
                if let Some(&slot) = self.slot_map.get(index) {
                    if let Some(button) = self.left_slots[slot].as_mut() {
                        (button.on_press)();
                    }
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

    /// Sidebar width in logical px (clamped to the live minimum
    /// and maximum, ignored while collapsed).
    pub fn width(mut self, px: f32) -> Self {
        self.width_setting = px.clamp(self.min_bar_w(), SIDEBAR_MAX_W);
        self
    }

    /// Sidebar width in logical px at runtime (clamped to the live
    /// minimum and maximum).
    pub fn set_width(&mut self, px: f32) {
        self.width_setting = px.clamp(self.min_bar_w(), SIDEBAR_MAX_W);
    }

    /// Current sidebar width setting in logical px.
    pub fn width_value(&self) -> f32 {
        self.eff_setting()
    }

    /// Live minimum column width: fits traffic plus every shown
    /// pill, never below the absolute floor. Shrinks when the dev
    /// hides slots or the toggle.
    pub fn min_bar_w(&self) -> f32 {
        let cluster = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0;
        let mut min = cluster + SIDEBAR_PAD;
        if self.right_bar_w() > 0.0 {
            min = min.max(SIDEBAR_PAD + self.right_bar_w() + SIDEBAR_PAD);
        }
        if self.left_bar_w() > 0.0 {
            min = min.max(cluster + SIDEBAR_PILL_GAP + self.left_bar_w() + SIDEBAR_PAD);
            if self.right_bar_w() > 0.0 {
                min = min.max(
                    cluster
                        + SIDEBAR_PILL_GAP
                        + self.left_bar_w()
                        + TOOLBAR_GAP
                        + self.right_bar_w()
                        + SIDEBAR_PAD,
                );
            }
        }
        min.max(SIDEBAR_MIN_W)
    }

    /// Width setting clamped to the live range.
    fn eff_setting(&self) -> f32 {
        self.width_setting.clamp(self.min_bar_w(), SIDEBAR_MAX_W)
    }

    /// Left-pill button in slot 0 or 1: icon plus press callback.
    /// Unset slots stay absent; out-of-range slots are ignored.
    pub fn left_button(
        mut self,
        slot: usize,
        icon: impl Into<String>,
        on_press: impl FnMut() + 'static,
    ) -> Self {
        self.set_left_button(slot, icon, on_press);
        self
    }

    /// Set a left-pill button after construction. Returns false for
    /// out-of-range slots (only 0 and 1 exist).
    pub fn set_left_button(
        &mut self,
        slot: usize,
        icon: impl Into<String>,
        on_press: impl FnMut() + 'static,
    ) -> bool {
        if slot >= self.left_slots.len() {
            return false;
        }
        self.left_slots[slot] = Some(LeftSlot {
            icon: icon.into(),
            on_press: Box::new(on_press),
        });
        self.sync_bars();
        true
    }

    /// Remove a left-pill button; the slot stays absent until set
    /// again. No-op for out-of-range slots.
    pub fn clear_left_button(&mut self, slot: usize) {
        if slot >= self.left_slots.len() {
            return;
        }
        self.left_slots[slot] = None;
        self.sync_bars();
    }

    /// Search row below the pills (default `true`). Hidden, the row
    /// is skipped and typing reaches the page.
    pub fn search_field(mut self, show: bool) -> Self {
        self.set_search_field(show);
        self
    }

    pub fn set_search_field(&mut self, show: bool) {
        self.show_search = show;
    }

    /// Fires with the full search text on every edit (live search).
    pub fn on_search(mut self, callback: impl FnMut(&str) + 'static) -> Self {
        self.on_search = Some(Box::new(callback));
        self
    }

    /// Current search text.
    pub fn search_text(&self) -> &str {
        self.search.text_value()
    }

    /// Programmatic search text (refilters, no `on_search`).
    pub fn set_search_text(&mut self, text: impl Into<String>) {
        self.search.set_text(text.into());
        self.poll_search();
    }

    /// True while the pointer wants the I-beam over the search row.
    pub fn search_text_cursor(&self) -> bool {
        self.show_search && !self.collapsed && self.search.wants_text_cursor()
    }

    /// Refilter visible rows when the query changed and fire
    /// `on_search`. Called from event paths and every draw.
    fn poll_search(&mut self) {
        if !self.show_search {
            return;
        }
        let text = self.search.text_value().to_string();
        if text == self.search_last {
            return;
        }
        self.search_last = text.clone();
        let query = text.to_lowercase();
        self.visible = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                query.is_empty() || item.label().to_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect();
        if let Some(callback) = self.on_search.as_mut() {
            callback(&text);
        }
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

    /// Fires with the collapsed state on every user collapse flip.
    pub fn on_collapse(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_collapse = Some(Box::new(callback));
        self
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

    /// Collapse programmatically (no callback, no animation: snaps
    /// at once; use `toggle_sidebar` for the user path).
    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
        self.anim_p = if collapsed { 1.0 } else { 0.0 };
        self.collapse_anim = None;
        self.sync_bars();
    }

    /// User collapse flip: slides plus fades toward the new state
    /// and fires `on_collapse`. Ignored while `collapsible(false)`
    /// (the collapse pill is gone; the toggle cell is the only UI
    /// path). `is_collapsed` flips at once; the visuals catch up
    /// over `SIDEBAR_COLLAPSE_SECONDS`.
    pub fn toggle_sidebar(&mut self) {
        if !self.collapsible {
            return;
        }
        self.collapsed = !self.collapsed;
        let target = if self.collapsed { 1.0 } else { 0.0 };
        self.collapse_anim = Some(TweenAnim::new(
            Tween::new(self.anim_p, target, SIDEBAR_COLLAPSE_SECONDS).easing(Easing::CubicOut),
        ));
        self.collapse_t0 = Instant::now();
        self.sync_bars();
        if let Some(callback) = self.on_collapse.as_mut() {
            callback(self.collapsed);
        }
    }

    /// Collapse progress 0 (expanded) to 1 (collapsed) for a
    /// from/to pair. Pure sampling helper for tests.
    pub fn collapse_sample(from: f32, to: f32, elapsed: f32) -> (f32, bool) {
        Tween::new(from, to, SIDEBAR_COLLAPSE_SECONDS)
            .easing(Easing::CubicOut)
            .sample(elapsed)
    }

    /// Advance the collapse animation to `elapsed` seconds since the
    /// last flip. `draw` feeds the live clock; tests feed fake time.
    pub fn update_progress(&mut self, elapsed: f32) {
        let (done, value) = match self.collapse_anim.as_mut() {
            Some(anim) => (anim.update(elapsed), *anim.value()),
            None => return,
        };
        if done {
            self.collapse_anim = None;
        }
        self.anim_p = value;
    }

    /// True while the collapse animation runs.
    pub fn is_animating(&self) -> bool {
        self.collapse_anim.is_some()
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

    /// Live theme forwarded to item icons, labels and the search
    /// row (toolbar pills keep their own icon grays per mode).
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        self.dark = dark;
        self.search
            .set_theme(self.glass_mode, accent, self.glass_amount);
    }

    /// Glass stage for the toolbar pills (Lens finish) and the
    /// search row (Frosted finish).
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.glass_mode = mode;
        self.glass_amount = amount;
        self.left_bar.set_theme(mode, amount);
        self.right_bar.set_theme(mode, amount);
        self.search.set_theme(mode, self.accent, amount);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.left_bar.set_focused(focused);
        self.right_bar.set_focused(focused);
        self.search.set_focused(focused);
    }

    /// True while the Lens toolbar pills are on screen: return it
    /// from `App::wants_backdrop` for the blur pass (same rule as
    /// the toolbar demo).
    pub fn wants_backdrop(&self) -> bool {
        true
    }

    /// Animated column width: full while expanded, zero while
    /// collapsed, sliding between during the fade.
    fn bar_w(&self) -> f32 {
        self.eff_setting() * (1.0 - self.anim_p)
    }

    /// True while an edge resize drag runs.
    pub fn is_resizing(&self) -> bool {
        self.resizing
    }

    /// True when the pointer should show the column-resize cursor:
    /// over the edge while expanded, the strip while collapsed, or
    /// mid-drag. The app maps this to `CursorKind::ResizeColumn`.
    pub fn wants_resize_cursor(&self, x: f64, y: f64) -> bool {
        self.resizing || self.resize_hit(x as f32, y as f32)
    }

    /// Resize grab test: the column edge band while expanded, the
    /// content left strip while collapsed.
    fn resize_hit(&self, x: f32, y: f32) -> bool {
        if y < self.y || y > self.y + self.height {
            return false;
        }
        if self.collapsed {
            x >= self.x && x <= self.x + SIDEBAR_REOPEN_HIT
        } else {
            (x - (self.x + self.eff_setting())).abs() <= SIDEBAR_RESIZE_HIT
        }
    }

    /// Follow the pointer while resizing. Dragging past the minimum
    /// snaps shut; from collapsed, grabbing the strip reopens.
    fn drag_resize(&mut self, x: f32) {
        let min = self.min_bar_w();
        let w = x - self.x;
        if self.collapsed {
            self.width_setting = w.clamp(min, SIDEBAR_MAX_W);
            return;
        }
        if w < min - SIDEBAR_CLOSE_SLOP {
            self.resizing = false;
            self.toggle_sidebar();
            return;
        }
        self.width_setting = w.clamp(min, SIDEBAR_MAX_W);
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
    /// sidebar traffic band up to the pill group (pills moved into
    /// the traffic row, so clicks there must reach the app, never
    /// start a drag); collapsed the content band up to the
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
            // Pill group (toggle plus left pill) sits in this band:
            // end the drag where it starts, like collapsed does.
            let left_w = self.left_bar_w();
            let pills_w = SIDEBAR_PAD
                + self.right_bar_w()
                + if left_w > 0.0 {
                    left_w + TOOLBAR_GAP
                } else {
                    0.0
                };
            let end = self.x + self.bar_w() - pills_w;
            (
                self.x + cut,
                self.y,
                (end - self.x - cut).max(0.0),
                SIDEBAR_TRAFFIC_TOP + TRAFFIC_SIZE + 6.0,
            )
        }
    }

    pub fn set_hover(&mut self, x: f32, y: f32) {
        // Mid-drag the pointer drives the width, not hover states.
        if self.resizing {
            self.drag_resize(x);
            return;
        }
        self.resize_hover = self.resize_hit(x, y);
        self.traffic_hover = self.traffic_index(x, y);
        self.left_bar.set_hover(x, y);
        self.right_bar.set_hover(x, y);
        if self.show_search && !self.collapsed {
            self.search.set_hover(x, y);
        }
        if let Some(page) = self.active_page_mut() {
            page.set_hover(x, y);
        }
    }

    fn toolbar_cy_for(&self, collapsed: bool) -> f32 {
        if collapsed {
            self.y + (SIDEBAR_TOOLBAR_H - TOOLBAR_HEIGHT) / 2.0
        } else {
            self.y + SIDEBAR_BAR_TOP
        }
    }

    /// Left pill icon count (set slots only).
    fn left_count(&self) -> usize {
        self.left_slots.iter().flatten().count()
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

    fn title_x(&self, p: f32) -> f32 {
        // Title slides between content (expanded) and traffic
        // (collapsed) while the column fades.
        let expanded_x = self.content_x() + SIDEBAR_PAD;
        let collapsed_x = self.traffic_end() + SIDEBAR_PAD;
        expanded_x * (1.0 - p) + collapsed_x * p
    }

    /// Right edge x of the traffic cluster (pills start after it,
    /// never on top of the lights).
    fn traffic_end(&self) -> f32 {
        self.x + TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0
    }

    fn place_bars(&mut self, fonts: &mut FontSystem) {
        let collapsed = self.collapsed;
        self.place_bars_for(fonts, collapsed);
    }

    /// Place both pills for either end state (the crossfade draws
    /// both layouts mid-flight, then restores the target one for
    /// hit-testing).
    fn place_bars_for(&mut self, fonts: &mut FontSystem, collapsed: bool) {
        let cy = self.toolbar_cy_for(collapsed);
        if collapsed {
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
            // Sidebar top row: back pill after traffic, single
            // toggle at the right edge (grouped with a gap).
            if self.right_bar_w() > 0.0 {
                let right_x = self.x + self.bar_w() - SIDEBAR_PAD - self.right_bar_w();
                self.right_bar.place(fonts, right_x, cy, self.right_bar_w(), TOOLBAR_HEIGHT);
                if self.left_bar_w() > 0.0 {
                    let min_x = self.traffic_end() + SIDEBAR_PILL_GAP;
                    let bar_x = (right_x - TOOLBAR_GAP - self.left_bar_w()).max(min_x);
                    self.left_bar.place(fonts, bar_x, cy, self.left_bar_w(), TOOLBAR_HEIGHT);
                }
            } else if self.left_bar_w() > 0.0 {
                let bar_x = self.traffic_end() + SIDEBAR_PILL_GAP;
                self.left_bar.place(fonts, bar_x, cy, self.left_bar_w(), TOOLBAR_HEIGHT);
            }
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let (x32, y32) = (x as f32, y as f32);
        // Edge resize starts here and swallows the press (pills and
        // page never see it). From collapsed, grabbing the strip
        // reopens with the collapse animation in reverse.
        if self.resize_hit(x32, y32) {
            if self.collapsed {
                if !self.collapsible {
                    return;
                }
                self.toggle_sidebar();
            }
            self.resizing = true;
            return;
        }
        self.left_bar.mouse_down(x, y);
        self.right_bar.mouse_down(x, y);
        // Search row focuses on inside clicks, deselects outside
        // (so item clicks steal focus back).
        if self.show_search && !self.collapsed {
            self.search.mouse_down(x, y);
        }
        // Item rows (sidebar visible only, filtered by search).
        if !self.collapsed && x32 >= self.x && x32 <= self.x + self.bar_w() {
            let top = self.y + SIDEBAR_ITEMS_TOP;
            if y32 >= top {
                let row = ((y32 - top) / SIDEBAR_ROW_H).floor() as usize;
                if let Some(&index) = self.visible.get(row) {
                    self.select(index);
                }
            }
        }
        if let Some(page) = self.active_page_mut() {
            page.mouse_down(x, y);
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        // A resize drag ends here; nothing was armed below.
        if self.resizing {
            self.resizing = false;
            return;
        }
        self.left_bar.mouse_up(x, y);
        self.right_bar.mouse_up(x, y);
        // Pill clicks land in shared pending state (applied here, so
        // unit tests never need a draw in between).
        self.drain_pending();
        if self.show_search && !self.collapsed {
            self.search.mouse_up(x, y);
        }
        self.poll_search();
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

    /// Printable text for the search row while it holds focus,
    /// else the active page (the app forwards its `text` here).
    pub fn page_text(&mut self, text: &str) {
        if self.search_focused() {
            self.search.type_text(text);
            self.poll_search();
        } else if let Some(page) = self.active_page_mut() {
            page.text(text);
        }
    }

    /// Key handling for the search row while it holds focus, else
    /// the active page. Returns true when consumed.
    pub fn page_key(&mut self, key: Key) -> bool {
        if self.search_focused() {
            let consumed = self.search.key(key);
            self.poll_search();
            return consumed;
        }
        if let Some(page) = self.active_page_mut() {
            page.key(key)
        } else {
            false
        }
    }

    /// True while the search row is shown, expanded and selected.
    fn search_focused(&self) -> bool {
        self.show_search && !self.collapsed && self.search.is_selected()
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
        // Advance the collapse slide plus fade.
        self.update_progress(self.collapse_t0.elapsed().as_secs_f32());
        // Refilter on typed text (fires `on_search` on change).
        self.poll_search();
        let p = self.anim_p;
        let cross = p > 0.001 && p < 0.999;
        let bar_w = self.bar_w();
        // Sidebar body (full height, square: the shell rounds the window).
        if bar_w > 0.0 {
            if cross {
                // Shrinking column: clip to the live width and fade out.
                let scale = fonts.scale as f64;
                let clip = Rect::new(
                    self.x as f64 * scale,
                    self.y as f64 * scale,
                    (self.x + bar_w) as f64 * scale,
                    (self.y + self.height) as f64 * scale,
                );
                scene.push_layer(
                    Fill::NonZero,
                    BlendMode::default(),
                    (1.0 - p).clamp(0.0, 1.0),
                    Affine::IDENTITY,
                    &clip,
                );
                self.paint_body(scene, fonts, images, bar_w);
                scene.pop_layer();
            } else {
                self.paint_body(scene, fonts, images, bar_w);
            }
        }
        // Traffic on top of everything (sidebar top when expanded,
        // content top-left when collapsed).
        self.render_traffic(scene, fonts);
        self.render_toolbar(scene, fonts, images, p, cross);
        // Active page below the toolbar.
        let (px0, py0, pw, ph) = self.page_rect();
        if pw > 0.0 && ph > 0.0 {
            if let Some(page) = self.active_page_mut() {
                page.place(fonts, px0, py0, pw, ph);
                page.draw(scene, fonts, images);
            }
        }
    }

    /// Sidebar body paint: background, content divider and items.
    fn paint_body(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        bar_w: f32,
    ) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.sidebar_bg()),
            None,
            &Rect::new(px(self.x), px(self.y), px(self.x + bar_w), px(self.y + self.height)),
        );
        // Divider between sidebar and content: accent plus wider
        // while the resize edge hovers or drags.
        let dx = self.x + bar_w;
        let grabbing = self.resizing || self.resize_hover;
        let (divider_w, divider_c) = if grabbing {
            (2.0, self.eff(self.accent))
        } else {
            (1.0, self.divider_color())
        };
        scene.stroke(
            &Stroke::new(divider_w as f64 * scale),
            Affine::IDENTITY,
            &Brush::Solid(divider_c),
            None,
            &Line::new((px(dx), px(self.y)), (px(dx), px(self.y + self.height))),
        );
        self.render_items(scene, fonts, images);
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
        // Search row first (clipped plus faded with the body).
        if self.show_search {
            self.search.draw(scene, fonts, images);
        }
        for (row, &i) in self.visible.iter().enumerate() {
            let item = &self.items[i];
            let ry = self.y + SIDEBAR_ITEMS_TOP + row as f32 * SIDEBAR_ROW_H;
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

    fn render_toolbar(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        p: f32,
        cross: bool,
    ) {
        // Pending pill actions apply here too (clicks handled
        // between frames still land before the next paint).
        self.drain_pending();
        let title = self.effective_title();
        let layout = fonts.layout_text_weighted(&title, SIDEBAR_TITLE_SIZE, self.eff(self.text_color()), 600.0, None);
        let (_, th) = FontSystem::layout_size(&layout);
        draw_layout(
            scene,
            &layout,
            self.title_x(p),
            self.y + (SIDEBAR_TOOLBAR_H - th / fonts.scale) / 2.0,
            fonts.scale,
        );
        if cross && !images.is_capture_pass() {
            // Crossfade mid-flight: expanded pills out, collapsed
            // pills in, then restore the target layout so hit cells
            // match what the user sees at rest.
            let target = self.collapsed;
            let scale = fonts.scale as f64;
            let area = Rect::new(
                self.x as f64 * scale,
                self.y as f64 * scale,
                (self.x + self.width) as f64 * scale,
                (self.y + self.height) as f64 * scale,
            );
            self.place_bars_for(fonts, false);
            scene.push_layer(
                Fill::NonZero,
                BlendMode::default(),
                (1.0 - p).clamp(0.0, 1.0),
                Affine::IDENTITY,
                &area,
            );
            self.draw_pills(scene, fonts, images);
            scene.pop_layer();
            self.place_bars_for(fonts, true);
            scene.push_layer(
                Fill::NonZero,
                BlendMode::default(),
                p.clamp(0.0, 1.0),
                Affine::IDENTITY,
                &area,
            );
            self.draw_pills(scene, fonts, images);
            scene.pop_layer();
            self.place_bars_for(fonts, target);
        } else {
            self.draw_pills(scene, fonts, images);
        }
    }

    /// Draw both pills at their placed rects.
    fn draw_pills(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.left_bar_w() > 0.0 {
            self.left_bar.draw(scene, fonts, images);
        }
        if self.right_bar_w() > 0.0 {
            self.right_bar.draw(scene, fonts, images);
        }
    }

    /// Search row placement (skipped while hidden or collapsed).
    fn layout_search(&mut self, fonts: &mut FontSystem) {
        if !self.show_search || self.collapsed {
            return;
        }
        self.search.place(
            fonts,
            self.x + SIDEBAR_PAD,
            self.y + SIDEBAR_SEARCH_TOP,
            (self.eff_setting() - SIDEBAR_PAD * 2.0).max(0.0),
            SIDEBAR_SEARCH_H,
        );
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
        self.layout_search(fonts);
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
    fn collapse_sample_eases_out() {
        let (start, running) = Sidebar::collapse_sample(0.0, 1.0, 0.0);
        assert_eq!(start, 0.0);
        assert!(!running);
        // CubicOut rushes ahead: halfway through time means 7/8 done.
        let (mid, running) = Sidebar::collapse_sample(0.0, 1.0, SIDEBAR_COLLAPSE_SECONDS / 2.0);
        assert!((mid - 0.875).abs() < 1e-6);
        assert!(!running);
        let (end, done) =
            Sidebar::collapse_sample(0.0, 1.0, SIDEBAR_COLLAPSE_SECONDS + 1.0);
        assert_eq!(end, 1.0);
        assert!(done);
    }

    #[test]
    fn toggle_animates_width_and_fade() {
        let mut bar = bar();
        bar.toggle_sidebar();
        assert!(bar.is_collapsed());
        assert!(bar.is_animating());
        // Mid-flight: column shrinks, content follows.
        bar.update_progress(SIDEBAR_COLLAPSE_SECONDS / 2.0);
        assert!(bar.is_animating());
        let mid_w = bar.bar_w();
        assert!(mid_w > 0.0 && mid_w < SIDEBAR_W);
        assert_eq!(bar.page_rect().0, mid_w);
        // End: snapped shut, animation cleared.
        bar.update_progress(SIDEBAR_COLLAPSE_SECONDS + 1.0);
        assert!(!bar.is_animating());
        assert_eq!(bar.bar_w(), 0.0);
        // Expand reverses back to full width.
        bar.toggle_sidebar();
        assert!(!bar.is_collapsed());
        bar.update_progress(SIDEBAR_COLLAPSE_SECONDS + 1.0);
        assert_eq!(bar.bar_w(), SIDEBAR_W);
    }

    #[test]
    fn set_collapsed_snaps_without_animation() {
        let mut bar = bar();
        bar.set_collapsed(true);
        assert!(!bar.is_animating());
        assert_eq!(bar.bar_w(), 0.0);
        bar.set_collapsed(false);
        assert_eq!(bar.bar_w(), SIDEBAR_W);
    }

    #[test]
    fn min_width_fits_shown_pills() {
        // No slots: traffic plus margin, floored.
        assert_eq!(bar().min_bar_w(), SIDEBAR_MIN_W);
        // Both slots plus toggle: cluster, pills, gaps and inset.
        let full = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .left_button(0, "chevron.left", || {})
            .left_button(1, "magnifyingglass", || {});
        assert_eq!(full.min_bar_w(), 89.0 + 12.0 + 76.0 + 4.0 + 44.0 + 16.0);
        // Hidden toggle shrinks the minimum.
        let no_toggle = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .toggle_button(false)
            .left_button(0, "chevron.left", || {})
            .left_button(1, "magnifyingglass", || {});
        assert_eq!(no_toggle.min_bar_w(), 89.0 + 12.0 + 76.0 + 16.0);
    }

    #[test]
    fn width_builder_clamps_to_live_range() {
        let wide = Sidebar::new(vec![SidebarItem::new("G", "gear")]).width(1000.0);
        assert_eq!(wide.width_value(), SIDEBAR_MAX_W);
        let narrow = Sidebar::new(vec![SidebarItem::new("G", "gear")]).width(10.0);
        assert_eq!(narrow.width_value(), SIDEBAR_MIN_W);
    }

    #[test]
    fn edge_drag_resizes_live() {
        let mut bar = bar();
        bar.mouse_down(240.0, 300.0);
        assert!(bar.is_resizing());
        bar.set_hover(320.0, 300.0);
        assert_eq!(bar.bar_w(), 320.0);
        bar.mouse_up(320.0, 300.0);
        assert!(!bar.is_resizing());
        assert_eq!(bar.width_value(), 320.0);
    }

    #[test]
    fn edge_drag_clamps_to_max() {
        let mut bar = bar();
        bar.mouse_down(240.0, 300.0);
        bar.set_hover(800.0, 300.0);
        bar.mouse_up(800.0, 300.0);
        assert_eq!(bar.width_value(), SIDEBAR_MAX_W);
    }

    #[test]
    fn edge_drag_past_min_snaps_shut() {
        let mut bar = bar();
        bar.mouse_down(240.0, 300.0);
        // Minimum is 120 here; 50 is past the close slop.
        bar.set_hover(50.0, 300.0);
        assert!(bar.is_collapsed());
        assert!(bar.is_animating());
        assert!(!bar.is_resizing());
    }

    #[test]
    fn collapsed_strip_grab_reopens() {
        let mut bar = bar();
        bar.set_collapsed(true);
        bar.mouse_down(4.0, 300.0);
        assert!(bar.is_resizing());
        assert!(!bar.is_collapsed());
        bar.set_hover(300.0, 300.0);
        bar.mouse_up(300.0, 300.0);
        assert!(!bar.is_resizing());
        assert_eq!(bar.width_value(), 300.0);
    }

    #[test]
    fn resize_cursor_only_over_edge() {
        let open = bar();
        assert!(open.wants_resize_cursor(240.0, 300.0));
        assert!(open.wants_resize_cursor(245.0, 300.0));
        assert!(!open.wants_resize_cursor(100.0, 300.0));
        let mut shut = bar();
        shut.set_collapsed(true);
        assert!(shut.wants_resize_cursor(4.0, 300.0));
        assert!(!shut.wants_resize_cursor(100.0, 300.0));
    }

    #[test]
    fn search_row_shown_by_default() {
        let bar = bar();
        let (sx, sy, sw, sh) = bar.search.rect();
        assert_eq!((sx, sy), (SIDEBAR_PAD, SIDEBAR_SEARCH_TOP));
        assert_eq!((sw, sh), (SIDEBAR_W - SIDEBAR_PAD * 2.0, SIDEBAR_SEARCH_H));
    }

    #[test]
    fn search_filters_and_click_selects_real_index() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let capture = seen.clone();
        let mut bar = bar().on_search(move |text| capture.borrow_mut().push(text.to_string()));
        // Focus the row, type, filter down to Security.
        bar.mouse_down(120.0, (SIDEBAR_SEARCH_TOP + SIDEBAR_SEARCH_H / 2.0) as f64);
        assert!(bar.search.is_selected());
        bar.page_text("sec");
        assert_eq!(bar.search_text(), "sec");
        assert_eq!(seen.borrow().as_slice(), ["sec"]);
        // First visible row is now Security (real index 1).
        bar.mouse_down(100.0, (SIDEBAR_ITEMS_TOP + 10.0) as f64);
        bar.mouse_up(100.0, (SIDEBAR_ITEMS_TOP + 10.0) as f64);
        assert_eq!(bar.selected_index(), 1);
        // Refocus and clear: Backspace deletes, then both rows return.
        bar.mouse_down(120.0, (SIDEBAR_SEARCH_TOP + SIDEBAR_SEARCH_H / 2.0) as f64);
        bar.page_key(Key::Backspace);
        bar.page_key(Key::Backspace);
        bar.page_key(Key::Backspace);
        assert_eq!(bar.search_text(), "");
        bar.mouse_down(100.0, (SIDEBAR_ITEMS_TOP + 10.0) as f64);
        bar.mouse_up(100.0, (SIDEBAR_ITEMS_TOP + 10.0) as f64);
        assert_eq!(bar.selected_index(), 0);
    }

    #[test]
    fn typing_reaches_page_without_search_focus() {
        let mut bar = bar();
        // Never focused: typing goes to the page, filter untouched.
        assert!(!bar.search.is_selected());
        bar.page_text("sec");
        assert_eq!(bar.search_text(), "");
        // Clicking an item steals focus back.
        bar.mouse_down(120.0, (SIDEBAR_SEARCH_TOP + SIDEBAR_SEARCH_H / 2.0) as f64);
        assert!(bar.search.is_selected());
        bar.mouse_down(100.0, (SIDEBAR_ITEMS_TOP + 10.0) as f64);
        bar.mouse_up(100.0, (SIDEBAR_ITEMS_TOP + 10.0) as f64);
        assert!(!bar.search.is_selected());
    }

    #[test]
    fn hidden_search_skips_row() {
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")]).search_field(false);
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        bar.mouse_down(120.0, (SIDEBAR_SEARCH_TOP + SIDEBAR_SEARCH_H / 2.0) as f64);
        bar.page_text("x");
        assert_eq!(bar.search_text(), "");
        assert!(!bar.search_text_cursor());
    }

    /// Center of a pill cell at `(bar_x, bar_y)`: leading layout,
    /// pad plus hit boxes with gaps, vertically centered.
    fn pill_cell(bar_x: f32, bar_y: f32, index: usize) -> (f64, f64) {
        let cx = bar_x + TOOLBAR_PAD_X + index as f32 * (TOOLBAR_HIT + TOOLBAR_GAP)
            + TOOLBAR_HIT / 2.0;
        let cy = bar_y + (TOOLBAR_HEIGHT - TOOLBAR_HIT) / 2.0 + TOOLBAR_HIT / 2.0;
        (cx as f64, cy as f64)
    }

    /// Toggle pill x in the expanded sidebar (single pill far right).
    fn toggle_bar_x() -> f32 {
        SIDEBAR_W - SIDEBAR_PAD - (TOOLBAR_PAD_X * 2.0 + TOOLBAR_HIT)
    }

    /// Toggle cell in the expanded sidebar (single pill far right).
    fn toggle_cell() -> (f64, f64) {
        pill_cell(toggle_bar_x(), SIDEBAR_BAR_TOP, 0)
    }

    /// Left pill x in the expanded sidebar (grouped left of toggle,
    /// never on top of traffic).
    fn left_bar_x(bar: &Sidebar) -> f32 {
        let min_x = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0 + SIDEBAR_PILL_GAP;
        (toggle_bar_x() - TOOLBAR_GAP - bar.left_bar_w()).max(min_x)
    }

    /// Back cell in the expanded sidebar (left pill first icon).
    fn back_cell(bar: &Sidebar) -> (f64, f64) {
        pill_cell(left_bar_x(bar), SIDEBAR_BAR_TOP, 0)
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
    fn hidden_toggle_keeps_slot_first() {
        use std::cell::Cell;
        use std::rc::Rc;

        let count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let taps = count.clone();
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .toggle_button(false)
            .left_button(0, "chevron.left", move || taps.set(taps.get() + 1));
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        // Slot 0 is the first left-pill icon (toggle lives far right).
        // Toggle hidden: the pill sits right after traffic.
        let min_x = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0 + SIDEBAR_PILL_GAP;
        let (bx, by) = pill_cell(min_x, SIDEBAR_BAR_TOP, 0);
        bar.mouse_down(bx, by);
        bar.mouse_up(bx, by);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn empty_left_pill_is_skipped() {
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")]).toggle_button(false);
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        assert_eq!(bar.left_bar_w(), 0.0);
        // Click where the pill would sit: nothing happens.
        bar.mouse_down(38.0, SIDEBAR_BAR_TOP as f64 + 18.0);
        bar.mouse_up(38.0, SIDEBAR_BAR_TOP as f64 + 18.0);
        assert!(!bar.is_collapsed());
        assert_eq!(bar.selected_index(), 0);
    }

    #[test]
    fn slot_button_fires_and_clears() {
        use std::cell::Cell;
        use std::rc::Rc;

        let count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let taps = count.clone();
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .left_button(0, "chevron.left", move || taps.set(taps.get() + 1));
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        let (bx, by) = back_cell(&bar);
        bar.mouse_down(bx, by);
        bar.mouse_up(bx, by);
        assert_eq!(count.get(), 1);
        // Cleared slot never fires (left pill stays empty there).
        bar.clear_left_button(0);
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        assert_eq!(bar.left_bar_w(), 0.0);
        bar.mouse_down(bx, by);
        bar.mouse_up(bx, by);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn out_of_range_slot_is_rejected() {
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")]);
        assert!(!bar.set_left_button(2, "x", || {}));
        bar.clear_left_button(7);
        assert_eq!(bar.left_bar_w(), 0.0);
    }

    #[test]
    fn second_slot_fires_its_callback() {
        use std::cell::Cell;
        use std::rc::Rc;

        let count: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let taps = count.clone();
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .left_button(0, "chevron.left", || {})
            .left_button(1, "magnifyingglass", move || taps.set(taps.get() + 1));
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        // Slot 1 is the second icon in the left pill.
        let dev_x = left_bar_x(&bar);
        let (bx, by) = pill_cell(dev_x, SIDEBAR_BAR_TOP, 1);
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
        // Drag ends where the pill group starts (toggle shown).
        let pills_w = SIDEBAR_PAD + bar.right_bar_w();
        assert_eq!(dw, SIDEBAR_W - pills_w - cluster_end);
        assert_eq!((dy, dh), (0.0, SIDEBAR_TRAFFIC_TOP + TRAFFIC_SIZE + 6.0));
    }

    /// Regression: pills live in the traffic band, so the expanded
    /// drag rect must not cover them, or `window.rs` steals the
    /// press as a window-drag and the toolbar never arms.
    #[test]
    fn drag_rect_leaves_pills_clickable() {
        let mut bar = Sidebar::new(vec![SidebarItem::new("G", "gear")])
            .left_button(0, "chevron.left", || {})
            .left_button(1, "magnifyingglass", || {});
        bar.place(&mut FontSystem::new(), 0.0, 0.0, 900.0, 600.0);
        let (dx, dy, dw, dh) = bar.drag_rect();
        let outside = |x: f64, y: f64| {
            x < dx as f64
                || x > (dx + dw) as f64
                || y < dy as f64
                || y > (dh + dy) as f64
        };
        let (tx, ty) = toggle_cell();
        assert!(outside(tx, ty), "toggle inside drag rect");
        let (bx, by) = back_cell(&bar);
        assert!(outside(bx, by), "slot 0 inside drag rect");
        let dev_x = left_bar_x(&bar);
        let (sx, sy) = pill_cell(dev_x, SIDEBAR_BAR_TOP, 1);
        assert!(outside(sx, sy), "slot 1 inside drag rect");
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
