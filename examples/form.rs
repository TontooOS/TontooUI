use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tontooui::elements::{
    BasicText, Button, ButtonStyle, Form, FormRow, FormSection, ScrollView, Titlebar,
    TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, CursorKind, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct FormDemo {
    bar: Titlebar,
    scroll: ScrollView,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    menu_open: Cell<bool>,
    text_cursor: Cell<bool>,
    status: Rc<RefCell<String>>,
    command: Option<WindowCommand>,
}

impl FormDemo {
    fn new() -> Self {
        // Basic form: one untitled group of text rows.
        let basic = Form::new().section(
            FormSection::new()
                .row(FormRow::text("Name", "").placeholder("Jane Appleseed"))
                .row(FormRow::text("Email", "").placeholder("jane@tontoo.os"))
                .row(FormRow::secure("Password", "")),
        );
        // Form sections: titled groups with toggles and a picker.
        let sections = Form::new()
            .section(
                FormSection::titled("Connection")
                    .row(FormRow::text("Username", "octo"))
                    .row(FormRow::text("Host", "tontoo.os"))
                    .row(FormRow::text("Port", "22")),
            )
            .section(
                FormSection::titled("Authentication")
                    .row(FormRow::secure("Password", ""))
                    .row(FormRow::toggle("Use SSH Key", false)),
            )
            .section(
                FormSection::titled("Options")
                    .row(FormRow::toggle("Enable notifications", true))
                    .row(
                        FormRow::picker(
                            "Protocol",
                            vec!["FTP".into(), "SFTP".into(), "WebDAV".into()],
                            1,
                        )
                        .icon("globe"),
                    ),
            );
        // Form with picker: profile rows plus a schedule section.
        let picker = Form::new()
            .section(
                FormSection::titled("Profile")
                    .row(FormRow::text("Full Name", ""))
                    .row(
                        FormRow::picker("Role", vec!["Admin".into(), "Editor".into(), "Viewer".into()], 0)
                            .icon("tag"),
                    ),
            )
            .section(
                FormSection::titled("Schedule").row(FormRow::text("Start Date", "25. 09. 2026")),
            );
        // Form button row: agreement plus centered Cancel/Save.
        // Buttons carry press callbacks: the status lands in the
        // titlebar through shared state.
        let status: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let saved = status.clone();
        let cancelled = status.clone();
        let buttons = Form::new()
            .section(
                FormSection::new().row(FormRow::text("Email Address", "")),
            )
            .section(
                FormSection::new()
                    .row(FormRow::toggle("I agree to the terms", false))
                    .footnote("Please read the terms before continuing."),
            )
            .section(
                FormSection::new().row(FormRow::buttons(vec![
                    Button::new("Cancel").style(ButtonStyle::Bordered).on_press(move || {
                        *cancelled.borrow_mut() = "Cancelled".to_string();
                    }),
                    Button::new("Save").style(ButtonStyle::BorderedProminent).on_press(move || {
                        *saved.borrow_mut() = "Saved".to_string();
                    }),
                ])),
            );
        let stack = VStack::new()
            .spacing(28.0)
            .child(BasicText::new("Basic Form"))
            .child(basic)
            .child(BasicText::new("Form Sections"))
            .child(sections)
            .child(BasicText::new("Form with Picker"))
            .child(picker)
            .child(BasicText::new("Form Button Row"))
            .child(buttons);
        Self {
            bar: Titlebar::new("Form"),
            scroll: ScrollView::new(stack),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            menu_open: Cell::new(false),
            text_cursor: Cell::new(false),
            status,
            command: None,
        }
    }

    fn each_form(&mut self, mut f: impl FnMut(&mut Form)) {
        if let Some(stack) = self.scroll.child_mut::<VStack>() {
            for index in 0..stack.len() {
                if let Some(form) = stack.child_mut::<Form>(index) {
                    f(form);
                }
            }
        }
    }

    fn each_label(&mut self, mut f: impl FnMut(&mut BasicText)) {
        if let Some(stack) = self.scroll.child_mut::<VStack>() {
            for index in 0..stack.len() {
                if let Some(label) = stack.child_mut::<BasicText>(index) {
                    f(label);
                }
            }
        }
    }

    fn refresh_flags(&mut self) {
        let mut open = false;
        let mut text = false;
        self.each_form(|form| {
            open |= form.wants_backdrop();
            text |= form.wants_text_cursor();
        });
        self.menu_open.set(open);
        self.text_cursor.set(text);
    }
}

impl App for FormDemo {
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
        self.each_label(|label| {
            label.set_theme(theme.mode);
            label.set_focused(focused);
        });
        self.each_form(|form| {
            form.set_theme(palette.accent, dark);
            form.set_glass(theme.mode, theme.glass);
            form.set_focused(focused);
            form.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        });

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        let status = self.status.borrow().clone();
        self.bar.set_title(if status.is_empty() {
            "Form".to_string()
        } else {
            format!("Form — {status}")
        });
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        self.scroll.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0,
            viewport.width - 48.0,
            (viewport.height - 47.0).max(0.0),
        );
        self.scroll.draw(scene, fonts, images);
        self.refresh_flags();
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
        // Glass picker panels need the blur pass while open.
        self.menu_open.get()
    }

    fn cursor(&self, _x: f64, _y: f64) -> CursorKind {
        if self.text_cursor.get() {
            CursorKind::Text
        } else {
            CursorKind::Default
        }
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            // Single delivery: the scroll view routes presses through
            // the stack into the forms. Forwarding to both would arm
            // every control twice (a menu would open and instantly
            // close again).
            None => self.scroll.mouse_down(x, y),
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.scroll.mouse_move(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.scroll.mouse_up(x, y);
    }

    fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        // Picker panels cap at the window and never scroll.
        self.scroll.mouse_wheel(dx, dy);
    }

    fn text(&mut self, text: &str) {
        self.each_form(|form| form.type_text(text));
    }

    fn key(&mut self, key: Key) {
        self.each_form(|form| {
            form.key(key);
        });
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Form", 760, 720, FormDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
