use tontooui::elements::{
    Align, BasicGroupBox, StyledGroupBox, Titlebar, Toggle, TrafficAction,
    View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct GroupBoxDemo {
    bar: Titlebar,
    stack: VStack,
    styled: StyledGroupBox,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl GroupBoxDemo {
    fn new() -> Self {
        // Reference row plus narrower boxes, centered.
        let stack = VStack::new()
            .align(Align::Center)
            .spacing(20.0)
            .child(BasicGroupBox::new("This is content inside a GroupBox."))
            .child(BasicGroupBox::new("Short note."))
            .child(BasicGroupBox::new(
                "A second line wraps inside the padding.",
            ));
        Self {
            bar: Titlebar::new("GroupBox"),
            stack,
            // Reference settings rows: checkboxes plus a switch and
            // an icon row.
            styled: StyledGroupBox::new()
                .check_row("Notifications", true)
                .check_row("Dark Mode", false)
                .check_row("Location Services", true)
                .toggle_row("Wi-Fi", true)
                .symbol_row("wifi", "Network"),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_group(&mut self, mut f: impl FnMut(&mut BasicGroupBox)) {
        for index in 0..self.stack.len() {
            if let Some(group) = self.stack.child_mut::<BasicGroupBox>(index) {
                f(group);
            }
        }
    }
}

impl App for GroupBoxDemo {
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
        let focused = self.focused;
        self.each_group(|group| {
            group.set_theme(theme.mode);
            group.set_focused(focused);
        });
        // Styled rows: box and labels follow the theme; leading
        // toggles and icons keep their own wiring.
        self.styled.set_theme(theme.mode);
        self.styled.set_focused(focused);
        for index in 0..self.styled.row_len() {
            if let Some(toggle) = self.styled.row_leading_mut::<Toggle>(index) {
                toggle.set_theme(palette.accent, theme.mode == ThemeMode::Dark);
                toggle.set_focused(focused);
            }
        }

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let (stack_w, stack_h) = self.stack.measure(fonts);
        let x = viewport.x + ((viewport.width - stack_w) / 2.0).max(0.0);
        self.stack.place(fonts, x, top + 20.0, stack_w, stack_h);
        self.stack.draw(scene, fonts, images);
        // Styled box below the basics, same centering.
        let (bw, bh) = self.styled.measure(fonts);
        let bx = viewport.x + ((viewport.width - bw) / 2.0).max(0.0);
        self.styled.place(fonts, bx, top + 20.0 + stack_h + 24.0, bw, bh);
        self.styled.draw(scene, fonts, images);
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
            Some(TrafficAction::Minimize) => {
                self.command = Some(WindowCommand::Minimize)
            }
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => self.styled.mouse_down(x, y),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.styled.mouse_up(x, y);
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.styled.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("GroupBox", 900, 480, GroupBoxDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
