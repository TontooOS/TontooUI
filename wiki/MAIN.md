# TontooUI – Wiki

TontooUI is a SwiftUI-inspired declarative UI layer for TontooOS. It is currently being rewritten from scratch on wgpu + Vello, with no UIKit layer and no GTK.

- Repository: tontoo-os/TontooLibs/TontooUI
- License: TCL
- Version: 26.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Wiki design system and conventions |

## Status

TontooUI was stripped to an empty crate as the starting point for a full rewrite:

- No UIKit layer. All UI is built directly in TontooUI.
- No GTK. The renderer moves to wgpu + Vello for real backdrop-blur
  LiquidGlass, GPU animation and full control over text and color.
- No public API exists yet. Feature pages return as features land, one page
  per feature, following [RULE.md](RULE.md).

## Quick Start

No working example exists yet. The crate has no sources and no dependencies
while the renderer foundation is built.

## Target Architecture

```
tontooui (single crate, no UIKit layer)
 |
 +-- renderer (wgpu + Vello: GPU compositing, backdrop-blur glass)
 +-- layout   (constraint / stack layout, no GTK dependency)
 +-- text     (native text rendering, SF Pro)
 +-- animation (spring physics, GPU-friendly ticks)
```

> **Note:** This is the rewrite target. None of it is implemented yet.

## Cross References

- [RULE.md](RULE.md) – wiki design system and conventions

## Changelog

- 2026-09-21: Stripped all Rust sources (`src/`, `examples/`) and removed the
  UIKit/GTK dependencies from `Cargo.toml` for the wgpu + Vello rewrite.
  Deleted all 38 feature pages documenting the removed code; this page is now
  a stub until features land again.