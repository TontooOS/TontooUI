use tontooui::prelude::*;

fn main() {
    let mut app = App::new("TontooUI Demo", 800, 600);

    app.set_root(
        VStack::new()
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
            .child(Slider::new(0.0, 100.0)
                .value(50.0)
                .step(1.0)
                .label("Volume")
                .on_change(|v| println!("Volume: {:.0}", v)))
            .child(Slider::new(0.0, 1.0)
                .value(0.7)
                .step(0.01)
                .label("Opacity")
                .on_change(|v| println!("Opacity: {:.2}", v)))
    );

    app.run();
}
