//! CircularGauge demo — SwiftUI "Accessory Circular Gauge" look (iOS/macOS).
//!
//! Built on the uikit App API with a declarative VStack/HStack layout and a
//! custom root widget. Three gauges (plain, float readout, percent readout
//! with min/max labels) are driven live by an interactive slider; a toggle
//! switches between the TontooOS light (`#ececec`) and dark (`#1d1d1d`)
//! themes.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use gtk::prelude::*;
use gtk::{self, Label as GtkLabel};
use gtk::Orientation;
use tontooui::prelude::*;
use uikit::widget::next_widget_id;

/// A raw-GTK piece wrapped as a uikit Widget so it can live in the
/// declarative tree.
struct GtkWrap(gtk::Widget);

impl Widget for GtkWrap {
    fn id(&self) -> WidgetId {
        next_widget_id()
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.0.clone()
    }
}

/// A gauge shared between the declarative tree and the slider callback, so
/// `set_value` can redraw it live after it was rendered.
struct GaugeHandle {
    gauge: Rc<RefCell<CircularGauge>>,
}

impl Widget for GaugeHandle {
    fn id(&self) -> WidgetId {
        self.gauge.borrow().id()
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.gauge.borrow().to_gtk()
    }
    fn is_interactive(&self) -> bool {
        false
    }
}

/// Update a CSS provider in place instead of registering a new one on every
/// theme toggle (the same pattern Slider uses for its per-frame styling).
fn update_cached_css(
    widget: &impl IsA<gtk::Widget>,
    css: &str,
    cache: &Rc<RefCell<Option<gtk::CssProvider>>>,
) {
    let mut borrow = cache.borrow_mut();
    if let Some(ref provider) = *borrow {
        provider.load_from_string(css);
    } else {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(css);
        widget.style_context().add_provider(
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION as u32,
        );
        *borrow = Some(provider);
    }
}

/// Page styling for one theme (background, card, state label, scale, toggle).
fn page_css(dark: bool) -> String {
    let bg = if dark { "#1d1d1d" } else { "#ececec" };
    let fg = if dark { "#d4d4d8" } else { "#3f3f46" };
    let muted = if dark { "#9a9aa2" } else { "#71717a" };
    let card_bg = if dark { "rgba(255,255,255,0.06)" } else { "rgba(255,255,255,0.6)" };
    let card_border = if dark { "rgba(255,255,255,0.05)" } else { "rgba(0,0,0,0.05)" };
    let btn_bg = if dark { "#27272a" } else { "#f4f4f5" };
    let btn_border = if dark { "rgba(255,255,255,0.08)" } else { "rgba(0,0,0,0.05)" };
    let accent = if dark { "#0a84ff" } else { "#007aff" };
    let (tr, tg, tb) = if dark { (63, 63, 70) } else { (212, 212, 212) };

    format!(
        ".page {{ background-color: {bg}; }}
         .cg-card {{ background-color: {card_bg}; border-radius: 16px; border: 1px solid {card_border}; }}
         .cg-state {{ font-family: 'SF Pro Display'; font-size: 12px; font-weight: 600; color: {muted}; }}
         .cg-state-val {{ font-family: 'SF Pro Mono'; font-size: 12px; color: {accent}; }}
         .cg-toggle {{ font-family: 'SF Pro Display'; font-size: 12px; font-weight: 500; color: {fg};
                      background-color: {btn_bg}; border-radius: 8px; padding: 6px 12px; border: 1px solid {btn_border}; }}
         .cg-scale trough {{ background-color: rgb({tr},{tg},{tb}); border-radius: 4px; min-height: 4px; }}
         .cg-scale highlight {{ background-color: {accent}; border-radius: 4px; min-height: 4px; }}
         .cg-scale slider {{ background-color: #ffffff; border: 1px solid rgba(0,0,0,0.15); border-radius: 50%;
                            min-width: 16px; min-height: 16px; box-shadow: 0 1px 4px rgba(0,0,0,0.25); }}",
        bg = bg,
        card_bg = card_bg,
        card_border = card_border,
        muted = muted,
        accent = accent,
        fg = fg,
        btn_bg = btn_bg,
        btn_border = btn_border,
        tr = tr,
        tg = tg,
        tb = tb,
    )
}

/// Build the inner page (gauges + slider card) for the current theme and
/// value, wiring the slider to the shared gauge handles.
fn build_content(value: Rc<RefCell<f32>>, dark: bool) -> gtk::Widget {
    let scheme = if dark { ColorScheme::Dark } else { ColorScheme::Light };

    let g1 = Rc::new(RefCell::new(
        CircularGauge::new().value(*value.borrow()).label("Foo").color_scheme(scheme),
    ));
    let g2 = Rc::new(RefCell::new(
        CircularGauge::new()
            .value(*value.borrow())
            .center(CircularGaugeCenter::Float(6))
            .label("Foo")
            .color_scheme(scheme),
    ));
    let g3 = Rc::new(RefCell::new(
        CircularGauge::new()
            .value(*value.borrow())
            .center(CircularGaugeCenter::Percent)
            .min_label("0")
            .max_label("100")
            .color_scheme(scheme),
    ));

    // ── Slider card ──
    let card = gtk::Box::new(Orientation::Vertical, 8);
    card.set_hexpand(true);
    card.set_margin_start(24);
    card.set_margin_end(24);
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.add_css_class("cg-card");

    let state_row = gtk::Box::new(Orientation::Horizontal, 0);
    let state_lbl = GtkLabel::new(Some("@State private var currentValue"));
    state_lbl.set_halign(gtk::Align::Start);
    state_lbl.set_hexpand(true);
    state_lbl.add_css_class("cg-state");
    let val_lbl = GtkLabel::new(Some(&format!("{:.2}", *value.borrow())));
    val_lbl.set_halign(gtk::Align::End);
    val_lbl.add_css_class("cg-state-val");
    state_row.append(&state_lbl);
    state_row.append(&val_lbl);
    card.append(&state_row);

    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
    scale.set_value(*value.borrow() as f64);
    scale.set_width_request(420);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    scale.add_css_class("cg-scale");
    scale.connect_value_changed({
        let value = value.clone();
        let g1 = g1.clone();
        let g2 = g2.clone();
        let g3 = g3.clone();
        let val_lbl = val_lbl.clone();
        move |s| {
            let v = s.value() as f32;
            *value.borrow_mut() = v;
            g1.borrow().set_value(v);
            g2.borrow().set_value(v);
            g3.borrow().set_value(v);
            val_lbl.set_text(&format!("{:.2}", v));
        }
    });
    card.append(&scale);

    // ── Declarative layout ──
    VStack::new()
        .alignment(HAlignment::Center)
        .spacing(28.0)
        .child(
            HStack::new()
                .spacing(28.0)
                .child(GaugeHandle { gauge: g1.clone() })
                .child(GaugeHandle { gauge: g2.clone() })
                .child(GaugeHandle { gauge: g3.clone() }),
        )
        .child(GtkWrap(card.upcast()))
        .to_gtk()
}

/// Root widget: fills the window, hosts the theme toggle and rebuilds the
/// inner content when the theme changes (keeping the slider value).
struct DemoRoot {
    id: WidgetId,
    value: Rc<RefCell<f32>>,
    dark: Rc<Cell<bool>>,
}

impl DemoRoot {
    fn new() -> Self {
        Self {
            id: next_widget_id(),
            value: Rc::new(RefCell::new(0.42)),
            dark: Rc::new(Cell::new(false)),
        }
    }
}

impl Widget for DemoRoot {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn to_gtk(&self) -> gtk::Widget {
        let page = gtk::Box::new(Orientation::Vertical, 0);
        page.set_hexpand(true);
        page.set_vexpand(true);
        page.add_css_class("page");

        let css_cache: Rc<RefCell<Option<gtk::CssProvider>>> = Rc::new(RefCell::new(None));
        update_cached_css(&page, &page_css(self.dark.get()), &css_cache);

        let overlay = gtk::Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);

        let host = gtk::Box::new(Orientation::Vertical, 0);
        host.set_hexpand(true);
        host.set_vexpand(true);
        host.set_halign(gtk::Align::Center);
        host.set_valign(gtk::Align::Center);
        overlay.set_child(Some(&host));

        host.append(&build_content(self.value.clone(), self.dark.get()));

        // ── Dark-mode toggle ──
        let btn = gtk::Button::with_label(if self.dark.get() { "Light Mode" } else { "Dark Mode" });
        btn.add_css_class("cg-toggle");
        btn.set_halign(gtk::Align::End);
        btn.set_valign(gtk::Align::Start);
        btn.set_margin_top(20);
        btn.set_margin_end(20);
        {
            let dark = self.dark.clone();
            let value = self.value.clone();
            let css_cache = css_cache.clone();
            let page = page.clone();
            let host = host.clone();
            btn.connect_clicked(move |_| {
                dark.set(!dark.get());
                update_cached_css(&page, &page_css(dark.get()), &css_cache);
                while let Some(c) = host.first_child() {
                    host.remove(&c);
                }
                host.append(&build_content(value.clone(), dark.get()));
            });
        }
        overlay.add_overlay(&btn);

        page.append(&overlay);
        page.upcast()
    }

    fn is_interactive(&self) -> bool {
        false
    }
}

fn main() {
    let mut app = App::new("Circular Gauge Demo", 560, 460);
    app.no_window_bar();
    app.set_root(DemoRoot::new());
    app.run();
}