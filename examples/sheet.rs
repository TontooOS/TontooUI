use std::cell::RefCell;
use std::rc::Rc;

use tontooui::elements::{
    Align, BasicSheet, BasicText, Button, ButtonStyle, SheetSize, Titlebar,
    TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct SheetDemo {
    bar: Titlebar,
    stack: VStack,
    sheet: BasicSheet<VStack>,
    open_size: Rc<RefCell<Option<SheetSize>>>,
    dismiss: Rc<RefCell<bool>>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl SheetDemo {
    fn new() -> Self {
        let open_size: Rc<RefCell<Option<SheetSize>>> = Rc::new(RefCell::new(None));
        let dismiss: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let flag_dismiss = dismiss.clone();
        let content = VStack::new()
            .align(Align::Center)
            .spacing(20.0)
            .child(BasicText::new("This is a sheet!"))
            .child(
                Button::new("Dismiss")
                    .style(ButtonStyle::Bordered)
                    .on_press(move || {
                        *flag_dismiss.borrow_mut() = true;
                    }),
            );
        let mut stack = VStack::new().spacing(16.0);
        for (label, size) in [
            ("Small Sheet", SheetSize::Small),
            ("25% Sheet", SheetSize::Quarter),
            ("50% Sheet", SheetSize::Half),
            ("75% Sheet", SheetSize::Large),
        ] {
            let flag = open_size.clone();
            stack = stack.child(
                Button::new(label)
                    .style(ButtonStyle::BorderedProminent)
                    .on_press(move || {
                        *flag.borrow_mut() = Some(size);
                    }),
            );
        }
        Self {
            bar: Titlebar::new("Sheet"),
            stack,
            sheet: BasicSheet::new(content),
            open_size,
            dismiss,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_button(&mut self, mut f: impl FnMut(&mut Button)) {
        for index in 0..self.stack.len() {
            if let Some(button) = self.stack.child_mut::<Button>(index) {
                f(button);
            }
        }
    }
}

impl App for SheetDemo {
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

        // Triggered from the app buttons: open the sheet in the
        // requested size, which fades in over the dimmed content.
        if let Some(size) = self.open_size.borrow_mut().take() {
            self.sheet.set_size(size);
            self.sheet.show();
        }
        if std::mem::replace(&mut *self.dismiss.borrow_mut(), false) {
            self.sheet.dismiss();
        }

        self.each_button(|button| {
            button.set_theme(palette.accent, dark);
            button.set_focused(focused);
        });
        self.sheet.set_theme(dark);
        self.sheet.set_focused(focused);
        if let Some(text) = self.sheet.child_mut().child_mut::<BasicText>(0) {
            text.set_theme(theme.mode);
            text.set_focused(focused);
        }
        if let Some(button) = self.sheet.child_mut().child_mut::<Button>(1) {
            button.set_theme(palette.accent, dark);
            button.set_focused(focused);
        }

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        // Gray, unclickable red light while a sheet is open.
        self.bar.set_modal_blocked(self.sheet.is_visible());
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let content_h = viewport.height - 31.0;
        let (stack_w, stack_h) = self.stack.measure(fonts);
        let cx = viewport.x + ((viewport.width - stack_w) / 2.0).max(0.0);
        self.stack.place(fonts, cx, top + 32.0, stack_w, stack_h);
        self.stack.draw(scene, fonts, images);

        // Modal overlay on top of everything below the titlebar.
        if self.sheet.is_visible() {
            self.sheet
                .set_viewport(viewport.x, top, viewport.width, content_h);
            self.sheet.draw(scene, fonts, images);
        }
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn wants_backdrop(&self) -> bool {
        // Sheets are solid cards; no blur pass needed.
        false
    }

    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        Some(self.bar.drag_rect())
    }

    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        self.command.take()
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        // Modal: only the sheet hears clicks while visible.
        if self.sheet.is_visible() {
            self.sheet.mouse_down(x, y);
            return;
        }
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => {
                self.command = Some(WindowCommand::Minimize)
            }
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => self.each_button(|button| button.mouse_down(x, y)),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if self.sheet.is_visible() {
            self.sheet.mouse_up(x, y);
            return;
        }
        self.each_button(|button| button.mouse_up(x, y));
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        // Content stays frozen while modal, except the sheet's own
        // buttons; the titlebar keeps its minimize/maximize hover.
        if self.sheet.is_visible() {
            if let Some(button) = self.sheet.child_mut().child_mut::<Button>(1) {
                button.set_hover(x as f32, y as f32);
            }
        } else {
            self.each_button(|button| button.set_hover(x as f32, y as f32));
        }
        self.bar.set_hover(x as f32, y as f32);
    }

    fn key(&mut self, key: Key) {
        // ESC closes the open sheet.
        if self.sheet.is_visible() {
            self.sheet.key(key);
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Sheet", 900, 640, SheetDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
