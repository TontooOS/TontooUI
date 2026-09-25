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
use super::basic::{AlertAction, AlertButton, AlertState};
use super::{
    ALERT_BUTTON_GAP, ALERT_BUTTON_H, ALERT_CANCEL, ALERT_DIM_ALPHA,
    ALERT_FADE_SECONDS, ALERT_MESSAGE_DARK, ALERT_MESSAGE_GAP,
    ALERT_MESSAGE_LIGHT, ALERT_MESSAGE_SIZE, ALERT_PAD, ALERT_RADIUS,
    ALERT_TITLE_DARK, ALERT_TITLE_GAP, ALERT_TITLE_LIGHT, ALERT_TITLE_SIZE,
    ALERT_WIDTH,
};
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::theme::{GlassAmount, ThemeMode};

/// Press event of an action alert: which button (`index` into the
/// defs, 0 left, 1 right) with its action and label.
#[derive(Clone, Debug, PartialEq)]
pub struct AlertEvent {
    pub index: usize,
    pub action: AlertAction,
    pub label: String,
}

/// Alert with actions: like `BasicAlert` (frosted card, dimmed
/// backdrop, engine fade, modal) but with the title and message
/// leading-aligned and exactly two side-by-side buttons with custom
/// tints (e.g. gray Cancel plus red Delete). A press returns the
/// button as an `AlertEvent`; the plain `BasicAlert` reports no such
/// event. Clicks outside or mid-fade are swallowed.
pub struct ActionAlert {
    title: String,
    message: String,
    defs: [AlertButton; 2],
    buttons: Vec<Button>,
    fired: Rc<RefCell<Option<usize>>>,
    glass: GlassContainer,
    state: AlertState,
    opacity: f32,
    anim: Option<TweenAnim<f32>>,
    t0: Instant,
    dark: bool,
    focused: bool,
    accent: Color,
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

impl ActionAlert {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        left: AlertButton,
        right: AlertButton,
    ) -> Self {
        let fired: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let mut alert = Self {
            title: title.into(),
            message: message.into(),
            defs: [left, right],
            buttons: Vec::new(),
            fired,
            glass: GlassContainer::new().glass_type(GlassType::Frosted),
            state: AlertState::Hidden,
            opacity: 0.0,
            anim: None,
            t0: Instant::now(),
            dark: true,
            focused: true,
            accent: super::ALERT_ACCENT,
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

    fn rebuild_buttons(&mut self) {
        self.buttons.clear();
        for (index, def) in self.defs.iter().enumerate() {
            // Custom tint wins (translucent style); otherwise Ok is
            // prominent blue and Cancel is tinted gray.
            let style = match (def.color, def.action) {
                (Some(_), _) => ButtonStyle::BorderedTinted,
                (None, AlertAction::Ok) => ButtonStyle::BorderedProminent,
                (None, AlertAction::Cancel) => ButtonStyle::BorderedTinted,
            };
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
            let accent = match (def.color, def.action) {
                (Some(color), _) => color,
                (None, AlertAction::Ok) => self.accent,
                (None, AlertAction::Cancel) => ALERT_CANCEL,
            };
            button.set_theme(accent, self.dark);
        }
    }

    /// Live theme: frost amount, dark mode and the default OK accent
    /// (custom button tints win).
    pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.accent = accent;
        self.glass.set_theme(mode, glass);
        self.apply_accents();
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
            let def = &self.defs[index.min(1)];
            AlertEvent {
                index: index.min(1),
                action: def.action,
                label: def.label.clone(),
            }
        })
    }

    fn text_width(&self) -> f32 {
        (ALERT_WIDTH - ALERT_PAD * 2.0).max(0.0)
    }

    fn text_colors(&self) -> (Color, Color) {
        if self.dark {
            (ALERT_TITLE_DARK, ALERT_MESSAGE_DARK)
        } else {
            (ALERT_TITLE_LIGHT, ALERT_MESSAGE_LIGHT)
        }
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty && self.title_layout.is_some() && self.message_layout.is_some() {
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

impl View for ActionAlert {
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
        // Leading-aligned title and message.
        let title_h = self
            .title_layout
            .as_ref()
            .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
            .unwrap_or(0.0);
        if self.title_layout.is_some() {
            let layout = self.title_layout.as_ref().expect("layout built");
            draw_layout(
                scene,
                layout,
                self.x + ALERT_PAD,
                self.y + ALERT_PAD,
                fonts.scale,
            );
        }
        if self.message_layout.is_some() {
            let layout = self.message_layout.as_ref().expect("layout built");
            draw_layout(
                scene,
                layout,
                self.x + ALERT_PAD,
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

    fn alert() -> ActionAlert {
        ActionAlert::new(
            "Delete Item?",
            "Are you sure you want to delete this item?",
            AlertButton::cancel("Cancel"),
            AlertButton::ok("Delete").color(Color::from_rgb8(0xff, 0x3b, 0x30)),
        )
    }

    #[test]
    fn keeps_both_defs_with_custom_tint() {
        let alert = alert();
        assert_eq!(alert.defs[0].action, AlertAction::Cancel);
        assert_eq!(alert.defs[1].action, AlertAction::Ok);
        assert_eq!(
            alert.defs[1].color,
            Some(Color::from_rgb8(0xff, 0x3b, 0x30))
        );
        assert!(!alert.is_visible());
        assert!(!alert.is_open());
    }

    #[test]
    fn hidden_alert_reports_no_event() {
        let mut alert = alert();
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

    #[test]
    fn compact_tokens_are_half() {
        assert_eq!(ALERT_WIDTH, 210.0);
        assert_eq!(ALERT_TITLE_SIZE, 8.5);
        assert_eq!(ALERT_MESSAGE_SIZE, 7.5);
        assert_eq!(ALERT_BUTTON_H, 22.0);
    }
}
