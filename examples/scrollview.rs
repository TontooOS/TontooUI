use tontooui::elements::{
    BasicText, ScrollView, Titlebar, TrafficAction, VStack, View,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

const ROWS: usize = 60;

struct ScrollViewDemo {
    bar: Titlebar,
    scroll: ScrollView,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ScrollViewDemo {
    fn new() -> Self {
        let mut stack = VStack::new().spacing(4.0);
        for i in 0..ROWS {
            stack = stack.child(BasicText::new(format!(
                "Row {:02} - the window keeps its size",
                i + 1
            )));
        }
        Self {
            bar: Titlebar::new("ScrollView"),
            scroll: ScrollView::new(stack),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }
}

impl App for ScrollViewDemo {
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
        let dark = theme.mode == ThemeMode::Dark;

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
        self.bar
            .set_title(format!("ScrollView - {}", self.scroll.offset() as i32));

        // Theme the rows through the wrapped stack.
        if let Some(stack) = self.scroll.child_mut::<VStack>() {
            for index in 0..stack.len() {
                if let Some(row) = stack.child_mut::<BasicText>(index) {
                    row.set_theme(theme.mode);
                    row.set_focused(self.focused);
                }
            }
        }
        self.scroll.set_theme(palette.accent, dark);
        self.scroll.set_focused(self.focused);

        // Fixed viewport: more rows never grow the window, the
        // ScrollView clips and scrolls instead.
        let area_y = viewport.y + 31.0 + 48.0;
        let area_h = (viewport.y + viewport.height - area_y - 16.0).max(80.0);
        self.scroll.place(
            fonts,
            viewport.x + 24.0,
            area_y,
            viewport.width - 48.0,
            area_h,
        );
        self.scroll.draw(scene, fonts, images);
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
        false
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => self.scroll.mouse_down(x, y),
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.scroll.mouse_move(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.scroll.mouse_up(x, y);
    }

    fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        self.scroll.mouse_wheel(dx, dy);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.scroll.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("ScrollView", 480, 420, ScrollViewDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
