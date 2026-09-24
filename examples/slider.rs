use tontooui::elements::{HStack, Slider, Titlebar, TrafficAction, View, VStack};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct SliderDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
    dragging: bool,
}

impl SliderDemo {
    fn new() -> Self {
        let colors = HStack::new()
            .spacing(16.0)
            .child(Slider::new(0.6, 0.0, 1.0).fill(Color::from_rgb8(0xff, 0x2d, 0x55)))
            .child(Slider::new(0.6, 0.0, 1.0).fill(Color::from_rgb8(0x34, 0xc7, 0x59)))
            .child(Slider::new(0.6, 0.0, 1.0).fill(Color::from_rgb8(0xaf, 0x52, 0xde)));
        let stack = VStack::new()
            .spacing(20.0)
            .child(
                Slider::new(0.5, 0.0, 1.0).value_text(|v| format!("Basic: {v:.2}")),
            )
            .child(
                Slider::new(10.0, 0.0, 100.0)
                    .step(10.0)
                    .show_ticks(true)
                    .value_text(|v| format!("Value: {v:.0}")),
            )
            .child(
                Slider::new(50.0, 0.0, 100.0)
                    .title("Temperature")
                    .min_label("0°")
                    .max_label("100°")
                    .value_text(|v| format!("Value: {v:.0}°")),
            )
            .child(
                Slider::new(3.0, 1.0, 5.0)
                    .step(1.0)
                    .show_ticks(true)
                    .value_text(|v| format!("Rating: {v:.0}/5")),
            )
            .child(colors)
            .child(
                Slider::new(0.6, 0.0, 1.0)
                    .glass(true)
                    .value_text(|v| format!("Glass: {:.0}%", v * 100.0)),
            );
        Self {
            bar: Titlebar::new("Slider"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
            dragging: false,
        }
    }

    fn each_slider(&mut self, mut f: impl FnMut(&mut Slider)) {
        let mut index = 0;
        loop {
            if let Some(slider) = self.stack.child_mut::<Slider>(index) {
                f(slider);
            } else if let Some(row) = self.stack.child_mut::<HStack>(index) {
                let mut inner = 0;
                loop {
                    match row.child_mut::<Slider>(inner) {
                        Some(slider) => f(slider),
                        None => break,
                    }
                    inner += 1;
                }
            } else {
                break;
            }
            index += 1;
        }
    }
}

impl App for SliderDemo {
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
        // Copy out before the mutable walk.
        let theme = self.watcher.theme();
        let dark = theme.mode == ThemeMode::Dark;
        let focused = self.focused;
        self.each_slider(|slider| {
            slider.set_theme(palette.accent, dark, theme.glass);
            slider.set_focused(focused);
        });

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        self.stack.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0,
            viewport.width - 48.0,
            (viewport.height - 47.0).max(0.0),
        );
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

    fn wants_backdrop(&self) -> bool {
        self.dragging
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => {
                let mut dragging = false;
                self.each_slider(|slider| {
                    slider.mouse_down(x, y);
                    if slider.is_dragging() {
                        dragging = true;
                    }
                });
                self.dragging = dragging;
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        let mut dragging = false;
        self.each_slider(|slider| {
            slider.mouse_move(x, y);
            if slider.is_dragging() {
                dragging = true;
            }
        });
        if dragging {
            self.dragging = true;
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_slider(|slider| slider.mouse_up(x, y));
        self.dragging = false;
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Slider", 900, 720, SliderDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
