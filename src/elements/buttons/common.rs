//! Shared button types, the iOS system palette and render helpers.
//!
//! The enums mirror the SwiftUI API surface 1:1 (`SwiftUI.ButtonRole.Role`,
//! `SwiftUI.ButtonSizing.Value`, `SwiftUI.ButtonBorderShape.Guts` and the
//! button style hierarchy `PlainButtonStyle` / `BorderedButtonStyle_Mac` /
//! `GlassButtonStyle` / `GlassProminentButtonStyle`).

use std::sync::Arc;

use gtk::prelude::*;
use gtk::{Button as GtkButton, Image as GtkImage, Label as GtkLabel};

use uikit::style::Color;

pub(crate) const BLUE_DARK: &str = "#0A84FF";
pub(crate) const BLUE_LIGHT: &str = "#007AFF";
pub(crate) const RED_DARK: &str = "#FF453A";
pub(crate) const RED_LIGHT: &str = "#FF3B30";

/// The purpose of a button, mirroring `SwiftUI.ButtonRole.Role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonRole {
    /// A button that destroys something — tinted red.
    Destructive,
    /// A button used to cancel an operation — default label "Cancel".
    Cancel,
    /// A button that confirms an operation — default label "Done".
    Confirm,
    /// A button that closes a window or sheet — default label "Close".
    Close,
}

impl ButtonRole {
    /// The system default label for a role, as used when the button is
    /// created with an empty label (matches the reference behavior).
    pub fn default_label(self) -> &'static str {
        match self {
            ButtonRole::Destructive => "Delete",
            ButtonRole::Cancel => "Cancel",
            ButtonRole::Confirm => "Done",
            ButtonRole::Close => "Close",
        }
    }

    pub(crate) fn is_destructive(self) -> bool {
        self == ButtonRole::Destructive
    }
}

/// The visual style of a button, mirroring the SwiftUI style hierarchy
/// (`PlainButtonStyle`, `BorderedButtonStyle_Mac`, `GlassButtonStyle`,
/// `GlassProminentButtonStyle`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// No background, tinted label only.
    Plain,
    /// Tinted translucent background with tinted label.
    Bordered,
    /// Solid tinted background with white label.
    BorderedProminent,
    /// Neutral glass background with tinted label.
    Glass,
    /// Solid tinted glass background with white label.
    GlassProminent,
    /// Resolves to the platform default (glass bordered).
    Automatic,
}

/// The shape of the button border, mirroring `SwiftUI.ButtonBorderShape.Guts`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonBorderShape {
    /// Platform default (capsule).
    Automatic,
    /// Fully rounded pill shape.
    Capsule,
    /// Rounded rectangle with the given corner radius.
    RoundedRectangle(f32),
    /// Perfect circle for icon-only buttons. Falls back to capsule when a
    /// label is present (a circle cannot fit text).
    Circle,
}

/// How a button sizes itself, mirroring `SwiftUI.ButtonSizing.Value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSizing {
    /// Hug the content.
    Fitted,
    /// Expand to fill the available width.
    Flexible,
    /// Platform default (same as fitted).
    Automatic,
}

pub(crate) fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let h = hex.trim_start_matches('#');
    let n = u32::from_str_radix(h, 16).unwrap_or(0);
    (
        ((n >> 16) & 0xFF) as f32 / 255.0,
        ((n >> 8) & 0xFF) as f32 / 255.0,
        (n & 0xFF) as f32 / 255.0,
    )
}

/// Resolved colors for one render pass.
pub(crate) struct ButtonColors {
    pub label: String,
    pub background: String,
    pub border: String,
    pub icon: (u8, u8, u8),
}

pub(crate) fn resolve_colors(style: ButtonStyle, tint_hex: &str, dark: bool) -> ButtonColors {
    let (tr, tg, tb) = hex_to_rgb(tint_hex);
    match style {
        ButtonStyle::Plain => ButtonColors {
            label: tint_hex.into(),
            background: "rgba(0,0,0,0)".into(),
            border: "none".into(),
            icon: ((tr * 255.0) as u8, (tg * 255.0) as u8, (tb * 255.0) as u8),
        },
        ButtonStyle::Bordered => {
            let alpha = if dark { 0.24 } else { 0.14 };
            ButtonColors {
                label: tint_hex.into(),
                background: format!(
                    "rgba({},{},{},{})",
                    (tr * 255.0) as u8,
                    (tg * 255.0) as u8,
                    (tb * 255.0) as u8,
                    alpha
                ),
                border: format!(
                    "1px solid rgba({},{},{},{})",
                    (tr * 255.0) as u8,
                    (tg * 255.0) as u8,
                    (tb * 255.0) as u8,
                    alpha + 0.15
                ),
                icon: ((tr * 255.0) as u8, (tg * 255.0) as u8, (tb * 255.0) as u8),
            }
        }
        ButtonStyle::BorderedProminent | ButtonStyle::GlassProminent => ButtonColors {
            label: "#FFFFFF".into(),
            background: tint_hex.into(),
            border: "none".into(),
            icon: (255, 255, 255),
        },
        ButtonStyle::Glass | ButtonStyle::Automatic => {
            let (bg, bd) = if dark {
                (
                    "rgba(255,255,255,0.14)",
                    "1px solid rgba(255,255,255,0.18)",
                )
            } else {
                ("rgba(0,0,0,0.06)", "1px solid rgba(0,0,0,0.12)")
            };
            ButtonColors {
                label: tint_hex.into(),
                background: bg.into(),
                border: bd.into(),
                icon: ((tr * 255.0) as u8, (tg * 255.0) as u8, (tb * 255.0) as u8),
            }
        }
    }
}

/// Effective tint hex for a button (role destructive maps to system red).
pub(crate) fn effective_tint_hex(
    role: Option<ButtonRole>,
    tint: Option<&Color>,
    dark: bool,
) -> String {
    if let Some(c) = tint {
        return c.to_hex();
    }
    if role.map(|r| r.is_destructive()).unwrap_or(false) {
        return if dark { RED_DARK } else { RED_LIGHT }.into();
    }
    if dark {
        BLUE_DARK
    } else {
        BLUE_LIGHT
    }
    .into()
}

/// Load an SF Symbol asset from CoreIcon and recolor it to `color` using its
/// alpha channel as a mask. Cached in the temp dir per (symbol, color).
#[cfg(feature = "coreicon")]
pub(crate) fn sf_icon_path(symbol: &str, color: (u8, u8, u8)) -> Option<String> {
    let coreicon_assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .join("CoreIcon/assets/icons");
    if !coreicon_assets.exists() {
        return None;
    }
    let src = coreicon_assets.join(symbol).with_extension("png");
    if !src.exists() {
        return None;
    }

    let key = format!(
        "btn_{}_{:02x}{:02x}{:02x}.png",
        symbol.replace('.', "_"),
        color.0,
        color.1,
        color.2
    );
    let out = std::env::temp_dir().join(key);
    if out.exists() {
        return Some(out.to_str()?.to_string());
    }

    let img = image::open(&src).ok()?;
    let mut rgba = img.to_rgba8();
    for p in rgba.pixels_mut() {
        if p[3] > 0 {
            p[0] = color.0;
            p[1] = color.1;
            p[2] = color.2;
        }
    }
    rgba.save(&out).ok()?;
    Some(out.to_str()?.to_string())
}

pub(crate) fn border_radius(shape: ButtonBorderShape) -> String {
    match shape {
        ButtonBorderShape::Capsule | ButtonBorderShape::Automatic => "9999px".into(),
        ButtonBorderShape::RoundedRectangle(r) => format!("{}px", r),
        ButtonBorderShape::Circle => "50%".into(),
    }
}

/// Resolve the shape used for rendering/sizing: `Circle` only applies to
/// icon-only buttons — with a label it degrades to `Capsule` so text buttons
/// render as normal pills instead of stretched ellipses.
pub(crate) fn effective_shape(shape: ButtonBorderShape, has_label: bool) -> ButtonBorderShape {
    match shape {
        ButtonBorderShape::Circle if has_label => ButtonBorderShape::Capsule,
        other => other,
    }
}

/// Shared GtkButton renderer used by [`crate::elements::buttons::Button`].
pub(crate) fn render_button_widget(
    label: &str,
    icon: &Option<String>,
    icon_size: f32,
    colors: &ButtonColors,
    shape: ButtonBorderShape,
    sizing: ButtonSizing,
    width: Option<f32>,
    height: Option<f32>,
    on_click: &Option<Arc<dyn Fn() + Send + Sync>>,
) -> gtk::Widget {
    let btn = GtkButton::new();

    if !label.is_empty() || icon.is_none() {
        let lbl = GtkLabel::new(Some(label));
        btn.set_child(Some(&lbl));
    }

    #[cfg(feature = "coreicon")]
    if let Some(symbol) = icon {
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        content.set_halign(gtk::Align::Center);
        content.set_valign(gtk::Align::Center);
        if let Some(path) = sf_icon_path(symbol, colors.icon) {
            let img = GtkImage::from_file(&path);
            img.set_pixel_size(icon_size.max(1.0) as i32);
            content.append(&img);
        }
        if !label.is_empty() {
            content.append(&GtkLabel::new(Some(label)));
        }
        btn.set_child(Some(&content));
    }

    let shape = effective_shape(shape, !label.is_empty());
    let radius = border_radius(shape);
    let circle = shape == ButtonBorderShape::Circle;
    let side = height.unwrap_or(38.0).max(width.unwrap_or(0.0));
    let min_w = width.unwrap_or(if circle { side } else { 0.0 });
    let min_h = height.unwrap_or(if circle { side } else { 32.0 });

    let css = format!(
        "button {{
            background: {bg};
            border: {border};
            border-radius: {radius};
            color: {fg};
            font-family: 'SF Pro Text';
            font-size: 15px;
            font-weight: 500;
            padding: {vp}px {hp}px;
            min-height: {mh}px;
            min-width: {mw}px;
        }}
        button:hover {{
            filter: brightness(1.15);
        }}
        button:active {{
            filter: brightness(0.85);
        }}
        button label {{
            color: {fg};
        }}",
        bg = colors.background,
        border = colors.border,
        radius = radius,
        fg = colors.label,
        vp = if circle { 0 } else { 6 },
        hp = if circle { 0 } else { 14 },
        mh = min_h,
        mw = min_w,
    );
    uikit::widget::apply_css(&btn, &css);

    if sizing == ButtonSizing::Flexible {
        btn.set_hexpand(true);
        btn.set_halign(gtk::Align::Fill);
    }

    if let Some(handler) = on_click {
        let handler = handler.clone();
        btn.connect_clicked(move |_| handler());
    }

    btn.upcast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_default_labels() {
        assert_eq!(ButtonRole::Cancel.default_label(), "Cancel");
        assert_eq!(ButtonRole::Close.default_label(), "Close");
        assert_eq!(ButtonRole::Confirm.default_label(), "Done");
        assert_eq!(ButtonRole::Destructive.default_label(), "Delete");
    }

    #[test]
    fn resolve_colors_shapes() {
        let c = resolve_colors(ButtonStyle::BorderedProminent, BLUE_DARK, true);
        assert_eq!(c.background, BLUE_DARK);
        assert_eq!(c.label, "#FFFFFF");
        let p = resolve_colors(ButtonStyle::Plain, BLUE_DARK, true);
        assert_eq!(p.background, "rgba(0,0,0,0)");
        assert_eq!(p.label, BLUE_DARK);
    }

    #[test]
    fn effective_tint_maps_destructive_to_red() {        assert_eq!(effective_tint_hex(Some(ButtonRole::Destructive), None, true), RED_DARK);
        assert_eq!(effective_tint_hex(Some(ButtonRole::Cancel), None, true), BLUE_DARK);
        assert_eq!(effective_tint_hex(None, None, false), BLUE_LIGHT);
        let green = Color::from_rgb(48, 209, 88);
        assert_eq!(effective_tint_hex(None, Some(&green), true), "#30d158");
    }

    #[test]
    fn circle_falls_back_to_capsule_with_label() {
        assert_eq!(
            effective_shape(ButtonBorderShape::Circle, true),
            ButtonBorderShape::Capsule
        );
        assert_eq!(
            effective_shape(ButtonBorderShape::Circle, false),
            ButtonBorderShape::Circle
        );
        assert_eq!(
            effective_shape(ButtonBorderShape::Capsule, true),
            ButtonBorderShape::Capsule
        );
    }
}
