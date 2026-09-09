//! TontooUI sidebar tabs demo — Apple `.sidebarAdaptable` TabView.
//!
//! Ports the SwiftUI reference:
//!
//! ```swift
//! TabView {
//!     Tab { Text("0") }
//!     TabSection("Foo") {
//!         Tab("1", systemImage: "1.circle") { Text("1") }
//!     }
//!     Tab("2", image: "cats24x24") { Text("2") }
//!     Tab { Text("3") } label: { Label("3", systemImage: "3.circle") }
//! }
//! .tabViewStyle(.sidebarAdaptable)
//! ```

use tontooui::prelude::*;
use tontooui::{Tab, TabSection, TabView};

fn main() {
    let mut app = App::new("ExploreSwiftUISandbox", 900, 600);
    app.no_window_bar();

    let tabs = TabView::new()
        .title("ExploreSwiftUISandbox")
        .tab(Tab::new(Text::new("0").font_size(28.0).bold()))
        .section(
            TabSection::new().header("Foo").tab(
                Tab::new(Text::new("1").font_size(28.0))
                    .title("1")
                    .system_image("1.circle"),
            ),
        )
        .tab(
            Tab::new(Text::new("2").font_size(28.0))
                .title("2")
                .image("cats24x24"),
        )
        .tab(
            Tab::new(Text::new("3").font_size(28.0))
                .title("3")
                .system_image("3.circle"),
        )
        .selected(0)
        .on_select(|i| println!("Tab selected: {}", i));

    app.set_root(tabs);
    app.run();
}
