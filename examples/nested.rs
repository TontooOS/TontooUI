use tontooui::elements::{MenuItem, NestedMenu, Titlebar, TrafficAction, View, VStack};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::text::draw_layout;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

fn action_label(menu: &NestedMenu) -> String {
    match menu.last_action() {
        Some(path) => path
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("/"),
        None => "-".to_string(),
    }
}

struct NestedDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    text: Color,
    last_action: String,
    menu_open: bool,
    command: Option<WindowCommand>,
}

impl NestedDemo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(16.0)
            .child(NestedMenu::new(
                "Share",
                vec![
                    MenuItem::section("Choose destination"),
                    MenuItem::submenu(
                        "Messages",
                        vec![
                            MenuItem::action("John"),
                            MenuItem::action("Jane"),
                            MenuItem::action("Bob"),
                        ],
                    ),
                    MenuItem::submenu("Social", vec![MenuItem::action("Post")]),
                    MenuItem::divider(),
                    MenuItem::action("More..."),
                ],
            ))
            .child(NestedMenu::new(
                "File",
                vec![
                    MenuItem::section("File Operations"),
                    MenuItem::action("New File"),
                    MenuItem::action("Open File"),
                    MenuItem::action("Save"),
                    MenuItem::divider(),
                    MenuItem::section("Edit Operations"),
                    MenuItem::action("Undo"),
                    MenuItem::action("Redo"),
                ],
            ));
        Self {
            bar: Titlebar::new("Nested"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            text: Color::WHITE,
            last_action: "-".to_string(),
            menu_open: false,
            command: None,
        }
    }

    fn each_menu(&mut self, mut f: impl FnMut(&mut NestedMenu)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<NestedMenu>(index) {
                Some(menu) => f(menu),
                None => break,
            }
            index += 1;
        }
    }

    fn refresh(&mut self) {
        let mut last = "-".to_string();
        let mut open = false;
        self.each_menu(|menu| {
            if menu.last_action().is_some() {
                last = action_label(menu);
            }
            open = open || menu.is_open();
        });
        self.last_action = last;
        self.menu_open = open;
    }
}

impl App for NestedDemo {
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
        let focused = self.focused;
        self.each_menu(|menu| {
            menu.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
            menu.set_theme(palette.accent, dark);
            menu.set_glass(theme.mode, theme.glass);
            menu.set_focused(focused);
        });
        self.refresh();

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
        self.bar
            .set_title(format!("Nested — {}", self.last_action));

        // Caption before the stack: open panels float above it.
        let caption = format!("Selected: {}", self.last_action);
        let layout = fonts.layout_text_weighted(&caption, 17.0, self.text, 600.0, None);
        let (tw, _) = FontSystem::layout_size(&layout);
        let cx = viewport.x + (viewport.width - tw / fonts.scale) / 2.0;
        let top = viewport.y + 31.0;
        draw_layout(scene, &layout, cx, top + 8.0, fonts.scale);

        self.stack.place(
            fonts,
            viewport.x + 24.0,
            top + 40.0,
            viewport.width - 48.0,
            (viewport.height - 71.0).max(0.0),
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
        self.menu_open
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => {
                self.each_menu(|menu| menu.mouse_down(x, y));
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.each_menu(|menu| menu.mouse_move(x, y));
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_menu(|menu| menu.mouse_up(x, y));
        self.refresh();
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.each_menu(|menu| menu.set_focused(focused));
    }
}

fn main() {
    if let Err(err) = run("Nested", 480, 420, NestedDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
