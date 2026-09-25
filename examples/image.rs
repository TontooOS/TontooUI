use tontooui::elements::{
    AppImage, HStack, ImageFit, ImageOverlay, SFSymbolImage, Titlebar,
    TrafficAction, UrlImage, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct ImageDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ImageDemo {
    fn new() -> Self {
        // SF Symbols: plain, sized up, hand-colored.
        let symbols = HStack::new()
            .spacing(32.0)
            .child(SFSymbolImage::new("star.fill"))
            .child(SFSymbolImage::new("heart.fill").size(40.0))
            .child(
                SFSymbolImage::new("bell.fill")
                    .size(40.0)
                    .color(Color::from_rgb8(0xff, 0x9f, 0x0a)),
            );
        // App resources: cover (crop) and fit (letterbox). The demo
        // asset lives in `./assets/demo-star.png` (dev / cargo run).
        let files = HStack::new()
            .spacing(32.0)
            .child(AppImage::new("demo-star", 200.0, 130.0))
            .child(AppImage::new("demo-star", 200.0, 130.0).fit(ImageFit::Fit))
            .child(AppImage::new("missing-file", 200.0, 130.0));
        // Remote images: a real photo (spinner, then image) and a
        // 404 URL (spinner, then "Error 404").
        let remote = HStack::new()
            .spacing(32.0)
            .child(UrlImage::new("https://picsum.photos/400/260", 200.0, 130.0))
            .child(UrlImage::new(
                "https://example.com/missing-image.png",
                200.0,
                130.0,
            ));
        // Overlay cards: resource photo with caption plus badge, and
        // a caption-only card.
        let cards = HStack::new()
            .spacing(32.0)
            .child(
                ImageOverlay::resource("demo-star", 220.0, 140.0)
                    .caption("Demo star")
                    .badge("heart.fill"),
            )
            .child(
                ImageOverlay::resource("demo-star", 220.0, 140.0)
                    .caption("Caption only"),
            );
        let stack = VStack::new()
            .spacing(28.0)
            .child(symbols)
            .child(files)
            .child(remote)
            .child(cards);
        Self {
            bar: Titlebar::new("Image"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn wire_row(row: &mut HStack, palette_text: Color, dark: bool, focused: bool) {
        for index in 0..row.len() {
            if let Some(symbol) = row.child_mut::<SFSymbolImage>(index) {
                symbol.set_theme(palette_text, dark);
                symbol.set_focused(focused);
            } else if let Some(image) = row.child_mut::<AppImage>(index) {
                image.set_theme(dark);
                image.set_focused(focused);
            } else if let Some(image) = row.child_mut::<UrlImage>(index) {
                image.set_theme(dark);
                image.set_focused(focused);
            } else if let Some(card) = row.child_mut::<ImageOverlay>(index) {
                card.set_theme(dark);
                card.set_focused(focused);
            }
        }
    }

    fn each_row(&mut self, mut f: impl FnMut(&mut HStack)) {
        for index in 0..self.stack.len() {
            if let Some(row) = self.stack.child_mut::<HStack>(index) {
                f(row);
            }
        }
    }
}

impl App for ImageDemo {
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
        let (text, focused) = (palette.text, self.focused);
        self.each_row(|row| Self::wire_row(row, text, dark, focused));

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
    if let Err(err) = run("Image", 900, 900, ImageDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
