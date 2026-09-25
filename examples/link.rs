use tontooui::elements::{
    Align, BasicLink, LinkStyle, LinkWithImage, StyledLink, Titlebar,
    TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::ThemeWatcher;
use vello::Scene;
use vello::peniko::Color;

struct LinkDemo {
    bar: Titlebar,
    stack: VStack,
    styled: StyledLink,
    bordered: StyledLink,
    card: LinkWithImage,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl LinkDemo {
    fn new() -> Self {
        // Reference rows: plain link plus icon link.
        let stack = VStack::new()
            .align(Align::Center)
            .spacing(16.0)
            .child(BasicLink::new("Visit Apple", "https://apple.com"))
            .child(
                BasicLink::new("Swift.org", "https://swift.org")
                    .icon("chevron.left.forwardslash.chevron.right"),
            );
        Self {
            bar: Titlebar::new("Link"),
            stack,
            styled: StyledLink::new("Styled Link", "https://example.com"),
            bordered: StyledLink::new("Border Link", "https://example.com")
                .style(LinkStyle::Border),
            card: LinkWithImage::new(
                "https://picsum.photos/320/200",
                "Download App",
                "https://example.com/download",
                320.0,
                200.0,
            ),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_link(&mut self, mut f: impl FnMut(&mut BasicLink)) {
        for index in 0..self.stack.len() {
            if let Some(link) = self.stack.child_mut::<BasicLink>(index) {
                f(link);
            }
        }
    }
}

impl App for LinkDemo {
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
        self.each_link(|link| link.set_focused(focused));
        self.styled.set_focused(focused);
        self.bordered.set_focused(focused);
        self.card.image_mut().set_theme(true);
        self.card.image_mut().set_focused(focused);
        self.card.link_mut().set_focused(focused);

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
        self.stack.place(fonts, x, top + 32.0, stack_w, stack_h);
        self.stack.draw(scene, fonts, images);
        // Styled pills below the basics.
        let (sw, sh) = self.styled.measure(fonts);
        let sx = viewport.x + ((viewport.width - sw) / 2.0).max(0.0);
        let mut y = top + 32.0 + stack_h + 28.0;
        self.styled.place(fonts, sx, y, sw, sh);
        self.styled.draw(scene, fonts, images);
        y += sh + 16.0;
        let (bw2, bh2) = self.bordered.measure(fonts);
        let bx2 = viewport.x + ((viewport.width - bw2) / 2.0).max(0.0);
        self.bordered.place(fonts, bx2, y, bw2, bh2);
        self.bordered.draw(scene, fonts, images);
        y += bh2 + 28.0;
        // Image card last.
        let (cw, ch) = self.card.measure(fonts);
        let cx = viewport.x + ((viewport.width - cw) / 2.0).max(0.0);
        self.card.place(fonts, cx, y, cw, ch);
        self.card.draw(scene, fonts, images);
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
            None => {
                self.each_link(|link| link.mouse_down(x, y));
                self.styled.mouse_down(x, y);
                self.bordered.mouse_down(x, y);
                self.card.mouse_down(x, y);
            }
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_link(|link| link.mouse_up(x, y));
        self.styled.mouse_up(x, y);
        self.bordered.mouse_up(x, y);
        self.card.mouse_up(x, y);
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.each_link(|link| link.set_hover(x as f32, y as f32));
        self.styled.set_hover(x as f32, y as f32);
        self.bordered.set_hover(x as f32, y as f32);
        self.card.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Link", 900, 480, LinkDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
