use tontooui::elements::{
    Capsule, Circle, CustomShape, HStack, Rectangle, RoundedRectangle, Titlebar,
    TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::ThemeWatcher;
use vello::Scene;
use vello::peniko::Color;

fn blue() -> Color {
    Color::from_rgb8(0x0b, 0x5c, 0xe6)
}

fn purple() -> Color {
    Color::from_rgb8(0xaf, 0x52, 0xde)
}

struct ShapesDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ShapesDemo {
    fn new() -> Self {
        // Reference rows per shape: solid fill, outline (unfilled),
        // gradient fill.
        let rects = HStack::new()
            .spacing(32.0)
            .child(Rectangle::new(240.0, 120.0).fill(blue()))
            .child(
                Rectangle::new(240.0, 120.0)
                    .stroke(Color::from_rgb8(0xee, 0x2c, 0x2c))
                    .filled(false),
            )
            .child(Rectangle::new(240.0, 120.0).linear_gradient(
                vec![Color::from_rgb8(0x1e, 0x6f, 0xf2), purple()],
                0.0,
            ));
        let circles = HStack::new()
            .spacing(32.0)
            .child(Circle::new(120.0))
            .child(
                Circle::new(120.0)
                    .stroke(Color::from_rgb8(0xff, 0x95, 0x00))
                    .filled(false),
            )
            .child(Circle::new(120.0).radial_gradient(vec![
                Color::WHITE,
                Color::from_rgb8(0x2e, 0x7c, 0xf6),
            ]));
        let capsules = HStack::new()
            .spacing(32.0)
            .child(Capsule::new(240.0, 90.0))
            .child(
                Capsule::new(240.0, 90.0)
                    .stroke(Color::from_rgb8(0x58, 0x56, 0xd6))
                    .filled(false),
            )
            .child(Capsule::new(240.0, 90.0).linear_gradient(
                vec![
                    Color::from_rgb8(0xff, 0xcc, 0x00),
                    Color::from_rgb8(0xff, 0x7a, 0x00),
                ],
                0.0,
            ));
        let rounded = HStack::new()
            .spacing(32.0)
            .child(RoundedRectangle::new(240.0, 120.0, 24.0).fill(blue()))
            .child(
                RoundedRectangle::new(240.0, 120.0, 24.0)
                    .stroke(Color::from_rgb8(0xee, 0x2c, 0x2c))
                    .filled(false),
            )
            .child(
                RoundedRectangle::new(240.0, 120.0, 24.0).linear_gradient(
                    vec![Color::from_rgb8(0x1e, 0x6f, 0xf2), purple()],
                    0.0,
                ),
            );
        let custom = HStack::new()
            .spacing(32.0)
            .child(CustomShape::triangle(120.0, 110.0).fill(blue()))
            .child(CustomShape::hexagon(120.0, 110.0).fill(purple()))
            .child(
                CustomShape::star(120.0, 110.0).linear_gradient(
                    vec![
                        Color::from_rgb8(0xff, 0xcc, 0x00),
                        Color::from_rgb8(0xff, 0x7a, 0x00),
                    ],
                    90.0,
                ),
            )
            .child(
                CustomShape::diamond(120.0, 110.0)
                    .stroke(Color::from_rgb8(0x58, 0x56, 0xd6))
                    .filled(false),
            );
        let stack = VStack::new()
            .spacing(28.0)
            .child(rects)
            .child(circles)
            .child(capsules)
            .child(rounded)
            .child(custom);
        Self {
            bar: Titlebar::new("Shapes"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn set_row_focused(row: &mut HStack, focused: bool) {
        for index in 0..row.len() {
            if let Some(shape) = row.child_mut::<Rectangle>(index) {
                shape.set_focused(focused);
            } else if let Some(shape) = row.child_mut::<Circle>(index) {
                shape.set_focused(focused);
            } else if let Some(shape) = row.child_mut::<RoundedRectangle>(index)
            {
                shape.set_focused(focused);
            } else if let Some(shape) = row.child_mut::<Capsule>(index) {
                shape.set_focused(focused);
            } else if let Some(shape) = row.child_mut::<CustomShape>(index) {
                shape.set_focused(focused);
            }
        }
    }

    fn each_row(&mut self, mut f: impl FnMut(&mut HStack)) {
        for index in 0..self.stack.len() {
            if let Some(row) = self.stack.child_mut::<HStack>(index) {
                f(row);
            }
        }
    }
}

impl App for ShapesDemo {
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
        let focused = self.focused;
        self.each_row(|row| Self::set_row_focused(row, focused));

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
            None => {}
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Shapes", 900, 900, ShapesDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
