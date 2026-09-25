use tontooui::elements::{
    DividerStyle, HStack, HorizontalDivider, Titlebar, TrafficAction, VerticalDivider,
    View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct DividerDemo {
    bar: Titlebar,
    stack: VStack,
    row: HStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl DividerDemo {
    fn new() -> Self {
        // Reference rows: default, red, thick, blue thick, padded.
        let stack = VStack::new()
            .spacing(24.0)
            .child(HorizontalDivider::styled(DividerStyle::Default))
            .child(HorizontalDivider::styled(DividerStyle::Red))
            .child(HorizontalDivider::styled(DividerStyle::Thick))
            .child(HorizontalDivider::styled(DividerStyle::BlueThick))
            .child(HorizontalDivider::styled(DividerStyle::Padded));
        let row = HStack::new()
            .spacing(24.0)
            .child(VerticalDivider::styled(DividerStyle::Default))
            .child(VerticalDivider::styled(DividerStyle::Red))
            .child(VerticalDivider::styled(DividerStyle::Thick))
            .child(VerticalDivider::styled(DividerStyle::BlueThick))
            .child(VerticalDivider::styled(DividerStyle::Padded));
        Self {
            bar: Titlebar::new("Divider"),
            stack,
            row,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_horizontal(&mut self, mut f: impl FnMut(&mut HorizontalDivider)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<HorizontalDivider>(index) {
                Some(line) => f(line),
                None => break,
            }
            index += 1;
        }
    }

    fn each_vertical(&mut self, mut f: impl FnMut(&mut VerticalDivider)) {
        let mut index = 0;
        loop {
            match self.row.child_mut::<VerticalDivider>(index) {
                Some(line) => f(line),
                None => break,
            }
            index += 1;
        }
    }
}

impl App for DividerDemo {
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
        self.each_horizontal(|line| {
            line.set_theme(palette.divider, dark);
            line.set_focused(focused);
        });
        self.each_vertical(|line| {
            line.set_theme(palette.divider, dark);
            line.set_focused(focused);
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
        self.stack.place(
            fonts,
            viewport.x,
            top + 16.0,
            viewport.width,
            stack_h,
        );
        self.stack.draw(scene, fonts, images);
        let row_y = top + 16.0 + stack_h + 24.0;
        self.row.place(
            fonts,
            viewport.x + 24.0,
            row_y,
            viewport.width - 48.0,
            160.0,
        );
        self.row.draw(scene, fonts, images);
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
    }
}

fn main() {
    if let Err(err) = run("Divider", 900, 480, DividerDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
