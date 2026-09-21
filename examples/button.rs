use std::cell::Cell;
use std::rc::Rc;

use tontooui::elements::{
    Align, Button, ButtonShape, ButtonStyle, Titlebar, TrafficAction, View, VStack,
    buttons::{BUTTON_BG_DARK, BUTTON_BG_LIGHT},
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct ButtonDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
    presses: Rc<Cell<usize>>,
}

impl ButtonDemo {
    fn new() -> Self {
        let presses = Rc::new(Cell::new(0));
        let counted = || {
            let presses = presses.clone();
            move || presses.set(presses.get() + 1)
        };
        let stack = VStack::new()
            .spacing(12.0)
            .align(Align::Center)
            .child(Button::new("Tap Me").on_press(counted()))
            .child(
                Button::new("Tap with Label")
                    .icon("hand.tap")
                    .style(ButtonStyle::BorderedProminent)
                    .on_press(counted()),
            )
            .child(
                Button::new("Tinted")
                    .style(ButtonStyle::BorderedTinted)
                    .on_press(counted()),
            )
            .child(Button::new("Plain").style(ButtonStyle::Plain).on_press(counted()))
            .child(
                Button::new("Capsule")
                    .shape(ButtonShape::Capsule)
                    .on_press(counted()),
            );
        Self {
            bar: Titlebar::new("Button"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
            presses,
        }
    }

    fn each_button(&mut self, f: impl Fn(&mut Button)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<Button>(index) {
                Some(button) => f(button),
                None => break,
            }
            index += 1;
        }
    }
}

impl App for ButtonDemo {
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
        let dark = self.watcher.theme().mode == ThemeMode::Dark;
        let (button_bg, button_text) = if dark {
            (BUTTON_BG_DARK, Color::WHITE)
        } else {
            (BUTTON_BG_LIGHT, Color::BLACK)
        };
        self.each_button(|button| {
            button.set_palette(button_bg, button_text);
            button.set_theme(palette.accent, dark);
        });
        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
        self.bar
            .set_title(format!("Button — {} presses", self.presses.get()));

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
            None => self.each_button(|button| button.mouse_down(x, y)),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_button(|button| button.mouse_up(x, y));
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.each_button(|button| button.set_hover(x as f32, y as f32));
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.each_button(|button| button.set_focused(focused));
    }
}

fn main() {
    if let Err(err) = run("Button", 800, 600, ButtonDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
