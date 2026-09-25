use tontooui::elements::{DisclosureGroup, Titlebar, TrafficAction, View, VStack};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct DisclosureDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl DisclosureDemo {
    fn new() -> Self {
        // Like the reference: one open group plus closed ones.
        let stack = VStack::new()
            .spacing(0.0)
            .child(
                DisclosureGroup::from_slice(
                    "Fruits",
                    &["Apple", "Banana", "Cherry", "Date"],
                )
                .open(true),
            )
            .child(DisclosureGroup::from_slice(
                "Vegetables",
                &["Carrot", "Lettuce"],
            ))
            .child(DisclosureGroup::from_slice("Grains", &["Rice", "Wheat"]));
        Self {
            bar: Titlebar::new("Disclosure"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_group(&mut self, mut f: impl FnMut(&mut DisclosureGroup)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<DisclosureGroup>(index) {
                Some(group) => f(group),
                None => break,
            }
            index += 1;
        }
    }
}

impl App for DisclosureDemo {
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
        self.each_group(|group| {
            group.set_theme(palette.divider, dark);
            group.set_focused(focused);
        });

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let (_, stack_h) = self.stack.measure(fonts);
        self.stack.place(fonts, viewport.x, top, viewport.width, stack_h);
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
            None => self.each_group(|group| group.mouse_down(x, y)),
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if self.bar.press(x as f32, y as f32).is_none() {
            self.each_group(|group| group.mouse_up(x, y));
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Disclosure", 900, 480, DisclosureDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
