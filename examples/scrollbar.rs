use tontooui::elements::{Scrollbar, Titlebar, TrafficAction, View};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::text::draw_layout;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Color, Fill};

const LINES: usize = 60;
const LINE_H: f32 = 22.0;

struct ScrollDemo {
    bar: Titlebar,
    scroller: Scrollbar,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    text: Color,
    content: (f32, f32),
    command: Option<WindowCommand>,
}

impl ScrollDemo {
    fn new() -> Self {
        Self {
            bar: Titlebar::new("Scrollbar"),
            scroller: Scrollbar::new(),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            text: Color::WHITE,
            content: (0.0, 0.0),
            command: None,
        }
    }

    fn line(&self, index: usize) -> String {
        format!("Line {:02} - Lorem ipsum dolor sit amet", index + 1)
    }
}

impl App for ScrollDemo {
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
        let top = viewport.y + 31.0;

        // Text area left, scrollbar at the right edge.
        let area_x = viewport.x + 24.0;
        let area_y = top + 64.0;
        let area_w = viewport.width - 48.0 - 16.0;
        let area_h = (viewport.height - 64.0 - 47.0 - 16.0).max(80.0);
        let total = LINES as f32 * LINE_H;
        // Only push new content (set_content flashes the bar).
        if (total, area_h) != self.content {
            self.content = (total, area_h);
            self.scroller.set_content(total, area_h);
        }
        self.scroller.set_theme(palette.accent, dark);
        self.scroller.set_focused(self.focused);
        self.scroller.set_rect(
            area_x + area_w + 6.0,
            area_y,
            10.0,
            area_h,
        );
        let offset = self.scroller.offset();

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
        self.bar
            .set_title(format!("Scrollbar - {}", offset as i32));

        // Caption: the bar only shows while scrolling.
        let caption = "Scroll, hover or drag - the bar fades out when idle";
        let layout = fonts.layout_text_weighted(caption, 13.0, self.text, 400.0, None);
        draw_layout(scene, &layout, area_x, top + 16.0, fonts.scale);

        // Clipped text column driven by the scrollbar offset.
        let clip = vello::kurbo::Rect::new(
            area_x as f64 * fonts.scale as f64,
            area_y as f64 * fonts.scale as f64,
            (area_x + area_w) as f64 * fonts.scale as f64,
            (area_y + area_h) as f64 * fonts.scale as f64,
        );
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
        let first = (offset / LINE_H).floor() as usize;
        for i in first..LINES.min(first + (area_h / LINE_H).ceil() as usize + 1) {
            let layout = fonts.layout_text_weighted(&self.line(i), 13.0, self.text, 400.0, None);
            draw_layout(
                scene,
                &layout,
                area_x,
                area_y + i as f32 * LINE_H - offset,
                fonts.scale,
            );
        }
        scene.pop_layer();

        self.scroller.draw(scene, fonts, images);
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
            None => {
                self.scroller.mouse_down(x, y);
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.scroller.mouse_move(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.scroller.mouse_up(x, y);
    }

    fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        self.scroller.mouse_wheel(dx, dy);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.scroller.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Scrollbar", 480, 420, ScrollDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
