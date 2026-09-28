use tontooui::elements::{
    Align, BasicOutlineGroup, BasicText, BasicTextField, Button, ButtonStyle, Gauge,
    HStack, InlinePicker, LinearProgress, MenuButton, OutlineNode, SecureField,
    SegmentedPicker, Slider, Stepper, Titlebar, Toggle, ToggleStyle, TrafficAction, View,
    VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{Accent, ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

/// Every accent in switcher order.
const ACCENTS: [Accent; 13] = [
    Accent::Multicolor,
    Accent::Blue,
    Accent::Red,
    Accent::Orange,
    Accent::Yellow,
    Accent::Green,
    Accent::Teal,
    Accent::Cyan,
    Accent::Indigo,
    Accent::Purple,
    Accent::Purple2,
    Accent::Pink,
    Accent::Gray,
];

fn accent_name(accent: Accent) -> &'static str {
    match accent {
        Accent::Multicolor => "Multicolor",
        Accent::Blue => "Blue",
        Accent::Red => "Red",
        Accent::Orange => "Orange",
        Accent::Yellow => "Yellow",
        Accent::Green => "Green",
        Accent::Teal => "Teal",
        Accent::Cyan => "Cyan",
        Accent::Indigo => "Indigo",
        Accent::Purple => "Purple",
        Accent::Purple2 => "Purple2",
        Accent::Pink => "Pink",
        Accent::Gray => "Gray",
    }
}

fn switcher_options() -> Vec<String> {
    let mut options = vec!["Auto (daemon)".to_string()];
    for accent in ACCENTS {
        options.push(accent_name(accent).to_string());
    }
    options
}

// VStack child indices (captions and rows interleave).
const ROW_SWITCHER: usize = 0;
const ROW_BUTTONS: usize = 2;
const ROW_TOGGLES: usize = 4;
const ROW_SLIDER_STEPPER: usize = 6;
const ROW_PROGRESS_GAUGE: usize = 8;
const ROW_FIELDS: usize = 10;
const ROW_PICKERS: usize = 12;
const ROW_OUTLINE: usize = 14;
const ROW_SEGMENTS: usize = 16;

struct AccentGallery {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    override_accent: Option<Accent>,
    command: Option<WindowCommand>,
}

impl AccentGallery {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(10.0)
            .align(Align::Leading)
            .child(SegmentedPicker::new("Accent", switcher_options()))
            .child(BasicText::new("Buttons"))
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(Button::new("Save").style(ButtonStyle::BorderedProminent))
                    .child(Button::new("Options").style(ButtonStyle::BorderedTinted)),
            )
            .child(BasicText::new("Toggles"))
            .child(
                HStack::new()
                    .spacing(24.0)
                    .child(Toggle::new("Wi-Fi").on(true))
                    .child(
                        Toggle::new("Airplane Mode")
                            .style(ToggleStyle::Checkbox)
                            .on(true),
                    ),
            )
            .child(BasicText::new("Slider & Stepper"))
            .child(
                HStack::new()
                    .spacing(24.0)
                    .child(Slider::new(0.6, 0.0, 1.0))
                    .child(Stepper::new(5.0, 0.0, 10.0)),
            )
            .child(BasicText::new("Progress & Gauge"))
            .child(
                HStack::new()
                    .spacing(24.0)
                    .child(LinearProgress::new())
                    .child(Gauge::new(0.7, 0.0, 1.0).title("Accent follows theme")),
            )
            .child(BasicText::new("Text fields"))
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(BasicTextField::new("Name"))
                    .child(SecureField::new("Password")),
            )
            .child(BasicText::new("Pickers"))
            .child(
                HStack::new()
                    .spacing(24.0)
                    .child(InlinePicker::from_slice("Choice", &["A", "B", "C"]))
                    .child(MenuButton::from_slice("Options", &["First", "Second"])),
            )
            .child(BasicText::new("Outline"))
            .child(BasicOutlineGroup::new(vec![
                OutlineNode::folder("Documents").expanded(true).children(vec![
                    OutlineNode::file("Notes.txt"),
                    OutlineNode::file("Sketch.png"),
                ]),
                OutlineNode::folder("Downloads")
                    .child(OutlineNode::file("Archive.zip")),
            ]))
            .child(BasicText::new("Segments"))
            .child(SegmentedPicker::from_slice("Size", &["Small", "Medium", "Large"]));
        Self {
            bar: Titlebar::new("Accent Gallery"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            override_accent: None,
            command: None,
        }
    }

    /// Apply `f` to every child of type `T` in a mixed row. Skips
    /// other types by index (the documented `len` pattern): stopping
    /// at the first mismatch would never reach elements past it
    /// (this left Gauge, Stepper, SecureField and MenuButton
    /// unthemed).
    fn each_in_row<T: View + 'static>(&mut self, row: usize, mut f: impl FnMut(&mut T)) {
        let len = self
            .stack
            .child_mut::<HStack>(row)
            .map_or(0, |hstack| hstack.len());
        for index in 0..len {
            if let Some(element) = self
                .stack
                .child_mut::<HStack>(row)
                .and_then(|hstack| hstack.child_mut::<T>(index))
            {
                f(element);
            }
        }
    }

    fn each_caption(&mut self, mut f: impl FnMut(&mut BasicText)) {
        for row in [1, 3, 5, 7, 9, 11, 13, 15] {
            if let Some(caption) = self.stack.child_mut::<BasicText>(row) {
                f(caption);
            }
        }
    }
}

impl App for AccentGallery {
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

        // Accent switcher: "Auto" follows the daemon, the rest previews
        // one accent locally without touching the daemon.
        if let Some(switcher) = self.stack.child_mut::<SegmentedPicker>(ROW_SWITCHER) {
            self.override_accent = match switcher.selected_index() {
                0 => None,
                index => ACCENTS.get(index - 1).copied(),
            };
        }
        let accent = self.override_accent.unwrap_or(theme.accent);
        let color = accent.color();

        if let Some(switcher) = self.stack.child_mut::<SegmentedPicker>(ROW_SWITCHER) {
            switcher.set_theme(color, dark);
            switcher.set_focused(focused);
        }
        self.each_caption(|caption| {
            caption.set_theme(theme.mode);
            caption.set_focused(focused);
        });
        self.each_in_row::<Button>(ROW_BUTTONS, |button| {
            button.set_theme(color, dark);
            button.set_focused(focused);
        });
        self.each_in_row::<Toggle>(ROW_TOGGLES, |toggle| {
            toggle.set_theme(color, dark);
            toggle.set_focused(focused);
        });
        self.each_in_row::<Slider>(ROW_SLIDER_STEPPER, |slider| {
            slider.set_theme(color, dark, theme.glass);
            slider.set_focused(focused);
        });
        self.each_in_row::<Stepper>(ROW_SLIDER_STEPPER, |stepper| {
            stepper.set_theme(color, dark);
            stepper.set_focused(focused);
        });
        self.each_in_row::<LinearProgress>(ROW_PROGRESS_GAUGE, |bar| {
            bar.set_theme(color, dark);
            bar.set_focused(focused);
            bar.set_progress(0.65);
        });
        self.each_in_row::<Gauge>(ROW_PROGRESS_GAUGE, |gauge| {
            gauge.set_theme(color, dark);
            gauge.set_focused(focused);
        });
        self.each_in_row::<BasicTextField>(ROW_FIELDS, |field| {
            field.set_theme(color, dark);
            field.set_focused(focused);
        });
        self.each_in_row::<SecureField>(ROW_FIELDS, |field| {
            field.set_theme(color, dark);
            field.set_focused(focused);
        });
        self.each_in_row::<InlinePicker>(ROW_PICKERS, |picker| {
            picker.set_theme(color, dark);
            picker.set_focused(focused);
        });
        self.each_in_row::<MenuButton>(ROW_PICKERS, |menu| {
            menu.set_theme(color, dark);
            menu.set_focused(focused);
        });
        if let Some(outline) = self.stack.child_mut::<BasicOutlineGroup>(ROW_OUTLINE) {
            outline.set_theme(color, dark);
            outline.set_focused(focused);
        }
        if let Some(segments) = self.stack.child_mut::<SegmentedPicker>(ROW_SEGMENTS) {
            segments.set_theme(color, dark);
            segments.set_focused(focused);
        }

        let source = if self.override_accent.is_some() {
            "preview"
        } else {
            "daemon"
        };
        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
        self.bar
            .set_title(format!("Accent Gallery — {} ({})", accent_name(accent), source));

        let top = viewport.y + 31.0;
        self.stack.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0,
            viewport.width - 48.0,
            (viewport.height - 47.0).max(0.0),
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

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => {
                if let Some(switcher) = self.stack.child_mut::<SegmentedPicker>(ROW_SWITCHER) {
                    switcher.mouse_down(x, y);
                }
                self.each_in_row::<Button>(ROW_BUTTONS, |button| button.mouse_down(x, y));
                self.each_in_row::<Toggle>(ROW_TOGGLES, |toggle| toggle.mouse_down(x, y));
                self.each_in_row::<Slider>(ROW_SLIDER_STEPPER, |slider| {
                    slider.mouse_down(x, y)
                });
                self.each_in_row::<Stepper>(ROW_SLIDER_STEPPER, |stepper| {
                    stepper.mouse_down(x, y)
                });
                self.each_in_row::<BasicTextField>(ROW_FIELDS, |field| {
                    field.mouse_down(x, y)
                });
                self.each_in_row::<SecureField>(ROW_FIELDS, |field| field.mouse_down(x, y));
                self.each_in_row::<InlinePicker>(ROW_PICKERS, |picker| {
                    picker.mouse_down(x, y)
                });
                self.each_in_row::<MenuButton>(ROW_PICKERS, |menu| menu.mouse_down(x, y));
                if let Some(outline) = self.stack.child_mut::<BasicOutlineGroup>(ROW_OUTLINE) {
                    outline.mouse_down(x, y);
                }
                if let Some(segments) = self.stack.child_mut::<SegmentedPicker>(ROW_SEGMENTS) {
                    segments.mouse_down(x, y);
                }
            }
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if let Some(switcher) = self.stack.child_mut::<SegmentedPicker>(ROW_SWITCHER) {
            switcher.mouse_up(x, y);
        }
        self.each_in_row::<Button>(ROW_BUTTONS, |button| button.mouse_up(x, y));
        self.each_in_row::<Toggle>(ROW_TOGGLES, |toggle| toggle.mouse_up(x, y));
        self.each_in_row::<Slider>(ROW_SLIDER_STEPPER, |slider| slider.mouse_up(x, y));
        self.each_in_row::<Stepper>(ROW_SLIDER_STEPPER, |stepper| stepper.mouse_up(x, y));
        self.each_in_row::<BasicTextField>(ROW_FIELDS, |field| field.mouse_up(x, y));
        self.each_in_row::<SecureField>(ROW_FIELDS, |field| field.mouse_up(x, y));
        self.each_in_row::<InlinePicker>(ROW_PICKERS, |picker| picker.mouse_up(x, y));
        self.each_in_row::<MenuButton>(ROW_PICKERS, |menu| menu.mouse_up(x, y));
        if let Some(segments) = self.stack.child_mut::<SegmentedPicker>(ROW_SEGMENTS) {
            segments.mouse_up(x, y);
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        if let Some(switcher) = self.stack.child_mut::<SegmentedPicker>(ROW_SWITCHER) {
            switcher.mouse_move(x, y);
        }
        self.each_in_row::<Button>(ROW_BUTTONS, |button| {
            button.set_hover(x as f32, y as f32)
        });
        self.each_in_row::<Toggle>(ROW_TOGGLES, |toggle| toggle.mouse_move(x, y));
        self.each_in_row::<Slider>(ROW_SLIDER_STEPPER, |slider| slider.mouse_move(x, y));
        self.each_in_row::<InlinePicker>(ROW_PICKERS, |picker| picker.mouse_move(x, y));
        self.each_in_row::<MenuButton>(ROW_PICKERS, |menu| menu.mouse_move(x, y));
        if let Some(segments) = self.stack.child_mut::<SegmentedPicker>(ROW_SEGMENTS) {
            segments.mouse_move(x, y);
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        if let Some(switcher) = self.stack.child_mut::<SegmentedPicker>(ROW_SWITCHER) {
            switcher.set_focused(focused);
        }
        self.each_in_row::<Button>(ROW_BUTTONS, |button| button.set_focused(focused));
        self.each_in_row::<Toggle>(ROW_TOGGLES, |toggle| toggle.set_focused(focused));
        self.each_in_row::<Slider>(ROW_SLIDER_STEPPER, |slider| slider.set_focused(focused));
        self.each_in_row::<Stepper>(ROW_SLIDER_STEPPER, |stepper| stepper.set_focused(focused));
        self.each_in_row::<LinearProgress>(ROW_PROGRESS_GAUGE, |bar| bar.set_focused(focused));
        self.each_in_row::<Gauge>(ROW_PROGRESS_GAUGE, |gauge| gauge.set_focused(focused));
        self.each_in_row::<BasicTextField>(ROW_FIELDS, |field| field.set_focused(focused));
        self.each_in_row::<SecureField>(ROW_FIELDS, |field| field.set_focused(focused));
        self.each_in_row::<InlinePicker>(ROW_PICKERS, |picker| picker.set_focused(focused));
        self.each_in_row::<MenuButton>(ROW_PICKERS, |menu| menu.set_focused(focused));
        if let Some(outline) = self.stack.child_mut::<BasicOutlineGroup>(ROW_OUTLINE) {
            outline.set_focused(focused);
        }
        if let Some(segments) = self.stack.child_mut::<SegmentedPicker>(ROW_SEGMENTS) {
            segments.set_focused(focused);
        }
        self.each_caption(|caption| caption.set_focused(focused));
    }
}

fn main() {
    if let Err(err) = run("Accent Gallery", 960, 800, AccentGallery::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
