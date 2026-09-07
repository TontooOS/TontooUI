//! TontooUI Slider demo — recreates the SwiftUI slider example cards:
//! tick content, custom ticks, stepped sliders with/without range labels,
//! the plain slider and color-customized sliders.

use tontooui::prelude::*;

fn section(title: &str, desc: &str, preview: impl Widget + 'static) -> impl Widget {
    VStack::new()
        .spacing(10.0)
        .child(preview)
        .child(Text::new(title).font_size(15.0).bold())
        .child(Text::new(desc).font_size(12.0).color(Color::from_rgb(142, 142, 147)).max_width(200.0))
}

fn main() {
    let mut app = App::new("TontooUI Sliders", 1080, 560);

    let _blue = Color::from_rgb(10, 132, 255);
    let red = Color::from_rgb(255, 69, 58);

    // 1 — SliderTickContentForEach: ticks from a collection
    let tick_foreach = Slider::new(0.0, 100.0)
        .value(55.0)
        .ticks(10)
        .width(220.0)
        .on_change(|v| println!("foreach: {:.0}", v));

    // 2 — Custom Slider Ticks: stepped slider with dense ticks
    let custom_ticks = Slider::new(0.0, 100.0)
        .value(100.0)
        .step(1.0)
        .ticks(20)
        .width(220.0)
        .on_change(|v| println!("ticks: {:.0}", v));

    // 3 — Stepped Min Max Label Slider
    let min_max = Slider::new(0.0, 100.0)
        .value(30.0)
        .step(1.0)
        .ticks(10)
        .min_label("0")
        .max_label("100")
        .width(190.0)
        .on_change(|v| println!("minmax: {:.0}", v));

    // 4 — Stepped Slider
    let stepped = Slider::new(0.0, 100.0)
        .value(45.0)
        .step(1.0)
        .ticks(10)
        .width(220.0)
        .on_change(|v| println!("stepped: {:.0}", v));

    // 5 — Slider (plain)
    let plain = Slider::new(0.0, 100.0)
        .value(35.0)
        .width(220.0)
        .on_change(|v| println!("plain: {:.0}", v));

    // 6 — Slider Color: red accent, with and without labels
    let color_rows = VStack::new()
        .spacing(14.0)
        .child(Slider::new(0.0, 100.0)
            .value(50.0)
            .accent_color(red)
            .width(220.0)
            .on_change(|v| println!("red: {:.0}", v)))
        .child(Slider::new(0.0, 100.0)
            .value(40.0)
            .accent_color(red)
            .ticks(10)
            .min_label("0")
            .max_label("100")
            .width(190.0)
            .on_change(|v| println!("red stepped: {:.0}", v)));

    app.set_root(
        VStack::new()
            .spacing(28.0)
            .child(Text::new("Slider").font_size(26.0).bold())
            .child(HStack::new().spacing(24.0)
                .child(section("SliderTickContentForEach",
                    "A type of slider content that creates content by iterating over a collection.", tick_foreach))
                .child(section("Custom Slider Ticks",
                    "Creates a slider to select a value from a given range, subject to a step increment.", custom_ticks))
                .child(section("Stepped Min Max Label Slider",
                    "Creates a slider to select a value from a given range, subject to a step increment.", min_max))
                .child(section("Stepped Slider",
                    "Creates a slider to select a value from a given range, subject to a step increment", stepped)))
            .child(HStack::new().spacing(24.0)
                .child(section("Slider",
                    "Creates a slider to select a value from a given range", plain))
                .child(section("Slider Color",
                    "Customizing the color of the slider", color_rows)))
    );

    app.run();
}
