use tontooui::elements::{Align, Toggle, ToggleStyle, Titlebar, TrafficAction, View, VStack};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct ToggleDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
    dragging: bool,
}

impl ToggleDemo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(16.0)
            .align(Align::Leading)
            .child(Toggle::new("Switch Style").on(true))
            .child(Toggle::new("Button Style").style(ToggleStyle::Button).on(true))
            .child(
                Toggle::new("Checkbox Style")
                    .style(ToggleStyle::Checkbox)
                    .on(true),
            )
            .child(Toggle::new("Airplane Mode").icon("airplane"))
            .child(Toggle::new("Wi-Fi").icon("wifi").on(true))
            .child(Toggle::new("Bluetooth"))
            .child(
                Toggle::new("Fixed green")
                    .fill(Color::from_rgb8(0x34, 0xc7, 0x59))
                    .on(true),
            );
        Self {
            bar: Titlebar::new("Toggle"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
            dragging: false,
        }
    }

    fn each_toggle(&mut self, mut f: impl FnMut(&mut Toggle)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<Toggle>(index) {
                Some(toggle) => f(toggle),
                None => break,
            }
            index += 1;
        }
    }

    fn on_count(&mut self) -> usize {
        let mut count = 0;
        self.each_toggle(|toggle| {
            if toggle.is_on() {
                count += 1;
            }
        });
        count
    }
}

impl App for ToggleDemo {
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
        self.each_toggle(|toggle| {
            toggle.set_theme(palette.accent, dark);
            toggle.set_focused(focused);
        });

        // Unified bar: same fill as the background, only a divider line.
        self.bar.set_palette(
            palette.bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
        let on_count = self.on_count();
        self.bar
            .set_title(format!("Toggle — {on_count} on"));

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
                self.each_toggle(|toggle| {
                    toggle.mouse_down(x, y);
                    if toggle.is_dragging() {
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
        self.each_toggle(|toggle| {
            toggle.mouse_move(x, y);
            if toggle.is_dragging() {
                dragging = true;
            }
        });
        if dragging {
            self.dragging = true;
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_toggle(|toggle| toggle.mouse_up(x, y));
        self.dragging = false;
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.each_toggle(|toggle| toggle.set_focused(focused));
    }
}

fn main() {
    if let Err(err) = run("Toggle", 800, 640, ToggleDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
