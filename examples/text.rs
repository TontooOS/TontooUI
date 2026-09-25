use tontooui::elements::{
    Align, BasicText, FormattedText, Span, TextAlignment, TextForeground, TextStyle, Titlebar,
    TrafficAction, View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

fn style_row(name: &str, style: TextStyle) -> BasicText {
    BasicText::new(name).style(style)
}

struct TextDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl TextDemo {
    fn new() -> Self {
        let stack = VStack::new()
            .spacing(8.0)
            .align(Align::Leading)
            .child(
                BasicText::new("Hello, SwiftUI!").alignment(TextAlignment::Center),
            )
            .child(
                BasicText::new(
                    "Multi-line text that spans across multiple lines to show how text wrapping works in SwiftUI.",
                )
                .alignment(TextAlignment::Center),
            )
            .child(BasicText::new("Fixed width text").width(200.0))
            .child(style_row("Large Title", TextStyle::LargeTitle))
            .child(style_row("Title 1", TextStyle::Title))
            .child(style_row("Title 2", TextStyle::Title2))
            .child(style_row("Title 3", TextStyle::Title3))
            .child(style_row("Headline", TextStyle::Headline))
            .child(style_row("Subheadline", TextStyle::Subheadline))
            .child(style_row("Body", TextStyle::Body))
            .child(style_row("Callout", TextStyle::Callout))
            .child(style_row("Footnote", TextStyle::Footnote))
            .child(style_row("Caption", TextStyle::Caption))
            .child(style_row("Caption 2", TextStyle::Caption2))
            .child(
                BasicText::new("Red Text")
                    .foreground_color(Color::from_rgb8(0xff, 0x3b, 0x30)),
            )
            .child(
                BasicText::new("Blue Text")
                    .foreground_color(Color::from_rgb8(0x00, 0x7a, 0xff)),
            )
            .child(
                BasicText::new("Green Text")
                    .foreground_color(Color::from_rgb8(0x34, 0xc7, 0x59)),
            )
            .child(BasicText::new("Gradient Text").foreground_gradient(vec![
                Color::from_rgb8(0xff, 0x3b, 0x30),
                Color::from_rgb8(0xff, 0xcc, 0x00),
                Color::from_rgb8(0x34, 0xc7, 0x59),
                Color::from_rgb8(0x00, 0x7a, 0xff),
                Color::from_rgb8(0xaf, 0x52, 0xde),
            ]))
            .child(BasicText::new("Primary").foreground(TextForeground::Primary))
            .child(BasicText::new("Secondary").foreground(TextForeground::Secondary))
            .child(BasicText::new("Tertiary").foreground(TextForeground::Tertiary))
            .child(FormattedText::markdown("**Bold Text**"))
            .child(FormattedText::markdown("*Italic Text*"))
            .child(FormattedText::markdown("***Bold & Italic***"))
            .child(FormattedText::spans(vec![Span::new("Underlined Text").underline()]))
            .child(FormattedText::spans(vec![Span::new("Underlined Red")
                .underline_color(Color::from_rgb8(0xff, 0x3b, 0x30))]))
            .child(FormattedText::markdown("~~Strikethrough Text~~"))
            .child(FormattedText::spans(vec![Span::new("Strikethrough Green")
                .strikethrough_color(Color::from_rgb8(0x34, 0xc7, 0x59))]))
            .child(FormattedText::markdown("`Monospaced`"))
            .child(
                FormattedText::markdown("[Link](https://example.com)")
                    .on_link(|url| println!("link pressed: {url}")),
            )
            .child(FormattedText::markdown("**Bold** and *italic* together"))
            .child(
                FormattedText::markdown(
                    "Truncated Text that keeps going past the box edge again and again",
                )
                .width(300.0)
                .line_limit(1),
            );
        Self {
            bar: Titlebar::new("Text"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn each_text(&mut self, mut f: impl FnMut(&mut BasicText)) {
        let mut index = 0;
        loop {
            match self.stack.child_mut::<BasicText>(index) {
                Some(text) => f(text),
                None => break,
            }
            index += 1;
        }
    }

    fn each_formatted(&mut self, mut f: impl FnMut(&mut FormattedText)) {
        let mut index = 0;
        loop {
            if let Some(text) = self.stack.child_mut::<FormattedText>(index) {
                f(text);
            } else if self.stack.child_mut::<BasicText>(index).is_none() {
                break;
            }
            index += 1;
        }
    }
}

impl App for TextDemo {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut tontooui::renderer::ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    ) {
        self.watcher.poll(time_secs);
        self.watcher.set_focused(self.focused, time_secs);
        let palette = self.watcher.palette(time_secs);
        self.bg = palette.bg;
        let dark = self.watcher.theme().mode == ThemeMode::Dark;
        let focused = self.focused;
        self.each_text(|text| {
            if dark {
                text.set_theme(ThemeMode::Dark);
            } else {
                text.set_theme(ThemeMode::Light);
            }
            text.set_focused(focused);
        });
        self.each_formatted(|text| {
            if dark {
                text.set_theme(ThemeMode::Dark);
            } else {
                text.set_theme(ThemeMode::Light);
            }
            text.set_focused(focused);
        });
        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        self.stack.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0,
            (viewport.width - 48.0).max(0.0),
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
                self.each_formatted(|text| text.mouse_down(x, y));
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.each_formatted(|text| text.mouse_up(x, y));
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Text", 800, 1150, TextDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
