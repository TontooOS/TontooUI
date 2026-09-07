//! TontooUI Toggle demo — the two non-list styles: SwitchToggleStyle
//! (label + trailing switch) and CheckboxToggleStyle (checkbox + label).

use tontooui::prelude::*;

fn section(title: &str, desc: &str, preview: impl Widget + 'static) -> impl Widget {
    VStack::new()
        .spacing(12.0)
        .child(preview)
        .child(Text::new(title).font_size(15.0).bold())
        .child(Text::new(desc).font_size(12.0).color(Color::from_rgb(142, 142, 147)).max_width(260.0))
}

fn main() {
    let mut app = App::new("TontooUI Toggles", 760, 420);

    // SwitchToggleStyle — leading label, trailing switch.
    let switches = VStack::new()
        .spacing(10.0)
        .child(Toggle::new("Foo1").value(true).width(150.0).on_change(|on| println!("switch Foo1: {on}")))
        .child(Toggle::new("Foo2").width(150.0).on_change(|on| println!("switch Foo2: {on}")))
        .child(Toggle::new("Wi-Fi").value(true).width(150.0).on_change(|on| println!("Wi-Fi: {on}")));

    // CheckboxToggleStyle — checkbox followed by its label.
    let checkboxes = VStack::new()
        .spacing(10.0)
        .child(
            Toggle::new("Foo1")
                .style(ToggleStyle::Checkbox)
                .width(150.0)
                .on_change(|on| println!("check Foo1: {on}")),
        )
        .child(
            Toggle::new("Foo2")
                .style(ToggleStyle::Checkbox)
                .value(true)
                .width(150.0)
                .on_change(|on| println!("check Foo2: {on}")),
        )
        .child(
            Toggle::new("Sync iCloud")
                .style(ToggleStyle::Checkbox)
                .width(150.0)
                .on_change(|on| println!("check Sync: {on}")),
        );

    app.set_root(
        VStack::new()
            .spacing(20.0)
            .child(Text::new("Toggle").font_size(26.0).bold())
            .child(
                HStack::new()
                    .spacing(24.0)
                    .child(section(
                        "SwitchToggleStyle",
                        "A toggle style that displays a leading label and a trailing switch.",
                        switches,
                    ))
                    .child(section(
                        "CheckboxToggleStyle",
                        "A toggle style that displays a checkbox followed by its label.",
                        checkboxes,
                    )),
            ),
    );

    app.run();
}
