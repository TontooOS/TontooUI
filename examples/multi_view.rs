use tontooui::elements::{
    Background, HStack, Padding, Spacer, Text, TextInput, Titlebar, TrafficAction, View,
    VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Key, Viewport, WindowCommand, run};
use vello::Scene;
use vello::peniko::Color;

struct Multi {
    bar: Titlebar,
    root: HStack,
    command: Option<WindowCommand>,
}

impl Multi {
    fn new() -> Self {
        let left = Background::new(
            Padding::all(
                VStack::new()
                    .spacing(8.0)
                    .child(
                        Text::new("Left Panel")
                            .size(20.0)
                            .color(Color::WHITE),
                    )
                    .child(
                        Text::new("Views inside views.")
                            .size(13.0)
                            .color(Color::from_rgb8(0x9a, 0x9a, 0x9e)),
                    )
                    .child(Spacer::new())
                    .child(
                        Text::new("Echo: ")
                            .size(14.0)
                            .color(Color::from_rgb8(0xff, 0x9f, 0x0a)),
                    ),
                12.0,
            ),
            Color::from_rgb8(0x2c, 0x2c, 0x2e),
        )
        .radius(12.0);
        let right = Background::new(
            Padding::all(
                VStack::new()
                    .spacing(8.0)
                    .child(
                        Text::new("Right Panel")
                            .size(20.0)
                            .color(Color::WHITE),
                    )
                    .child(TextInput::new().placeholder("Type here...")),
                12.0,
            ),
            Color::from_rgb8(0x2c, 0x2c, 0x2e),
        )
        .radius(12.0);
        let root = HStack::new().spacing(12.0).child(left).child(right);
        Self {
            bar: Titlebar::new("Multi View"),
            root,
            command: None,
        }
    }

    fn left_stack(&mut self) -> Option<&mut VStack> {
        self.root
            .child_mut::<Background>(0)?
            .child_mut::<Padding>()?
            .child_mut::<VStack>()
    }

    fn right_stack(&mut self) -> Option<&mut VStack> {
        self.root
            .child_mut::<Background>(1)?
            .child_mut::<Padding>()?
            .child_mut::<VStack>()
    }

    fn input_text(&mut self) -> String {
        self.right_stack()
            .and_then(|stack| stack.child_mut::<TextInput>(1))
            .map(|input| input.text().to_owned())
            .unwrap_or_default()
    }
}

impl App for Multi {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        viewport: Viewport,
        _time: f64,
    ) {
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let current = self.input_text();
        if let Some(stack) = self.left_stack() {
            if let Some(echo) = stack.child_mut::<Text>(3) {
                echo.set_content(format!("Echo: {current}"));
            }
        }
        if let Some(stack) = self.right_stack() {
            if let Some(input) = stack.child_mut::<TextInput>(1) {
                input.set_bounds(0.0, 0.0, 360.0, 44.0);
            }
        }
        let top = viewport.y + 31.0;
        self.root.place(
            fonts,
            viewport.x + 8.0,
            top + 12.0,
            viewport.width - 16.0,
            (viewport.height - 43.0).max(0.0),
        );
        self.root.draw(scene, fonts);
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
                if let Some(stack) = self.right_stack() {
                    if let Some(input) = stack.child_mut::<TextInput>(1) {
                        input.mouse_down(x, y);
                    }
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
        if let Some(stack) = self.right_stack() {
            if let Some(input) = stack.child_mut::<TextInput>(1) {
                input.insert(text);
            }
        }
    }

    fn key(&mut self, key: Key) {
        if let Some(stack) = self.right_stack() {
            if let Some(input) = stack.child_mut::<TextInput>(1) {
                match key {
                    Key::Backspace => input.backspace(),
                    Key::Left => input.move_left(),
                    Key::Right => input.move_right(),
                    Key::Enter | Key::Escape => input.mouse_down(-1.0, -1.0),
                }
            }
        }
    }
}

fn main() {
    if let Err(err) = run("Multi View", 900, 600, Multi::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
