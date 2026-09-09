//! Apply GlassContainer to a text input: the input itself stays fully
//! functional, its background is replaced with liquid glass.
//!
//! No backdrop at all: the containers lie directly on the app background.

use tontooui::prelude::*;

fn main() {
    let system = ColorScheme::detect_system();
    let is_dark = system == ColorScheme::Dark;
    let mut app = App::new("Glass Text Field", 640, 420);
    app.set_color_scheme(system);

    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    let title = Text::new("Apply Glass Container").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    // Plain field for contrast: input paints its own background.
    let plain = TextInput::new("Plain input...")
        .width(360.0)
        .on_change(|t| println!("plain: {}", t));

    // Same field on glass: transparent input, lying on the app background.
    let glass_field = GlassContainer::new(
        TextInput::new("Glass input...")
            .transparent()
            .width(320.0)
            .on_change(|t| println!("glass: {}", t)),
    )
    .size(380.0, 64.0)
    .radius(20.0);

    // Glass also works for plain content.
    let glass_label = GlassContainer::new(Text::new("Content stays, background becomes glass").font_size(12.0))
        .size(380.0, 56.0)
        .radius(28.0);

    let root = VStack::new()
        .spacing(16.0)
        .child(title)
        .child(Text::new("Type in both fields — the glass one keeps the live input.").font_size(11.0).color(desc_c))
        .child(plain)
        .child(glass_field)
        .child(glass_label);
    app.set_root(root);
    app.run();
}
