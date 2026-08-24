//! TontooUI Button demo — recreates the SwiftUI button example gallery:
//! roles with default labels, initializers, glass styles, tint, roles,
//! styles, sizing, RenameButton, EditButton, PasteButton and border shapes.

use tontooui::prelude::*;

fn section(title: &str, desc: &str, preview: impl Widget + 'static) -> impl Widget {
    VStack::new()
        .spacing(10.0)
        .child(preview)
        .child(Text::new(title).font_size(15.0).bold())
        .child(Text::new(desc).font_size(12.0).color(Color::from_rgb(142, 142, 147)).max_width(240.0))
}

fn main() {
    let mut app = App::new("TontooUI Buttons", 1180, 780);

    // 1 — Role Button With Default Label
    let roles = VStack::new()
        .spacing(8.0)
        .child(HStack::new().spacing(8.0)
            .child(Button::new("").role(ButtonRole::Close).icon("xmark").style(ButtonStyle::Plain))
            .child(Button::new("").role(ButtonRole::Close).icon("xmark").style(ButtonStyle::Glass))
            .child(Button::new("").role(ButtonRole::Confirm).icon("checkmark")
                .style(ButtonStyle::BorderedProminent).border_shape(ButtonBorderShape::Circle).height(36.0))
            .child(Button::new("").role(ButtonRole::Destructive).icon("xmark.bin").style(ButtonStyle::Glass)))
        .child(HStack::new().spacing(10.0)
            .child(Button::new("").role(ButtonRole::Cancel).style(ButtonStyle::Plain))
            .child(Button::new("").role(ButtonRole::Close).style(ButtonStyle::Plain))
            .child(Button::new("").role(ButtonRole::Confirm).style(ButtonStyle::Plain))
            .child(Button::new("").role(ButtonRole::Destructive).style(ButtonStyle::Plain)));

    // 2 — Button (three ways to initialize)
    let initializers = HStack::new().spacing(10.0)
        .child(Button::new("Tap Me").style(ButtonStyle::Plain))
        .child(Button::new("Tap Me").icon("hand.tap").style(ButtonStyle::Plain))
        .child(Button::new("Tap Me").style(ButtonStyle::Glass))
        .child(Button::new("Tap Me").icon("hand.tap").style(ButtonStyle::Glass));

    // 3 — Glass Button Styles
    let glass = HStack::new().spacing(8.0)
        .child(Button::new("Tap Me").style(ButtonStyle::Glass))
        .child(Button::new("Tinted Tap Me").style(ButtonStyle::Glass)
            .tint(Color::from_rgb(255, 69, 58)))
        .child(Button::new("Tap Me").style(ButtonStyle::GlassProminent))
        .child(Button::new("Tap Me").style(ButtonStyle::GlassProminent)
            .tint(Color::from_rgb(255, 69, 58)));

    // 4 — Button Tint
    let blue = Color::from_rgb(10, 132, 255);
    let red = Color::from_rgb(255, 69, 58);
    let green = Color::from_rgb(48, 209, 88);
    let tint_rows = VStack::new()
        .spacing(8.0)
        .child(HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).tint(blue))
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).tint(red))
            .child(Button::new("Tap Me").style(ButtonStyle::Glass).tint(blue))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).tint(blue)))
        .child(HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).tint(red))
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).tint(green))
            .child(Button::new("Tap Me").style(ButtonStyle::Glass).tint(red))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).tint(red)))
        .child(HStack::new().spacing(8.0)
            .child(Button::new("Tap Me").style(ButtonStyle::Plain).tint(green))
            .child(Button::new("Tap Me").style(ButtonStyle::Plain))
            .child(Button::new("Tap Me").style(ButtonStyle::Glass).tint(green))
            .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).tint(green)));

    // 5 — Button Roles
    let role_rows = HStack::new().spacing(8.0)
        .child(Button::new("Tap Me").style(ButtonStyle::Plain))
        .child(Button::new("Tap Me").style(ButtonStyle::Plain).role(ButtonRole::Destructive))
        .child(Button::new("Tap Me").style(ButtonStyle::Glass).role(ButtonRole::Cancel))
        .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).role(ButtonRole::Destructive));

    // 6 — Button Styles
    let styles = HStack::new().spacing(8.0)
        .child(Button::new("Tap Me").style(ButtonStyle::Plain))
        .child(Button::new("Tap Me").style(ButtonStyle::Bordered))
        .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent))
        .child(Button::new("Tap Me").style(ButtonStyle::Glass));

    // 7 — Button Sizing
    let sizing = VStack::new()
        .spacing(8.0)
        .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).sizing(ButtonSizing::Fitted))
        .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).sizing(ButtonSizing::Flexible));

    // 8 — RenameButton
    let rename = HStack::new().spacing(10.0)
        .child(Text::new("foo").font_size(15.0))
        .child(RenameButton::new().on_rename(|| println!("Rename triggered")));

    // 9 — EditButton
    let edit = HStack::new().spacing(10.0)
        .child(EditButton::new())
        .child(Text::new("(toggles Edit / Done)").font_size(12.0).color(Color::from_rgb(142, 142, 147)));

    // 10 — PasteButton
    let paste = PasteButton::new().on_paste(|text| match text {
        Some(t) => println!("Pasted: {}", t),
        None => println!("Pasteboard holds no text"),
    });

    // 11 — Button Border Shape
    let shapes = HStack::new().spacing(8.0)
        .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).border_shape(ButtonBorderShape::Capsule))
        .child(Button::new("Tap Me").style(ButtonStyle::BorderedProminent).border_shape(ButtonBorderShape::RoundedRectangle(8.0)))
        .child(Button::new("").style(ButtonStyle::BorderedProminent).icon("hand.tap")
            .border_shape(ButtonBorderShape::Circle).height(36.0));

    app.set_root(
        VStack::new()
            .spacing(24.0)
            .child(Text::new("TontooUI — SwiftUI Button Gallery").font_size(22.0).bold())
            .child(HStack::new().spacing(20.0)
                .child(section("Role Button With Default Label",
                    "Creates a button that displays a default label.", roles))
                .child(section("Button",
                    "Three ways to initialize a button", initializers))
                .child(section("Glass Button Styles",
                    "A button style that applies glass border artwork based on the button's context", glass))
                .child(section("Button Tint",
                    "Sets the buttons tint color", tint_rows)))
            .child(HStack::new().spacing(20.0)
                .child(section("Button Roles",
                    "A value that describes the purpose of a button", role_rows))
                .child(section("Button Styles",
                    "A type that applies standard interaction behavior and a custom appearance", styles))
                .child(section("Button Sizing",
                    "The preferred sizing behavior of buttons in the view hierarchy", sizing))
                .child(section("RenameButton",
                    "A button that triggers a standard rename action", rename)))
            .child(HStack::new().spacing(20.0)
                .child(section("EditButton",
                    "A button that toggles the edit mode environment value", edit))
                .child(section("PasteButton",
                    "A system button that reads items from the pasteboard and delivers it to a closure", paste))
                .child(section("Button Border Shape",
                    "A shape used to draw a button's border.", shapes)))
    );

    app.run();
}
