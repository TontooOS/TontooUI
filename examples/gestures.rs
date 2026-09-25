use std::cell::RefCell;
use std::rc::Rc;

use tontooui::elements::{
    Align, BasicText, Circle, GestureArea, HStack, Rectangle, SFSymbolImage,
    Titlebar, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct GesturesDemo {
    bar: Titlebar,
    stack: VStack,
    taps: Rc<RefCell<u32>>,
    longs: Rc<RefCell<u32>>,
    doubles: Rc<RefCell<u32>>,
    hovers: Rc<RefCell<u32>>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl GesturesDemo {
    fn new() -> Self {
        let taps: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let longs: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let doubles: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let hovers: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let tap_count = taps.clone();
        let long_count = longs.clone();
        let double_count = doubles.clone();
        let hover_count = hovers.clone();
        // Tap pad: quick press and release counts up; a quick pair
        // additionally doubles.
        let tap = GestureArea::new(
            Rectangle::new(220.0, 110.0)
                .fill(Color::from_rgb8(0x00, 0x7a, 0xff)),
        )
        .on_tap(move || {
            *tap_count.borrow_mut() += 1;
        })
        .on_double_tap(move || {
            *double_count.borrow_mut() += 1;
        });
        // Long-press pad: hold without wandering.
        let long = GestureArea::new(
            Circle::new(110.0).fill(Color::from_rgb8(0x34, 0xc7, 0x59)),
        )
        .on_long_press(move || {
            *long_count.borrow_mut() += 1;
        });
        // Drag pad: the rectangle follows the pointer.
        let drag = GestureArea::new(
            Rectangle::new(160.0, 90.0)
                .fill(Color::from_rgb8(0xaf, 0x52, 0xde)),
        )
        .draggable(true);
        // Magnify pad: the wheel zooms the symbol around the center.
        let zoom = GestureArea::new(
            SFSymbolImage::new("star.fill")
                .size(72.0)
                .color(Color::from_rgb8(0xff, 0x9f, 0x0a)),
        )
        .zoomable(true);
        // Hover pad: entering the rect counts up (exit passes false).
        let hover = GestureArea::new(
            Rectangle::new(160.0, 90.0)
                .fill(Color::from_rgb8(0x30, 0xb0, 0xc7)),
        )
        .on_hover(move |inside| {
            if inside {
                *hover_count.borrow_mut() += 1;
            }
        });
        let stack = VStack::new()
            .align(Align::Center)
            .spacing(20.0)
            .child(
                HStack::new()
                    .align(Align::Center)
                    .spacing(32.0)
                    .child(
                        VStack::new()
                            .align(Align::Center)
                            .spacing(8.0)
                            .child(BasicText::new("Tap / double tap"))
                            .child(tap),
                    )
                    .child(
                        VStack::new()
                            .align(Align::Center)
                            .spacing(8.0)
                            .child(BasicText::new("Long press"))
                            .child(long),
                    ),
            )
            .child(
                HStack::new()
                    .align(Align::Center)
                    .spacing(32.0)
                    .child(
                        VStack::new()
                            .align(Align::Center)
                            .spacing(8.0)
                            .child(BasicText::new("Drag me"))
                            .child(drag),
                    )
                    .child(
                        VStack::new()
                            .align(Align::Center)
                            .spacing(8.0)
                            .child(BasicText::new("Wheel to zoom"))
                            .child(zoom),
                    ),
            )
            .child(
                HStack::new()
                    .align(Align::Center)
                    .spacing(32.0)
                    .child(
                        VStack::new()
                            .align(Align::Center)
                            .spacing(8.0)
                            .child(BasicText::new("Hover me"))
                            .child(hover),
                    ),
            )
            .child(BasicText::new(""));
        Self {
            bar: Titlebar::new("Gestures"),
            stack,
            taps,
            longs,
            doubles,
            hovers,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn status_index(&mut self) -> Option<usize> {
        // Status line is the last stack child.
        let len = self.stack.len();
        if len == 0 {
            None
        } else {
            Some(len - 1)
        }
    }
}

impl App for GesturesDemo {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    ) {
        self.watcher.poll(time_secs);
        self.watcher.set_focused(self.focused, time_secs);
        let palette = self.watcher.palette(time_secs);
        self.bg = palette.bg;
        let theme = self.watcher.theme();
        let dark = theme.mode == ThemeMode::Dark;
        let focused = self.focused;

        // Theme the labels and the symbol; read live gesture state
        // for the status line.
        let mut drag_text = String::new();
        let mut zoom_text = String::new();
        for row in 0..self.stack.len() {
            if let Some(hstack) = self.stack.child_mut::<HStack>(row) {
                for col in 0..hstack.len() {
                    if let Some(col) = hstack.child_mut::<VStack>(col) {
                        for index in 0..col.len() {
                            if let Some(label) =
                                col.child_mut::<BasicText>(index)
                            {
                                label.set_theme(theme.mode);
                                label.set_focused(focused);
                            } else if let Some(area) = col
                                .child_mut::<GestureArea<Rectangle>>(index)
                            {
                                let (dx, dy) = area.drag_offset();
                                if dx != 0.0 || dy != 0.0 {
                                    drag_text =
                                        format!("drag ({dx:.0}, {dy:.0})");
                                }
                            } else if let Some(area) = col
                                .child_mut::<GestureArea<SFSymbolImage>>(index)
                            {
                                area.child_mut().set_theme(palette.text, dark);
                                area.child_mut().set_focused(focused);
                                let s = area.scale_value();
                                if (s - 1.0).abs() > 0.001 {
                                    zoom_text = format!("zoom {s:.2}x");
                                }
                            } else if let Some(area) = col
                                .child_mut::<GestureArea<Circle>>(index)
                            {
                                area.child_mut().set_focused(focused);
                            }
                        }
                    }
                }
            }
        }
        let status = format!(
            "taps {}   doubles {}   longs {}   hovers {}{} {}",
            self.taps.borrow(),
            self.doubles.borrow(),
            self.longs.borrow(),
            self.hovers.borrow(),
            if drag_text.is_empty() {
                String::new()
            } else {
                format!("   {drag_text}")
            },
            zoom_text,
        );
        if let Some(last) = self.status_index() {
            if let Some(line) = self.stack.child_mut::<BasicText>(last) {
                line.set_text(status);
                line.set_theme(theme.mode);
                line.set_focused(focused);
            }
        }

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let (stack_w, stack_h) = self.stack.measure(fonts);
        let x = viewport.x + ((viewport.width - stack_w) / 2.0).max(0.0);
        self.stack.place(fonts, x, top + 20.0, stack_w, stack_h);
        self.stack.draw(scene, fonts, images);
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        Some(self.bar.drag_rect())
    }

    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        self.command.take()
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => {
                self.command = Some(WindowCommand::Minimize)
            }
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => self.forward_gesture(x, y, 0),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.forward_gesture(x, y, 1);
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.forward_gesture(x, y, 2);
    }

    fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        for row in 0..self.stack.len() {
            let mut hit = false;
            if let Some(hstack) = self.stack.child_mut::<HStack>(row) {
                for col in 0..hstack.len() {
                    if let Some(col) = hstack.child_mut::<VStack>(col) {
                        for index in 0..col.len() {
                            if let Some(area) = col
                                .child_mut::<GestureArea<SFSymbolImage>>(index)
                            {
                                area.mouse_wheel(dx, dy);
                                hit = true;
                            }
                        }
                    }
                }
            }
            let _ = hit;
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

impl GesturesDemo {
    /// Forward presses to every gesture area (0 down, 1 up, 2 move).
    fn forward_gesture(&mut self, x: f64, y: f64, kind: u8) {
        for row in 0..self.stack.len() {
            if let Some(hstack) = self.stack.child_mut::<HStack>(row) {
                for col in 0..hstack.len() {
                    if let Some(col) = hstack.child_mut::<VStack>(col) {
                        for index in 0..col.len() {
                            if let Some(area) =
                                col.child_mut::<GestureArea<Rectangle>>(index)
                            {
                                match kind {
                                    0 => area.mouse_down(x, y),
                                    1 => area.mouse_up(x, y),
                                    _ => area.mouse_move(x, y),
                                }
                            } else if let Some(area) =
                                col.child_mut::<GestureArea<Circle>>(index)
                            {
                                match kind {
                                    0 => area.mouse_down(x, y),
                                    1 => area.mouse_up(x, y),
                                    _ => area.mouse_move(x, y),
                                }
                            } else if let Some(area) = col
                                .child_mut::<GestureArea<SFSymbolImage>>(index)
                            {
                                match kind {
                                    0 => area.mouse_down(x, y),
                                    1 => area.mouse_up(x, y),
                                    _ => area.mouse_move(x, y),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    if let Err(err) = run("Gestures", 900, 900, GesturesDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
