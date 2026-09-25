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
| Button | [Button.md](Button.md) | Standard button with CoreIcon SF Symbols |
| Gauge | [Gauge.md](Gauge.md) | Basic, linear, circular and capacity gauges |
| Glass | [Glass.md](Glass.md) | Liquid glass container and backdrop blur |
| Layout | [Layout.md](Layout.md) | VStack, HStack, ZStack, modifiers, `View` |
| Menu | [Menu.md](Menu.md) | Simple dropdown with action rows, picker base |
| Renderer | [Renderer.md](Renderer.md) | Window shell, frame pipeline, backdrop blur |
| Picker | [Picker.md](Picker.md) | Segmented, inline, menu and date pickers |
| Progress | [Progress.md](Progress.md) | Linear progress bar with chase buffer |
| Scrollbar | [Scrollbar.md](Scrollbar.md) | Overlay side bar with fade, drag and page jump |
| Slider | [Slider.md](Slider.md) | Slider with steps, labels, ticks, glass |
| Theme | [Theme.md](Theme.md) | Live dark/light, accent, glass stage |
| Titlebar | [Titlebar.md](Titlebar.md) | Custom decoration bar with drag region |
| Toggle | [Toggle.md](Toggle.md) | Switch, button and checkbox styles |

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
