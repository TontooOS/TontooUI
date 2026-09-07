//! TontooUI System Theme Demo — zeigt ob System-Standard Dark oder Light ist.
//!
//! Nur TontooUI (`tontooui::prelude::*`), UIKit/GTK bleiben hinter TontooUI versteckt.
//! Fenster-Chrome mit Ampeln (Standard), SF Pro Display, #1d1d1d dark / #ececec light.

use tontooui::prelude::*;

fn main() {
    // System-Standard ermitteln — TontooOS nutzt gsettings / env
    let system = ColorScheme::detect_system();
    let is_dark = system == ColorScheme::Dark;

    let mut app = App::new("System Theme", 480, 260);
    // Window-Hintergrund an System anpassen (#1d1d1d dark, #ececec light)
    app.set_color_scheme(system);

    // Texte in SF Pro, Farben passend zum Scheme
    let title = if is_dark { "Dark Mode" } else { "Light Mode" };
    let subtitle = if is_dark {
        "System-Standard: Dark (#1d1d1d)"
    } else {
        "System-Standard: Light (#ececec)"
    };
    let hint = match system {
        ColorScheme::Dark => "Erkannt via ColorScheme::detect_system()",
        ColorScheme::Light => "Erkannt via ColorScheme::detect_system()",
    };

    let fg_main = if is_dark {
        Color::from_hex("#ececec").unwrap()
    } else {
        Color::from_hex("#1d1d1d").unwrap()
    };
    let fg_sub = if is_dark {
        Color::from_rgb(142, 142, 147)
    } else {
        Color::from_rgb(110, 110, 115)
    };

    // Kleines Icon als Typo-Hinweis — SF Pro Display wird von TontooUI automatisch gesetzt
    let badge_text = if is_dark { "⬤ Dark" } else { "○ Light" };

    let root = VStack::new()
        .spacing(14.0)
        .child(Text::new(badge_text).font_size(13.0).color(fg_sub))
        .child(Text::new(title).font_size(32.0).bold().color(fg_main))
        .child(Text::new(subtitle).font_size(14.0).color(fg_main))
        .child(Text::new(hint).font_size(11.0).color(fg_sub))
        .child(
            Text::new("SF Pro Display • Ampeln sichtbar • nur TontooUI")
                .font_size(10.0)
                .color(fg_sub),
        );

    app.set_root(root);
    app.run();
}
