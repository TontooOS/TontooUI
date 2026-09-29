use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{BlendMode, Brush, Color, Fill};

use super::super::buttons::{Button, ButtonShape, ButtonStyle};
use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
use super::super::textfield::{
    FieldCore, SEARCH_FONT_SIZE, SEARCH_PAD_X, TEXTFIELD_CARET_W, accent_color, caret_blink,
    placeholder_color, resolve_press_single, selection_brush,
};
use super::basic::AlertState;
use super::{
    ALERT_ACCENT, ALERT_BUTTON_GAP, ALERT_BUTTON_H, ALERT_CANCEL, ALERT_DIM_ALPHA,
    ALERT_FADE_SECONDS, ALERT_FIELD_GAP, ALERT_FIELD_H, ALERT_MESSAGE_DARK, ALERT_MESSAGE_GAP,
    ALERT_MESSAGE_LIGHT, ALERT_MESSAGE_SIZE, ALERT_PAD, ALERT_RADIUS, ALERT_TITLE_DARK,
    ALERT_TITLE_GAP, ALERT_TITLE_LIGHT, ALERT_TITLE_SIZE, ALERT_WIDTH, AlertAction,
    AlertButton, fade_sample,
};
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{CTFrame, FontSystem, draw_layout};
use crate::renderer::window::Key;
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Modal text input alert: a frosted LiquidGlass card centered over
/// the app with a dimmed backdrop, a semibold title, a message, a
/// clear (`Lens`) glass pill with a single-line text input (no icon,
/// full width for typing) and Cancel + OK action buttons. Cannot be
/// dismissed by clicking outside — only the buttons (or Enter for
/// OK) close it. Entrance and exit fade through an engine tween; the
/// app triggers it with `show` (the field autofocuses with the caret
/// at the end), reads the result from `mouse_up` and the typed text
/// from `text_value`. Typing, Backspace and caret keys arrive
/// through `type_text` and `key`. Alert buttons react to clicks
/// only: hover does nothing.
pub struct TextInputAlert {
    title: String,
    message: String,
    defs: Vec<AlertButton>,
    buttons: Vec<Button>,
    fired: Rc<RefCell<Option<AlertAction>>>,
    core: FieldCore,
    glass: GlassContainer,
    field_glass: GlassContainer,
    autofocus: bool,
    field_hovered: bool,
    state: AlertState,
    opacity: f32,
    anim: Option<TweenAnim<f32>>,
    t0: Instant,
    dark: bool,
    focused: bool,
    accent: Color,
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
    fx: f32,
    fy: f32,
    fw: f32,
    fh: f32,
    title_layout: Option<CTFrame>,
    message_layout: Option<CTFrame>,
    layout_scale: f32,
    dirty: bool,
}

impl TextInputAlert {
    fn build(
        title: String,
        message: String,
        placeholder: String,
        defs: Vec<AlertButton>,
    ) -> Self {
        let fired: Rc<RefCell<Option<AlertAction>>> = Rc::new(RefCell::new(None));
        let mut alert = Self {
            title,
            message,
            defs,
            buttons: Vec::new(),
            fired,
            core: FieldCore::new(placeholder),
            glass: GlassContainer::new().glass_type(GlassType::Frosted),
            field_glass: GlassContainer::new().glass_type(GlassType::Lens),
            autofocus: false,
            field_hovered: false,
            state: AlertState::Hidden,
            opacity: 0.0,
            anim: None,
            t0: Instant::now(),
            dark: true,
            focused: true,
            accent: ALERT_ACCENT,
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
            fx: 0.0,
            fy: 0.0,
            fw: 0.0,
            fh: 0.0,
            title_layout: None,
            message_layout: None,
            layout_scale: 0.0,
            dirty: true,
        };
        alert.rebuild_buttons();
        alert
    }

    /// Default variant: title, message, placeholder and Cancel + OK.
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self::build(
            title.into(),
            message.into(),
            placeholder.into(),
            vec![AlertButton::cancel("Cancel"), AlertButton::ok("OK")],
        )
    }

    /// Custom buttons: one fills the row, two share it (in def
    /// order). Empty falls back to a single OK.
    pub fn buttons(
        title: impl Into<String>,
        message: impl Into<String>,
        placeholder: impl Into<String>,
        buttons: Vec<AlertButton>,
    ) -> Self {
        let mut defs = buttons;
        if defs.is_empty() {
            defs.push(AlertButton::ok("OK"));
        }
        defs.truncate(2);
        Self::build(title.into(), message.into(), placeholder.into(), defs)
    }

    fn rebuild_buttons(&mut self) {
        self.buttons.clear();
        for def in &self.defs {
            let style = match (def.color, def.action) {
                (Some(_), _) => ButtonStyle::BorderedTinted,
                (None, AlertAction::Ok) => ButtonStyle::BorderedProminent,
                (None, AlertAction::Cancel) => ButtonStyle::BorderedTinted,
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

    /// Live theme: frost amount, dark mode and the OK accent (also
    /// the field caret accent and pill frost).
    pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.accent = accent;
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
        self.field_glass.set_theme(mode, glass);
        self.core.set_style(accent, self.dark);
        self.apply_accents();
        self.dirty = true;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
        self.field_glass.set_focused(focused);
        self.core.set_focused_flag(focused);
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

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.core.set_placeholder_text(placeholder.into());
    }

    /// Programmatic text (caret to end, no selection change).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.core.set_text(text.into());
    }

    /// Current field text (read-only). The app reads this after OK.
    pub fn text_value(&self) -> &str {
        self.core.text_value()
    }

    /// True while the field holds the caret.
    pub fn is_selected(&self) -> bool {
        self.core.is_selected()
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

    /// Trigger the alert: fades in from transparent and autofocuses
    /// the field with the caret at the end. Safe to call when
    /// already visible (restarts the entrance).
    pub fn show(&mut self) {
        self.anim = Some(TweenAnim::new(
            Tween::new(0.0_f32, 1.0, ALERT_FADE_SECONDS).easing(Easing::CubicOut),
        ));
        self.t0 = Instant::now();
        self.state = AlertState::Opening;
        self.autofocus = true;
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

    /// True while fully open (accepts clicks and keys).
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
        fade_sample(elapsed)
    }

    pub fn opacity_value(&self) -> f32 {
        self.opacity
    }

    fn field_hit(&self, x: f32, y: f32) -> bool {
        x >= self.fx && x <= self.fx + self.fw && y >= self.fy && y <= self.fy + self.fh
    }

    /// Press handling. Forwards to the field and the buttons only
    /// while fully open; clicks outside or mid-fade are swallowed.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.state != AlertState::Open {
            return;
        }
        if self.field_hit(x as f32, y as f32) {
            self.core.press(x as f32, y as f32);
        } else {
            self.core.deselect();
        }
        for button in &mut self.buttons {
            button.mouse_down(x, y);
        }
    }

    /// Button result once per click (`None` while hidden, mid-fade
    /// or without a button hit). Releases a field hold either way.
    pub fn mouse_up(&mut self, x: f64, y: f64) -> Option<AlertAction> {
        self.core.release();
        if self.state != AlertState::Open {
            return None;
        }
        for button in &mut self.buttons {
            button.mouse_up(x, y);
        }
        self.take_action()
    }

    /// Take a pending button action without a click (e.g. after
    /// `key` confirmed with Enter). Returns `None` when nothing
    /// fired.
    pub fn take_action(&mut self) -> Option<AlertAction> {
        self.fired.borrow_mut().take()
    }

    /// Hover tracking for the I-beam cursor over the field.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.field_hovered =
            self.state == AlertState::Open && self.field_hit(x as f32, y as f32);
        self.core.set_hovered(self.field_hovered);
    }

    /// Type printable text at the caret (the app forwards its
    /// `text` here while the field is selected).
    pub fn type_text(&mut self, content: &str) {
        if self.state == AlertState::Open && self.core.is_selected() {
            self.core.insert(content);
        }
    }

    /// Key handling while open: Enter confirms with the first OK
    /// button (when the defs hold one), everything else goes to the
    /// field (Backspace, caret motion, clipboard, ESC deselects).
    /// Returns true when consumed. The app forwards its `key` here.
    pub fn key(&mut self, key: Key) -> bool {
        if self.state != AlertState::Open {
            return false;
        }
        if key == Key::Enter {
            if self
                .defs
                .iter()
                .any(|def| def.action == AlertAction::Ok)
            {
                *self.fired.borrow_mut() = Some(AlertAction::Ok);
                return true;
            }
            return false;
        }
        self.core.handle_key(key)
    }

    /// True while the pointer hovers the field: the app returns the
    /// I-beam cursor from `App::cursor` then.
    pub fn wants_text_cursor(&self) -> bool {
        self.field_hovered
    }

    fn text_width(&self) -> f32 {
        (ALERT_WIDTH - ALERT_PAD * 2.0).max(0.0)
    }

    fn text_color(&self) -> Color {
        let base = if self.dark {
            Color::from_rgb8(0xd8, 0xd9, 0xd9)
        } else {
            Color::from_rgb8(0x27, 0x27, 0x27)
        };
        if self.focused {
            base
        } else {
            desaturate(base)
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
        let message_h = self
            .message_layout
            .as_ref()
            .map(|l| FontSystem::layout_size(l).1 / fonts.scale)
            .unwrap_or(0.0);
        ALERT_PAD + title_h + ALERT_TITLE_GAP + message_h + ALERT_FIELD_GAP + ALERT_FIELD_H
            + ALERT_MESSAGE_GAP
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
        let field_h = ALERT_FIELD_H;
        self.fx = cx + ALERT_PAD;
        self.fy = cy + ALERT_PAD + title_h + ALERT_TITLE_GAP + message_h + ALERT_FIELD_GAP;
        self.fw = (ALERT_WIDTH - ALERT_PAD * 2.0).max(0.0);
        self.fh = field_h;
        self.field_glass.set_bounds(self.fx, self.fy, self.fw, self.fh);
        self.field_glass
            .set_radius((self.fw.min(self.fh) / 2.0).max(0.0));
        // Autofocus: press the pill's far end so the caret lands at
        // the end of prefilled text; draw resolves it to a caret.
        if self.autofocus {
            self.autofocus = false;
            self.core
                .press(self.fx + self.fw - 1.0, self.fy + self.fh / 2.0);
        }
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

    fn draw_field(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if self.fw <= 0.0 || self.fh <= 0.0 {
            return;
        }
        self.field_glass.draw(scene, fonts, images);
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let text = self.text_color();
        let ix = self.fx + SEARCH_PAD_X;
        let iw = (self.fw - SEARCH_PAD_X * 2.0).max(0.0);
        {
            let echo = self.core.echo();
            let origin_x = ix - self.core.scroll();
            resolve_press_single(
                &mut self.core,
                fonts,
                &echo,
                SEARCH_FONT_SIZE,
                text,
                origin_x,
            );
        }
        let placeholder = placeholder_color(&self.core);
        let clip = RoundedRect::new(
            px(ix),
            px(self.fy),
            px(ix + iw),
            px(self.fy + self.fh),
            px(4.0),
        );
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
        let th = {
            let (layout, _) =
                self.core
                    .ensure_layout(fonts, SEARCH_FONT_SIZE, text, placeholder, None);
            FontSystem::layout_size(layout).1
        };
        let ty = self.fy + (self.fh - th / fonts.scale) / 2.0;
        let caret_x = self.core.caret_x(fonts, SEARCH_FONT_SIZE, text);
        self.core.track_caret(caret_x, iw);
        let scroll = self.core.scroll();
        {
            let (layout, _) =
                self.core
                    .ensure_layout(fonts, SEARCH_FONT_SIZE, text, placeholder, None);
            draw_layout(scene, layout, ix - scroll, ty, fonts.scale);
        }
        if let Some((a, b)) = self.core.selection_range() {
            let x0 = ix - scroll + self.core.echo_advance(fonts, a, SEARCH_FONT_SIZE, text);
            let x1 = ix - scroll + self.core.echo_advance(fonts, b, SEARCH_FONT_SIZE, text);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(selection_brush(&self.core)),
                None,
                &vello::kurbo::Rect::new(px(x0), px(ty), px(x1), px(ty + th / fonts.scale)),
            );
        }
        if self.core.is_selected() && caret_blink() {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(accent_color(&self.core)),
                None,
                &vello::kurbo::Rect::new(
                    px(ix - scroll + caret_x),
                    px(ty),
                    px(ix - scroll + caret_x + TEXTFIELD_CARET_W),
                    px(ty + th / fonts.scale),
                ),
            );
        }
        scene.pop_layer();
    }
}

impl View for TextInputAlert {
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
            // the dimmed app without the card, texts, field or buttons.
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
        self.draw_field(scene, fonts, images);
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

    fn alert() -> TextInputAlert {
        TextInputAlert::new("Rename", "Enter a new name.", "Untitled")
    }

    #[test]
    fn default_variant_has_cancel_ok_and_empty_text() {
        let alert = alert();
        assert_eq!(alert.defs.len(), 2);
        assert_eq!(alert.defs[0].action, AlertAction::Cancel);
        assert_eq!(alert.defs[1].action, AlertAction::Ok);
        assert_eq!(alert.text_value(), "");
        assert!(!alert.is_selected());
        assert!(!alert.is_visible());
        assert!(!alert.is_open());
    }

    #[test]
    fn custom_buttons_clamp_and_fall_back() {
        let alert = TextInputAlert::buttons(
            "Title",
            "Message",
            "Placeholder",
            vec![
                AlertButton::cancel("Cancel"),
                AlertButton::ok("OK"),
                AlertButton::ok("Extra"),
            ],
        );
        assert_eq!(alert.defs.len(), 2);
        let alert = TextInputAlert::buttons("Title", "Message", "Placeholder", vec![]);
        assert_eq!(alert.defs.len(), 1);
        assert_eq!(alert.defs[0].action, AlertAction::Ok);
    }

    #[test]
    fn set_text_round_trips_and_typing_needs_selection() {
        let mut alert = alert();
        alert.set_text("hello");
        assert_eq!(alert.text_value(), "hello");
        // Unselected: typing is ignored.
        alert.type_text(" world");
        assert_eq!(alert.text_value(), "hello");
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
    fn hidden_alert_swallows_input() {
        let mut alert = alert();
        assert_eq!(alert.mouse_up(10.0, 10.0), None);
        alert.mouse_down(10.0, 10.0);
        assert_eq!(alert.mouse_up(10.0, 10.0), None);
        assert!(!alert.key(Key::Enter));
        assert!(!alert.key(Key::Backspace));
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
