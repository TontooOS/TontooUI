use tontooui::elements::{
    BasicToolbar, FileImage, HStack, ImageFit, Slider, Titlebar, ToolbarItem,
    ToolbarPlacement, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

/// Window background photo, vendored under `examples/assets` (the
/// Unsplash source needs network; the file keeps the demo offline).
/// Cover fit behind the sliders; the opaque body keeps the shell
/// clip to the rounded window corners.
const BG_PHOTO_FILE: &str = "slider-bg.jpg";

struct SliderDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    bg_image: FileImage,
    command: Option<WindowCommand>,
}

impl SliderDemo {
    fn new() -> Self {
        let colors = HStack::new()
            .spacing(16.0)
            .child(Slider::new(0.6, 0.0, 1.0).fill(Color::from_rgb8(0xff, 0x2d, 0x55)))
            .child(Slider::new(0.6, 0.0, 1.0).fill(Color::from_rgb8(0x34, 0xc7, 0x59)))
            .child(Slider::new(0.6, 0.0, 1.0).fill(Color::from_rgb8(0xaf, 0x52, 0xde)));
        let stack = VStack::new()
            .spacing(20.0)
            .child(
                Slider::new(0.5, 0.0, 1.0).value_text(|v| format!("Basic: {v:.2}")),
            )
            .child(
                Slider::new(10.0, 0.0, 100.0)
                    .step(10.0)
                    .show_ticks(true)
                    .value_text(|v| format!("Value: {v:.0}")),
            )
            .child(
                Slider::new(50.0, 0.0, 100.0)
                    .title("Temperature")
                    .min_label("0°")
                    .max_label("100°")
                    .value_text(|v| format!("Value: {v:.0}°")),
            )
            .child(
                Slider::new(3.0, 1.0, 5.0)
                    .step(1.0)
                    .show_ticks(true)
                    .value_text(|v| format!("Rating: {v:.0}/5")),
            )
            .child(colors)
            .child(
                Slider::new(0.6, 0.0, 1.0)
                    .glass(true)
                    .value_text(|v| format!("Glass: {:.0}%", v * 100.0)),
            )
            .child(
                BasicToolbar::from_items(vec![
                    ToolbarItem::icon("chevron.left"),
                    ToolbarItem::divider(),
                    ToolbarItem::icon("chevron.right"),
                ])
                .placement(ToolbarPlacement::Leading),
            )
            .child(
                BasicToolbar::from_icons(vec!["heart".to_string()])
                    .placement(ToolbarPlacement::Center),
            )
            .child(
                BasicToolbar::from_icons(vec![
                    "xmark".to_string(),
                    "star".to_string(),
                    "checkmark".to_string(),
                ])
                .placement(ToolbarPlacement::Trailing),
            );
        Self {
            bar: Titlebar::new("Slider"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            bg_image: FileImage::new(bg_photo_path(), 900.0, 720.0)
                .fit(ImageFit::Cover)
                .radius(0.0),
            command: None,
        }
    }

    fn each_toolbar(&mut self, mut f: impl FnMut(&mut BasicToolbar)) {
        // Toolbars sit after the sliders/HStack row: probe by type.
        for index in 0..32 {
            if let Some(bar) = self.stack.child_mut::<BasicToolbar>(index) {
                f(bar);
            }
        }
    }

    fn each_slider(&mut self, mut f: impl FnMut(&mut Slider)) {
        let mut index = 0;
        loop {
            if let Some(slider) = self.stack.child_mut::<Slider>(index) {
                f(slider);
            } else if let Some(row) = self.stack.child_mut::<HStack>(index) {
                let mut inner = 0;
                loop {
                    match row.child_mut::<Slider>(inner) {
                        Some(slider) => f(slider),
                        None => break,
                    }
                    inner += 1;
                }
            } else {
                break;
            }
            index += 1;
        }
    }
}

impl App for SliderDemo {
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
        // Copy out before the mutable walk.
        let theme = self.watcher.theme();
        let dark = theme.mode == ThemeMode::Dark;
        let focused = self.focused;
        self.each_slider(|slider| {
            slider.set_theme(palette.accent, dark, theme.glass);
            slider.set_focused(focused);
        });
        self.each_toolbar(|bar| {
            bar.set_theme(theme.mode, theme.glass);
            bar.set_focused(focused);
        });

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        // Photo background behind everything (cover fit over the
        // full viewport; the opaque body keeps the shell clip to
        // the rounded window corners).
        self.bg_image.set_theme(dark);
        self.bg_image.place(
            fonts,
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
        );
        self.bg_image.draw(scene, fonts, images);

        let top = viewport.y + 31.0;
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
        // Glass slider knobs and Lens toolbars need the blur pass.
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
                self.each_slider(|slider| {
                    slider.mouse_down(x, y);
                });
                self.each_toolbar(|bar| bar.mouse_down(x, y));
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.each_slider(|slider| {
            slider.mouse_move(x, y);
        });
        self.each_toolbar(|bar| bar.mouse_move(x as f32, y as f32));
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_slider(|slider| slider.mouse_up(x, y));
        self.each_toolbar(|bar| bar.mouse_up(x, y));
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.bg_image.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Slider", 900, 720, SliderDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

/// Absolute path of the vendored background photo, resolved from
/// the crate dir so the example runs from any working directory.
fn bg_photo_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("assets")
        .join(BG_PHOTO_FILE)
}
