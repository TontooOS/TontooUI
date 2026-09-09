//! TontooUI demo — sidebar navigation with a detail area.
//!
//! Clicking a sidebar row swaps the detail content on the right
//! (only the detail box rebuilds, like the sidebar playground).

use gtk::prelude::*;
use std::cell::RefCell;
use tontooui::prelude::*;
#[cfg(feature = "coreicon")]
use tontooui::SidebarIcon;
use uikit::layout::padding;
use uikit::style::Padding;
use uikit::widget::{Widget as UIKitWidget, WidgetId, next_widget_id};

// The detail holder lives on the GTK main thread; `on_select` only swaps
// its child, so the sidebar keeps all state.
thread_local! {
    static DETAIL_SLOT: RefCell<Option<gtk::Box>> = RefCell::new(None);
}

/// Layout slot for the detail area: renders the holder box stored in
/// `DETAIL_SLOT`, so page swaps never touch the window layout.
struct DetailSlot {
    id: WidgetId,
    box_: gtk::Box,
}

impl UIKitWidget for DetailSlot {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.box_.clone().upcast()
    }
    fn expand_vertically(&self) -> bool {
        true
    }
    fn flex_weight(&self) -> f32 {
        1.0
    }
}

/// Wrap a ready-made GTK widget so it can live in a TontooUI layout.
struct Raw {
    id: WidgetId,
    widget: gtk::Widget,
}

impl UIKitWidget for Raw {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn to_gtk(&self) -> gtk::Widget {
        self.widget.clone()
    }
}

fn page(title: &str, body: Vec<gtk::Widget>) -> gtk::Widget {
    let mut col = VStack::new()
        .spacing(16.0)
        .child(Text::new(title).font_size(24.0).bold());
    for w in body {
        col = col.child(Raw {
            id: next_widget_id(),
            widget: w,
        });
    }
    // Breathing room: titles no longer touch the window edge.
    UIKitWidget::to_gtk(&padding(Padding::new(28.0, 24.0, 24.0, 24.0)).child(col))
}

fn gtk_of(w: impl UIKitWidget + 'static) -> gtk::Widget {
    UIKitWidget::to_gtk(&w)
}

/// Detail content per sidebar index (see the sidebar item order in `main`).
fn detail_page(index: usize) -> gtk::Widget {
    match index {
        0 => page(
            "Wi-Fi",
            vec![
                gtk_of(TextInput::new("Enter your name...")
                    .on_change(|text| println!("Name: {}", text))
                    .on_submit(|text| println!("Submitted: {}", text))),
                gtk_of(TextInput::new("Email...")
                    .on_change(|text| println!("Email: {}", text))),
                gtk_of(TextInput::new("Password...")
                    .password()
                    .on_submit(|_text| println!("Password submitted"))),
            ],
        ),
        1 => page(
            "Bluetooth",
            vec![
                gtk_of(Text::new("Channel:").font_size(16.0).bold()),
                gtk_of(
                    WheelPicker::new()
                        .items(["1", "2", "3", "4", "5", "6"])
                        .selected("3")
                        .on_change(|val| println!("Picked: {}", val)),
                ),
            ],
        ),
        2 => page(
            "Network",
            vec![
                gtk_of(Text::new("Sliders:").font_size(16.0).bold()),
                gtk_of(
                    Slider::new(0.0, 1.0)
                        .value(0.7)
                        .step(0.01)
                        .label("Opacity")
                        .track_color(Color::from_rgb(60, 60, 60))
                        .on_change(|v| println!("Opacity: {:.2}", v)),
                ),
            ],
        ),
        3 => page(
            "VPN",
            vec![
                gtk_of(Text::new("Loading Indicators:").font_size(16.0).bold()),
                gtk_of(ProgressView::new().label("Foo")),
                gtk_of(
                    ProgressView::new()
                        .value(0.42)
                        .label("Foo")
                        .sub_label("bar")
                        .accent_color(Color::from_rgb(0, 122, 255)),
                ),
            ],
        ),
        4 => page(
            "Battery",
            vec![gtk_of(
                Toggle::new("Low Power Mode")
                    .on_change(|v| println!("Low Power Mode: {}", v)),
            )],
        ),
        5 => page(
            "Sound",
            vec![gtk_of(
                Slider::new(6.0, 46.0)
                    .value(50.0)
                    .step(1.0)
                    .label("Volume")
                    .accent_color(Color::from_rgb(255, 149, 0))
                    .track_color(Color::from_rgb(45, 45, 45))
                    .on_change(|v| println!("Volume: {:.0}", v)),
            )],
        ),
        6 => page(
            "Chat",
            vec![gtk_of(
                TextInput::new("Message...")
                    .on_change(|text| println!("Message: {}", text))
                    .on_submit(|text| println!("Sent: {}", text)),
            )],
        ),
        _ => page(
            "General",
            vec![
                gtk_of(Text::new("Content Unavailable:").font_size(16.0).bold()),
                gtk_of(
                    ContentUnavailableView::new()
                        .query("foo")
                        .width(320.0)
                        .height(250.0),
                ),
            ],
        ),
    }
}

fn show_page(index: usize) {
    let widget = detail_page(index);
    DETAIL_SLOT.with(|slot| {
        if let Some(holder) = slot.borrow().as_ref() {
            while let Some(child) = holder.first_child() {
                holder.remove(&child);
            }
            holder.append(&widget);
        }
    });
}

fn main() {
    // GTK widgets (the detail holder) are created below, before `app.run()`
    // initializes GTK itself — so initialize explicitly first.
    let _ = gtk::init();

    let mut app = App::new("TontooUI Demo", 900, 620);
    app.no_window_bar();

    #[cfg(feature = "coreicon")]
    let sidebar = Sidebar::new()
        .item("Wi-Fi", SidebarIcon::file("../CoreIcon/examples/app_icon_demo/images_dark.png"))
        .item("Bluetooth", SidebarIcon::sf("antenna.radiowaves.left.and.right", Color::from_rgb(0, 122, 255)))
        .item("Network", SidebarIcon::sf("globe", Color::from_rgb(0, 122, 255)))
        .item("VPN", SidebarIcon::sf("lock.shield", Color::from_rgb(0, 122, 255)))
        .item("Battery", SidebarIcon::sf("battery.100", Color::from_rgb(52, 199, 89)))
        .item("Sound", SidebarIcon::sf_gradient("speaker.wave.2.fill",
            coreicon::Gradient::linear_two(
                coreicon::Color::new(1.0, 0.27, 0.23, 1.0),
                coreicon::Color::new(1.0, 0.62, 0.04, 1.0))))
        .item("Chat", SidebarIcon::file("../CoreIcon/examples/app_icon_demo/images_dark.png"))
        .section("System")
        .item("General", SidebarIcon::sf("gearshape", Color::from_rgb(142, 142, 147)))
        .selected(0)
        .on_select(|i| {
            println!("Sidebar selected: {}", i);
            show_page(i);
        });

    #[cfg(not(feature = "coreicon"))]
    let sidebar = Sidebar::new()
        .item("Wi-Fi", "wifi.circle.fill.png")
        .item("Bluetooth", "antenna.radiowaves.left.and.right.png")
        .selected(0)
        .on_select(|i| {
            println!("Sidebar selected: {}", i);
            show_page(i);
        });

    // Detail holder: filled by `show_page`, swapped on every click.
    let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    holder.set_hexpand(true);
    holder.set_vexpand(true);
    DETAIL_SLOT.with(|slot| *slot.borrow_mut() = Some(holder.clone()));
    let detail = DetailSlot {
        id: next_widget_id(),
        box_: holder,
    };

    show_page(0);

    app.set_root(
        HStack::new()
            .spacing(0.0)
            .child(sidebar)
            .child(detail),
    );

    app.run();
}
