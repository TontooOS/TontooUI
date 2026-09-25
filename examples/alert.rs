use std::cell::RefCell;
use std::rc::Rc;

use tontooui::elements::{
    ActionAlert, AlertAction, AlertButton, BasicAlert, BasicToolbar, Button,
    ButtonStyle, ConfirmationDialog, IconAlert, Titlebar, TrafficAction, View,
    VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct AlertDemo {
    bar: Titlebar,
    stack: VStack,
    toolbar: BasicToolbar,
    alert_ok: BasicAlert,
    alert_both: BasicAlert,
    alert_action: ActionAlert,
    alert_confirm: ConfirmationDialog,
    alert_icon: IconAlert,
    show_ok: Rc<RefCell<bool>>,
    show_both: Rc<RefCell<bool>>,
    show_action: Rc<RefCell<bool>>,
    show_confirm: Rc<RefCell<bool>>,
    show_icon: Rc<RefCell<bool>>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl AlertDemo {
    fn new() -> Self {
        let show_ok: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_both: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_action: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_confirm: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_icon: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let flag_ok = show_ok.clone();
        let flag_both = show_both.clone();
        let flag_action = show_action.clone();
        let flag_confirm = show_confirm.clone();
        let flag_icon = show_icon.clone();
        let stack = VStack::new()
            .spacing(16.0)
            .child(
                Button::new("Show OK Alert")
                    .style(ButtonStyle::BorderedProminent)
                    .on_press(move || {
                        *flag_ok.borrow_mut() = true;
                    }),
            )
            .child(
                Button::new("Show OK / Cancel Alert")
                    .style(ButtonStyle::Bordered)
                    .on_press(move || {
                        *flag_both.borrow_mut() = true;
                    }),
            )
            .child(
                Button::new("Show Action Alert")
                    .style(ButtonStyle::Bordered)
                    .on_press(move || {
                        *flag_action.borrow_mut() = true;
                    }),
            )
            .child(
                Button::new("Show Confirmation Dialog")
                    .style(ButtonStyle::Bordered)
                    .on_press(move || {
                        *flag_confirm.borrow_mut() = true;
                    }),
            )
            .child(
                Button::new("Show Icon Alert")
                    .style(ButtonStyle::Bordered)
                    .on_press(move || {
                        *flag_icon.borrow_mut() = true;
                    }),
            );
        Self {
            bar: Titlebar::new("Alert"),
            stack,
            toolbar: BasicToolbar::new().icons(vec![
                "star.fill".to_string(),
                "heart.fill".to_string(),
            ]),
            alert_ok: BasicAlert::ok("Alert Title", "This is a basic alert message."),
            alert_both: BasicAlert::buttons(
                "Delete Item?",
                "This cannot be undone.",
                vec![AlertButton::cancel("Cancel"), AlertButton::ok("OK")],
            ),
            alert_action: ActionAlert::new(
                "Delete Item?",
                "Are you sure you want to delete this item?",
                AlertButton::cancel("Cancel"),
                AlertButton::ok("Delete")
                    .color(Color::from_rgb8(0xff, 0x3b, 0x30)),
            ),
            alert_confirm: ConfirmationDialog::new(
                "Choose Action",
                vec![
                    AlertButton::ok("Option 1"),
                    AlertButton::ok("Option 2"),
                    AlertButton::ok("Option 3"),
                ],
            ),
            alert_icon: IconAlert::new(
                "lock.fill",
                "Authentication Required",
                "Enter an administrator name and password to continue.",
                vec![AlertButton::ok("Use Password...")],
            ),
            show_ok,
            show_both,
            show_action,
            show_confirm,
            show_icon,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    /// The basic alert currently on screen, if any.
    fn visible(&mut self) -> Option<&mut BasicAlert> {
        if self.alert_ok.is_visible() {
            Some(&mut self.alert_ok)
        } else if self.alert_both.is_visible() {
            Some(&mut self.alert_both)
        } else {
            None
        }
    }

    fn any_visible(&self) -> bool {
        self.alert_ok.is_visible()
            || self.alert_both.is_visible()
            || self.alert_action.is_visible()
            || self.alert_confirm.is_visible()
            || self.alert_icon.is_visible()
    }

    fn each_button(&mut self, mut f: impl FnMut(&mut Button)) {
        for index in 0..self.stack.len() {
            if let Some(button) = self.stack.child_mut::<Button>(index) {
                f(button);
            }
        }
    }
}

impl App for AlertDemo {
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

        // Triggered from the app buttons: open the alert, which fades
        // in over the dimmed content.
        if std::mem::replace(&mut *self.show_ok.borrow_mut(), false) {
            self.alert_both.dismiss();
            self.alert_action.dismiss();
            self.alert_confirm.dismiss();
            self.alert_icon.dismiss();
            self.alert_ok.show();
        }
        if std::mem::replace(&mut *self.show_both.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_action.dismiss();
            self.alert_confirm.dismiss();
            self.alert_icon.dismiss();
            self.alert_both.show();
        }
        if std::mem::replace(&mut *self.show_action.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_both.dismiss();
            self.alert_confirm.dismiss();
            self.alert_icon.dismiss();
            self.alert_action.show();
        }
        if std::mem::replace(&mut *self.show_confirm.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_both.dismiss();
            self.alert_action.dismiss();
            self.alert_icon.dismiss();
            self.alert_confirm.show();
        }
        if std::mem::replace(&mut *self.show_icon.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_both.dismiss();
            self.alert_action.dismiss();
            self.alert_confirm.dismiss();
            self.alert_icon.show();
        }

        self.each_button(|button| {
            button.set_theme(palette.accent, dark);
            button.set_focused(focused);
        });
        self.toolbar.set_theme(theme.mode, theme.glass);
        self.toolbar.set_focused(focused);
        for alert in [&mut self.alert_ok, &mut self.alert_both] {
            alert.set_theme(theme.mode, palette.accent, theme.glass);
            alert.set_focused(focused);
        }
        self.alert_action
            .set_theme(theme.mode, palette.accent, theme.glass);
        self.alert_action.set_focused(focused);
        self.alert_confirm
            .set_theme(theme.mode, palette.accent, theme.glass);
        self.alert_confirm.set_focused(focused);
        self.alert_icon.set_theme(theme.mode, palette.text, theme.glass);
        self.alert_icon.set_focused(focused);

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        // Gray, unclickable red light while an alert is open.
        self.bar.set_modal_blocked(self.any_visible());
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let content_h = viewport.height - 31.0;
        let (stack_w, stack_h) = self.stack.measure(fonts);
        let cx = viewport.x + ((viewport.width - stack_w) / 2.0).max(0.0);
        self.stack.place(fonts, cx, top + 32.0, stack_w, stack_h);
        self.stack.draw(scene, fonts, images);
        // Toolbar below the buttons: frozen behind the dim while open.
        let (tb_w, tb_h) = self.toolbar.measure(fonts);
        self.toolbar.place(
            fonts,
            viewport.x + ((viewport.width - tb_w) / 2.0).max(0.0),
            top + 32.0 + stack_h + 32.0,
            tb_w,
            tb_h,
        );
        self.toolbar.draw(scene, fonts, images);

        // Modal overlay on top of everything below the titlebar.
        if self.any_visible() {
            if self.alert_icon.is_visible() {
                let alert = &mut self.alert_icon;
                alert.set_viewport(viewport.x, top, viewport.width, content_h);
                alert.draw(scene, fonts, images);
            } else if self.alert_confirm.is_visible() {
                let alert = &mut self.alert_confirm;
                alert.set_viewport(viewport.x, top, viewport.width, content_h);
                alert.draw(scene, fonts, images);
            } else if self.alert_action.is_visible() {
                let alert = &mut self.alert_action;
                alert.set_viewport(viewport.x, top, viewport.width, content_h);
                alert.draw(scene, fonts, images);
            } else if let Some(alert) = self.visible() {
                alert.set_viewport(viewport.x, top, viewport.width, content_h);
                alert.draw(scene, fonts, images);
            }
        }
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn wants_backdrop(&self) -> bool {
        // Frosted alert cards need the blur pass while visible.
        self.any_visible()
    }

    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        Some(self.bar.drag_rect())
    }

    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        self.command.take()
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        // Modal: only the alert hears clicks while visible.
        if self.any_visible() {
            if self.alert_icon.is_visible() {
                self.alert_icon.mouse_down(x, y);
            } else if self.alert_confirm.is_visible() {
                self.alert_confirm.mouse_down(x, y);
            } else if self.alert_action.is_visible() {
                self.alert_action.mouse_down(x, y);
            } else if let Some(alert) = self.visible() {
                alert.mouse_down(x, y);
            }
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
        if self.any_visible() {
            // Both event dialogs report their button, then close.
            if self.alert_icon.is_visible() {
                if self.alert_icon.mouse_up(x, y).is_some() {
                    self.alert_icon.dismiss();
                }
                return;
            }
            if self.alert_confirm.is_visible() {
                if self.alert_confirm.mouse_up(x, y).is_some() {
                    self.alert_confirm.dismiss();
                }
                return;
            }
            if self.alert_action.is_visible() {
                if self.alert_action.mouse_up(x, y).is_some() {
                    self.alert_action.dismiss();
                }
                return;
            }
            let action = self.visible().and_then(|alert| alert.mouse_up(x, y));
            match action {
                // Any button closes the alert with a fade-out.
                Some(AlertAction::Ok) | Some(AlertAction::Cancel) => {
                    if let Some(alert) = self.visible() {
                        alert.dismiss();
                    }
                }
                None => {}
            }
            return;
        }
        self.each_button(|button| button.mouse_up(x, y));
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        // Content stays frozen while modal; the titlebar keeps its
        // minimize/maximize hover.
        if !self.any_visible() {
            self.each_button(|button| button.set_hover(x as f32, y as f32));
        }
        self.bar.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Alert", 900, 640, AlertDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
