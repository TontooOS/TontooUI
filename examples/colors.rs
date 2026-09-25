use tontooui::elements::{
    ALL_SYSTEM_COLORS, GradientPaint, Titlebar, TrafficAction,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::text::draw_layout;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::ThemeWatcher;
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

/// Swatch chip size and label size in logical px.
const CHIP: f32 = 56.0;
const CHIP_GAP: f32 = 12.0;
const NAME_SIZE: f32 = 11.0;
/// Gradient bar height, gap and radius in logical px.
const BAR_H: f32 = 88.0;
const BAR_GAP: f32 = 20.0;
const BAR_RADIUS: f32 = 16.0;
const BAR_LABEL_SIZE: f32 = 15.0;

struct ColorsDemo {
    bar: Titlebar,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ColorsDemo {
    fn new() -> Self {
        Self {
            bar: Titlebar::new("Colors"),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn label(
        &self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        text: &str,
        size: f32,
        color: Color,
        cx: f32,
        y: f32,
    ) {
        let layout = fonts.layout_text(text, size, color, None);
        let (tw, _) = FontSystem::layout_size(&layout);
        draw_layout(scene, &layout, cx - tw / fonts.scale / 2.0, y, fonts.scale);
    }
}

impl App for ColorsDemo {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    ) {
        self.watcher.poll(time_secs);
        self.watcher.set_focused(self.focused, time_secs);
        let palette = self.watcher.palette(time_secs);
        self.bg = palette.bg;

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let mut y = viewport.y + 31.0 + 24.0;

        // Swatch grid: every system color with its name.
        let total_w = ALL_SYSTEM_COLORS.len() as f32 * CHIP
            + (ALL_SYSTEM_COLORS.len() - 1) as f32 * CHIP_GAP;
        let mut x = viewport.x + ((viewport.width - total_w) / 2.0).max(0.0);
        for color in ALL_SYSTEM_COLORS {
            let chip = RoundedRect::new(
                px(x),
                px(y),
                px(x + CHIP),
                px(y + CHIP),
                px(CHIP / 3.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(color.color()),
                None,
                &chip,
            );
            self.label(
                scene,
                fonts,
                color.name(),
                NAME_SIZE,
                palette.text,
                x + CHIP / 2.0,
                y + CHIP + 6.0,
            );
            x += CHIP + CHIP_GAP;
        }
        y += CHIP + 6.0 + NAME_SIZE * 1.25 + 28.0;

        // Reference gradient bars with centered labels.
        let bars = [
            (GradientPaint::preset_linear(), "Linear Gradient"),
            (GradientPaint::preset_vertical(), "Vertical Gradient"),
            (GradientPaint::preset_radial(), "Radial Gradient"),
            (GradientPaint::preset_angular(), "Angular Gradient"),
        ];
        let bx = viewport.x + 24.0;
        let bw = (viewport.width - 48.0).max(0.0);
        for (paint, caption) in bars {
            let bar = RoundedRect::new(
                px(bx),
                px(y),
                px(bx + bw),
                px(y + BAR_H),
                px(BAR_RADIUS),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &paint.brush(bx, y, bw, BAR_H, fonts.scale),
                None,
                &bar,
            );
            self.label(
                scene,
                fonts,
                caption,
                BAR_LABEL_SIZE,
                Color::WHITE,
                bx + bw / 2.0,
                y + (BAR_H - BAR_LABEL_SIZE) / 2.0,
            );
            y += BAR_H + BAR_GAP;
        }
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
    if let Err(err) = run("Colors", 960, 800, ColorsDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
