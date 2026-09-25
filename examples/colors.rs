use std::cell::RefCell;
use std::rc::Rc;

use tontooui::elements::{
    ALL_SYSTEM_COLORS, Button, ButtonStyle, ColorPicker, GradientPaint,
    Titlebar, TrafficAction, View,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::text::draw_layout;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

/// Swatch chip size and label size in logical px.
const CHIP: f32 = 56.0;
const CHIP_GAP: f32 = 12.0;
const NAME_SIZE: f32 = 11.0;
/// Gradient bar height, gap and radius in logical px.
const BAR_H: f32 = 88.0;
const BAR_GAP: f32 = 20.0;
const BAR_RADIUS: f32 = 16.0;
const BAR_LABEL_SIZE: f32 = 15.0;

struct ColorsDemo {
    bar: Titlebar,
    pick_button: Button,
    picker: ColorPicker,
    picked: Color,
    show_picker: Rc<RefCell<bool>>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl ColorsDemo {
    fn new() -> Self {
        let show_picker: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let flag = show_picker.clone();
        Self {
            bar: Titlebar::new("Colors"),
            pick_button: Button::new("Pick a color")
                .style(ButtonStyle::BorderedProminent)
                .on_press(move || {
                    *flag.borrow_mut() = true;
                }),
            picker: ColorPicker::new().color(Color::from_rgb8(0x30, 0xb0, 0xc7)),
            picked: Color::from_rgb8(0x30, 0xb0, 0xc7),
            show_picker,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn label(
        &self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        text: &str,
        size: f32,
        color: Color,
        cx: f32,
        y: f32,
    ) {
        let layout = fonts.layout_text(text, size, color, None);
        let (tw, _) = FontSystem::layout_size(&layout);
        draw_layout(scene, &layout, cx - tw / fonts.scale / 2.0, y, fonts.scale);
    }
}

impl App for ColorsDemo {
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

        if std::mem::replace(&mut *self.show_picker.borrow_mut(), false) {
            self.picker.show();
        }
        self.pick_button.set_theme(palette.accent, dark);
        self.pick_button.set_focused(focused);
        self.picker.set_theme(theme.mode, theme.glass);
        self.picker.set_focused(focused);
        self.picked = self.picker.selected();

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let mut y = viewport.y + 31.0 + 24.0;

        // Pick row: button plus a swatch of the selection.
        let (bwn, bhn) = self.pick_button.measure(fonts);
        let bx = viewport.x + ((viewport.width - (bwn + 12.0 + 40.0)) / 2.0).max(0.0);
        self.pick_button.place(fonts, bx, y, bwn, bhn);
        self.pick_button.draw(scene, fonts, images);
        let swatch = RoundedRect::new(
            px(bx + bwn + 12.0),
            px(y),
            px(bx + bwn + 12.0 + 40.0),
            px(y + bhn),
            px(10.0),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.picked),
            None,
            &swatch,
        );
        y += bhn + 24.0;

        // Swatch grid: every system color with its name.
        let total_w = ALL_SYSTEM_COLORS.len() as f32 * CHIP
            + (ALL_SYSTEM_COLORS.len() - 1) as f32 * CHIP_GAP;
        let mut x = viewport.x + ((viewport.width - total_w) / 2.0).max(0.0);
        for color in ALL_SYSTEM_COLORS {
            let chip = RoundedRect::new(
                px(x),
                px(y),
                px(x + CHIP),
                px(y + CHIP),
                px(CHIP / 3.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(color.color()),
                None,
                &chip,
            );
            self.label(
                scene,
                fonts,
                color.name(),
                NAME_SIZE,
                palette.text,
                x + CHIP / 2.0,
                y + CHIP + 6.0,
            );
            x += CHIP + CHIP_GAP;
        }
        y += CHIP + 6.0 + NAME_SIZE * 1.25 + 28.0;

        // Reference gradient bars with centered labels.
        let bars = [
            (GradientPaint::preset_linear(), "Linear Gradient"),
            (GradientPaint::preset_vertical(), "Vertical Gradient"),
            (GradientPaint::preset_radial(), "Radial Gradient"),
            (GradientPaint::preset_angular(), "Angular Gradient"),
        ];
        let bx = viewport.x + 24.0;
        let bw = (viewport.width - 48.0).max(0.0);
        for (paint, caption) in bars {
            let bar = RoundedRect::new(
                px(bx),
                px(y),
                px(bx + bw),
                px(y + BAR_H),
                px(BAR_RADIUS),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &paint.brush(bx, y, bw, BAR_H, fonts.scale),
                None,
                &bar,
            );
            self.label(
                scene,
                fonts,
                caption,
                BAR_LABEL_SIZE,
                Color::WHITE,
                bx + bw / 2.0,
                y + (BAR_H - BAR_LABEL_SIZE) / 2.0,
            );
            y += BAR_H + BAR_GAP;
        }

        // Frosted picker popup on top while open.
        if self.picker.is_visible() {
            self.picker.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
            self.picker.draw(scene, fonts, images);
        }
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn wants_backdrop(&self) -> bool {
        // Frosted picker needs the blur pass while open.
        self.picker.is_visible()
    }

    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        Some(self.bar.drag_rect())
    }

    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        self.command.take()
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        // Popup first while open (outside clicks dismiss it).
        if self.picker.is_visible() {
            self.picker.mouse_down(x, y);
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
            None => self.pick_button.mouse_down(x, y),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if self.picker.is_visible() {
            self.picker.mouse_up(x, y);
            return;
        }
        self.pick_button.mouse_up(x, y);
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        if self.picker.is_visible() {
            self.picker.mouse_move(x, y);
        } else {
            self.pick_button.set_hover(x as f32, y as f32);
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Colors", 960, 800, ColorsDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
