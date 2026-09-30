use tontooui::elements::{
    Align, BarSwitcher, BarSwitcherItem, Titlebar, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::kurbo::{Affine, Point, Rect};
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};

struct BarSwitcherDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    text: Color,
    command: Option<WindowCommand>,
    view_label: String,
}

impl BarSwitcherDemo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(16.0)
            .align(Align::Center)
            .child(
                BarSwitcher::from_items(vec![
                    BarSwitcherItem::both("square.grid.2x2", "Grid"),
                    BarSwitcherItem::icon("list.bullet"),
                    BarSwitcherItem::text("Settings"),
                ])
                .on_select(|index| println!("bar switcher selected {index}")),
            );
        Self {
            bar: Titlebar::new("BarSwitcher"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            text: Color::WHITE,
            command: None,
            view_label: "Grid".to_string(),
        }
    }

    fn switcher_mut(&mut self) -> Option<&mut BarSwitcher> {
        self.stack.child_mut::<BarSwitcher>(0)
    }

    fn refresh_view_label(&mut self) {
        if let Some(switcher) = self.switcher_mut() {
            let label = switcher
                .selected_item()
                .and_then(|item| item.label.clone())
                .unwrap_or_else(|| format!("Item {}", switcher.selected_index()));
            self.view_label = label;
        }
    }
}

impl App for BarSwitcherDemo {
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
        let theme = self.watcher.theme();
        let dark = theme.mode == ThemeMode::Dark;
        self.bg = draw_wallpaper(scene, fonts, viewport, dark);
        self.text = palette.text;
        let focused = self.focused;
        if let Some(switcher) = self.switcher_mut() {
            switcher.set_theme(theme.mode, theme.glass);
            switcher.set_focused(focused);
        }
        self.refresh_view_label();

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        self.stack.place(
            fonts,
            viewport.x,
            top + 24.0,
            viewport.width,
            (viewport.height - 55.0).max(0.0),
        );
        self.stack.draw(scene, fonts, images);

        // Active view caption under the switcher, swapped on selection.
        let caption = format!("View: {}", self.view_label);
        let layout = fonts.layout_text_weighted(&caption, 17.0, self.text, 600.0, None);
        let (tw, _) = FontSystem::layout_size(&layout);
        let cx = viewport.x + (viewport.width - tw / fonts.scale) / 2.0;
        let cy = top + 24.0 + 36.0 + 16.0;
        tontooui::renderer::text::draw_layout(scene, &layout, cx, cy, fonts.scale);
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn wants_backdrop(&self) -> bool {
        true
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
            None => {
                if let Some(switcher) = self.switcher_mut() {
                    switcher.mouse_down(x, y);
                }
            }
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if let Some(switcher) = self.switcher_mut() {
            switcher.mouse_up(x, y);
        }
        self.refresh_view_label();
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        if let Some(switcher) = self.switcher_mut() {
            switcher.mouse_move(x as f32, y as f32);
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("BarSwitcher", 800, 600, BarSwitcherDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

/// Wallpaper gradient covering the viewport, theme-aware: saturated
/// indigo/violet/teal in dark mode, pastel in light mode. Returns the
/// top stop so `background()` matches the window edges.
fn draw_wallpaper(
    scene: &mut Scene,
    fonts: &FontSystem,
    viewport: Viewport,
    dark: bool,
) -> Color {
    let stops = if dark {
        [
            Color::from_rgb8(0x43, 0x34, 0x9e),
            Color::from_rgb8(0x7c, 0x3a, 0xed),
            Color::from_rgb8(0x0e, 0x74, 0x90),
        ]
    } else {
        [
            Color::from_rgb8(0xc7, 0xd2, 0xfe),
            Color::from_rgb8(0xf0, 0xab, 0xfc),
            Color::from_rgb8(0x99, 0xf6, 0xe4),
        ]
    };
    let scale = fonts.scale as f64;
    let gradient = Gradient::new_linear(
        Point::new(
            viewport.x as f64 * scale,
            viewport.y as f64 * scale,
        ),
        Point::new(
            (viewport.x + viewport.width) as f64 * scale,
            (viewport.y + viewport.height) as f64 * scale,
        ),
    )
    .with_stops([
        ColorStop {
            offset: 0.0,
            color: stops[0].into(),
        },
        ColorStop {
            offset: 0.55,
            color: stops[1].into(),
        },
        ColorStop {
            offset: 1.0,
            color: stops[2].into(),
        },
    ]);
    let rect = Rect::new(
        viewport.x as f64 * scale,
        viewport.y as f64 * scale,
        (viewport.x + viewport.width) as f64 * scale,
        (viewport.y + viewport.height) as f64 * scale,
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(gradient),
        None,
        &rect,
    );
    stops[0]
}
