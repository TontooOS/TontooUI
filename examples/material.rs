use tontooui::elements::{
    ALL_MATERIALS, BasicText, GradientPaint, Material, Padding, TextAlignment,
    TextStyle, Titlebar, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::ThemeWatcher;
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};

struct MaterialDemo {
    bar: Titlebar,
    rows: Vec<Material<Padding>>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl MaterialDemo {
    fn new() -> Self {
        // One title bar plus one subtitle bar per thickness, like
        // the reference rows.
        let rows = ALL_MATERIALS
            .iter()
            .map(|kind| {
                let title = format!("{} Material", kind.name());
                let subtitle = match kind {
                    tontooui::elements::MaterialKind::UltraThin => {
                        "This text has a very subtle blur effect behind it."
                    }
                    tontooui::elements::MaterialKind::Thin => {
                        "A slightly more prominent blur effect."
                    }
                    tontooui::elements::MaterialKind::Regular => {
                        "The standard material blur effect."
                    }
                    tontooui::elements::MaterialKind::Thick => {
                        "A heavy blur that obscures more content behind it."
                    }
                    tontooui::elements::MaterialKind::UltraThick => {
                        "The heaviest blur, almost completely opaque."
                    }
                };
                Material::new(
                    Padding::all(
                        VStack::new()
                            .spacing(16.0)
                            .child(
                                BasicText::new(title)
                                    .style(TextStyle::Title2)
                                    .alignment(TextAlignment::Center)
                                    .foreground_color(Color::WHITE),
                            )
                            .child(
                                BasicText::new(subtitle)
                                    .alignment(TextAlignment::Center)
                                    .foreground_color(Color::WHITE),
                            ),
                        20.0,
                    ),
                    *kind,
                )
            })
            .collect();
        Self {
            bar: Titlebar::new("Material"),
            rows,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }
}

impl App for MaterialDemo {
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
        let focused = self.focused;
        for row in &mut self.rows {
            row.set_theme(true);
            row.set_focused(focused);
        }

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        // Colorful backdrop so the veils read: blue to purple.
        let top = viewport.y + 31.0;
        let bg_brush = GradientPaint::preset_linear().brush(
            viewport.x,
            top,
            viewport.width,
            viewport.height - 31.0,
            fonts.scale,
        );
        let scale = fonts.scale as f64;
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &bg_brush,
            None,
            &Rect::new(
                viewport.x as f64 * scale,
                top as f64 * scale,
                (viewport.x + viewport.width) as f64 * scale,
                (viewport.y + viewport.height) as f64 * scale,
            ),
        );

        // Stacked bars, centered, fixed widths.
        let mut y = top + 28.0;
        for row in &mut self.rows {
            let (w, h) = row.measure(fonts);
            let bw = w.max(420.0);
            let bx = viewport.x + ((viewport.width - bw) / 2.0).max(0.0);
            row.place(fonts, bx, y, bw, h);
            row.draw(scene, fonts, images);
            y += h + 20.0;
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
    if let Err(err) = run("Material", 900, 1100, MaterialDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
