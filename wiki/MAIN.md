# TontooUI – Wiki

SwiftUI-inspired declarative UI layer for TontooOS: elements, layout,
theme, animation and a Vello/WGPU renderer.

- Repository: https://github.com/TontooOS/Libs
- License: TCL
- Version: 26.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Animation | [Animation.md](Animation.md) | Frame clock, tweens, springs, decay |
| Alerts | [Alerts.md](Alerts.md) | Modal frosted alert with OK/Cancel actions |
| Button | [Button.md](Button.md) | Standard button with CoreIcon SF Symbols |
| Colors | [Colors.md](Colors.md) | System colors and linear/radial/angular gradients |
| ContentUnavailable | [ContentUnavailable.md](ContentUnavailable.md) | Empty-state placeholder with refresh |
| Divider | [Divider.md](Divider.md) | Full-bleed horizontal and vertical dividers |
| Gauge | [Gauge.md](Gauge.md) | Basic, linear, circular and capacity gauges |
| Glass | [Glass.md](Glass.md) | Liquid glass container and backdrop blur |
| Gestures | [Gestures.md](Gestures.md) | Tap, long press, drag and magnify areas |
| Groupbox | [Groupbox.md](Groupbox.md) | Basic group box with centered text |
| Images | [Images.md](Images.md) | SF Symbol, app resource, URL and overlay card |
| Layout | [Layout.md](Layout.md) | VStack, HStack, ZStack, modifiers, `View` |
| List | [List.md](List.md) | Static text list with row dividers |
| Link | [Link.md](Link.md) | Blue link opening the default browser |
| Label | [Label.md](Label.md) | Icon, image, styled and icon-only labels |
| Menu | [Menu.md](Menu.md) | Simple dropdown with action rows, picker base |
| Material | [Material.md](Material.md) | Translucent material veils in five thicknesses |
| Renderer | [Renderer.md](Renderer.md) | Window shell, frame pipeline, backdrop blur |
| Picker | [Picker.md](Picker.md) | Segmented, inline, menu and date pickers |
| Progress | [Progress.md](Progress.md) | Linear progress bar with chase buffer |
| Scrollbar | [Scrollbar.md](Scrollbar.md) | Overlay side bar with fade, drag and page jump |
| ScrollView | [ScrollView.md](ScrollView.md) | Clipped scroll container with integrated scrollbar |
| Sheets | [Sheets.md](Sheets.md) | Modal sheet with sizes, custom background, ESC |
| Shapes | [Shapes.md](Shapes.md) | Rectangle, circle, rounded, capsule and custom shapes |
| Slider | [Slider.md](Slider.md) | Slider with steps, labels, ticks, glass |
| Stepper | [Stepper.md](Stepper.md) | Basic stepper with step size, range and limit dimming |
| Theme | [Theme.md](Theme.md) | Live dark/light, accent, glass stage |
| Text | [Text.md](Text.md) | Basic text with styles and foregrounds |
| Textfield | [Textfield.md](Textfield.md) | Slim and large single-line fields |
| Titlebar | [Titlebar.md](Titlebar.md) | Custom decoration bar with drag region |
| Toggle | [Toggle.md](Toggle.md) | Switch, button and checkbox styles |
| Toolbar | [Toolbar.md](Toolbar.md) | Small clear-glass icon toolbar |

## Quick Start

```rust
use tontooui::elements::Titlebar;
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, run};
use vello::Scene;

struct Hello {
    bar: Titlebar,
}

impl App for Hello {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut tontooui::renderer::ImageLoader<'_>,
        viewport: Viewport,
        _t: f64,
    ) {
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
    }
}

fn main() {
    run("Hello", 800, 600, Hello {
        bar: Titlebar::new("Hello"),
    }).unwrap();
}
```

See [Renderer.md](Renderer.md) for the shell and [Layout.md](Layout.md)
for the `View` tree.
