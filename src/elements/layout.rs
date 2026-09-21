use std::any::Any;

use vello::Scene;

use crate::renderer::text::FontSystem;

/// Cross-axis alignment inside stacks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Leading,
    Center,
    Trailing,
}

/// Layout protocol. Inherent `draw` methods on concrete elements take
/// precedence over the trait method, so existing direct callers keep working
/// while stacks use the trait through `Box<dyn Element>`.
pub trait Element {
    /// Intrinsic logical size.
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32);
    /// Assign a logical rect. Stacks measure first (fonts needed for text),
    /// then pass intrinsic size except for flex children, which share the
    /// remaining space.
    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32);
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem);
    /// Share of remaining space. Zero means fixed intrinsic size.
    fn flex(&self) -> f32 {
        0.0
    }
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Vertical stack. Children keep intrinsic size; flex children share the
/// leftover height. Cross-axis width is capped at the stack width and
/// positioned per `align`.
pub struct VStack {
    spacing: f32,
    align: Align,
    children: Vec<Box<dyn Element>>,
}

/// Horizontal stack. Mirrors `VStack` along the x axis.
pub struct HStack {
    spacing: f32,
    align: Align,
    children: Vec<Box<dyn Element>>,
}

/// Overlay stack. All children share the same rect at intrinsic size,
/// positioned per `align` on both axes.
pub struct ZStack {
    align: Align,
    children: Vec<Box<dyn Element>>,
}

/// Flexible empty space. Takes a share of the remaining stack space
/// proportional to `factor`.
pub struct Spacer {
    min: f32,
    factor: f32,
}

macro_rules! stack_boilerplate {
    ($name:ident) => {
        impl $name {
            pub fn new() -> Self {
                Self {
                    spacing: 8.0,
                    align: Align::Leading,
                    children: Vec::new(),
                }
            }

            pub fn spacing(mut self, px: f32) -> Self {
                self.spacing = px;
                self
            }

            pub fn align(mut self, align: Align) -> Self {
                self.align = align;
                self
            }

            pub fn child(mut self, child: impl Element + 'static) -> Self {
                self.children.push(Box::new(child));
                self
            }

            /// Access a child by index for state updates (typing, toggles).
            pub fn child_mut<T: Element + 'static>(
                &mut self,
                index: usize,
            ) -> Option<&mut T> {
                self.children
                    .get_mut(index)?
                    .as_any_mut()
                    .downcast_mut::<T>()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

stack_boilerplate!(VStack);
stack_boilerplate!(HStack);

impl ZStack {
    pub fn new() -> Self {
        Self {
            align: Align::Center,
            children: Vec::new(),
        }
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn child_mut<T: Element + 'static>(&mut self, index: usize) -> Option<&mut T> {
        self.children
            .get_mut(index)?
            .as_any_mut()
            .downcast_mut::<T>()
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Spacer {
    pub fn new() -> Self {
        Self {
            min: 0.0,
            factor: 1.0,
        }
    }

    pub fn min_size(mut self, px: f32) -> Self {
        self.min = px;
        self
    }

    pub fn factor(mut self, factor: f32) -> Self {
        self.factor = factor;
        self
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Spacer {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.min, self.min)
    }

    fn place(&mut self, _fonts: &mut FontSystem, _x: f32, _y: f32, _w: f32, _h: f32) {}

    fn draw(&mut self, _scene: &mut Scene, _fonts: &mut FontSystem) {}

    fn flex(&self) -> f32 {
        self.factor
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn cross_offset(align: Align, total: f32, used: f32) -> f32 {
    match align {
        Align::Leading => 0.0,
        Align::Center => ((total - used) / 2.0).max(0.0),
        Align::Trailing => (total - used).max(0.0),
    }
}

impl Element for VStack {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        let n = self.children.len();
        let spacing = self.spacing;
        for (i, child) in self.children.iter_mut().enumerate() {
            let (cw, ch) = child.measure(fonts);
            w = w.max(cw);
            h += ch;
            if i + 1 < n {
                h += spacing;
            }
        }
        (w, h)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
        let mut sizes = Vec::with_capacity(self.children.len());
        let mut fixed = 0.0;
        let mut total_flex = 0.0;
        for child in self.children.iter_mut() {
            let size = child.measure(fonts);
            if child.flex() > 0.0 {
                total_flex += child.flex();
            } else {
                fixed += size.1;
            }
            sizes.push(size);
        }
        if !self.children.is_empty() {
            fixed += self.spacing * (self.children.len() - 1) as f32;
        }
        let remaining = (height - fixed).max(0.0);
        let align = self.align;
        let spacing = self.spacing;

        let mut cy = y;
        for (child, size) in self.children.iter_mut().zip(sizes.iter()) {
            let child_h = if child.flex() > 0.0 && total_flex > 0.0 {
                remaining * child.flex() / total_flex
            } else {
                size.1
            };
            let child_w = size.0.min(width);
            child.place(fonts, x + cross_offset(align, width, child_w), cy, child_w, child_h);
            cy += child_h + spacing;
        }
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        for child in self.children.iter_mut() {
            child.draw(scene, fonts);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Element for HStack {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        let n = self.children.len();
        let spacing = self.spacing;
        for (i, child) in self.children.iter_mut().enumerate() {
            let (cw, ch) = child.measure(fonts);
            w += cw;
            h = h.max(ch);
            if i + 1 < n {
                w += spacing;
            }
        }
        (w, h)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
        let mut sizes = Vec::with_capacity(self.children.len());
        let mut fixed = 0.0;
        let mut total_flex = 0.0;
        for child in self.children.iter_mut() {
            let size = child.measure(fonts);
            if child.flex() > 0.0 {
                total_flex += child.flex();
            } else {
                fixed += size.0;
            }
            sizes.push(size);
        }
        if !self.children.is_empty() {
            fixed += self.spacing * (self.children.len() - 1) as f32;
        }
        let remaining = (width - fixed).max(0.0);
        let align = self.align;
        let spacing = self.spacing;

        let mut cx = x;
        for (child, size) in self.children.iter_mut().zip(sizes.iter()) {
            let child_w = if child.flex() > 0.0 && total_flex > 0.0 {
                remaining * child.flex() / total_flex
            } else {
                size.0
            };
            let child_h = size.1.min(height);
            child.place(fonts, cx, y + cross_offset(align, height, child_h), child_w, child_h);
            cx += child_w + spacing;
        }
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        for child in self.children.iter_mut() {
            child.draw(scene, fonts);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Element for ZStack {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        for child in self.children.iter_mut() {
            let (cw, ch) = child.measure(fonts);
            w = w.max(cw);
            h = h.max(ch);
        }
        (w, h)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
        let mut sizes = Vec::with_capacity(self.children.len());
        for child in self.children.iter_mut() {
            sizes.push(child.measure(fonts));
        }
        let align = self.align;
        for (child, size) in self.children.iter_mut().zip(sizes.iter()) {
            let cw = size.0.min(width);
            let ch = size.1.min(height);
            child.place(
                fonts,
                x + cross_offset(align, width, cw),
                y + cross_offset(align, height, ch),
                cw,
                ch,
            );
        }
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        for child in self.children.iter_mut() {
            child.draw(scene, fonts);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
