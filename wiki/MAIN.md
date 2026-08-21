# TontooUI -- Wiki

TontooUI is a SwiftUI-inspired declarative UI layer for TontooOS, built on top of TontooUIKit. It provides pre-made elements with a clean builder API for building native GTK4 applications.

- Repository: tontoo-os/TontooLibs/TontooUI
- License: MIT
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Wiki design system and conventions |
| TextInput | [TextInput.md](TextInput.md) | Single-line text input field |
| WheelPicker | [WheelPicker.md](WheelPicker.md) | macOS/iOS scroll wheel picker |
| Slider | [Slider.md](Slider.md) | Spring-physics slider with white pill thumb, dark glass while pressed |
| ProgressView | [ProgressView.md](ProgressView.md) | Loading indicator: spinner or progress ring |
| Sidebar | [Sidebar.md](Sidebar.md) | macOS-style sidebar with traffic lights, search, items, color/gradient background |
| ContentUnavailableView | [ContentUnavailableView.md](ContentUnavailableView.md) | Empty state with icon, title and hint message |
| CircularGauge | [CircularGauge.md](CircularGauge.md) | iOS/macOS accessory circular gauge with live value |

## Quick Start

```rust
use tontooui::prelude::*;

fn main() {
    let mut app = App::new("My App", 800, 600);
    app.set_root(
        VStack::new()
            .spacing(8.0)
            .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
            .child(TextInput::new("Enter text...")
                .on_change(|text| println!("Changed: {}", text)))
    );
    app.run();
}
```

See [TextInput.md](TextInput.md), [WheelPicker.md](WheelPicker.md), and [Slider.md](Slider.md) for details.

## Architecture

```
tontooui (SwiftUI-style layer)
 |
 +-- TextInput       (placeholder, password, on_change, on_submit)
 +-- WheelPicker     (spring physics, snap-to-center, 3D fade)
 +-- Slider          (spring physics, white pill thumb, dark glass while pressed)
 +-- ProgressView    (indeterminate spinner / determinate ring)
 +-- Sidebar         (traffic lights, search, selectable item list)
+-- ContentUnavailableView (empty state with icon, title, message)
+-- CircularGauge (270-degree ring gauge, live value)
 |
 +-- uikit (backend)
      +-- View, Widget, ViewContent
      +-- App, ViewController
      +-- Layout (VStack, HStack, ZStack)
      +-- Animation (Animator, Spring)
```

## Performance Notes

TontooUI is designed so long-running apps do not accumulate work over time:

- `Slider` and `WheelPicker` run their 60 FPS tick loop only while the physics
  are active (dragging, scrolling, wobbling or settling). As soon as the element
  settles the loop stops itself and is restarted by the next interaction, so an
  idle element does not wake the main loop or invalidate layout at 60 FPS
  forever.
- `Slider` applies its per-frame thumb styling through a single cached
  `gtk::CssProvider` that is updated in place, instead of registering a new
  CSS provider on every frame.

## Cross References

- [TextInput.md](TextInput.md) -- text input element
- [WheelPicker.md](WheelPicker.md) -- scroll wheel picker
- [Slider.md](Slider.md) -- spring-physics slider
- [ProgressView.md](ProgressView.md) -- loading indicator
- [Sidebar.md](Sidebar.md) -- sidebar with traffic lights and item list
- [ContentUnavailableView.md](ContentUnavailableView.md) -- empty state with icon, title, message
- [CircularGauge.md](CircularGauge.md) -- 270-degree ring gauge with live value
