use tontooui::prelude::*;
#[cfg(feature = "coreicon")]
use tontooui::SidebarIcon;

fn main() {
    let mut app = App::new("TontooUI Demo", 800, 600);
    app.no_window_bar();

    #[cfg(feature = "coreicon")]
    let sidebar = Sidebar::new()
        .item("Wi-Fi", SidebarIcon::sf("wifi.circle.fill", Color::from_rgb(0, 122, 255)))
        .item("Bluetooth", SidebarIcon::sf("antenna.radiowaves.left.and.right", Color::from_rgb(0, 122, 255)))
        .item("Network", SidebarIcon::sf("globe", Color::from_rgb(0, 122, 255)))
        .item("VPN", SidebarIcon::sf("lock.shield", Color::from_rgb(0, 122, 255)))
        .item("Battery", SidebarIcon::sf("battery.100", Color::from_rgb(52, 199, 89)))
        .item("General", SidebarIcon::sf("gearshape", Color::from_rgb(142, 142, 147)))
        .item("Sound", SidebarIcon::sf_gradient("speaker.wave.2.fill",
            coreicon::Gradient::linear_two(
                coreicon::Color::new(1.0, 0.27, 0.23, 1.0),
                coreicon::Color::new(1.0, 0.62, 0.04, 1.0))))
        .background_gradient(coreicon::Gradient::linear_two(
            coreicon::Color::new(0.04, 0.18, 0.08, 1.0),
            coreicon::Color::new(0.12, 0.50, 0.24, 1.0)))
        .selected(0)
        .on_select(|i| println!("Sidebar selected: {}", i));

    #[cfg(not(feature = "coreicon"))]
    let sidebar = Sidebar::new()
        .item("Wi-Fi", "wifi.circle.fill.png")
        .item("Bluetooth", "antenna.radiowaves.left.and.right.png")
        .selected(0)
        .on_select(|i| println!("Sidebar selected: {}", i));

    app.set_root(
        HStack::new()
            .spacing(0.0)
            .child(sidebar)
            .child(VStack::new()
                .spacing(16.0)
                .child(Text::new("TontooUI Demo").font_size(24.0).bold())
                .child(TextInput::new("Enter your name...")
                    .on_change(|text| println!("Name: {}", text))
                    .on_submit(|text| println!("Submitted: {}", text)))
                .child(TextInput::new("Email...")
                    .on_change(|text| println!("Email: {}", text)))
                .child(TextInput::new("Password...")
                    .password()
                    .on_submit(|_text| println!("Password submitted")))
                .child(Text::new("Wheel Picker:").font_size(16.0).bold())
                .child(WheelPicker::new()
                    .items(["1", "2", "3", "4", "5", "6"])
                    .selected("3")
                    .on_change(|val| println!("Picked: {}", val)))
                .child(Text::new("Sliders:").font_size(16.0).bold())
                .child(Slider::new(6.0, 46.0)
                    .value(50.0)
                    .step(1.0)
                    .label("Volume")
                    .accent_color(Color::from_rgb(255, 149, 0))
                    .track_color(Color::from_rgb(45, 45, 45))
                    .on_change(|v| println!("Volume: {:.0}", v)))
                .child(Slider::new(0.0, 1.0)
                    .value(0.7)
                    .step(0.01)
                    .label("Opacity")
                    .track_color(Color::from_rgb(60, 60, 60))
                    .on_change(|v| println!("Opacity: {:.2}", v)))
                .child(Text::new("Loading Indicators:").font_size(16.0).bold())
                .child(ProgressView::new().label("Foo"))
                .child(ProgressView::new()
                    .value(0.42)
                    .label("Foo")
                    .sub_label("bar")
                    .accent_color(Color::from_rgb(0, 122, 255)))
                .child(Text::new("Content Unavailable:").font_size(16.0).bold())
                .child(ContentUnavailableView::new()
                    .query("foo")
                    .width(320.0)
                    .height(250.0)))
    );

    app.run();
}
