use std::any::Any;

use parley::Layout;
use vello::Scene;
use vello::peniko::Color;

use super::layout::View;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};

/// Static text label. Layout is cached and rebuilt when content, size,
/// color or wrap width change.
pub struct Text {
    content: String,
    size: f32,
    color: Color,
    x: f32,
    y: f32,
    max_width: Option<f32>,
    layout: Option<Layout<SolidBrush>>,
    dirty: bool,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            size: 15.0,
            color: Color::WHITE,
            x: 0.0,
            y: 0.0,
            max_width: None,
            layout: None,
            dirty: true,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.dirty = true;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self.dirty = true;
        self
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn wrap(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self.dirty = true;
        self
    }

    /// Move without rebuilding the layout.
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    /// Recolor without moving. Marks the layout dirty on change (used for
    /// live theme and accent updates).
    pub fn set_color(&mut self, color: Color) {
        if color != self.color {
            self.color = color;
            self.dirty = true;
        }
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        let content = content.into();
        if content != self.content {
            self.content = content;
            self.dirty = true;
        }
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if self.dirty || self.layout.is_none() {
            self.layout = Some(fonts.layout_text(
                &self.content,
                self.size,
                self.color,
                self.max_width,
            ));
            self.dirty = false;
        }
    }

    /// Logical size of the laid out text.
    pub fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.measured_size(fonts)
    }

    pub fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        self.render(scene, fonts);
    }

    fn measured_size(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.ensure_layout(fonts);
        let layout = self.layout.as_ref().expect("layout built");
        let scale = fonts.scale;
        (
            FontSystem::layout_size(layout).0 / scale,
            FontSystem::layout_size(layout).1 / scale,
        )
    }

    fn render(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        self.ensure_layout(fonts);
        let layout = self.layout.as_ref().expect("layout built");
        draw_layout(scene, layout, self.x, self.y, fonts.scale);
    }
}

impl View for Text {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.measured_size(fonts)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, _w: f32, _h: f32) {
        self.set_position(x, y);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        self.render(scene, fonts);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
