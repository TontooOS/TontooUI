use parley::Layout;
use vello::Scene;
use vello::kurbo::{Affine, Line, Point, RoundedRect, RoundedRectRadii, Stroke};
use vello::peniko::{Brush, Color, Fill};

use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::renderer::window::WINDOW_CORNER_RADIUS;

/// Opaque titlebar background, dark / light. Dark is lighter than the
/// `#1d1d1d` window body, light is darker than the `#ececec` body.
pub const TITLEBAR_BG_DARK: Color = Color::from_rgb8(0x2c, 0x2c, 0x2e);
pub const TITLEBAR_BG_LIGHT: Color = Color::from_rgb8(0xde, 0xde, 0xe1);
/// Title text, dark / light.
pub const TITLEBAR_TEXT_DARK: Color = Color::from_rgb8(0xf5, 0xf5, 0xf7);
pub const TITLEBAR_TEXT_LIGHT: Color = Color::from_rgb8(0x1e, 0x1e, 0x1e);
/// Bottom divider, dark / light.
pub const TITLEBAR_DIVIDER_DARK: Color = Color::from_rgba8(255, 255, 255, 36);
pub const TITLEBAR_DIVIDER_LIGHT: Color = Color::from_rgba8(0, 0, 0, 31);

/// Titlebar height in logical px.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitlebarHeight {
    /// 31 px default (17 px button size + 14 px padding).
    Standard,
    /// 44 px Mac variant.
    Mac,
}

impl TitlebarHeight {
    pub fn px(self) -> f32 {
        match self {
            TitlebarHeight::Standard => 31.0,
            TitlebarHeight::Mac => 44.0,
        }
    }
}

/// Custom decoration bar: opaque background with top-only rounded corners,
/// 1 px bottom divider and a centered semibold title. No traffic lights;
/// dragging is handled by the shell through [`Titlebar::bounds`].
pub struct Titlebar {
    title: String,
    height: TitlebarHeight,
    x: f32,
    y: f32,
    width: f32,
    layout: Option<Layout<SolidBrush>>,
    dirty: bool,
}

impl Titlebar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            height: TitlebarHeight::Standard,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            layout: None,
            dirty: true,
        }
    }

    pub fn height(mut self, height: TitlebarHeight) -> Self {
        self.height = height;
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if title != self.title {
            self.title = title;
            self.dirty = true;
        }
    }

    /// Place the bar. Width usually spans the content viewport.
    pub fn set_rect(&mut self, x: f32, y: f32, width: f32) {
        if width != self.width {
            self.dirty = true;
        }
        self.x = x;
        self.y = y;
        self.width = width;
    }

    /// Logical hit rect for window dragging: (x, y, width, height).
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height.px())
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if self.dirty || self.layout.is_none() {
            self.layout = Some(fonts.layout_text_weighted(
                &self.title,
                13.0,
                TITLEBAR_TEXT_DARK,
                600.0,
                None,
            ));
            self.dirty = false;
        }
    }

    pub fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let bar_h = self.height.px();
        let radius = WINDOW_CORNER_RADIUS as f64 * scale;

        let bg = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.width),
            px(self.y + bar_h),
            RoundedRectRadii::new(radius, radius, 0.0, 0.0),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(TITLEBAR_BG_DARK),
            None,
            &bg,
        );

        // 1 px bottom divider, stopping before the rounded corners.
        let divider = Line::new(
            Point::new(px(self.x) + radius, px(self.y + bar_h) - 0.5 * scale),
            Point::new(
                px(self.x + self.width) - radius,
                px(self.y + bar_h) - 0.5 * scale,
            ),
        );
        scene.stroke(
            &Stroke::new(scale),
            Affine::IDENTITY,
            &Brush::Solid(TITLEBAR_DIVIDER_DARK),
            None,
            &divider,
        );

        self.ensure_layout(fonts);
        let layout = self.layout.as_ref().expect("layout built");
        let (tw, th) = FontSystem::layout_size(layout);
        let tx = self.x + (self.width - tw / fonts.scale) / 2.0;
        let ty = self.y + (bar_h - th / fonts.scale) / 2.0;
        draw_layout(scene, layout, tx, ty, fonts.scale);
    }
}
