use tontooui::elements::{
    Align, BasicToolbar, Titlebar, ToolbarPlacement, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::ThemeWatcher;
use vello::Scene;
use vello::peniko::Color;

struct ToolbarDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ToolbarDemo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(16.0)
            .align(Align::Center)
            .child(
                BasicToolbar::from_icons(vec![
                    "chevron.left".to_string(),
                    "line.3.horizontal".to_string(),
                ])
                .placement(ToolbarPlacement::Leading)
                .on_action(|index| println!("basic toolbar action {index}")),
            )
            .child(
                BasicToolbar::from_icons(vec!["heart".to_string()])
                    .placement(ToolbarPlacement::Center),
            )
            .child(
                BasicToolbar::from_icons(vec![
                    "xmark".to_string(),
                    "star".to_string(),
                    "checkmark".to_string(),
                ])
                .placement(ToolbarPlacement::Trailing),
            );
        Self {
            bar: Titlebar::new("Toolbar"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_toolbar(&mut self, mut f: impl FnMut(&mut BasicToolbar)) {
        for index in 0..3 {
            if let Some(bar) = self.stack.child_mut::<BasicToolbar>(index) {
                f(bar);
            }
        }
    }
}

impl App for ToolbarDemo {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut tontooui::renderer::ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    ) {
        self.watcher.poll(time_secs);
        self.watcher.set_focused(self.focused, time_secs);
        let palette = self.watcher.palette(time_secs);
        self.bg = palette.bg;
        let theme = self.watcher.theme();
        let focused = self.focused;
        self.each_toolbar(|bar| {
            bar.set_theme(theme.mode, theme.glass);
            bar.set_focused(focused);
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
            viewport.x,
            top + 24.0,
            viewport.width,
            (viewport.height - 55.0).max(0.0),
        );
        self.stack.draw(scene, fonts, images);
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn wants_backdrop(&self) -> bool {
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
            None => self.each_toolbar(|bar| bar.mouse_down(x, y)),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_toolbar(|bar| bar.mouse_up(x, y));
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.each_toolbar(|bar| bar.mouse_move(x as f32, y as f32));
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Toolbar", 800, 600, ToolbarDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
