use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tontooui::elements::{
    BasicText, SIDEBAR_ICON_GAP, SIDEBAR_ICON_SIZE, SIDEBAR_LABEL_SIZE, SIDEBAR_ROW_H,
    Sidebar, SidebarItem, Slider, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, CursorKind, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct SidebarDemo {
    sidebar: Rc<RefCell<Sidebar>>,
    history: Rc<RefCell<Vec<usize>>>,
    go_back: Rc<Cell<bool>>,
    jump_to: Rc<Cell<Option<usize>>>,
    row_h: Rc<Cell<f64>>,
    row_icon: Rc<Cell<f64>>,
    row_gap: Rc<Cell<f64>>,
    row_label: Rc<Cell<f64>>,
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
        // Live row metrics tuned on the General page; draw applies
        // them (slider callbacks never touch the sidebar directly).
        let row_h: Rc<Cell<f64>> = Rc::new(Cell::new(SIDEBAR_ROW_H as f64));
        let row_icon: Rc<Cell<f64>> = Rc::new(Cell::new(SIDEBAR_ICON_SIZE as f64));
        let row_gap: Rc<Cell<f64>> = Rc::new(Cell::new(SIDEBAR_ICON_GAP as f64));
        let row_label: Rc<Cell<f64>> = Rc::new(Cell::new(SIDEBAR_LABEL_SIZE as f64));
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
            .page(Self::sizes_page(
                row_h.clone(),
                row_icon.clone(),
                row_gap.clone(),
                row_label.clone(),
            ))
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
            .left_button(0, "chevron.left", {
                let go_back = go_back.clone();
                move || go_back.set(true)
            })
            .left_button(1, "magnifyingglass", {
                let jump_to = jump_to.clone();
                move || jump_to.set(Some(3))
            }),
        ));
        Self {
            sidebar,
            history,
            go_back,
            jump_to,
            row_h,
            row_icon,
            row_gap,
            row_label,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    /// General page: sliders tuning the sidebar row sizes live.
    fn sizes_page(
        row_h: Rc<Cell<f64>>,
        row_icon: Rc<Cell<f64>>,
        row_gap: Rc<Cell<f64>>,
        row_label: Rc<Cell<f64>>,
    ) -> VStack {
        fn slider(
            title: &str,
            initial: f64,
            min: f64,
            max: f64,
            cell: Rc<Cell<f64>>,
        ) -> Slider {
            Slider::new(initial, min, max)
                .step(0.5)
                .title(title)
                .value_text(|v| format!("{v:.1}"))
                .on_change(move |v| cell.set(v))
        }
        VStack::new()
            .spacing(16.0)
            .child(BasicText::new("General settings"))
            .child(BasicText::new("System appearance and behavior."))
            .child(slider(
                "Row height",
                row_h.get(),
                16.0,
                64.0,
                row_h.clone(),
            ))
            .child(slider(
                "Icon size",
                row_icon.get(),
                8.0,
                40.0,
                row_icon.clone(),
            ))
            .child(slider("Icon gap", row_gap.get(), 0.0, 24.0, row_gap.clone()))
            .child(slider(
                "Label size",
                row_label.get(),
                8.0,
                28.0,
                row_label.clone(),
            ))
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
            sidebar.set_glass(theme.mode, theme.glass);
            sidebar.set_focused(focused);
            // Live row sizes from the General sliders.
            sidebar.set_row_metrics(
                self.row_h.get() as f32,
                self.row_icon.get() as f32,
                self.row_gap.get() as f32,
                self.row_label.get() as f32,
            );
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

    fn wants_backdrop(&self) -> bool {
        // Lens toolbar pills need the blur pass while visible.
        self.sidebar.borrow().wants_backdrop()
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

    fn cursor(&self, x: f64, y: f64) -> CursorKind {
        let bar = self.sidebar.borrow();
        if bar.wants_resize_cursor(x, y) {
            CursorKind::ResizeColumn
        } else if bar.search_text_cursor() {
            CursorKind::Text
        } else {
            CursorKind::Default
        }
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
