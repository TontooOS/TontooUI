use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{BlendMode, Brush, Color, Fill};

use super::super::layout::View;
use super::{
    SHEET_BG_DARK, SHEET_BG_LIGHT, SHEET_BORDER_DARK, SHEET_BORDER_LIGHT,
    SHEET_DIM_ALPHA, SHEET_FADE_SECONDS, SHEET_RADIUS, SHEET_SHADOW,
    SHEET_SHADOW_BLUR, SHEET_SHADOW_DY,
};
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::renderer::window::Key;
use crate::theme::desaturate;

/// Sheet size: intrinsic content (`Small`) or a centered fraction of
/// the viewport in both directions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SheetSize {
    /// Intrinsic content size, centered.
    #[default]
    Small,
    /// 25% of the viewport width and height, centered.
    Quarter,
    /// 50% of the viewport width and height, centered.
    Half,
    /// 75% of the viewport width and height, centered.
    Large,
}

impl SheetSize {
    /// Viewport fraction (width and height), or `None` for intrinsic.
    pub fn fraction(&self) -> Option<f32> {
        match self {
            Self::Small => None,
            Self::Quarter => Some(0.25),
            Self::Half => Some(0.5),
            Self::Large => Some(0.75),
        }
    }
}

/// Visibility of the sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SheetState {
    #[default]
    Hidden,
    Opening,
    Open,
    Closing,
}

/// Basic modal sheet: a small card with any child content (text,
/// buttons, ...), centered over the app with a dimmed backdrop. ESC
/// closes it, or a close button the app puts into the content.
/// `Quarter`, `Half` and `Large` fill 25%, 50% or 75% of the viewport
/// in both directions for rich content; the card background is
/// changeable. Entrance and exit fade through an engine tween; the
/// app triggers it with `show` and routes input while visible. Like
/// alerts, an open sheet blocks the red traffic light (drive
/// `Titlebar::set_modal_blocked` from `is_visible`) and clicks
/// outside are swallowed.
pub struct BasicSheet<V> {
    child: V,
    size: SheetSize,
    bg: Color,
    bg_manual: bool,
    border: Color,
    dark: bool,
    focused: bool,
    state: SheetState,
    opacity: f32,
    anim: Option<TweenAnim<f32>>,
    t0: Instant,
    vx: f32,
    vy: f32,
    vw: f32,
    vh: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl<V: View> BasicSheet<V> {
    pub fn new(child: V) -> Self {
        Self {
            child,
            size: SheetSize::Small,
            bg: SHEET_BG_DARK,
            bg_manual: false,
            border: SHEET_BORDER_DARK,
            dark: true,
            focused: true,
            state: SheetState::Hidden,
            opacity: 0.0,
            anim: None,
            t0: Instant::now(),
            vx: 0.0,
            vy: 0.0,
            vw: 0.0,
            vh: 0.0,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Card size: intrinsic content or a viewport fraction.
    pub fn size(mut self, size: SheetSize) -> Self {
        self.size = size;
        self
    }

    /// Card background fill. Without it the fill follows the theme.
    pub fn background(mut self, color: Color) -> Self {
        self.bg = color;
        self.bg_manual = true;
        self
    }

    /// Live theme for the default fill and border.
    pub fn set_theme(&mut self, dark: bool) {
        self.dark = dark;
        if !self.bg_manual {
            self.bg = if dark { SHEET_BG_DARK } else { SHEET_BG_LIGHT };
        }
        self.border = if dark {
            SHEET_BORDER_DARK
        } else {
            SHEET_BORDER_LIGHT
        };
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_size(&mut self, size: SheetSize) {
        self.size = size;
    }

    pub fn set_background(&mut self, color: Color) {
        self.bg = color;
        self.bg_manual = true;
    }

    pub fn clear_background(&mut self) {
        self.bg_manual = false;
        self.bg = if self.dark { SHEET_BG_DARK } else { SHEET_BG_LIGHT };
    }

    /// Wrapped content for state updates (labels, toggles, ...).
    pub fn child_mut(&mut self) -> &mut V {
        &mut self.child
    }

    /// Viewport the dim covers and the card centers in (usually the
    /// content area below the titlebar).
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vx = x;
        self.vy = y;
        self.vw = w;
        self.vh = h;
    }

    /// Trigger the sheet: fades in from transparent.
    pub fn show(&mut self) {
        self.anim = Some(TweenAnim::new(
            Tween::new(0.0_f32, 1.0, SHEET_FADE_SECONDS).easing(Easing::CubicOut),
        ));
        self.t0 = Instant::now();
        self.state = SheetState::Opening;
    }

    /// Start the fade-out. The sheet hides itself when done.
    pub fn dismiss(&mut self) {
        if self.state == SheetState::Hidden {
            return;
        }
        self.anim = Some(TweenAnim::new(
            Tween::new(self.opacity, 0.0, SHEET_FADE_SECONDS).easing(Easing::CubicOut),
        ));
        self.t0 = Instant::now();
        self.state = SheetState::Closing;
    }

    /// True while fully open (accepts input).
    pub fn is_open(&self) -> bool {
        self.state == SheetState::Open
    }

    /// True while anything shows (fading in, open or fading out).
    /// Drive `Titlebar::set_modal_blocked` from this.
    pub fn is_visible(&self) -> bool {
        self.state != SheetState::Hidden
    }

    pub fn opacity_value(&self) -> f32 {
        self.opacity
    }

    /// Card rect for the current viewport in logical px. Pure helper
    /// for tests: (x, y, width, height).
    pub fn card_rect(&mut self, fonts: &mut FontSystem) -> (f32, f32, f32, f32) {
        let (w, h) = match self.size.fraction() {
            Some(f) => ((self.vw * f).max(0.0), (self.vh * f).max(0.0)),
            None => self.child.measure(fonts),
        };
        (
            self.vx + (self.vw - w) / 2.0,
            self.vy + (self.vh - h) / 2.0,
            w,
            h,
        )
    }

    /// Press handling. Forwards to the content only while fully open;
    /// clicks outside or mid-fade are swallowed.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.state != SheetState::Open {
            return;
        }
        self.child.mouse_down(x, y);
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        if self.state != SheetState::Open {
            return;
        }
        self.child.mouse_up(x, y);
    }

    /// Key handling. ESC while visible dismisses and reports true;
    /// anything else reports false. The app forwards its `key` here.
    pub fn key(&mut self, key: Key) -> bool {
        if self.state == SheetState::Hidden {
            return false;
        }
        if key == Key::Escape {
            self.dismiss();
            return true;
        }
        false
    }

    fn layout_card(&mut self, fonts: &mut FontSystem) {
        let (cx, cy, w, h) = self.card_rect(fonts);
        self.x = cx;
        self.y = cy;
        self.placed_w = w;
        self.placed_h = h;
        self.child.place(fonts, cx, cy, w, h);
    }
}

impl<V: View + 'static> View for BasicSheet<V> {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        // Intrinsic content size; fractions resolve against the
        // viewport on draw.
        self.child.measure(fonts)
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
        if self.state == SheetState::Hidden {
            return;
        }
        // Advance the fade.
        let elapsed = self.t0.elapsed().as_secs_f32();
        if let Some(anim) = self.anim.as_mut() {
            if anim.update(elapsed) {
                match self.state {
                    SheetState::Opening => self.state = SheetState::Open,
                    SheetState::Closing => {
                        self.state = SheetState::Hidden;
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
            // Backdrop capture: paint only the dim, so a blur pass
            // sees the dimmed app without the card or its content.
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(0, 0, 0, SHEET_DIM_ALPHA)),
                None,
                &dim,
            );
            return;
        }
        self.layout_card(fonts);
        // Whole overlay (dim plus card) fades as one layer.
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
            &Brush::Solid(Color::from_rgba8(0, 0, 0, SHEET_DIM_ALPHA)),
            None,
            &dim,
        );
        // Soft drop shadow under the card.
        let card = Rect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
        );
        let radius = SHEET_RADIUS as f64 * scale;
        scene.draw_blurred_rounded_rect(
            Affine::translate((0.0, SHEET_SHADOW_DY as f64 * scale)),
            card,
            SHEET_SHADOW,
            radius,
            SHEET_SHADOW_BLUR as f64 * scale,
        );
        let body = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
            radius,
        );
        // Inactive windows desaturate the card like the palette.
        let (bg, border) = if self.focused {
            (self.bg, self.border)
        } else {
            (desaturate(self.bg), desaturate(self.border))
        };
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(bg),
            None,
            &body,
        );
        scene.stroke(
            &Stroke::new(1.0 * scale),
            Affine::IDENTITY,
            &Brush::Solid(border),
            None,
            &body,
        );
        self.child.draw(scene, fonts, images);
        scene.pop_layer();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::BasicText;
    use crate::renderer::text::FontSystem;

    fn sheet() -> BasicSheet<BasicText> {
        BasicSheet::new(BasicText::new("This is a sheet!"))
    }

    #[test]
    fn fractions_match_quarters() {
        assert_eq!(SheetSize::Small.fraction(), None);
        assert_eq!(SheetSize::Quarter.fraction(), Some(0.25));
        assert_eq!(SheetSize::Half.fraction(), Some(0.5));
        assert_eq!(SheetSize::Large.fraction(), Some(0.75));
    }

    #[test]
    fn sizes_center_in_viewport() {
        let mut fonts = FontSystem::new();
        let mut large = BasicSheet::new(BasicText::new("x")).size(SheetSize::Large);
        large.set_viewport(0.0, 0.0, 800.0, 600.0);
        assert_eq!(large.card_rect(&mut fonts), (100.0, 75.0, 600.0, 450.0));
        let mut quarter = BasicSheet::new(BasicText::new("x")).size(SheetSize::Quarter);
        quarter.set_viewport(0.0, 0.0, 800.0, 600.0);
        assert_eq!(quarter.card_rect(&mut fonts), (300.0, 225.0, 200.0, 150.0));
    }

    #[test]
    fn hidden_sheet_ignores_keys_and_clicks() {
        let mut sheet = sheet();
        assert!(!sheet.key(Key::Escape));
        assert!(!sheet.is_visible());
        sheet.mouse_down(10.0, 10.0);
        sheet.mouse_up(10.0, 10.0);
        assert!(!sheet.is_visible());
    }

    #[test]
    fn escape_dismisses_visible_sheet() {
        let mut sheet = sheet();
        sheet.show();
        // Any non-hidden state consumes ESC.
        assert!(sheet.key(Key::Escape));
        assert!(!sheet.key(Key::Enter));
    }

    #[test]
    fn custom_background_wins_over_theme() {
        let custom = Color::from_rgb8(0x3a, 0x3a, 0x3c);
        let mut sheet = sheet().background(custom);
        sheet.set_theme(false);
        assert_eq!(sheet.bg, custom);
        sheet.clear_background();
        assert_eq!(sheet.bg, SHEET_BG_LIGHT);
    }
}
