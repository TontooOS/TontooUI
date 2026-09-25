use tontooui::elements::{
    BasicText, BasicTextField, LargeTextEditor, LargeTextField, SearchField,
    SecureField, TextEditor, Titlebar, TrafficAction, View,
};use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct TextFieldDemo {
    bar: Titlebar,
    basic: BasicTextField,
    large: LargeTextField,
    secure: SecureField,
    editor: TextEditor,
    search: SearchField,
    large_editor: LargeTextEditor,
    status: BasicText,
    pending: Vec<Key>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl TextFieldDemo {
    fn new() -> Self {
        Self {
            bar: Titlebar::new("TextField"),
            basic: BasicTextField::new("Enter text here"),
            large: LargeTextField::new("Placeholder"),
            secure: SecureField::new("Password"),
            editor: TextEditor::new("Write something…"),
            search: SearchField::new("Search items…"),
            large_editor: LargeTextEditor::new("Start typing here…"),
            pending: Vec::new(),
            status: BasicText::new("Click a field, type, ESC or outside click deselects."),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }
}

impl App for TextFieldDemo {
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
        self.basic.set_theme(palette.accent, dark);
        self.basic.set_focused(focused);
        self.large.set_theme(palette.accent, dark);
        self.large.set_focused(focused);
        self.secure.set_theme(palette.accent, dark);
        self.secure.set_focused(focused);
        self.search
            .set_theme(theme.mode, palette.accent, theme.glass);
        self.search.set_focused(focused);
        self.editor.set_theme(palette.accent, dark);
        self.editor.set_focused(focused);
        self.large_editor.set_theme(palette.accent, dark);
        self.large_editor.set_focused(focused);
        self.status.set_theme(theme.mode);
        self.status.set_focused(focused);
        self.status.set_text(format!(
            "basic: \"{}\"   large: \"{}\"   secure: {} chars",
            self.basic.text_value(),
            self.large.text_value(),
            self.secure.text_value().chars().count()
        ));
        // Drain buffered keys in order (editors need fonts).
        for key in std::mem::take(&mut self.pending) {
            if !self.basic.key(key)
                && !self.large.key(key)
                && !self.secure.key(key)
                && !self.search.key(key)
            {
                self.editor.key(fonts, key);
            }
        }

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        // Both variants full-bleed like the reference rows.
        let top = viewport.y + 31.0;
        let content_w = viewport.width - 48.0;
        let cx = viewport.x + 24.0;
        let mut y = top + 24.0;
        let (sw, sh) = self.status.measure(fonts);
        self.status.place(fonts, cx, y, sw, sh);
        self.status.draw(scene, fonts, images);
        y += sh + 20.0;
        let (_, bh) = self.basic.measure(fonts);
        self.basic.place(fonts, cx, y, content_w, bh);
        self.basic.draw(scene, fonts, images);
        y += bh + 20.0;
        let (_, lh) = self.large.measure(fonts);
        self.large.place(fonts, cx, y, content_w, lh);
        self.large.draw(scene, fonts, images);
        y += lh + 20.0;
        let (_, sh2) = self.secure.measure(fonts);
        self.secure.place(fonts, cx, y, content_w, sh2);
        self.secure.draw(scene, fonts, images);
        y += sh2 + 20.0;
        // Editor takes a fixed tall box.
        self.editor.place(fonts, cx, y, content_w, 180.0);
        self.editor.draw(scene, fonts, images);
        y += 200.0;
        // Search capsule full-bleed.
        let (_, sh3) = self.search.measure(fonts);
        self.search.place(fonts, cx, y, content_w, sh3);
        self.search.draw(scene, fonts, images);
        y += sh3 + 20.0;
        // Large inset editor last.
        self.large_editor.place(fonts, cx, y, content_w, 220.0);
        self.large_editor.draw(scene, fonts, images);
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn wants_backdrop(&self) -> bool {
        // Frosted search capsule needs the blur pass.
        true
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
            Some(TrafficAction::Minimize) => {
                self.command = Some(WindowCommand::Minimize)
            }
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            // Every press reaches all fields: inside selects,
            // anywhere else deselects.
            None => {
                self.basic.mouse_down(x, y);
                self.large.mouse_down(x, y);
                self.secure.mouse_down(x, y);
                self.search.mouse_down(x, y);
                self.editor.mouse_down(x, y);
                self.large_editor.mouse_down(x, y);
            }
        }
    }

    fn text(&mut self, text: &str) {
        self.basic.type_text(text);
        self.large.type_text(text);
        self.secure.type_text(text);
        self.search.type_text(text);
        self.editor.type_text(text);
        self.large_editor.type_text(text);
    }

    fn key(&mut self, key: Key) {
        // Buffered: the editor needs fonts for vertical caret
        // motion, which only `draw` has. Drained below in order.
        self.pending.push(key);
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("TextField", 900, 480, TextFieldDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
