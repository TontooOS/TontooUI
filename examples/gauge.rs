use tontooui::elements::{Gauge, LinearGauge, Titlebar, TrafficAction, View, VStack};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct GaugeDemo {
    bar: Titlebar,
    stack: VStack,
    linear: LinearGauge,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl GaugeDemo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(24.0)
            .child(
                Gauge::new(0.6, 0.0, 1.0)
                    .title("Progress")
                    .fill(Color::from_rgb8(0x34, 0xc7, 0x59)),
            )
            .child(Gauge::new(0.35, 0.0, 1.0).title("Accent follows theme"))
            .child(
                Gauge::new(82.0, 0.0, 100.0)
                    .title("Storage")
                    .fill(Color::from_rgb8(0xff, 0x2d, 0x55)),
            )
            .child(
                Gauge::new(72.0, 0.0, 100.0)
                    .title("Temperature")
                    .min_label("0°")
                    .max_label("100°")
                    .value_text(|v| format!("{v:.0}°"))
                    .fill(Color::from_rgb8(0x34, 0xc7, 0x59)),
            );
        Self {
            bar: Titlebar::new("Gauge"),
            stack,
            linear: LinearGauge::new(60.0, 0.0, 100.0).value_text(|v| format!("{v:.0}%")),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_gauge(&mut self, mut f: impl FnMut(&mut Gauge)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<Gauge>(index) {
                Some(gauge) => f(gauge),
                None => break,
            }
            index += 1;
        }
    }

}

impl App for GaugeDemo {
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
        self.each_gauge(|gauge| {
            gauge.set_theme(palette.accent, dark);
            gauge.set_focused(focused);
        });
        self.linear.set_theme(palette.accent, dark);
        self.linear.set_focused(focused);

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let (_, stack_h) = self.stack.measure(fonts);
        self.stack.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0,
            viewport.width - 48.0,
            stack_h,
        );
        self.stack.draw(scene, fonts, images);
        self.linear.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0 + stack_h + 24.0,
            viewport.width - 48.0,
            40.0,
        );
        self.linear.draw(scene, fonts, images);
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
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
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
        self.each_gauge(|gauge| gauge.set_focused(focused));
        self.linear.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Gauge", 900, 420, GaugeDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
