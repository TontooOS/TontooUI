use tontooui::elements::{Text, TextInput};
use tontooui::renderer::window::{Key, View, run};
use tontooui::renderer::FontSystem;
use vello::Scene;
use vello::peniko::Color;

struct Demo {
    title: Text,
    subtitle: Text,
    input: TextInput,
    echo: Text,
}

impl Demo {
    fn new() -> Self {
        Self {
            title: Text::new("TontooUI Renderer Test")
                .size(28.0)
                .color(Color::WHITE)
                .at(32.0, 36.0),
            subtitle: Text::new("Click the field and type. Esc clears focus.")
                .size(14.0)
                .color(Color::from_rgb8(0x9a, 0x9a, 0x9e))
                .at(32.0, 76.0),
            input: TextInput::new()
                .bounds(32.0, 120.0, 420.0, 44.0)
                .placeholder("Type here..."),
            echo: Text::new("Echo: ")
                .size(15.0)
                .color(Color::from_rgb8(0xff, 0x9f, 0x0a))
                .at(32.0, 184.0),
        }
    }
}

impl View for Demo {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _width: f32,
        _height: f32,
        time_secs: f64,
    ) {
        self.title.draw(scene, fonts);
        self.subtitle.draw(scene, fonts);
        // 530 ms cursor blink.
        let blink_on = (time_secs * 1000.0 / 530.0) as u64 % 2 == 0;
        self.input.draw(scene, fonts, blink_on);
        let shown = format!("Echo: {}", self.input.text());
        self.echo.set_content(shown);
        self.echo.draw(scene, fonts);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.input.mouse_down(x, y);
    }

    fn text(&mut self, text: &str) {
        self.input.insert(text);
    }

    fn key(&mut self, key: Key) {
        match key {
            Key::Backspace => self.input.backspace(),
            Key::Left => self.input.move_left(),
            Key::Right => self.input.move_right(),
            Key::Enter | Key::Escape => {
                // Blur by focusing nothing: send a click outside any field.
                self.input.mouse_down(-1.0, -1.0);
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
