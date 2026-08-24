//! TontooUI Toolbar demo — recreates the SwiftUI toolbar example cards:
//! ToolbarSpacer, ToolbarItem shared background visibility and title-area
//! item placement.

use tontooui::prelude::*;

fn section(title: &str, desc: &str, preview: impl Widget + 'static) -> impl Widget {
    VStack::new()
        .spacing(10.0)
        .child(preview)
        .child(Text::new(title).font_size(15.0).bold())
        .child(Text::new(desc).font_size(12.0).color(Color::from_rgb(142, 142, 147)).max_width(240.0))
}

fn main() {
    let mut app = App::new("TontooUI Toolbars", 900, 420);

    // 1 — ToolbarSpacer: [^ v] | [...] groups in one glass capsule row
    let spacer_toolbar = Toolbar::new()
        .item(ToolbarItem::new("chevron.up").on_click(|| println!("chevron up")))
        .item(ToolbarItem::new("chevron.down").on_click(|| println!("chevron down")))
        .spacer(ToolbarSpacer::fixed())
        .item(ToolbarItem::new("ellipsis").on_click(|| println!("ellipsis")));

    // 2 — ToolbarItem shared background: glass circles, one bare item
    let shared_toolbar = Toolbar::new()
        .item(ToolbarItem::new("person.2.circle").on_click(|| println!("account")))
        .spacer(ToolbarSpacer::fixed())
        .item(ToolbarItem::new("face.smiling").on_click(|| println!("face")))
        .spacer(ToolbarSpacer::fixed())
        .item(ToolbarItem::new("heart").shared_background(false).on_click(|| println!("bare item, no glass")));

    // 3 — Title toolbar item placement: custom content in the title area
    let title_toolbar = Toolbar::new()
        .item(ToolbarItem::new("").placement(ToolbarItemPlacement::Principal)
            .content(VStack::new().spacing(0.0)
                .child(Text::new("2").font_size(13.0))
                .child(Text::new("3").font_size(13.0))
                .child(Text::new("1").font_size(13.0))
                .child(Text::new("4").font_size(13.0))))
        .item(ToolbarItem::new("plus").on_click(|| println!("plus")))
        .item(ToolbarItem::new("gearshape").on_click(|| println!("settings")));

    app.set_root(
        VStack::new()
            .spacing(28.0)
            .child(Text::new("Toolbars").font_size(26.0).bold())
            .child(HStack::new().spacing(24.0)
                .child(section("ToolbarSpacer",
                    "A standard space item in toolbars", spacer_toolbar))
                .child(section("ToolbarItem Shared Background",
                    "Controls the visibility of the glass background effect on items in the toolbar", shared_toolbar))
                .child(section("Title Toolbar Item Placement",
                    "Places content in the title area", title_toolbar)))
    );

    app.run();
}
