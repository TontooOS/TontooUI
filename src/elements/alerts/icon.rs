use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use parley::Layout;
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{BlendMode, Brush, Color, Fill};

use super::super::buttons::{Button, ButtonShape, ButtonStyle};
use super::super::glass::{GlassContainer, GlassType};
use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::action::AlertEvent;
use super::basic::{AlertAction, AlertButton, AlertState};
use super::{
    ALERT_BUTTON_GAP, ALERT_BUTTON_H, ALERT_CANCEL, ALERT_DIM_ALPHA,
    ALERT_FADE_SECONDS, ALERT_ICON_GAP, ALERT_ICON_SIZE, ALERT_MESSAGE_DARK,
    ALERT_MESSAGE_GAP, ALERT_MESSAGE_LIGHT, ALERT_MESSAGE_SIZE, ALERT_PAD,
    ALERT_RADIUS, ALERT_TITLE_DARK, ALERT_TITLE_GAP, ALERT_TITLE_LIGHT,
    ALERT_TITLE_SIZE, ALERT_WIDTH,
};
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::theme::{GlassAmount, ThemeMode};

/// Alert with an SF icon: like the other alerts (frosted card, dimmed
/// backdrop, engine fade, modal) but with an SF Symbol on the left, a
/// leading-aligned title plus message on its right, and any number of
/// full-width action buttons stacked below. A press returns the
/// button as an `AlertEvent` (stack index, action, label); clicks
/// outside or mid-fade are swallowed.
pub struct IconAlert {
    icon: String,
    icon_color: Option<Color>,
    symbol: SFSymbolImage,
    title: String,
    message: String,
    defs: Vec<AlertButton>,
    buttons: Vec<Button>,
    fired: Rc<RefCell<Option<usize>>>,
    glass: GlassContainer,
    state: AlertState,
    opacity: f32,
    anim: Option<TweenAnim<f32>>,
    t0: Instant,
    dark: bool,
    focused: bool,
    text_color: Color,
    vx: f32,
    vy: f32,
    vw: f32,
    vh: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
    title_layout: Option<Layout<SolidBrush>>,
    message_layout: Option<Layout<SolidBrush>>,
    layout_scale: f32,
    dirty: bool,
}

impl IconAlert {
    /// Icon alert with `actions` stacked below the icon/text row (at
    /// least one; empty falls back to a single OK). A trailing Cancel
    /// button is added by default; rename it with `cancel` or drop it
    /// with `no_cancel`.
    pub fn new(
        icon: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        actions: Vec<AlertButton>,
    ) -> Self {
        let mut defs = actions;
        if defs.is_empty() {
            defs.push(AlertButton::ok("OK"));
        }
        defs.push(AlertButton::cancel("Cancel"));
        Self::with_defs(icon.into(), title.into(), message.into(), defs)
    }

    fn with_defs(
        icon: String,
        title: String,
        message: String,
        defs: Vec<AlertButton>,
    ) -> Self {
        let fired: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let mut alert = Self {
            icon: icon.clone(),
            icon_color: None,
            symbol: SFSymbolImage::new(icon).size(ALERT_ICON_SIZE),
            title,
            message,
            defs,
            buttons: Vec::new(),
            fired,
            glass: GlassContainer::new().glass_type(GlassType::Frosted),
            state: AlertState::Hidden,
            opacity: 0.0,
            anim: None,
            t0: Instant::now(),
            dark: true,
            focused: true,
            text_color: ALERT_TITLE_DARK,
            vx: 0.0,
            vy: 0.0,
            vw: 0.0,
            vh: 0.0,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
            title_layout: None,
            message_layout: None,
            layout_scale: 0.0,
            dirty: true,
        };
        alert.rebuild_buttons();
        alert
    }

    /// Optional icon tint. Without it the glyph follows the theme
    /// text color.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self.rebuild_symbol();
        self
    }

    pub fn set_icon_color(&mut self, color: Option<Color>) {
        self.icon_color = color;
        self.rebuild_symbol();
    }

    pub fn icon_value(&self) -> &str {
        &self.icon
    }

    pub fn icon_color_value(&self) -> Option<Color> {
        self.icon_color
    }

    /// Rename the trailing cancel button.
    pub fn cancel(mut self, label: impl Into<String>) -> Self {
        if let Some(last) = self.defs.last_mut() {
            last.label = label.into();
            last.action = AlertAction::Cancel;
            last.color = None;
        }
        self.rebuild_buttons();
        self.dirty = true;
        self
    }

    /// Drop the trailing cancel button (actions only).
    pub fn no_cancel(mut self) -> Self {
        self.defs.pop();
        if self.defs.is_empty() {
            self.defs.push(AlertButton::ok("OK"));
        }
        self.rebuild_buttons();
        self.dirty = true;
        self
    }

    fn rebuild_symbol(&mut self) {
        let mut symbol = SFSymbolImage::new(self.icon.clone()).size(ALERT_ICON_SIZE);
        if let Some(color) = self.icon_color {
            symbol = symbol.color(color);
        }
        symbol.set_theme(self.text_color, self.dark);
        symbol.set_focused(self.focused);
        self.symbol = symbol;
    }

    fn rebuild_buttons(&mut self) {
        self.buttons.clear();
        for (index, def) in self.defs.iter().enumerate() {
            // Custom tint wins (translucent style); otherwise every
            // button is tinted gray, like the system prompt.
            let style = ButtonStyle::BorderedTinted;
            let fired = self.fired.clone();
            self.buttons.push(
                Button::new(def.label.clone())
                    .style(style)
                    .shape(ButtonShape::Capsule)
                    .hover_effect(false)
                    .on_press(move || {
                        *fired.borrow_mut() = Some(index);
                    }),
            );
        }
        for button in &mut self.buttons {
            button.set_focused(self.focused);
        }
        self.apply_accents();
    }

    fn apply_accents(&mut self) {
        for (button, def) in self.buttons.iter_mut().zip(self.defs.iter()) {
            button.set_theme(def.color.unwrap_or(ALERT_CANCEL), self.dark);
        }
    }

    /// Live theme: frost amount, dark mode and the default text color
    /// (custom icon and button tints win).
    pub fn set_theme(&mut self, mode: ThemeMode, text: Color, glass: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.text_color = text;
        self.glass.set_theme(mode, glass);
        self.rebuild_symbol();
        self.apply_accents();
        self.dirty = true;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
        self.symbol.set_focused(focused);
        for button in &mut self.buttons {
            button.set_focused(focused);
        }
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if title != self.title {
            self.title = title;
            self.dirty = true;
        }
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        let message = message.into();
        if message != self.message {
            self.message = message;
            self.dirty = true;
        }
    }

    /// Button defs in stack order (trailing cancel last, if present).
    pub fn defs_value(&self) -> &[AlertButton] {
        &self.defs
    }

    /// Viewport the dim covers and the card centers in (usually the
    /// content area below the titlebar).
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vx = x;
        self.vy = y;
        self.vw = w;
        self.vh = h;
    }

    /// Trigger the alert: fades in from transparent.
    pub fn show(&mut self) {
        self.anim = Some(TweenAnim::new(
            Tween::new(0.0_f32, 1.0, ALERT_FADE_SECONDS).easing(Easing::CubicOut),
        ));
        self.t0 = Instant::now();
        self.state = AlertState::Opening;
    }

    /// Start the fade-out. The alert hides itself when done.
    pub fn dismiss(&mut self) {
        if self.state == AlertState::Hidden {
            return;
        }
        self.anim = Some(TweenAnim::new(
            Tween::new(self.opacity, 0.0, ALERT_FADE_SECONDS).easing(Easing::CubicOut),
        ));
        self.t0 = Instant::now();
        self.state = AlertState::Closing;
    }

    /// True while fully open (accepts button clicks).
    pub fn is_open(&self) -> bool {
        self.state == AlertState::Open
    }

    /// True while anything shows (fading in, open or fading out).
    /// Drive `Titlebar::set_modal_blocked` from this.
    pub fn is_visible(&self) -> bool {
        self.state != AlertState::Hidden
    }

    pub fn opacity_value(&self) -> f32 {
        self.opacity
    }

    /// Press handling. Forwards to the buttons only while fully open;
    /// clicks outside or mid-fade are swallowed and return `None`.
    /// Returns the pressed button as an event once per click.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.state != AlertState::Open {
            return;
        }
        for button in &mut self.buttons {
            button.mouse_down(x, y);
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) -> Option<AlertEvent> {
        if self.state != AlertState::Open {
            return None;
        }
        for button in &mut self.buttons {
            button.mouse_up(x, y);
        }
        self.fired.borrow_mut().take().map(|index| {
            let index = index.min(self.defs.len().saturating_sub(1));
            let def = &self.defs[index];
            AlertEvent {
                index,
                action: def.action,
                label: def.label.clone(),
            }
        })
    }

    fn text_width(&self) -> f32 {
        (ALERT_WIDTH - ALERT_PAD * 2.0 - ALERT_ICON_SIZE - ALERT_ICON_GAP).max(0.0)
    }

    fn text_colors(&self) -> (Color, Color) {
        if self.dark {
            (ALERT_TITLE_DARK, ALERT_MESSAGE_DARK)
        } else {
            (ALERT_TITLE_LIGHT, ALERT_MESSAGE_LIGHT)
        }
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty
            && self.title_layout.is_some()
            && self.message_layout.is_some()
            && self.layout_scale == fonts.scale
        {
            return;
        }
        let max = self.text_width();
        let (title_color, message_color) = self.text_colors();
        self.title_layout = Some(fonts.layout_text_weighted(
            &self.title,
            ALERT_TITLE_SIZE,
            title_color,
            600.0,
            Some(max),
        ));
        self.message_layout = Some(fonts.layout_text_weighted(
            &self.message,
            ALERT_MESSAGE_SIZE,
            message_color,
            400.0,
            Some(max),
        ));
        self.layout_scale = fonts.scale;
        self.dirty = false;
    }

    fn header_height(&mut self, fonts: &mut FontSystem) -> f32 {
        self.ensure_layout(fonts);
        let title_h = self
            .title_layout
            .as_ref()
            .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
            .unwrap_or(0.0);
        let message_h = self
            .message_layout
            .as_ref()
            .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
            .unwrap_or(0.0);
        let text_h = title_h + ALERT_TITLE_GAP + message_h;
        text_h.max(ALERT_ICON_SIZE)
    }

    fn card_height(&mut self, fonts: &mut FontSystem) -> f32 {
        let header = self.header_height(fonts);
        let n = self.buttons.len().max(1) as f32;
        ALERT_PAD + header + ALERT_MESSAGE_GAP
            + n * ALERT_BUTTON_H
            + (n - 1.0) * ALERT_BUTTON_GAP
            + ALERT_PAD
    }

    fn layout_card(&mut self, fonts: &mut FontSystem) {
        let height = self.card_height(fonts);
        let cx = self.vx + (self.vw - ALERT_WIDTH) / 2.0;
        let cy = self.vy + (self.vh - height) / 2.0;
        self.x = cx;
        self.y = cy;
        self.placed_w = ALERT_WIDTH;
        self.placed_h = height;
        self.glass.set_bounds(cx, cy, ALERT_WIDTH, height);
        self.glass.set_radius(ALERT_RADIUS);
        self.symbol.place(fonts, cx + ALERT_PAD, cy + ALERT_PAD, ALERT_ICON_SIZE, ALERT_ICON_SIZE);
        let bw = ALERT_WIDTH - ALERT_PAD * 2.0;
        let mut by = cy + ALERT_PAD + self.header_height(fonts) + ALERT_MESSAGE_GAP;
        for button in self.buttons.iter_mut() {
            button.place(fonts, cx + ALERT_PAD, by, bw, ALERT_BUTTON_H);
            by += ALERT_BUTTON_H + ALERT_BUTTON_GAP;
        }
    }
}

impl View for IconAlert {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        (ALERT_WIDTH, self.card_height(fonts))
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        // Placement is informational: the card centers in the viewport
        // set via `set_viewport` on every draw.
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.state == AlertState::Hidden {
            return;
        }
        // Advance the fade.
        let elapsed = self.t0.elapsed().as_secs_f32();
        if let Some(anim) = self.anim.as_mut() {
            if anim.update(elapsed) {
                match self.state {
                    AlertState::Opening => self.state = AlertState::Open,
                    AlertState::Closing => {
                        self.state = AlertState::Hidden;
                        self.opacity = 0.0;
                        self.anim = None;
                        return;
                    }
                    _ => {}
                }
            }
            self.opacity = *anim.value();
        }
        if self.vw <= 0.0 || self.vh <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        // Dim the app behind the card.
        let dim = Rect::new(
            px(self.vx),
            px(self.vy),
            px(self.vx + self.vw),
            px(self.vy + self.vh),
        );
        if images.is_capture_pass() {
            // Backdrop capture: paint only the dim, so the blur sees
            // the dimmed app without the card, icon, texts or buttons.
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(0, 0, 0, ALERT_DIM_ALPHA)),
                None,
                &dim,
            );
            return;
        }
        self.layout_card(fonts);
        // Whole overlay (dim plus frost card) fades as one layer.
        scene.push_layer(
            Fill::NonZero,
            BlendMode::default(),
            self.opacity.clamp(0.0, 1.0),
            Affine::IDENTITY,
            &dim,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(Color::from_rgba8(0, 0, 0, ALERT_DIM_ALPHA)),
            None,
            &dim,
        );
        self.glass.draw(scene, fonts, images);
        // Icon left, texts leading-aligned on its right.
        self.symbol.draw(scene, fonts, images);
        let tx = self.x + ALERT_PAD + ALERT_ICON_SIZE + ALERT_ICON_GAP;
        let title_h = self
            .title_layout
            .as_ref()
            .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
            .unwrap_or(0.0);
        if self.title_layout.is_some() {
            let layout = self.title_layout.as_ref().expect("layout built");
            draw_layout(scene, layout, tx, self.y + ALERT_PAD, fonts.scale);
        }
        if self.message_layout.is_some() {
            let layout = self.message_layout.as_ref().expect("layout built");
            draw_layout(
                scene,
                layout,
                tx,
                self.y + ALERT_PAD + title_h + ALERT_TITLE_GAP,
                fonts.scale,
            );
        }
        for button in &mut self.buttons {
            button.draw(scene, fonts, images);
        }
        scene.pop_layer();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alert() -> IconAlert {
        IconAlert::new(
            "lock.fill",
            "Authentication Required",
            "Enter an administrator name and password to continue.",
            vec![AlertButton::ok("Use Password...")],
        )
    }

    #[test]
    fn keeps_icon_and_actions_plus_cancel() {
        let alert = alert();
        assert_eq!(alert.icon_value(), "lock.fill");
        assert_eq!(alert.icon_color_value(), None);
        assert_eq!(alert.defs_value().len(), 2);
        assert_eq!(alert.defs_value()[1].action, AlertAction::Cancel);
        assert!(!alert.is_visible());
    }

    #[test]
    fn icon_color_is_optional() {
        let alert = alert().icon_color(Color::from_rgb8(0xff, 0x9f, 0x0a));
        assert_eq!(
            alert.icon_color_value(),
            Some(Color::from_rgb8(0xff, 0x9f, 0x0a))
        );
    }

    #[test]
    fn hidden_alert_reports_no_event() {
        let mut alert = alert();
        dialog_mouse(&mut alert);
        assert_eq!(alert.mouse_up(10.0, 10.0), None);
    }

    fn dialog_mouse(alert: &mut IconAlert) {
        alert.mouse_down(10.0, 10.0);
    }

    #[test]
    fn intrinsic_width_matches_token() {
        let mut alert = alert();
        let mut fonts = FontSystem::new();
        let (w, h) = alert.measure(&mut fonts);
        assert_eq!(w, ALERT_WIDTH);
        assert!(h > ALERT_BUTTON_H * 2.0);
    }
}
