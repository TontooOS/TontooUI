//! ProgressView demo — 1:1 aus dem Screenshot
//! 4 Varianten direkt auf dem Background (#1d1d1d / #ececec), nur TontooUI API,
//! Ampeln bleiben sichtbar.

use tontooui::prelude::*;
use tontooui::{ProgressView, ProgressViewStyle};

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

// ── Previews wie im Screenshot ──

fn preview_initializers() -> impl Widget {
    // Screenshot erste Karte: 3 Bars — 42% + zwei Foo/bar Varianten
    VStack::new()
        .spacing(14.0)
        .child(ProgressView::new().value(0.42).label("Foo").progress_view_style(ProgressViewStyle::Linear).width(180.0))
        .child(ProgressView::new().value(0.65).progress_view_style(ProgressViewStyle::Linear).width(180.0))
        .child(VStack::new().spacing(4.0)
            .child(ProgressView::new().value(0.42).label("Foo").sub_label("bar").progress_view_style(ProgressViewStyle::Linear).width(180.0))
        )
}

fn preview_colors() -> impl Widget {
    // Screenshot zweite Karte: tint customization — roter Bar, rote Labels
    VStack::new()
        .spacing(14.0)
        .child(ProgressView::new().value(0.5).label("Foo").sub_label("Foo").tint(Color::from_rgb(255, 69, 58)).progress_view_style(ProgressViewStyle::Circular).size(36.0))
        .child(ProgressView::new().value(0.7).label("Foo").sub_label("bar").tint(Color::from_rgb(255, 69, 58)).progress_view_style(ProgressViewStyle::Linear).width(180.0))
}

fn preview_circular() -> impl Widget {
    // "The style of a progress view that uses a circular gauge"
    VStack::new()
        .spacing(12.0)
        .child(ProgressView::new().progress_view_style(ProgressViewStyle::Circular).size(36.0))
        .child(ProgressView::new().label("Foo").progress_view_style(ProgressViewStyle::Circular).size(36.0))
        .child(ProgressView::new().label("Foo").sub_label("Foo").progress_view_style(ProgressViewStyle::Circular).size(36.0))
        .child(ProgressView::new().label("Foo").sub_label("bar").value(0.4).progress_view_style(ProgressViewStyle::Circular).size(36.0))
}

fn preview_linear() -> impl Widget {
    // "A progress view that visually indicates its progress using a horizontal bar"
    VStack::new()
        .spacing(12.0)
        .child(ProgressView::new().label("Foo").progress_view_style(ProgressViewStyle::Linear).width(180.0))
        .child(ProgressView::new().value(0.55).label("Foo").sub_label("bar").progress_view_style(ProgressViewStyle::Linear).width(180.0))
}

fn main() {
    let mut app = App::new("ProgressView", 960, 420);
    // kein no_window_bar() — Ampeln sichtbar, reiner TontooUI API Gebrauch
    let title = Text::new("ProgressView").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row = HStack::new()
        .spacing(24.0)
        .child(cell("ProgressView", "The different ProgressView initializers", "initializer", preview_initializers()))
        .child(cell("ProgressView Colors", "Customizing the colors of the ProgressView", "style", preview_colors()))
        .child(cell("CircularProgressViewStyle", "The style of a progress view that uses a circular gauge to indicate the partial...", "style", preview_circular()))
        .child(cell("LinearProgressViewStyle", "A progress view that visually indicates its progress using a horizontal bar", "style", preview_linear()));

    let grid = VStack::new().spacing(28.0).child(row);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
