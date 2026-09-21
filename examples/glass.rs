use tontooui::elements::{GlassContainer, Titlebar, TrafficAction, View};
use tontooui::renderer::ImageLoader;
use tontooui::theme::ThemeWatcher;
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use vello::Scene;

struct GlassDemo {
    bar: Titlebar,
    glass: GlassContainer,
    watcher: ThemeWatcher,
    focused: bool,
    command: Option<WindowCommand>,
}

impl GlassDemo {
    fn new() -> Self {
        Self {
            bar: Titlebar::new("Liquid Glass"),
            glass: GlassContainer::new().radius(28.0),
            watcher: ThemeWatcher::new(),
            focused: true,
            command: None,
        }
    }
}

impl App for GlassDemo {
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
        let theme = self.watcher.theme();

        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        // Glass stage follows the system setting live.
        let gw = 440.0;
        let gh = 220.0;
        let gx = viewport.x + (viewport.width - gw) / 2.0;
        let gy = viewport.y + 31.0 + (viewport.height - 31.0 - gh) / 2.0;
        self.glass.set_bounds(gx, gy, gw, gh);
        self.glass.set_theme(theme.mode, theme.glass);
        self.glass.set_focused(self.focused);
        self.glass.draw(scene, fonts, images);
    }

    fn transparent_body(&self) -> bool {
        true
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
    }
}

fn main() {
    if let Err(err) = run("Liquid Glass", 800, 600, GlassDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
