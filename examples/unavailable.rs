use tontooui::elements::{
    BasicText, ContentUnavailable, SearchEmpty, Titlebar, TrafficAction,
    View,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::ThemeWatcher;
use vello::Scene;
use vello::peniko::Color;

struct UnavailableDemo {
    bar: Titlebar,
    status: BasicText,
    empty: ContentUnavailable,
    plain: ContentUnavailable,
    search: SearchEmpty,
    refreshes: u32,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl UnavailableDemo {
    fn new() -> Self {
        Self {
            bar: Titlebar::new("Empty"),
            status: BasicText::new("refreshes: 0"),
            empty: ContentUnavailable::new(
                "tray",
                "No Data",
                "There is no data to display yet. Pull down to refresh.",
            ),
            plain: ContentUnavailable::new(
                "star.fill",
                "No Favorites",
                "Stars you tap will show up here.",
            )
            .refresh(false),
            search: SearchEmpty::new(
                "magnifyingglass",
                "No Results",
                "Check the spelling or try a new search.",
            ),
            refreshes: 0,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }
}

impl App for UnavailableDemo {
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

        if self.empty.take_refreshed() {
            self.refreshes += 1;
            self.empty.finish_refresh();
        }
        self.empty.set_theme(theme.mode, palette.accent);
        self.empty.set_focused(focused);
        self.plain.set_theme(theme.mode, palette.accent);
        self.plain.set_focused(focused);
        self.search.set_theme(theme.mode);
        self.search.set_focused(focused);
        self.status
            .set_text(format!("refreshes: {}", self.refreshes));
        self.status.set_theme(theme.mode);
        self.status.set_focused(focused);

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        // Status line, then both variants stacked and centered.
        let top = viewport.y + 31.0;
        let (sw, sh) = self.status.measure(fonts);
        self.status.place(
            fonts,
            viewport.x + ((viewport.width - sw) / 2.0).max(0.0),
            top + 12.0,
            sw,
            sh,
        );
        self.status.draw(scene, fonts, images);

        let (ew, eh) = self.empty.measure(fonts);
        let (pw, ph) = self.plain.measure(fonts);
        let (qw, qh) = self.search.measure(fonts);
        let col_w = ew.max(pw).max(qw);
        let cx = viewport.x + ((viewport.width - col_w) / 2.0).max(0.0);
        let mut y = top + 12.0 + sh + 24.0;
        // Refresh variant, plain variant, search variant below.
        self.empty.place(fonts, cx, y, col_w, eh);
        self.empty.draw(scene, fonts, images);
        y += eh + 40.0;
        self.plain.place(fonts, cx, y, col_w, ph);
        self.plain.draw(scene, fonts, images);
        y += ph + 40.0;
        self.search.place(fonts, cx, y, col_w, qh);
        self.search.draw(scene, fonts, images);
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
                self.empty.mouse_down(x, y);
                self.plain.mouse_down(x, y);
            }
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.empty.mouse_up(x, y);
        self.plain.mouse_up(x, y);
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.empty.set_hover(x as f32, y as f32);
        self.plain.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Empty", 900, 900, UnavailableDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
