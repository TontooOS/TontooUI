use tontooui::elements::{
    BasicText, HStack, Stepper, StepperOrientation, Titlebar, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct StepperDemo {
    bar: Titlebar,
    rows: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl StepperDemo {
    fn new() -> Self {
        let rows = VStack::new()
            .spacing(20.0)
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(BasicText::new("Basic: 5"))
                    .child(Stepper::new(5.0, 0.0, 10.0)),
            )
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(BasicText::new("Tens: 30"))
                    .child(Stepper::new(30.0, 0.0, 100.0).step(10.0)),
            )
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(BasicText::new("Horizontal: 50"))
                    .child(
                        Stepper::new(50.0, 0.0, 100.0)
                            .step(5.0)
                            .orientation(StepperOrientation::Horizontal),
                    ),
            );
        Self {
            bar: Titlebar::new("Stepper"),
            rows,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_stepper(&mut self, mut f: impl FnMut(&mut Stepper)) {
        let mut index = 0;
        loop {
            let next = match self.rows.child_mut::<HStack>(index) {
                Some(row) => {
                    if let Some(stepper) = row.child_mut::<Stepper>(1) {
                        f(stepper);
                    }
                    true
                }
                None => false,
            };
            if !next {
                break;
            }
            index += 1;
        }
    }

    fn sync_labels(&mut self) {
        let mut values = Vec::new();
        self.each_stepper(|stepper| values.push(stepper.value()));
        for (index, value) in values.iter().enumerate() {
            if let Some(row) = self.rows.child_mut::<HStack>(index) {
                if let Some(text) = row.child_mut::<BasicText>(0) {
                    let label = match index {
                        0 => format!("Basic: {value:.0}"),
                        1 => format!("Tens: {value:.0}"),
                        _ => format!("Horizontal: {value:.0}"),
                    };
                    text.set_text(label);
                }
            }
        }
    }
}

impl App for StepperDemo {
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
        self.each_stepper(|stepper| {
            stepper.set_theme(palette.accent, dark);
            stepper.set_focused(focused);
        });
        let mut index = 0;
        loop {
            let next = match self.rows.child_mut::<HStack>(index) {
                Some(row) => {
                    if let Some(text) = row.child_mut::<BasicText>(0) {
                        text.set_theme(theme.mode);
                        text.set_focused(focused);
                    }
                    true
                }
                None => false,
            };
            if !next {
                break;
            }
            index += 1;
        }
        self.sync_labels();

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        self.rows.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0,
            viewport.width - 48.0,
            (viewport.height - 47.0).max(0.0),
        );
        self.rows.draw(scene, fonts, images);
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
            None => self.each_stepper(|stepper| stepper.mouse_down(x, y)),
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.each_stepper(|stepper| stepper.set_hover(x as f32, y as f32));
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_stepper(|stepper| stepper.mouse_up(x, y));
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Stepper", 600, 400, StepperDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
