use parley::Layout;
use vello::Scene;
use vello::kurbo::{Affine, Circle, Line, Point, RoundedRect, RoundedRectRadii, Stroke};
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

/// Traffic light button size, spacing and left margin in logical px.
pub const TRAFFIC_SIZE: f32 = 12.0;
pub const TRAFFIC_GAP: f32 = 10.0;
pub const TRAFFIC_LEFT: f32 = 18.0;

/// Active traffic light colors: close, minimize, maximize.
pub const TRAFFIC_CLOSE: Color = Color::from_rgb8(0xff, 0x5f, 0x56);
pub const TRAFFIC_MINIMIZE: Color = Color::from_rgb8(0xff, 0xbd, 0x2e);
pub const TRAFFIC_MAXIMIZE: Color = Color::from_rgb8(0x27, 0xc9, 0x3f);
/// Inactive (unfocused window) traffic light color.
pub const TRAFFIC_INACTIVE: Color = Color::from_rgb8(0x88, 0x88, 0x88);
/// Glyph color drawn on hover, 68% of the button size.
pub const TRAFFIC_GLYPH: Color = Color::from_rgba8(0, 0, 0, 150);

/// Traffic light action triggered by click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrafficAction {
    Close,
    Minimize,
    Maximize,
}

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
    hover: bool,
    focused: bool,
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
            hover: false,
            focused: true,
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

    /// Drag rect minus the left traffic light cluster, so button clicks
    /// never start a window drag.
    pub fn drag_rect(&self) -> (f32, f32, f32, f32) {
        let cut = TRAFFIC_LEFT + TRAFFIC_SIZE * 3.0 + TRAFFIC_GAP * 2.0;
        (
            self.x + cut,
            self.y,
            (self.width - cut).max(0.0),
            self.height.px(),
        )
    }

    /// Center of traffic light `index` (0 close, 1 minimize, 2 maximize).
    fn button_center(&self, index: usize) -> (f32, f32) {
        let cx = self.x + TRAFFIC_LEFT + TRAFFIC_SIZE / 2.0
            + index as f32 * (TRAFFIC_SIZE + TRAFFIC_GAP);
        let cy = self.y + self.height.px() / 2.0;
        (cx, cy)
    }

    fn button_at(&self, x: f32, y: f32) -> Option<TrafficAction> {
        let actions = [
            TrafficAction::Close,
            TrafficAction::Minimize,
            TrafficAction::Maximize,
        ];
        for (index, action) in actions.iter().enumerate() {
            let (cx, cy) = self.button_center(index);
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= (TRAFFIC_SIZE / 2.0 + 3.0).powi(2) {
                return Some(*action);
            }
        }
        None
    }

    /// Update group hover from logical cursor position.
    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.hover = self.button_at(x, y).is_some();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Click handling. Returns the traffic light action when a button was
    /// hit, otherwise `None`.
    pub fn press(&mut self, x: f32, y: f32) -> Option<TrafficAction> {
        self.button_at(x, y)
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

        let colors = if self.focused {
            [TRAFFIC_CLOSE, TRAFFIC_MINIMIZE, TRAFFIC_MAXIMIZE]
        } else {
            [TRAFFIC_INACTIVE; 3]
        };
        for (index, color) in colors.iter().enumerate() {
            let (cx, cy) = self.button_center(index);
            let circle = Circle::new(
                (px(cx), px(cy)),
                (TRAFFIC_SIZE / 2.0 * fonts.scale) as f64,
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(*color),
                None,
                &circle,
            );
            if self.hover {
                self.draw_glyph(scene, index, px(cx), px(cy), scale);
            }
        }
    }

    /// Hover glyph at physical center (`cx`, `cy`): x, minus or plus.
    /// Glyph size is 68% of the button diameter.
    fn draw_glyph(&self, scene: &mut Scene, index: usize, cx: f64, cy: f64, scale: f64) {
        let half = TRAFFIC_SIZE as f64 * 0.68 / 2.0 * scale;
        let style = Stroke::new(1.3 * scale);
        let brush = Brush::Solid(TRAFFIC_GLYPH);
        let mut line = |x0: f64, y0: f64, x1: f64, y1: f64| {
            scene.stroke(
                &style,
                Affine::IDENTITY,
                &brush,
                None,
                &Line::new(Point::new(x0, y0), Point::new(x1, y1)),
            );
        };
        match index {
            0 => {
                line(cx - half, cy - half, cx + half, cy + half);
                line(cx - half, cy + half, cx + half, cy - half);
            }
            1 => {
                line(cx - half, cy, cx + half, cy);
            }
            _ => {
                line(cx - half, cy, cx + half, cy);
                line(cx, cy - half, cx, cy + half);
            }
        }
    }
}
