use tontooui::elements::{
    Align, BasicLabel, HStack, IconLabel, ImageLabel, StyledLabel,
    Titlebar, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct LabelDemo {
    bar: Titlebar,
    stack: VStack,
    image: ImageLabel,
    status: StyledLabel,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl LabelDemo {
    fn new() -> Self {
        let green = Color::from_rgb8(0x34, 0xc7, 0x59);
        let blue = Color::from_rgb8(0x0a, 0x84, 0xff);
        // Reference rows: outline icons plus titles.
        let stack = VStack::new()
            .align(Align::Center)
            .spacing(24.0)
            .child(
                HStack::new()
                    .align(Align::Center)
                    .spacing(48.0)
                    .child(
                        VStack::new()
                            .spacing(14.0)
                            .child(BasicLabel::new("star", "Star"))
                            .child(BasicLabel::new("heart", "Heart"))
                            .child(BasicLabel::new("bookmark", "Bookmark"))
                            .child(BasicLabel::new("envelope", "Mail")),
                    )
                    .child(
                        VStack::new()
                            .align(Align::Center)
                            .spacing(14.0)
                            .child(IconLabel::new("star"))
                            .child(IconLabel::new("heart").icon_color(green))
                            .child(IconLabel::new("bookmark").icon_color(blue)),
                    ),
            );
        Self {
            bar: Titlebar::new("Label"),
            stack,
            // Async URL image plus title, tinted like the reference.
            image: ImageLabel::new(
                "https://picsum.photos/96/96",
                "Custom Title",
            )
            .tint(blue),
            status: StyledLabel::status("Downloaded", green),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn theme_rows(&mut self, mode: ThemeMode, focused: bool) {
        for row in 0..self.stack.len() {
            if let Some(hstack) = self.stack.child_mut::<HStack>(row) {
                for col in 0..hstack.len() {
                    if let Some(vstack) = hstack.child_mut::<VStack>(col) {
                        for index in 0..vstack.len() {
                            if let Some(label) =
                                vstack.child_mut::<BasicLabel>(index)
                            {
                                label.set_theme(mode);
                                label.set_focused(focused);
                            } else if let Some(icon) =
                                vstack.child_mut::<IconLabel>(index)
                            {
                                icon.set_theme(mode);
                                icon.set_focused(focused);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl App for LabelDemo {
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
        self.theme_rows(theme.mode, focused);
        self.image.set_theme(theme.mode);
        self.image.set_focused(focused);
        self.status.set_theme(theme.mode);
        self.status.set_focused(focused);

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
        self.stack.place(fonts, x, top + 24.0, stack_w, stack_h);
        self.stack.draw(scene, fonts, images);

        // Image row plus status row below the basics.
        let (iw, ih) = self.image.measure(fonts);
        let ix = viewport.x + ((viewport.width - iw) / 2.0).max(0.0);
        let mut y = top + 24.0 + stack_h + 32.0;
        self.image.place(fonts, ix, y, iw, ih);
        self.image.draw(scene, fonts, images);
        y += ih + 20.0;
        let (sw, sh) = self.status.measure(fonts);
        let sx = viewport.x + ((viewport.width - sw) / 2.0).max(0.0);
        self.status.place(fonts, sx, y, sw, sh);
        self.status.draw(scene, fonts, images);
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
    if let Err(err) = run("Label", 900, 900, LabelDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
