use tontooui::elements::{
    BasicList, DisclosureGroup, ListRow, Titlebar, TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct ListDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ListDemo {
    fn new() -> Self {
        // Full category showcase: plain rows, sections, badges, row
        // styles and disclosure groups.
        let stack = VStack::new()
            .spacing(24.0)
            .child(BasicList::from_slice(&[
                "Row 1", "Row 2", "Row 3", "Row 4", "Row 5", "Row 6", "Row 7",
                "Row 8", "Row 9", "Row 10",
            ]))
            .child(BasicList::from_rows(vec![
                ListRow::item("Item 1"),
                ListRow::item("Item 2"),
                ListRow::section("Grouped"),
                ListRow::item("Item 3"),
                ListRow::item("Item 4"),
            ]))
            .child(BasicList::from_rows(vec![
                ListRow::item("Inbox").badge("5"),
                ListRow::item("Drafts").badge("12"),
                ListRow::item("Sent"),
                ListRow::item("Trash").badge("100"),
            ]))
            .child(BasicList::from_rows(vec![
                ListRow::item("Default Row"),
                ListRow::item("Custom Background").background(Color::from_rgba8(
                    0x00, 0x7a, 0xff, 38,
                )),
                ListRow::item("Tinted Item")
                    .text_color(Color::from_rgb8(0x64, 0xd2, 0xff))
                    .no_divider(),
            ]))
            .child(
                DisclosureGroup::from_slice(
                    "Fruits",
                    &["Apple", "Banana", "Cherry", "Date"],
                )
                .open(true),
            )
            .child(DisclosureGroup::from_slice(
                "Vegetables",
                &["Carrot", "Lettuce"],
            ))
            .child(DisclosureGroup::from_slice("Grains", &["Rice", "Wheat"]));
        Self {
            bar: Titlebar::new("List"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_list(&mut self, mut f: impl FnMut(&mut BasicList)) {
        // Mixed stack: skip type mismatches instead of stopping at
        // the first one.
        for index in 0..self.stack.len() {
            if let Some(list) = self.stack.child_mut::<BasicList>(index) {
                f(list);
            }
        }
    }

    fn each_group(&mut self, mut f: impl FnMut(&mut DisclosureGroup)) {
        // Mixed stack: skip type mismatches instead of stopping at
        // the first one.
        for index in 0..self.stack.len() {
            if let Some(group) = self.stack.child_mut::<DisclosureGroup>(index) {
                f(group);
            }
        }
    }
}

impl App for ListDemo {
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
        self.each_list(|list| {
            list.set_theme(palette.divider, dark);
            list.set_focused(focused);
        });
        self.each_group(|group| {
            group.set_theme(palette.divider, dark);
            group.set_focused(focused);
        });

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let (_, stack_h) = self.stack.measure(fonts);
        self.stack.place(fonts, viewport.x, top, viewport.width, stack_h);
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
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => self.each_group(|group| group.mouse_down(x, y)),
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if self.bar.press(x as f32, y as f32).is_none() {
            self.each_group(|group| group.mouse_up(x, y));
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("List", 900, 1100, ListDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
