//! TontooUI Gauge demo — all 11 SwiftUI Gauge variants 1:1
//! Gauges sit directly on the app background (#1d1d1d dark / #ececec light),
//! no card wrapper. Pure TontooUI API only — traffic lights stay visible.

use tontooui::prelude::*;
use tontooui::{Gauge, GaugeStyle};

// ── Cell helper — pure TontooUI API, no gtk, no card background ──
// Each cell is just a VStack directly on the background: badge + preview + title + desc.
// This satisfies "Gauges direkt auf dem background".

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    // Light Mode #ececec vs Dark #1d1d1d — title/desc adapt live via Gauge fix; cell texts adapt at build
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(
            HStack::new().spacing(0.0)
                .child(Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap()))
        )
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(200.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(200.0))
}

// ── preview builders — Widget only, directly on background ──

fn preview_min_max_current() -> impl Widget {
    Gauge::new(42.0).in_range(0.0, 100.0)
        .label("Foo").current_value_label("42").minimum_value_label("0").maximum_value_label("100")
        .width(200.0)
}

fn preview_current_value() -> impl Widget {
    Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").current_value_label("0.42000").width(200.0)
}

fn preview_plain() -> impl Widget {
    Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").width(200.0)
}

fn preview_colors() -> impl Widget {
    VStack::new().spacing(10.0)
        .child(
            HStack::new().spacing(18.0)
                .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircularCapacity).tint(Color::from_rgb(48, 209, 88)).width(56.0))
                .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircularCapacity).tint(Color::from_rgb(255, 159, 10)).width(56.0))
        )
        .child(Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").current_value_label("42").tint(Color::from_rgb(255, 204, 0)).width(180.0))
        .child(Gauge::new(40.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").current_value_label("42").tint(Color::from_rgb(48, 209, 88)).width(180.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").current_value_label("42").tint(Color::from_rgb(255, 69, 58)).width(180.0))
}

fn preview_circular_style() -> impl Widget {
    VStack::new().spacing(12.0)
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::Circular).width(92.0))
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").current_value_label("0.42000").gauge_style(GaugeStyle::Circular).width(92.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::Circular).width(92.0))
}

fn preview_linear_style() -> impl Widget {
    VStack::new().spacing(14.0)
        .child(Gauge::new(0.42).in_range(0.0, 1.0).gauge_style(GaugeStyle::Linear).width(180.0))
        .child(Gauge::new(0.42).in_range(0.0, 1.0).current_value_label("0.42000").gauge_style(GaugeStyle::Linear).width(180.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::Linear).width(180.0))
}

fn preview_accessory_linear_capacity() -> impl Widget {
    VStack::new().spacing(14.0)
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryLinearCapacity).width(180.0))
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").current_value_label("0.42000").gauge_style(GaugeStyle::AccessoryLinearCapacity).width(180.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::AccessoryLinearCapacity).width(180.0))
}

fn preview_accessory_linear() -> impl Widget {
    VStack::new().spacing(14.0)
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryLinear).width(180.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryLinear).width(180.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::AccessoryLinear).width(180.0))
}

fn preview_linear_capacity() -> impl Widget {
    VStack::new().spacing(14.0)
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::LinearCapacity).width(180.0))
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").current_value_label("0.42000").gauge_style(GaugeStyle::LinearCapacity).width(180.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::LinearCapacity).width(180.0))
}

fn preview_accessory_circular_capacity() -> impl Widget {
    HStack::new().spacing(10.0)
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryCircularCapacity).tint(Color::from_rgb(10,132,255)).width(56.0))
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").current_value_label("0.42000").gauge_style(GaugeStyle::AccessoryCircularCapacity).width(56.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircularCapacity).width(56.0))
}

fn preview_accessory_circular() -> impl Widget {
    HStack::new().spacing(10.0)
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryCircular).width(56.0))
        .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").current_value_label("0.42000").gauge_style(GaugeStyle::AccessoryCircular).width(56.0))
        .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircular).width(56.0))
}

fn main() {
    let mut app = App::new("TontooUI Gauges", 1080, 860);
    // kein `no_window_bar()` — Ampeln bleiben sichtbar (TontooOS Standard)

    let title = Text::new("Gauge").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new().spacing(24.0)
        .child(cell("Min Max Current Value Gauge", "Creates a gauge showing a value within a range and describes the gauge's...", "initializer", preview_min_max_current()))
        .child(cell("Current Value Gauge", "Creates a gauge showing a value within a range and that describes the gauge's...", "initializer", preview_current_value()))
        .child(cell("Gauge", "Creates a gauge showing a value within a range and describes the gauge's...", "initializer", preview_plain()))
        .child(cell("Gauge Colors", "Customizing the colors of different styles with minimal effort", "style", preview_colors()));
    let row2 = HStack::new().spacing(24.0)
        .child(cell("CircularGaugeStyle", "A gauge style that displays an open ring with a marker that appears at a point...", "style", preview_circular_style()))
        .child(cell("LinearGaugeStyle", "A gauge style that displays a bar with a marker that appears at a point along t...", "style", preview_linear_style()))
        .child(cell("AccessoryLinearCapacityGaugeS...", "A gauge style that displays bar that fills from leading to trailing edges as the...", "style", preview_accessory_linear_capacity()))
        .child(cell("AccessoryLinearGaugeStyle", "A gauge style that displays bar with a marker that appears at a point along t...", "style", preview_accessory_linear()));
    let row3 = HStack::new().spacing(24.0)
        .child(cell("LinearCapacityGaugeStyle", "A gauge style that displays a bar that fills from leading to trailing edges as t...", "style", preview_linear_capacity()))
        .child(cell("AccessoryCircularCapacityGauge...", "A gauge style that displays a closed ring that's partially filled in to indicate the...", "style", preview_accessory_circular_capacity()))
        .child(cell("AccessoryCircularGaugeStyle", "A gauge style that displays an open ring with a marker that appears at a point...", "style", preview_accessory_circular()));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2).child(row3);
    let root = VStack::new().spacing(18.0).child(title).child(grid);

    app.set_root(root);
    app.run();
}
