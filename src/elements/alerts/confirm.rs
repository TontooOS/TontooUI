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
use super::action::AlertEvent;
use super::basic::{AlertAction, AlertButton, AlertState};
use super::{
    ALERT_BUTTON_GAP, ALERT_BUTTON_H, ALERT_CANCEL, ALERT_DIM_ALPHA,
    ALERT_FADE_SECONDS, ALERT_PAD, ALERT_RADIUS, ALERT_TITLE_DARK,
    ALERT_TITLE_GAP, ALERT_TITLE_LIGHT, ALERT_TITLE_SIZE, ALERT_WIDTH,
};
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::theme::{GlassAmount, ThemeMode};

/// Confirmation dialog: like the other alerts (frosted card, dimmed
/// backdrop, engine fade, modal) but with only a centered title and a
/// vertical stack of full-width option buttons plus a trailing
/// cancel. Made for more than two actions. A press returns the button
/// as an `AlertEvent` (stack index, action, label); clicks outside or
/// mid-fade are swallowed.
pub struct ConfirmationDialog {
    title: String,
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
    layout_scale: f32,
    dirty: bool,
}

impl ConfirmationDialog {
    /// Dialog with a title and stacked `options` (at least one; empty
    /// falls back to a single OK). A trailing Cancel button is added
    /// by default; rename it with `cancel` or drop it with
    /// `no_cancel`.
    pub fn new(title: impl Into<String>, options: Vec<AlertButton>) -> Self {
        let mut defs = options;
        if defs.is_empty() {
            defs.push(AlertButton::ok("OK"));
        }
        defs.push(AlertButton::cancel("Cancel"));
        Self::with_defs(title.into(), defs)
    }

    fn with_defs(title: String, defs: Vec<AlertButton>) -> Self {
        let fired: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let mut dialog = Self {
            title,
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
            layout_scale: 0.0,
            dirty: true,
        };
        dialog.rebuild_buttons();
        dialog
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

    /// Drop the trailing cancel button (options only).
    pub fn no_cancel(mut self) -> Self {
        self.defs.pop();
        if self.defs.is_empty() {
            self.defs.push(AlertButton::ok("OK"));
        }
        self.rebuild_buttons();
        self.dirty = true;
        self
    }

    fn rebuild_buttons(&mut self) {
        self.buttons.clear();
        for (index, def) in self.defs.iter().enumerate() {
            // Custom tint wins (translucent style); otherwise the
            // first option is prominent blue and the rest (plus the
            // trailing cancel) are tinted gray.
            let style = match (def.color, index) {
                (Some(_), _) => ButtonStyle::BorderedTinted,
                (None, 0) => ButtonStyle::BorderedProminent,
                _ => ButtonStyle::BorderedTinted,
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
        for (index, (button, def)) in self.buttons.iter_mut().zip(self.defs.iter()).enumerate() {
            let accent = match (def.color, index) {
                (Some(color), _) => color,
                (None, 0) => self.accent,
                _ => ALERT_CANCEL,
            };
            button.set_theme(accent, self.dark);
        }
    }

    /// Live theme: frost amount, dark mode and the first-option accent
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

    /// Trigger the dialog: fades in from transparent.
    pub fn show(&mut self) {
        self.anim = Some(TweenAnim::new(
            Tween::new(0.0_f32, 1.0, ALERT_FADE_SECONDS).easing(Easing::CubicOut),
        ));
        self.t0 = Instant::now();
        self.state = AlertState::Opening;
    }

    /// Start the fade-out. The dialog hides itself when done.
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

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if !self.dirty && self.title_layout.is_some() && self.layout_scale == fonts.scale {
            return;
        }
        let max = (ALERT_WIDTH - ALERT_PAD * 2.0).max(0.0);
        let title_color = if self.dark {
            ALERT_TITLE_DARK
        } else {
            ALERT_TITLE_LIGHT
        };
        self.title_layout = Some(fonts.layout_text_weighted(
            &self.title,
            ALERT_TITLE_SIZE,
            title_color,
            600.0,
            Some(max),
        ));
        self.layout_scale = fonts.scale;
        self.dirty = false;
    }

    fn card_height(&mut self, fonts: &mut FontSystem) -> f32 {
        self.ensure_layout(fonts);
        let title_h = self
            .title_layout
            .as_ref()
            .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
            .unwrap_or(0.0);
        let n = self.buttons.len().max(1) as f32;
        ALERT_PAD + title_h + ALERT_TITLE_GAP
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
        let bw = ALERT_WIDTH - ALERT_PAD * 2.0;
        let mut by = cy + ALERT_PAD
            + self
                .title_layout
                .as_ref()
                .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
                .unwrap_or(0.0)
            + ALERT_TITLE_GAP;
        for button in self.buttons.iter_mut() {
            button.place(fonts, cx + ALERT_PAD, by, bw, ALERT_BUTTON_H);
            by += ALERT_BUTTON_H + ALERT_BUTTON_GAP;
        }
    }
}

impl View for ConfirmationDialog {
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
            // the dimmed app without the card, title or buttons.
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
        // Centered title on top.
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

    fn dialog() -> ConfirmationDialog {
        ConfirmationDialog::new(
            "Choose Action",
            vec![
                AlertButton::ok("Option 1"),
                AlertButton::ok("Option 2"),
                AlertButton::ok("Option 3"),
            ],
        )
    }

    #[test]
    fn stacks_options_plus_trailing_cancel() {
        let dialog = dialog();
        assert_eq!(dialog.defs_value().len(), 4);
        assert_eq!(dialog.defs_value()[0].label, "Option 1");
        assert_eq!(dialog.defs_value()[3].action, AlertAction::Cancel);
        assert!(!dialog.is_visible());
    }

    #[test]
    fn empty_options_fall_back_to_ok() {
        let dialog = ConfirmationDialog::new("Title", vec![]).no_cancel();
        assert_eq!(dialog.defs_value().len(), 1);
        assert_eq!(dialog.defs_value()[0].action, AlertAction::Ok);
    }

    #[test]
    fn cancel_renames_trailing_button() {
        let dialog = ConfirmationDialog::new("Title", vec![AlertButton::ok("A")])
            .cancel("Abort");
        let defs = dialog.defs_value();
        assert_eq!(defs.last().map(|d| d.label.as_str()), Some("Abort"));
        assert_eq!(defs.last().map(|d| d.action), Some(AlertAction::Cancel));
    }

    #[test]
    fn hidden_dialog_reports_no_event() {
        let mut dialog = dialog();
        dialog.mouse_down(10.0, 10.0);
        assert_eq!(dialog.mouse_up(10.0, 10.0), None);
    }

    #[test]
    fn intrinsic_width_matches_token() {
        let mut dialog = dialog();
        let mut fonts = FontSystem::new();
        let (w, h) = dialog.measure(&mut fonts);
        assert_eq!(w, ALERT_WIDTH);
        assert!(h > ALERT_BUTTON_H * 4.0);
    }
}
