//! TontooUI Glass demo — everything around the Liquid Glass effect.
//!
//! Showcases `GlassEffect` (default + tinted), the core
//! `ViewModifierExt::glassEffect` chain, `ButtonStyle::Glass` styles and the
//! glass capsule `Toolbar`. All elements render directly on the window
//! background (#1d1d1d dark / #ececec light) with SF Pro.

use tontooui::prelude::*;

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(
            HStack::new().spacing(0.0).child(
                Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap()),
            ),
        )
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(220.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(220.0))
}

fn main() {
    let system = ColorScheme::detect_system();
    let is_dark = system == ColorScheme::Dark;
    let mut app = App::new("TontooUI Glass", 1180, 820);
    app.set_color_scheme(system);

    let title = Text::new("GlassEffect").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());
    let subtitle = Text::new("Applies the Liquid Glass effect to a view").font_size(11.0).color(
        if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() },
    );

    // Row 1 — GlassEffect palette widget: default + tints.
    let blue = Color::from_rgb(10, 132, 255);
    let accent = Color::from_hex("#FF6B2B").unwrap();
    let row1 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Glass effect",
            "Default frosted white (0.28 dark / 0.22 light).",
            "modifier",
            GlassEffect::new(),
        ))
        .child(cell(
            "Tinted blue",
            "GlassEffect with a blue tint color.",
            "modifier",
            GlassEffect::new().tint(blue),
        ))
        .child(cell(
            "Tinted accent",
            "GlassEffect with the TontooOS orange accent.",
            "modifier",
            GlassEffect::new().tint(accent),
        ))
        .child(core_modifier_cell());

    // Row 2 — Glass buttons + glass toolbar.
    let glass_buttons = VStack::new()
        .spacing(8.0)
        .child(
            HStack::new().spacing(8.0)
                .child(Button::new("Tap Me").style(ButtonStyle::Glass))
                .child(Button::new("Tap Me").style(ButtonStyle::GlassProminent)),
        )
        .child(
            HStack::new().spacing(8.0)
                .child(Button::new("Tinted").style(ButtonStyle::Glass).tint(blue))
                .child(Button::new("Tinted").style(ButtonStyle::GlassProminent).tint(accent)),
        );

    let glass_toolbar = Toolbar::new()
        .item(ToolbarItem::new("chevron.up").on_click(|| println!("glass up")))
        .item(ToolbarItem::new("chevron.down").on_click(|| println!("glass down")))
        .spacer(ToolbarSpacer::fixed())
        .item(ToolbarItem::new("heart").shared_background(false).on_click(|| println!("bare item, no glass")));

    let row2 = HStack::new()
        .spacing(24.0)
        .child(cell(
            "Glass buttons",
            "Glass border artwork based on the button context.",
            "style",
            glass_buttons,
        ))
        .child(cell(
            "Glass toolbar",
            "Shared glass capsule, last item renders bare.",
            "toolbar",
            glass_toolbar,
        ))
        .child(cell(
            "Materials",
            "Frosted materials used below the glass effect.",
            "type",
            Materials::new(),
        ));

    let root = VStack::new()
        .spacing(18.0)
        .child(title)
        .child(subtitle)
        .child(row1)
        .child(row2);
    app.set_root(root);
    app.run();
}

fn core_modifier_cell() -> impl Widget {
    // Core backing: attach the modifier to a raw View, then read it back.
    // This proves the declarative chain works without the palette widget.
    let view = View::empty().glassEffect(Some(Color::from_rgb(10, 132, 255)));
    let mods = get_modifiers(view.id());
    let state = format!(
        "enabled={} tint_set={}",
        mods.glass_effect_enabled,
        mods.glass_tint.is_some()
    );

    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(
            HStack::new().spacing(0.0).child(
                Text::new("modifier").font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap()),
            ),
        )
        .child(GlassEffect::new().tint(Color::from_rgb(10, 132, 255)))
        .child(Text::new("Core glassEffect()").font_size(11.0).bold().color(title_c).max_width(220.0))
        .child(
            Text::new(format!("View::empty().glassEffect(Some(blue)) — {}", state))
                .font_size(9.0)
                .color(desc_c)
                .max_width(220.0),
        )
}
