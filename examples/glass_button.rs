//! Small round glass icon buttons: a circle GlassContainer with a bare
//! icon button inside, lying directly on the app background.

use tontooui::prelude::*;

fn round_icon_button(icon: &str, tint: Color, label: &str) -> GlassContainer {
    let msg = format!("glass button {}", label);
    GlassContainer::new(
        Button::new("")
            .icon(icon)
            .icon_size(40.0)
            .style(ButtonStyle::Plain)
            .tint(tint)
            .height(56.0)
            .on_click(move || println!("{}", msg)),
    )
    .size(96.0, 96.0)
    .radius(48.0)
}

fn main() {
    let system = ColorScheme::detect_system();
    let is_dark = system == ColorScheme::Dark;
    let mut app = App::new("Glass Icon Buttons", 560, 380);
    app.set_color_scheme(system);

    let icon_tint = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    let title = Text::new("Glass Icon Buttons").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row = HStack::new()
        .spacing(20.0)
        .child(round_icon_button("chevron.left", icon_tint, "back"))
        .child(round_icon_button("plus", icon_tint, "add"))
        .child(round_icon_button("gearshape", icon_tint, "settings"));

    let root = VStack::new()
        .spacing(16.0)
        .child(title)
        .child(Text::new("Round 96px glass, bare icon inside — clicks print below.").font_size(11.0).color(desc_c))
        .child(row);
    app.set_root(root);
    app.run();
}
