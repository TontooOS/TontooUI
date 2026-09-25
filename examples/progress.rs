use tontooui::elements::{LinearProgress, Titlebar, TrafficAction, View, VStack};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct ProgressDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ProgressDemo {
    fn new() -> Self {
        // Like the reference: pink bar fed by the app.
        let stack = VStack::new().spacing(24.0).child(
            LinearProgress::new()
                .speed(0.12)
                .fill(Color::from_rgb8(0xff, 0x2d, 0x99)),
        );
        Self {
            bar: Titlebar::new("Progress"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_bar(&mut self, mut f: impl FnMut(&mut LinearProgress)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<LinearProgress>(index) {
                Some(bar) => f(bar),
                None => break,
            }
            index += 1;
        }
    }
}

impl App for ProgressDemo {
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
        self.each_bar(|bar| {
            bar.set_theme(palette.accent, dark);
            bar.set_focused(focused);
        });

        // Fake app: reports jumpy progress with stalls every 6 s. The
        // bar chases slowly and glides through the stalls.
        let cycle = time_secs % 12.0;
        let target = if cycle < 6.0 { cycle / 6.0 * 0.7 } else { 0.7 + (cycle - 6.0) / 6.0 * 0.3 };
        self.each_bar(|bar| {
            if bar.displayed() >= 1.0 && target < 0.05 {
                bar.set_progress(0.0);
            } else {
                bar.set_progress(target);
            }
        });
        self.bar.set_title(format!("Progress — {:.0}%", target * 100.0));

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
        self.each_bar(|bar| bar.set_focused(focused));
    }
}

fn main() {
    if let Err(err) = run("Progress", 900, 200, ProgressDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
