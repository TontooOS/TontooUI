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
use super::super::layout::View;
use super::{
    ALERT_ACCENT, ALERT_BUTTON_GAP, ALERT_BUTTON_H, ALERT_DIM_ALPHA,
    ALERT_FADE_SECONDS, ALERT_MESSAGE_DARK, ALERT_MESSAGE_GAP,
    ALERT_MESSAGE_LIGHT, ALERT_MESSAGE_SIZE, ALERT_PAD, ALERT_RADIUS,
    ALERT_TITLE_DARK, ALERT_TITLE_GAP, ALERT_TITLE_LIGHT, ALERT_TITLE_SIZE,
    ALERT_WIDTH,
};
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::theme::{GlassAmount, ThemeMode};

/// Dismiss action of an alert button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertAction {
    Ok,
    Cancel,
}

/// One alert button: label plus action. `Ok` renders prominent blue,
/// `Cancel` renders bordered gray.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertButton {
    pub label: String,
    pub action: AlertAction,
}

impl AlertButton {
    pub fn ok(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: AlertAction::Ok,
        }
    }

    pub fn cancel(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: AlertAction::Cancel,
        }
    }
}

/// Visibility of the alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AlertState {
    #[default]
    Hidden,
    Opening,
    Open,
    Closing,
}

/// Basic modal alert: frosted LiquidGlass card centered over the app
/// with a dimmed backdrop, a semibold title, a message and one (OK)
/// or two (Cancel + OK) action buttons. Cannot be dismissed by
/// clicking outside — only the buttons close it. Entrance and exit
/// fade through an engine tween; the app triggers it with `show`
/// (e.g. from its own buttons) and reads the result from `mouse_up`.
/// Alert buttons react to clicks only: hover does nothing.
pub struct BasicAlert {
    title: String,
    message: String,
    defs: Vec<AlertButton>,
    buttons: Vec<Button>,
    fired: Rc<RefCell<Option<AlertAction>>>,
    glass: GlassContainer,
    state: AlertState,
    opacity: f32,
    anim: Option<TweenAnim<f32>>,
    t0: Instant,
    dark: bool,
    focused: bool,
    title_color: Color,
    message_color: Color,
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
    dirty: bool,
}

impl BasicAlert {
    fn build(title: String, message: String, defs: Vec<AlertButton>) -> Self {
        let fired: Rc<RefCell<Option<AlertAction>>> = Rc::new(RefCell::new(None));
        let mut alert = Self {
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
            title_color: ALERT_TITLE_DARK,
            message_color: ALERT_MESSAGE_DARK,
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
            dirty: true,
        };
        alert.rebuild_buttons();
        alert
    }

    /// Basic variant: title, message and a single OK button.
    pub fn ok(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::build(
            title.into(),
            message.into(),
            vec![AlertButton::ok("OK")],
        )
    }

    /// Custom buttons: one fills the row, two share it (Cancel left,
    /// OK right). Empty falls back to a single OK.
    pub fn buttons(
        title: impl Into<String>,
        message: impl Into<String>,
        buttons: Vec<AlertButton>,
    ) -> Self {
        let mut defs = buttons;
        if defs.is_empty() {
            defs.push(AlertButton::ok("OK"));
        }
        defs.truncate(2);
        Self::build(title.into(), message.into(), defs)
    }

    fn rebuild_buttons(&mut self) {
        self.buttons.clear();
        for def in &self.defs {
            let style = match def.action {
                AlertAction::Ok => ButtonStyle::BorderedProminent,
                AlertAction::Cancel => ButtonStyle::Bordered,
            };
            let action = def.action;
            let fired = self.fired.clone();
            self.buttons.push(
                Button::new(def.label.clone())
                    .style(style)
                    .shape(ButtonShape::Capsule)
                    .hover_effect(false)
                    .on_press(move || {
                        *fired.borrow_mut() = Some(action);
                    }),
            );
        }
        for button in &mut self.buttons {
            button.set_theme(ALERT_ACCENT, self.dark);
            button.set_focused(self.focused);
        }
    }

    /// Live theme: frost amount, dark mode and the OK accent.
    pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.title_color = if self.dark {
            ALERT_TITLE_DARK
        } else {
            ALERT_TITLE_LIGHT
        };
        self.message_color = if self.dark {
            ALERT_MESSAGE_DARK
        } else {
            ALERT_MESSAGE_LIGHT
        };
        self.glass.set_theme(mode, glass);
        for button in &mut self.buttons {
            button.set_theme(accent, self.dark);
        }
        self.dirty = true;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
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

    /// Viewport the dim covers and the card centers in (usually the
    /// content area below the titlebar).
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vx = x;
        self.vy = y;
        self.vw = w;
        self.vh = h;
    }

    pub fn viewport(&self) -> (f32, f32, f32, f32) {
        (self.vx, self.vy, self.vw, self.vh)
    }

    /// Trigger the alert: fades in from transparent. Safe to call
    /// when already visible (restarts the entrance).
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

    /// Current fade opacity (0..1). Pure sampling helper for tests.
    pub fn opacity_at(&self, elapsed: f32) -> f32 {
        Tween::new(0.0_f32, 1.0, ALERT_FADE_SECONDS)
            .easing(Easing::CubicOut)
            .sample(elapsed)
            .0
    }

    pub fn opacity_value(&self) -> f32 {
        self.opacity
    }

    /// Press handling. Forwards to the buttons only while fully open;
    /// clicks outside or mid-fade are swallowed and return `None`.
    /// Returns the clicked action once per click.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.state != AlertState::Open {
            return;
        }
        for button in &mut self.buttons {
            button.mouse_down(x, y);
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) -> Option<AlertAction> {
        if self.state != AlertState::Open {
            return None;
        }
        for button in &mut self.buttons {
            button.mouse_up(x, y);
        }
        self.fired.borrow_mut().take()
    }

    fn text_width(&self) -> f32 {
        (ALERT_WIDTH - ALERT_PAD * 2.0).max(0.0)
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty && self.title_layout.is_some() && self.message_layout.is_some() {
            return;
        }
        let max = self.text_width();
        self.title_layout = Some(fonts.layout_text_weighted(
            &self.title,
            ALERT_TITLE_SIZE,
            self.title_color,
            600.0,
            Some(max),
        ));
        self.message_layout = Some(fonts.layout_text_weighted(
            &self.message,
            ALERT_MESSAGE_SIZE,
            self.message_color,
            400.0,
            Some(max),
        ));
        self.dirty = false;
    }

    fn card_height(&mut self, fonts: &mut FontSystem) -> f32 {
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
        ALERT_PAD + title_h + ALERT_TITLE_GAP + message_h + ALERT_MESSAGE_GAP
            + ALERT_BUTTON_H
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
        let btn_y = cy + height - ALERT_PAD - ALERT_BUTTON_H;
        if self.buttons.len() == 1 {
            self.buttons[0].place(
                fonts,
                cx + ALERT_PAD,
                btn_y,
                ALERT_WIDTH - ALERT_PAD * 2.0,
                ALERT_BUTTON_H,
            );
        } else {
            let bw = (ALERT_WIDTH - ALERT_PAD * 2.0 - ALERT_BUTTON_GAP) / 2.0;
            for (index, button) in self.buttons.iter_mut().enumerate() {
                button.place(
                    fonts,
                    cx + ALERT_PAD + index as f32 * (bw + ALERT_BUTTON_GAP),
                    btn_y,
                    bw,
                    ALERT_BUTTON_H,
                );
            }
        }
    }
}

impl View for BasicAlert {
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
        self.layout_card(fonts);
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        // Whole overlay (dim plus frost card) fades as one layer.
        let clip = Rect::new(
            px(self.vx),
            px(self.vy),
            px(self.vx + self.vw),
            px(self.vy + self.vh),
        );
        scene.push_layer(
            Fill::NonZero,
            BlendMode::default(),
            self.opacity.clamp(0.0, 1.0),
            Affine::IDENTITY,
            &clip,
        );
        // Dim the app behind the card.
        let dim = Rect::new(
            px(self.vx),
            px(self.vy),
            px(self.vx + self.vw),
            px(self.vy + self.vh),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(Color::from_rgba8(0, 0, 0, ALERT_DIM_ALPHA)),
            None,
            &dim,
        );
        self.glass.draw(scene, fonts, images);
        // Centered title and message.
        let title_h = self
            .title_layout
            .as_ref()
            .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
            .unwrap_or(0.0);
        if self.title_layout.is_some() {
            let layout = self.title_layout.as_ref().expect("layout built");
            let (tw, _) = FontSystem::layout_size(layout);
            draw_layout(
                scene,
                layout,
                self.x + (ALERT_WIDTH - tw / fonts.scale) / 2.0,
                self.y + ALERT_PAD,
                fonts.scale,
            );
        }
        if self.message_layout.is_some() {
            let layout = self.message_layout.as_ref().expect("layout built");
            let (tw, _) = FontSystem::layout_size(layout);
            draw_layout(
                scene,
                layout,
                self.x + (ALERT_WIDTH - tw / fonts.scale) / 2.0,
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
    use crate::renderer::text::FontSystem;

    fn alert() -> BasicAlert {
        BasicAlert::ok("Alert Title", "This is a basic alert message.")
    }

    #[test]
    fn ok_variant_has_single_ok_button() {
        let alert = alert();
        assert_eq!(alert.defs.len(), 1);
        assert_eq!(alert.defs[0].action, AlertAction::Ok);
        assert!(!alert.is_visible());
        assert!(!alert.is_open());
    }

    #[test]
    fn two_buttons_share_cancel_ok() {
        let alert = BasicAlert::buttons(
            "Title",
            "Message",
            vec![AlertButton::cancel("Cancel"), AlertButton::ok("OK")],
        );
        assert_eq!(alert.defs.len(), 2);
        assert_eq!(alert.defs[0].action, AlertAction::Cancel);
        // More than two clamp to the first two.
        let alert = BasicAlert::buttons(
            "Title",
            "Message",
            vec![
                AlertButton::cancel("Cancel"),
                AlertButton::ok("OK"),
                AlertButton::ok("Extra"),
            ],
        );
        assert_eq!(alert.defs.len(), 2);
    }

    #[test]
    fn empty_buttons_fall_back_to_ok() {
        let alert = BasicAlert::buttons("Title", "Message", vec![]);
        assert_eq!(alert.defs.len(), 1);
        assert_eq!(alert.defs[0].action, AlertAction::Ok);
    }

    #[test]
    fn fade_ramps_through_engine() {
        let alert = alert();
        assert_eq!(alert.opacity_at(0.0), 0.0);
        let mid = alert.opacity_at(ALERT_FADE_SECONDS / 2.0);
        assert!(mid > 0.0 && mid < 1.0);
        assert_eq!(alert.opacity_at(ALERT_FADE_SECONDS * 2.0), 1.0);
    }

    #[test]
    fn clicks_outside_open_do_nothing() {
        let mut alert = alert();
        // Hidden alerts swallow everything.
        assert_eq!(alert.mouse_up(10.0, 10.0), None);
        alert.mouse_down(10.0, 10.0);
        assert_eq!(alert.mouse_up(10.0, 10.0), None);
    }

    #[test]
    fn intrinsic_width_matches_token() {
        let mut alert = alert();
        let mut fonts = FontSystem::new();
        let (w, h) = alert.measure(&mut fonts);
        assert_eq!(w, ALERT_WIDTH);
        assert!(h > ALERT_BUTTON_H + ALERT_PAD * 2.0);
    }
}
