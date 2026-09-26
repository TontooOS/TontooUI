use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tontooui::elements::{
    BasicText, Button, ButtonShape, Sidebar, SidebarItem, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct SidebarDemo {
    sidebar: Rc<RefCell<Sidebar>>,
    history: Rc<RefCell<Vec<usize>>>,
    go_back: Rc<Cell<bool>>,
    jump_to: Rc<Cell<Option<usize>>>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl SidebarDemo {
    fn new() -> Self {
        let history: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(vec![0]));
        let go_back: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let jump_to: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
        // Shared so press callbacks (back, dev button, selection)
        // can reach the sidebar from `'static` closures.
        let sidebar: Rc<RefCell<Sidebar>> = Rc::new(RefCell::new(
            Sidebar::new(vec![
                SidebarItem::new("General", "gear"),
                SidebarItem::new("Security", "lock.fill"),
                SidebarItem::new("Privacy", "hand.raised.fill"),
                SidebarItem::new("Notifications", "bell.fill"),
                SidebarItem::new("Storage", "internaldrive"),
            ])
            .page(Self::page("General", "System appearance and behavior."))
            .page(Self::page("Security", "Passwords and encryption."))
            .page(Self::page("Privacy", "Tracking and permissions."))
            .page(Self::page("Notifications", "Banners, sounds and badges."))
            .page(Self::page("Storage", "Disks and usage."))
            .on_select({
                let history = history.clone();
                move |index| {
                    let mut trail = history.borrow_mut();
                    if trail.last() != Some(&index) {
                        trail.push(index);
                    }
                }
            })
            .on_back({
                let go_back = go_back.clone();
                move || go_back.set(true)
            }),
        ));
        // Dev toolbar button: jumps to Notifications. Added after
        // construction; the callback only sets a flag (selecting
        // here would re-enter the borrowed sidebar, draw applies it).
        sidebar.borrow_mut().add_toolbar_button(
            Button::new("")
                .shape(ButtonShape::Circle)
                .icon("magnifyingglass")
                .icon_size(20.0)
                .on_press({
                    let jump_to = jump_to.clone();
                    move || jump_to.set(Some(3))
                }),
        );
        Self {
            sidebar,
            history,
            go_back,
            jump_to,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn page(title: &str, body: &str) -> VStack {
        VStack::new()
            .spacing(8.0)
            .child(BasicText::new(format!("{title} settings")))
            .child(BasicText::new(body))
    }
}

impl App for SidebarDemo {
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
        {
            let mut sidebar = self.sidebar.borrow_mut();
            sidebar.set_theme(palette.accent, dark);
            sidebar.set_focused(focused);
        }
        // Back navigation: pop the trail and select the previous
        // item (`on_select` skips the push since it is already last).
        if self.go_back.take() {
            let previous = {
                let mut trail = self.history.borrow_mut();
                trail.pop();
                trail.last().copied().unwrap_or(0)
            };
            self.sidebar.borrow_mut().select(previous);
        }
        // Dev search button: jump applied here, never re-entered
        // from the press callback itself.
        if let Some(index) = self.jump_to.take() {
            self.sidebar.borrow_mut().select(index);
        }
        // No titlebar: the sidebar owns the decoration (traffic
        // lights live in it) and fills the whole viewport.
        self.sidebar.borrow_mut().place(
            fonts,
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
        );
        self.sidebar.borrow_mut().draw(scene, fonts, images);
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        Some(self.sidebar.borrow().drag_rect())
    }

    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        self.command.take()
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        // Bind first: the borrow guard would otherwise live into the
        // else branch and panic on the second borrow.
        let traffic = self.sidebar.borrow_mut().press(x, y);
        if let Some(action) = traffic {
            match action {
                TrafficAction::Close => self.command = Some(WindowCommand::Close),
                TrafficAction::Minimize => self.command = Some(WindowCommand::Minimize),
                TrafficAction::Maximize => {
                    self.command = Some(WindowCommand::ToggleMaximize)
                }
            }
        } else {
            self.sidebar.borrow_mut().mouse_down(x, y);
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.sidebar.borrow_mut().set_hover(x as f32, y as f32);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.sidebar.borrow_mut().mouse_up(x, y);
    }

    fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        self.sidebar.borrow_mut().mouse_wheel(dx, dy);
    }

    fn text(&mut self, text: &str) {
        self.sidebar.borrow_mut().page_text(text);
    }

    fn key(&mut self, key: Key) {
        self.sidebar.borrow_mut().page_key(key);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.sidebar.borrow_mut().set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Sidebar", 900, 620, SidebarDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
