//! Temporary probe: prints the wheel picker's real allocation under Xvfb.

use gtk::prelude::*;
use tontooui::WheelPicker;
use uikit::widget::Widget;

fn main() {
    let app = gtk::Application::builder().application_id("dev.tontoo.probe").build();

    app.connect_activate(|app| {
        let win = gtk::ApplicationWindow::new(app);
        win.set_default_size(800, 600);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.set_hexpand(true);
        vbox.set_vexpand(true);

        let wheel = WheelPicker::new().items(["1", "2", "3", "4", "5", "6"]).selected("3");
        let w = wheel.to_gtk();
        // Mimic VStack default alignment (Leading -> Start).
        w.set_halign(gtk::Align::Start);
        vbox.append(&w);

        win.set_child(Some(&vbox));
        win.present();

        let w2 = w.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
            let alloc = w2.allocation();
            let (min, nat, _, _) = w2.measure(gtk::Orientation::Horizontal, -1);
            println!(
                "WHEEL_ALLOC width={:.0} height={:.0} natural_width={} min_width={}",
                alloc.width(),
                alloc.height(),
                nat,
                min
            );
            glib::ControlFlow::Break
        });
    });

    app.run_with_args::<&str>(&[]);
}