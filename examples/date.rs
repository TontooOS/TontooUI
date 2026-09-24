use tontooui::elements::{DatePicker, Titlebar, TrafficAction, View, VStack, DATE_MONTHS};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::text::draw_layout;
use tontooui::renderer::window::{App, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct DateDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    text: Color,
    caption: String,
    command: Option<WindowCommand>,
}

impl DateDemo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(16.0)
            .child(DatePicker::new().selected(2026, 7, 16));
        Self {
            bar: Titlebar::new("Date"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            text: Color::WHITE,
            caption: "16 July 2026".to_string(),
            command: None,
        }
    }

    fn each_picker(&mut self, mut f: impl FnMut(&mut DatePicker)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<DatePicker>(index) {
                Some(picker) => f(picker),
                None => break,
            }
            index += 1;
        }
    }

    fn refresh_caption(&mut self) {
        if let Some(first) = self.stack.child_mut::<DatePicker>(0) {
            let (y, m, d) = first.selected_date();
            self.caption = format!("{d} {} {y}", DATE_MONTHS[(m - 1) as usize]);
        }
    }
}

impl App for DateDemo {
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
        self.text = palette.text;
        let theme = self.watcher.theme();
        let dark = theme.mode == ThemeMode::Dark;
        let focused = self.focused;
        self.each_picker(|picker| {
            picker.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
            picker.set_theme(palette.accent, dark);
            picker.set_glass(theme.mode, theme.glass);
            picker.set_focused(focused);
        });
        self.refresh_caption();

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
        self.bar.set_title(format!("Date — {}", self.caption));

        // Caption under the calendar, like the reference screenshots.
        let text = format!("Selected: {}", self.caption);
        let layout = fonts.layout_text_weighted(&text, 17.0, self.text, 600.0, None);
        let (tw, _) = FontSystem::layout_size(&layout);
        let cx = viewport.x + (viewport.width - tw / fonts.scale) / 2.0;
        let top = viewport.y + 31.0;
        let cy = top + 16.0 + 236.0 + 16.0;
        draw_layout(scene, &layout, cx, cy, fonts.scale);

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
        // Glass calendar always on screen.
        true
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => {
                self.each_picker(|picker| picker.mouse_down(x, y));
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.each_picker(|picker| picker.mouse_move(x, y));
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_picker(|picker| picker.mouse_up(x, y));
        self.refresh_caption();
    }

    fn text(&mut self, text: &str) {
        self.each_picker(|picker| picker.text(text));
    }

    fn key(&mut self, key: Key) {
        self.each_picker(|picker| picker.key(key));
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.each_picker(|picker| picker.set_focused(focused));
    }
}

fn main() {
    if let Err(err) = run("Date", 480, 520, DateDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
