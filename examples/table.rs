use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tontooui::elements::{
    BasicTable, ContextMenu, Menu, TableColumn, TableHit, Titlebar, TrafficAction, View,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct TableDemo {
    bar: Titlebar,
    table: Rc<RefCell<BasicTable>>,
    pending: Rc<Cell<Option<(usize, usize)>>>,
    menu: ContextMenu,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl TableDemo {
    fn new() -> Self {
        let columns = vec![
            TableColumn::new("Name").weight(1.4).editable(true),
            TableColumn::new("Age").weight(0.6).min_width(60.0),
            // Custom comparator demo: shortest city name first.
            TableColumn::new("City")
                .weight(1.0)
                .sort_by(|a, b| a.len().cmp(&b.len())),
            TableColumn::new("Email").weight(1.6).min_width(200.0).editable(true),
        ];
        let people = [
            ("Alice", "28", "Berlin", "alice@tontoo.os"),
            ("Bob", "34", "Munich", "bob@tontoo.os"),
            ("Charlie", "22", "Hamburg", "charlie@tontoo.os"),
            ("Diana", "31", "Frankfurt", "diana@tontoo.os"),
            ("Eve", "25", "Ulm", "eve@tontoo.os"),
            ("Frank", "41", "Cologne", "frank@tontoo.os"),
            ("Grace", "29", "Stuttgart", "grace@tontoo.os"),
            ("Heidi", "33", "Dresden", "heidi@tontoo.os"),
            ("Ivan", "27", "Leipzig", "ivan@tontoo.os"),
            ("Judy", "24", "Bremen", "judy@tontoo.os"),
            ("Karl", "45", "Hanover", "karl@tontoo.os"),
            ("Lena", "26", "Nuremberg", "lena@tontoo.os"),
            ("Mike", "38", "Duisburg", "mike@tontoo.os"),
            ("Nina", "23", "Bochum", "nina@tontoo.os"),
            ("Oscar", "36", "Wuppertal", "oscar@tontoo.os"),
            ("Paula", "30", "Bonn", "paula@tontoo.os"),
            ("Quinn", "32", "Mainz", "quinn@tontoo.os"),
            ("Rita", "28", "Freiburg", "rita@tontoo.os"),
            ("Steve", "39", "Kassel", "steve@tontoo.os"),
            ("Tina", "21", "Aachen", "tina@tontoo.os"),
        ];
        let rows: Vec<Vec<String>> = people
            .iter()
            .map(|(n, a, c, e)| [n, a, c, e].iter().map(|s| s.to_string()).collect())
            .collect();
        let table = Rc::new(RefCell::new(
            BasicTable::new(columns, rows).selectable(true),
        ));
        let pending: Rc<Cell<Option<(usize, usize)>>> = Rc::new(Cell::new(None));
        // Manual context menu wiring: right-click stores the cell,
        // the action starts inline editing on it.
        let menu = ContextMenu::basic(
            (0.0, 0.0, 0.0, 0.0),
            Menu::from_slice("Cell", &["Edit Cell"]).on_action({
                let table = Rc::clone(&table);
                let pending = Rc::clone(&pending);
                move |index| {
                    if index == 0 {
                        if let Some((row, col)) = pending.get() {
                            table.borrow_mut().begin_edit(row, col);
                        }
                    }
                }
            }),
        );
        Self {
            bar: Titlebar::new("Table"),
            table,
            pending,
            menu,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }
}

impl App for TableDemo {
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
            let mut table = self.table.borrow_mut();
            table.set_theme(palette.accent, dark);
            table.set_focused(focused);
        }
        self.menu
            .set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        self.menu.set_theme(palette.accent, dark);
        self.menu.set_glass(theme.mode, theme.glass);
        self.menu.set_focused(focused);

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        let selected = self.table.borrow().selected_rows().len();
        let sort = self.table.borrow().sort_state().map(|(c, asc)| {
            let name = ["Name", "Age", "City", "Email"][c.min(3)];
            format!("sorted by {name} {}", if asc { "up" } else { "down" })
        });
        self.bar.set_title(match sort {
            Some(s) => format!("Table — {selected} selected · {s}"),
            None => format!("Table — {selected} selected"),
        });
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        {
            let mut table = self.table.borrow_mut();
            table.place(
                fonts,
                viewport.x + 16.0,
                top + 12.0,
                viewport.width - 32.0,
                (viewport.height - 43.0).max(0.0),
            );
            table.draw(scene, fonts, images);
            let (tx, ty, tw, th) = table.rect();
            drop(table);
            self.menu.set_area((tx, ty, tw, th));
        }
        // Context overlay last so its panel floats above the table.
        self.menu.place(fonts, viewport.x, viewport.y, viewport.width, viewport.height);
        self.menu.draw(scene, fonts, images);
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
        self.menu.is_open()
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => {
                self.table.borrow_mut().mouse_down(x, y);
                self.menu.mouse_down(x, y);
            }
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        self.table.borrow_mut().set_hover(x as f32, y as f32);
        self.menu.mouse_move(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.table.borrow_mut().mouse_up(x, y);
        self.menu.mouse_up(x, y);
    }

    fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        if self.menu.is_open() {
            self.menu.mouse_wheel(dx, dy);
        } else {
            self.table.borrow_mut().mouse_wheel(dx, dy);
        }
    }

    fn context_click(&mut self, x: f64, y: f64) {
        let hit = self.table.borrow().cell_at(x, y);
        match hit {
            Some(TableHit::Cell(row, col)) if self.table.borrow().is_cell_editable(row, col) => {
                self.pending.set(Some((row, col)));
                self.menu.context_click(x, y);
            }
            _ => self.menu.close(),
        }
    }

    fn text(&mut self, text: &str) {
        self.table.borrow_mut().type_text(text);
    }

    fn key(&mut self, key: Key) {
        self.table.borrow_mut().key(key);
    }

    fn set_modifiers(&mut self, ctrl: bool, shift: bool) {
        self.table.borrow_mut().set_modifiers(ctrl, shift);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
        self.table.borrow_mut().set_focused(focused);
        self.menu.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Table", 760, 560, TableDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
