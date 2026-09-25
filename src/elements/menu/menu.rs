use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::{BUTTON_BG_DARK, BUTTON_BG_LIGHT};
use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Menu button height in logical px.
pub const MENU_BUTTON_H: f32 = 24.0;
/// Menu button corner radius in logical px.
pub const MENU_BUTTON_RADIUS: f32 = 6.0;
/// Button, row and leading label size in logical px.
pub const MENU_FONT_SIZE: f32 = 13.0;
/// Gap between the leading label and the button in logical px.
pub const MENU_GAP: f32 = 9.0;
/// Horizontal text padding inside the button in logical px.
pub const MENU_BTN_PAD_X: f32 = 10.0;
/// Chevron box width inside the button in logical px.
pub const MENU_CHEV_W: f32 = 12.0;
/// Gap between button text and chevron in logical px.
pub const MENU_CHEV_GAP: f32 = 8.0;
/// Menu panel padding on every side in logical px.
pub const MENU_PAD: f32 = 6.0;
/// Menu row height in logical px.
pub const MENU_ROW_H: f32 = 26.0;
/// Vertical gap between menu rows in logical px.
pub const MENU_ROW_SPACING: f32 = 2.0;
/// Menu panel corner radius in logical px (round, like the reference).
pub const MENU_RADIUS: f32 = 9.0;
/// Column reserved for the checkmark in logical px.
pub const MENU_CHECK_COL: f32 = 20.0;
/// Gap between checkmark column and row text in logical px.
pub const MENU_TEXT_GAP: f32 = 6.0;
/// Gap between button and menu panel in logical px.
pub const MENU_PANEL_GAP: f32 = 4.0;
/// Checkmark glyph width in logical px (fixed, independent of size).
pub const MENU_CHECK_W: f32 = 10.0;
/// Checkmark glyph height in logical px (fixed, independent of size).
pub const MENU_CHECK_H: f32 = 7.5;
/// Menu edge shadow blur in logical px (heavy, like the reference).
pub const MENU_SHADOW_BLUR: f32 = 24.0;
/// Default hover fill (theme accent blue).
pub const MENU_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Button chevron style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuChevron {
    /// Single down chevron, like the simple dropdown reference.
    #[default]
    Down,
    /// Up/down chevron pair, like the macOS pop-up button.
    Both,
}

/// Simple dropdown menu: a button with fixed text plus a chevron
/// that opens a frosted glass panel. Rows are action buttons: every
/// row click fires `on_action` (no selection state). An optional
/// checkmark row (`checked`) and leading label exist for picker
/// wrappers reusing this base.
///
/// The closed button has no hover state, like macOS. The menu panel
/// uses the `Frosted` glass finish (no clear background) with a heavy
/// edge shadow, and it is always clamped into the viewport passed via
/// `set_viewport`: glass samples the in-app backdrop, so the panel
/// must never leave the window. Apps must call `set_viewport` every
/// frame (see `examples/menu.rs`) and return `is_open()` from
/// `App::wants_backdrop` so the shell runs the blur pass.
pub struct Menu {
    label: String,
    button: String,
    anchor: Option<(f32, f32)>,
    chevron: MenuChevron,
    options: Vec<String>,
    open: bool,
    checked: Option<usize>,
    last_action: Option<usize>,
    pub(crate) hover: Color,
    hover_manual: bool,
    dark: bool,
    text_color: Color,
    text_dim: Color,
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
    menu_x: f32,
    menu_y: f32,
    menu_w: f32,
    menu_h: f32,
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
    armed_button: bool,
    armed_row: Option<usize>,
    pub(crate) hovered: Option<usize>,
    disabled: bool,
    focused: bool,
    on_action: Option<Box<dyn FnMut(usize)>>,
}

impl Menu {
    pub fn new(button: impl Into<String>, options: Vec<String>) -> Self {
        let mut glass = GlassContainer::new();
        glass.set_glass_type(GlassType::Frosted);
        Self {
            label: String::new(),
            button: button.into(),
            anchor: None,
            chevron: MenuChevron::Down,
            options,
            open: false,
            checked: None,
            last_action: None,
            hover: MENU_ACCENT,
            hover_manual: false,
            dark: true,
            text_color: Color::WHITE,
            text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
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
            menu_x: 0.0,
            menu_y: 0.0,
            menu_w: 0.0,
            menu_h: 0.0,
            vp_x: 0.0,
            vp_y: 0.0,
            vp_w: f32::MAX,
            vp_h: f32::MAX,
            armed_button: false,
            armed_row: None,
            hovered: None,
            disabled: false,
            focused: true,
            on_action: None,
        }
    }

    /// Convenience constructor from a slice (e.g. `["Option 1", "Option 2"]`).
    pub fn from_slice(button: impl Into<String>, options: &[&str]) -> Self {
        Self::new(
            button,
            options.iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Optional leading label before the button (empty by default).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Button chevron style (down chevron by default).
    pub fn chevron(mut self, chevron: MenuChevron) -> Self {
        self.chevron = chevron;
        self
    }

    /// Checkmark row, if any (plain action rows by default).
    pub fn checked(mut self, row: Option<usize>) -> Self {
        self.checked = row;
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

    /// Fires on every row click (press plus release on the same row).
    pub fn on_action(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_action = Some(Box::new(callback));
        self
    }

    /// Live theme: hover fill plus mode grays and label colors. A
    /// manually set hover wins over the system accent.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.hover_manual {
            self.hover = accent;
        }
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.text_dim = Color::from_rgb8(0x9a, 0x9a, 0x9e);
        } else {
            self.text_color = Color::BLACK;
            self.text_dim = Color::from_rgb8(0x6e, 0x6e, 0x72);
        }
    }

    /// Glass stage for the menu panel (frost tint per setting).
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

    /// Fixed button text (stays put on row clicks, unlike a picker).
    pub fn set_button(&mut self, button: impl Into<String>) {
        self.button = button.into();
    }

    pub fn button_text(&self) -> &str {
        &self.button
    }

    pub fn set_chevron(&mut self, chevron: MenuChevron) {
        self.chevron = chevron;
    }

    pub fn set_checked(&mut self, row: Option<usize>) {
        self.checked = row;
    }

    /// Float the menu at a point instead of its button (context
    /// mode): the button hides, takes no layout space and never hits,
    /// while the panel anchors at the point. `None` restores the
    /// button-anchored dropdown.
    pub fn set_anchor(&mut self, point: Option<(f32, f32)>) {
        self.anchor = point;
    }

    /// Window bounds the menu is clamped into. Apps must call this
    /// every frame with the current viewport; without it the panel
    /// cannot be kept inside the window.
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
        if !self.disabled && !self.options.is_empty() {
            self.open = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.armed_button = false;
        self.armed_row = None;
        self.hovered = None;
    }

    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    pub fn option(&self, index: usize) -> Option<&str> {
        self.options.get(index).map(|s| s.as_str())
    }

    /// Last clicked row, if any (also readable without a callback).
    pub fn last_action(&self) -> Option<usize> {
        self.last_action
    }

    /// Button rect (x, y, width, height) after layout.
    pub fn button_rect(&self) -> (f32, f32, f32, f32) {
        (self.btn_x, self.btn_y, self.btn_w, MENU_BUTTON_H)
    }

    /// Menu panel rect (x, y, width, height) after layout.
    pub fn menu_rect(&self) -> (f32, f32, f32, f32) {
        (self.menu_x, self.menu_y, self.menu_w, self.menu_h)
    }

    /// Row content rect (x, y, width, height) after layout, if the row
    /// exists. Independent of open state (hit testing still needs it).
    pub fn row_rect(&self, row: usize) -> Option<(f32, f32, f32, f32)> {
        if row >= self.options.len() {
            return None;
        }
        Some((
            self.menu_x,
            self.menu_y + MENU_PAD + row as f32 * self.row_stride(),
            self.menu_w,
            MENU_ROW_H,
        ))
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
        for option in &self.options {
            widest = widest.max(self.text_w(fonts, option));
        }
        // Wide enough for the longest row: opening the menu must not
        // resize the button.
        widest + MENU_BTN_PAD_X * 2.0 + MENU_CHEV_GAP + MENU_CHEV_W
    }

    fn row_stride(&self) -> f32 {
        MENU_ROW_H + MENU_ROW_SPACING
    }

    /// Left indent for the check column: only reserved when a
    /// checkmark exists, otherwise row text starts at the panel
    /// padding.
    fn check_indent(&self) -> f32 {
        if self.checked.is_some() {
            MENU_CHECK_COL + MENU_TEXT_GAP
        } else {
            0.0
        }
    }

    fn row_y(&self, row: usize) -> f32 {
        self.menu_y + MENU_PAD + row as f32 * self.row_stride()
    }

    /// Menu panel rect, clamped into the viewport so the glass never
    /// samples outside the window. Prefers below the button, falls
    /// back above it, then clamps.
    fn layout_menu(&mut self, fonts: &mut FontSystem) {
        if self.options.is_empty() {
            self.menu_w = 0.0;
            self.menu_h = 0.0;
            return;
        }
        let mut content_w: f32 = 0.0;
        for option in &self.options {
            content_w = content_w.max(self.check_indent() + self.text_w(fonts, option));
        }
        let mut w = (content_w + MENU_PAD * 2.0).max(self.btn_w);
        let mut h = MENU_PAD * 2.0
            + self.options.len() as f32 * MENU_ROW_H
            + (self.options.len() as f32 - 1.0) * MENU_ROW_SPACING;
        // Never larger than the window itself.
        w = w.min(self.vp_w).max(0.0);
        h = h.min(self.vp_h).max(0.0);
        let mut x = self.btn_x;
        if x + w > self.vp_x + self.vp_w {
            x = self.vp_x + self.vp_w - w;
        }
        if x < self.vp_x {
            x = self.vp_x;
        }
        let below = self.btn_y + MENU_BUTTON_H + MENU_PANEL_GAP;
        let mut y = below;
        if y + h > self.vp_y + self.vp_h {
            y = self.btn_y - MENU_PANEL_GAP - h;
        }
        if y < self.vp_y {
            y = self.vp_y;
        }
        self.menu_x = x;
        self.menu_y = y;
        self.menu_w = w;
        self.menu_h = h;
        self.glass.set_bounds(x, y, w, h);
        self.glass.set_radius(MENU_RADIUS);
    }

    fn button_hit(&self, x: f32, y: f32) -> bool {
        if self.anchor.is_some() {
            return false;
        }
        x >= self.btn_x
            && x <= self.btn_x + self.btn_w
            && y >= self.btn_y
            && y <= self.btn_y + MENU_BUTTON_H
    }

    fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.open || self.options.is_empty() {
            return None;
        }
        if x < self.menu_x
            || x > self.menu_x + self.menu_w
            || y < self.menu_y
            || y > self.menu_y + self.menu_h
        {
            return None;
        }
        let rel = y - (self.menu_y + MENU_PAD);
        if rel < 0.0 {
            return None;
        }
        let row = (rel / self.row_stride()).floor() as usize;
        if row >= self.options.len() {
            return None;
        }
        // Ignore hits in padding and row gaps.
        let within = rel - row as f32 * self.row_stride();
        if within > MENU_ROW_H {
            return None;
        }
        Some(row)
    }

    fn notify(&mut self, row: usize) {
        self.last_action = Some(row);
        if let Some(callback) = self.on_action.as_mut() {
            callback(row);
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
        // Open menu: arm the row under the pointer, if any. A press
        // anywhere else closes the menu on release.
        self.armed_button = false;
        self.armed_row = self.row_at(x, y);
    }

    /// Hover only exists inside the open menu; the closed button
    /// never highlights, like macOS.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if self.open {
            self.hovered = self.row_at(x as f32, y as f32);
        } else {
            self.hovered = None;
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    /// Scrolling is not needed: panels cap at the window size.
    pub fn mouse_wheel(&mut self, _dx: f64, _dy: f64) {}

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
        let armed = self.armed_row.take();
        self.armed_button = false;
        if let Some(row) = armed {
            // Press + release on the same row fires and closes.
            if self.row_at(x, y) == Some(row) {
                self.notify(row);
                self.close();
                return;
            }
        }
        // Any other release (button toggle, outside click) closes.
        self.close();
    }

    fn draw_check(&self, scene: &mut Scene, x: f32, y: f32, scale: f32, color: Color) {
        // Fixed-size checkmark, independent of the menu size.
        let px = |v: f32| v as f64 * scale as f64;
        let mut path = BezPath::new();
        path.move_to((px(x), px(y + MENU_CHECK_H * 0.55)));
        path.line_to((px(x + MENU_CHECK_W * 0.38), px(y + MENU_CHECK_H)));
        path.line_to((px(x + MENU_CHECK_W), px(y)));
        let mut stroke = Stroke::new(2.0 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(
            &stroke,
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &path,
        );
    }

    fn draw_chevron_down(&self, scene: &mut Scene, x: f32, cy: f32, scale: f32, color: Color) {
        // Single down chevron, like the simple dropdown reference.
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
        scene.stroke(
            &stroke,
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &path,
        );
    }

    fn draw_chevron_both(&self, scene: &mut Scene, x: f32, cy: f32, scale: f32, color: Color) {
        // Up/down chevron pair, like the macOS pop-up button.
        let px = |v: f32| v as f64 * scale as f64;
        let w = 7.0;
        let h = 4.0;
        let gap = 2.0;
        let cx = x + MENU_CHEV_W / 2.0;
        let mut up = BezPath::new();
        up.move_to((px(cx - w / 2.0), px(cy - gap / 2.0)));
        up.line_to((px(cx), px(cy - gap / 2.0 - h)));
        up.line_to((px(cx + w / 2.0), px(cy - gap / 2.0)));
        let mut down = BezPath::new();
        down.move_to((px(cx - w / 2.0), px(cy + gap / 2.0)));
        down.line_to((px(cx), px(cy + gap / 2.0 + h)));
        down.line_to((px(cx + w / 2.0), px(cy + gap / 2.0)));
        let mut stroke = Stroke::new(1.5 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        for path in [&up, &down] {
            scene.stroke(
                &stroke,
                Affine::IDENTITY,
                &Brush::Solid(color),
                None,
                path,
            );
        }
    }
}

impl View for Menu {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        // Anchored context menus float above content and take no
        // layout space.
        if self.anchor.is_some() {
            return (0.0, 0.0);
        }
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
        if let Some((ax, ay)) = self.anchor {
            self.btn_x = ax;
            self.btn_y = ay;
            self.btn_w = self.button_w(fonts);
            self.layout_menu(fonts);
            return;
        }
        let (label_w, label_h) = self.label_size(fonts);
        self.label_x = x;
        self.label_y = y + (h - label_h) / 2.0;
        self.btn_x = if label_w > 0.0 { x + label_w + MENU_GAP } else { x };
        self.btn_y = y + (h - MENU_BUTTON_H) / 2.0;
        self.btn_w = self.button_w(fonts).min((w - (self.btn_x - x)).max(0.0));
        self.layout_menu(fonts);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        // Capture pass: the glass skips itself so the blur sees only
        // what sits behind the menu; rows must skip too.
        if images.is_capture_pass() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        if self.options.is_empty() {
            return;
        }
        // Viewport may change between frames: re-clamp every draw.
        self.layout_menu(fonts);

        if self.anchor.is_none() {
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
        }

        // Button with fixed text (no hover state). Hidden in anchor
        // mode, where the panel floats at the anchor point.
        if self.anchor.is_none() {
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
            let chev_x = self.btn_x + self.btn_w - MENU_BTN_PAD_X - MENU_CHEV_W;
            let chev_y = self.btn_y + MENU_BUTTON_H / 2.0;
            match self.chevron {
                MenuChevron::Down => self.draw_chevron_down(
                    scene,
                    chev_x,
                    chev_y,
                    fonts.scale,
                    self.eff(self.text_dim),
                ),
                MenuChevron::Both => self.draw_chevron_both(
                    scene,
                    chev_x,
                    chev_y,
                    fonts.scale,
                    self.eff(self.text_dim),
                ),
            }
        }

        if !self.open {
            return;
        }

        // Heavy edge shadow, then the frosted glass panel (opaque
        // finish, no clear background).
        let rect = vello::kurbo::Rect::new(
            px(self.menu_x),
            px(self.menu_y),
            px(self.menu_x + self.menu_w),
            px(self.menu_y + self.menu_h),
        );
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            rect,
            Color::from_rgba8(0, 0, 0, 70),
            px(MENU_RADIUS),
            MENU_SHADOW_BLUR as f64 * scale,
        );
        self.glass.draw(scene, fonts, images);

        // Rows on top of the glass, clipped to the panel so text
        // never spills outside it (e.g. when the panel is clamped
        // into a small viewport).
        let clip = vello::kurbo::Rect::new(
            px(self.menu_x),
            px(self.menu_y + MENU_PAD),
            px(self.menu_x + self.menu_w),
            px(self.menu_y + self.menu_h - MENU_PAD),
        );
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
        for (i, option) in self.options.clone().iter().enumerate() {
            let ry = self.row_y(i);
            let is_hovered = Some(i) == self.hovered && !self.disabled;
            if is_hovered {
                let hl = RoundedRect::new(
                    px(self.menu_x + MENU_PAD / 2.0),
                    px(ry),
                    px(self.menu_x + self.menu_w - MENU_PAD / 2.0),
                    px(ry + MENU_ROW_H),
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
            let text_x = self.menu_x + MENU_PAD + self.check_indent();
            if self.checked == Some(i) {
                self.draw_check(
                    scene,
                    self.menu_x + MENU_PAD + (MENU_CHECK_COL - MENU_CHECK_W) / 2.0,
                    ry + (MENU_ROW_H - MENU_CHECK_H) / 2.0,
                    fonts.scale,
                    self.eff(color),
                );
            }
            let layout = fonts.layout_text_weighted(
                option,
                MENU_FONT_SIZE,
                self.eff(color),
                400.0,
                None,
            );
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                text_x,
                ry + (MENU_ROW_H - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }
        scene.pop_layer();
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

    fn menu() -> Menu {
        Menu::from_slice("Options", &["Option 1", "Option 2", "Option 3"])
    }

    fn placed() -> (Menu, FontSystem) {
        let mut m = menu();
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 0.0, 0.0, w, h);
        (m, fonts)
    }

    fn button_center(m: &Menu) -> (f64, f64) {
        let (x, y, w, _) = m.button_rect();
        ((x + w / 2.0) as f64, (y + MENU_BUTTON_H / 2.0) as f64)
    }

    fn row_center(m: &Menu, row: usize) -> (f64, f64) {
        let (x, y, w, h) = m.row_rect(row).expect("row exists");
        ((x + w / 2.0) as f64, (y + h / 2.0) as f64)
    }

    #[test]
    fn button_text_stays_fixed() {
        let (mut m, _) = placed();
        assert_eq!(m.button_text(), "Options");
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        assert!(m.is_open());
        let (rx, ry) = row_center(&m, 1);
        m.mouse_down(rx, ry);
        m.mouse_up(rx, ry);
        // Action rows never rewrite the button text.
        assert_eq!(m.button_text(), "Options");
        assert_eq!(m.last_action(), Some(1));
        assert!(!m.is_open());
    }

    #[test]
    fn action_fires_every_click() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut m = menu().on_action(move |_| count.set(count.get() + 1));
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 0.0, 0.0, w, h);
        // Release without press does nothing.
        m.mouse_up(1000.0, 1000.0);
        assert_eq!(fires.get(), 0);
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        let (rx, ry) = row_center(&m, 0);
        m.mouse_down(rx, ry);
        m.mouse_up(rx, ry);
        assert_eq!(fires.get(), 1);
        // Same row again fires again (no selection state).
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        m.mouse_down(rx, ry);
        m.mouse_up(rx, ry);
        assert_eq!(fires.get(), 2);
    }

    #[test]
    fn closed_button_has_no_hover() {
        let (mut m, _) = placed();
        let (bx, by) = button_center(&m);
        m.mouse_move(bx, by);
        assert_eq!(m.hovered, None);
    }

    #[test]
    fn outside_click_closes() {
        let (mut m, _) = placed();
        let (bx, by) = button_center(&m);
        m.mouse_down(bx, by);
        m.mouse_up(bx, by);
        assert!(m.is_open());
        m.mouse_down(700.0, 500.0);
        m.mouse_up(700.0, 500.0);
        assert!(!m.is_open());
        assert_eq!(m.last_action(), None);
    }

    #[test]
    fn menu_stays_inside_viewport() {
        let mut m = menu();
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 200.0, 120.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 150.0, 90.0, w, h);
        let (x, y, mw, mh) = (m.menu_x, m.menu_y, m.menu_w, m.menu_h);
        assert!(x >= 0.0 && y >= 0.0 && x + mw <= 200.0 && y + mh <= 120.0);
    }

    #[test]
    fn check_column_only_when_checked() {
        // No icons, no indent: plain rows start at the panel padding.
        assert_eq!(menu().check_indent(), 0.0);
        assert_eq!(
            menu().checked(Some(0)).check_indent(),
            MENU_CHECK_COL + MENU_TEXT_GAP
        );
    }

    #[test]
    fn checked_row_marks_without_state() {
        let mut m = menu().checked(Some(2));
        assert_eq!(m.checked, Some(2));
        m.set_checked(None);
        assert_eq!(m.checked, None);
    }

    #[test]
    fn manual_hover_wins_over_theme_accent() {
        let mut m = Menu::from_slice("Options", &["A"]).hover_fill(Color::from_rgb8(0x34, 0xc7, 0x59));
        m.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(m.hover, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = Menu::from_slice("Options", &["A"]);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.hover, Color::from_rgb8(0xff, 0x2d, 0x55));
    }

    #[test]
    fn empty_options_never_open() {
        let mut m = Menu::new("Options", Vec::new());
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (w, h) = m.measure(&mut fonts);
        m.place(&mut fonts, 0.0, 0.0, w.max(1.0), h.max(1.0));
        m.mouse_down(2.0, 2.0);
        m.mouse_up(2.0, 2.0);
        assert!(!m.is_open());
    }

    #[test]
    fn anchor_floats_without_button() {
        let mut m = menu();
        let mut fonts = FontSystem::new();
        m.set_viewport(0.0, 0.0, 800.0, 600.0);
        m.set_anchor(Some((400.0, 300.0)));
        // Anchored menus take no layout space.
        assert_eq!(m.measure(&mut fonts), (0.0, 0.0));
        m.place(&mut fonts, 0.0, 0.0, 0.0, 0.0);
        // Button never hits in anchor mode.
        m.mouse_down(400.0, 300.0);
        m.mouse_up(400.0, 300.0);
        assert!(!m.is_open());
        // Programmatic open anchors the panel at the point (below
        // it, like under a button) and inside the viewport.
        m.open();
        m.place(&mut fonts, 0.0, 0.0, 0.0, 0.0);
        let (x, y, w, h) = m.menu_rect();
        assert!(x <= 400.0 && 400.0 <= x + w);
        assert!(y >= 300.0 || y + h <= 300.0);
        assert!(x >= 0.0 && y >= 0.0 && x + w <= 800.0 && y + h <= 600.0);
        m.set_anchor(None);
        assert_ne!(m.measure(&mut fonts), (0.0, 0.0));
    }

    #[test]
    fn chevron_defaults_down() {
        assert_eq!(menu().chevron, MenuChevron::Down);
        let both = menu().chevron(MenuChevron::Both);
        assert_eq!(both.chevron, MenuChevron::Both);
    }
}
