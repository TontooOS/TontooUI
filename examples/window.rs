use tontooui::elements::{Text, TextInput, Titlebar, TrafficAction, View, VStack};
use tontooui::renderer::window::{App, Key, Viewport, WindowCommand, run};
use tontooui::renderer::FontSystem;
use tontooui::theme::ThemeWatcher;
use vello::Scene;
use vello::peniko::Color;

struct Demo {
    bar: Titlebar,
    stack: VStack,
    command: Option<WindowCommand>,
    watcher: ThemeWatcher,
    bg: Color,
}

impl Demo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(8.0)
            .child(
                Text::new("TontooUI Renderer Test")
                    .size(28.0)
                    .color(Color::WHITE),
            )
            .child(
                Text::new("Click the field and type. Esc clears focus.")
                    .size(14.0)
                    .color(Color::from_rgb8(0x9a, 0x9a, 0x9e)),
            )
            .child(TextInput::new().placeholder("Type here..."))
            .child(
                Text::new("Echo: ")
                    .size(15.0)
                    .color(Color::from_rgb8(0xff, 0x9f, 0x0a)),
            );
        Self {
            bar: Titlebar::new("TontooUI"),
            stack,
            command: None,
            watcher: ThemeWatcher::new(),
            bg: tontooui::renderer::window::BACKGROUND,
        }
    }

    fn input_text(&mut self) -> String {
        self.stack
            .child_mut::<TextInput>(2)
            .map(|input| input.text().to_owned())
            .unwrap_or_default()
    }
}

impl App for Demo {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        viewport: Viewport,
        time_secs: f64,
    ) {
        // Live theme: poll the daemon, crossfade the palette on change.
        self.watcher.poll(time_secs);
        let palette = self.watcher.palette(time_secs);
        self.bg = palette.bg;
        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let current = self.input_text();
        if let Some(echo) = self.stack.child_mut::<Text>(3) {
            echo.set_content(format!("Echo: {current}"));
            echo.set_color(palette.accent);
        }
        // Input keeps its intrinsic 300 px width; stretch it to the stack.
        if let Some(input) = self.stack.child_mut::<TextInput>(2) {
            input.set_bounds(0.0, 0.0, 420.0, 44.0);
        }
        let top = viewport.y + 31.0;
        self.stack.place(
            fonts,
            viewport.x + 8.0,
            top + 12.0,
            viewport.width - 16.0,
            (viewport.height - 43.0).max(0.0),
        );
        // 530 ms cursor blink for direct-drawn inputs is handled by the
        // element; stacks draw steady. Unused here but kept for reference.
        let _ = time_secs;
        self.stack.draw(scene, fonts);
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
            None => {
                if let Some(input) = self.stack.child_mut::<TextInput>(2) {
                    input.mouse_down(x, y);
                }
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.bar.set_focused(focused);
    }

    fn text(&mut self, text: &str) {
        if let Some(input) = self.stack.child_mut::<TextInput>(2) {
            input.insert(text);
        }
    }

    fn key(&mut self, key: Key) {
        if let Some(input) = self.stack.child_mut::<TextInput>(2) {
            match key {
                Key::Backspace => input.backspace(),
                Key::Left => input.move_left(),
                Key::Right => input.move_right(),
                Key::Enter | Key::Escape => {
                    // Blur by focusing nothing: send a click outside any field.
                    input.mouse_down(-1.0, -1.0);
                }
            }
        }
    }
}

fn main() {
    if let Err(err) = run("TontooUI", 800, 600, Demo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
