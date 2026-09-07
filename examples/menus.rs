//! Menu demo — 1:1 aus dem Screenshot
//! 8 Varianten direkt auf dem Background (#1d1d1d / #ececec), LiquidGlass/Transparent,
//! Light/Dark, SF Pro. Menus öffnen bei Klick / ContextMenu bei Rechtsklick.
//! Nur TontooUI API, Ampeln bleiben sichtbar.

use tontooui::prelude::*;
use tontooui::{Menu, MenuEntry, MenuItem, ContextMenu};

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(HStack::new().spacing(0.0).child(Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap())))
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(200.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(200.0))
}

// common 3 items as in screenshot
fn std_entries() -> Vec<MenuEntry> {
    vec![
        MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")),
        MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")),
        MenuEntry::Item(MenuItem::new("Button 1")),
    ]
}

fn preview_menu_action_button() -> impl Widget {
    // Acts like a button and opens the menu on a secondary gesture — Menu with 3 items
    Menu::new("Menu")
        .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")))
        .entry(MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")))
        .entry(MenuEntry::Item(MenuItem::new("Button 1")))
}

fn preview_menu() -> impl Widget {
    // A control for presenting a menu of actions.
    Menu::new("Menu")
        .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")))
        .entry(MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")))
        .entry(MenuEntry::Item(MenuItem::new("Button 1")))
}

fn preview_context_preview() -> impl Widget {
    // Context Menu Preview — custom preview pill + menu
    let target = Text::new("Long-press / Right-click").font_size(11.0).color(Color::from_hex("#8e8e93").unwrap());
    ContextMenu::new(target)
        .preview(Text::new("Preview").font_size(11.0).bold())
        .entries(std_entries())
}

fn preview_context() -> impl Widget {
    // Adds a context menu to a view.
    let target = Button::new("Buttone").on_click(|| println!("Buttone pressed"));
    ContextMenu::new(target).entries(std_entries())
}

fn preview_fixed_order() -> impl Widget {
    // Order items from top to bottom — fixed order Button 1,2,3
    Menu::new("Menu")
        .entry(MenuEntry::Item(MenuItem::new("Button 1")))
        .entry(MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")))
        .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")))
}

fn preview_nested() -> impl Widget {
    // A Menu inside a Menu — "Other" submenu
    Menu::new("Menu")
        .submenu("Other", vec![MenuEntry::Item(MenuItem::new("Button 1"))])
        .entry(MenuEntry::Item(MenuItem::new("Button 1")))
}

fn preview_divider() -> impl Widget {
    // A Menu with a Divider
    Menu::new("Menu")
        .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")))
        .entry(MenuEntry::Divider)
        .entry(MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")))
        .entry(MenuEntry::Item(MenuItem::new("Button 1")))
}

fn preview_section() -> impl Widget {
    // A Section inside a Menu — section "foo"
    Menu::new("Menu")
        .section_items(Some("foo".to_string()), vec![
            MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")),
            MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")),
            MenuEntry::Item(MenuItem::new("Button 1")),
        ])
}

fn main() {
    let mut app = App::new("Menu", 1080, 620);
    // kein no_window_bar — Ampeln bleiben sichtbar

    let title = Text::new("Menu").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new().spacing(24.0)
        .child(cell("Menu Action Button", "Acts like a button and opens the menu on a secondary gesture", "initializer", preview_menu_action_button()))
        .child(cell("Menu", "A control for presenting a menu of actions.", "initializer", preview_menu()))
        .child(cell("Context Menu Preview", "A context menu with custom preview", "modifier", preview_context_preview()))
        .child(cell("Context Menu", "Adds a context menu to a view.", "modifier", preview_context()));

    let row2 = HStack::new().spacing(24.0)
        .child(cell("Menu Fixed Order", "Order items from top to bottom", "modifier", preview_fixed_order()))
        .child(cell("Nested Menu", "A Menu inside a Menu", "content", preview_nested()))
        .child(cell("Menu Divider", "A Menu with a Divider", "content", preview_divider()))
        .child(cell("Menu Section", "A Section inside a Menu", "content", preview_section()));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
