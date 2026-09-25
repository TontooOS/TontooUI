use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, Rect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::dividers::{DIVIDER_DARK, DIVIDER_FILL};
use super::super::layout::View;
use super::basic::{BasicList, LIST_DIVIDER_H, LIST_FONT_SIZE, LIST_ROW_H, ListRow};
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Expand/collapse time in seconds.
pub const DISCLOSURE_ANIM_SECONDS: f32 = 0.25;
/// Default children indent in logical px.
pub const DISCLOSURE_INDENT: f32 = 20.0;
/// Chevron glyph width/height in logical px.
pub const DISCLOSURE_CHEV_W: f32 = 7.0;
pub const DISCLOSURE_CHEV_H: f32 = 10.0;
/// Chevron stroke width in logical px.
pub const DISCLOSURE_CHEV_STROKE: f32 = 1.8;
/// Left pad before the chevron in logical px.
pub const DISCLOSURE_CHEV_PAD: f32 = 6.0;
/// Gap between chevron and header title in logical px.
pub const DISCLOSURE_CHEV_GAP: f32 = 8.0;
/// Generous chevron hit width in logical px. The glyph is tiny, so
/// the whole leading box toggles.
pub const DISCLOSURE_HIT_W: f32 = 28.0;

/// Expandable group: a header row (chevron plus title) with an
/// indented `BasicList` below it, like the reference (Fruits open
/// with Apple/Banana/Cherry/Date, Vegetables and Grains closed).
/// Only a press plus release on the chevron box toggles the group;
/// clicks on the title or children do nothing.
///
/// Toggling animates: the children height tweens between 0 and full
/// (`DISCLOSURE_ANIM_SECONDS`, `CubicOut`) while the chevron morphs
/// from `>` to `v`, and rows below glide down. Children draw into a
/// clip layer, so partially revealed rows never spill outside the
/// animated bounds.
pub struct DisclosureGroup {
    title: String,
    child: BasicList,
    open: bool,
    progress: f32,
    anim: Option<TweenAnim<f32>>,
    anim_time: f32,
    last_draw: Option<Instant>,
    indent: f32,
    title_color: Color,
    title_manual: bool,
    chevron_color: Color,
    chevron_manual: bool,
    divider_color: Color,
    divider_manual: bool,
    dark: bool,
    focused: bool,
    disabled: bool,
    armed: bool,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl DisclosureGroup {
    pub fn new(title: impl Into<String>, children: Vec<ListRow>) -> Self {
        Self {
            title: title.into(),
            child: BasicList::from_rows(children),
            open: false,
            progress: 0.0,
            anim: None,
            anim_time: 0.0,
            last_draw: None,
            indent: DISCLOSURE_INDENT,
            title_color: Color::WHITE,
            title_manual: false,
            chevron_color: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            chevron_manual: false,
            divider_color: DIVIDER_DARK,
            divider_manual: false,
            dark: true,
            focused: true,
            disabled: false,
            armed: false,
            on_toggle: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Convenience constructor from a slice (e.g. `["Apple", "Banana"]`).
    pub fn from_slice(title: impl Into<String>, children: &[&str]) -> Self {
        Self::new(
            title,
            children.iter().map(|s| ListRow::item(*s)).collect(),
        )
    }

    /// Initial open state (closed by default).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self.progress = if open { 1.0 } else { 0.0 };
        self.anim = None;
        self
    }

    /// Children indent in logical px (clamped to >= 0).
    pub fn indent(mut self, px: f32) -> Self {
        self.indent = px.max(0.0);
        self
    }

    /// Manual header title color: wins over the theme.
    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = color;
        self.title_manual = true;
        self
    }

    /// Manual chevron color: wins over the theme dim text.
    pub fn chevron_color(mut self, color: Color) -> Self {
        self.chevron_color = color;
        self.chevron_manual = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Fires on every user toggle (chevron click) with the new state.
    /// Programmatic `set_open` does not fire.
    pub fn on_toggle(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(callback));
        self
    }

    /// Live theme: header title, chevron dim and dividers. Manually
    /// set colors win over the system ones. Forwards divider and
    /// mode to the children list.
    pub fn set_theme(&mut self, divider: Color, dark: bool) {
        self.dark = dark;
        if !self.title_manual {
            self.title_color = if dark {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }
        if !self.chevron_manual {
            self.chevron_color = if dark {
                Color::from_rgb8(0x9a, 0x9a, 0x9e)
            } else {
                Color::from_rgb8(0x6e, 0x6e, 0x72)
            };
        }
        if !self.divider_manual {
            self.divider_color = divider;
        }
        self.child.set_theme(divider, dark);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.child.set_focused(focused);
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_children(&mut self, children: Vec<ListRow>) {
        self.child.set_list_rows(children);
    }

    pub fn children(&self) -> &[ListRow] {
        self.child.rows()
    }

    pub fn push_child(&mut self, row: ListRow) {
        self.child.push_list_row(row);
    }

    /// Advanced access to the children list (row styles, badges,
    /// manual colors).
    pub fn list_mut(&mut self) -> &mut BasicList {
        &mut self.child
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Animated progress: 0.0 closed, 1.0 open, between mid-tween.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Open or close with animation. No callback (use `toggle` for
    /// the user path or call `on_toggle` yourself).
    pub fn set_open(&mut self, open: bool) {
        if self.open == open {
            return;
        }
        self.open = open;
        let target = if open { 1.0 } else { 0.0 };
        self.anim = Some(TweenAnim::new(
            Tween::new(self.progress, target, DISCLOSURE_ANIM_SECONDS)
                .easing(Easing::CubicOut),
        ));
        self.anim_time = 0.0;
    }

    /// User toggle: flips with animation and fires `on_toggle`.
    pub fn toggle(&mut self) {
        if self.disabled {
            return;
        }
        let open = !self.open;
        self.set_open(open);
        if let Some(callback) = self.on_toggle.as_mut() {
            callback(open);
        }
    }

    /// Press starts a toggle only inside the chevron box.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        self.armed = self.chevron_hit(x as f32, y as f32);
    }

    pub fn mouse_move(&mut self, _x: f64, _y: f64) {}

    /// Release on the chevron box after pressing there toggles.
    pub fn mouse_up(&mut self, x: f64, y: f64) {
        if self.disabled {
            self.armed = false;
            return;
        }
        if self.armed && self.chevron_hit(x as f32, y as f32) {
            self.toggle();
        }
        self.armed = false;
    }

    fn chevron_hit(&self, x: f32, y: f32) -> bool {
        x >= self.x
            && x <= self.x + DISCLOSURE_HIT_W
            && y >= self.y
            && y <= self.y + LIST_ROW_H
    }

    /// Children content height in logical px.
    fn children_height(&self) -> f32 {
        self.child.content_height()
    }

    fn advance(&mut self, now: Instant) {
        let dt = match self.last_draw {
            Some(last) => now.saturating_duration_since(last).as_secs_f32().min(0.5),
            None => 0.0,
        };
        self.last_draw = Some(now);
        if let Some(mut anim) = self.anim.take() {
            self.anim_time += dt;
            let done = anim.update(self.anim_time);
            self.progress = *anim.value();
            if !done {
                self.anim = Some(anim);
            }
        } else {
            self.progress = if self.open { 1.0 } else { 0.0 };
        }
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }

    fn draw_chevron(
        &self,
        scene: &mut Scene,
        cx: f32,
        cy: f32,
        scale: f32,
        color: Color,
    ) {
        // Morph from ">" (progress 0) to "v" (progress 1) by
        // interpolating the three stroke points.
        let p = self.progress.clamp(0.0, 1.0);
        let hw = DISCLOSURE_CHEV_W / 2.0;
        let hh = DISCLOSURE_CHEV_H / 2.0;
        let lerp = |a: f32, b: f32| a + (b - a) * p;
        let px = |v: f32| v as f64 * scale as f64;
        let mut path = BezPath::new();
        path.move_to((px(lerp(cx - hw, cx - hw)), px(lerp(cy - hh, cy - hh))));
        path.line_to((px(lerp(cx + hw, cx)), px(lerp(cy, cy + hh))));
        path.line_to((px(lerp(cx - hw, cx + hw)), px(lerp(cy + hh, cy - hh))));
        let mut stroke = Stroke::new(DISCLOSURE_CHEV_STROKE as f64 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }
}

impl Default for DisclosureGroup {
    fn default() -> Self {
        Self::new(String::new(), Vec::new())
    }
}

impl View for DisclosureGroup {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        let revealed = self.progress * (self.children_height() + LIST_DIVIDER_H);
        (
            DIVIDER_FILL,
            LIST_ROW_H + LIST_DIVIDER_H + revealed,
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        let cw = (w - self.indent).max(0.0);
        self.child.place(
            fonts,
            x + self.indent,
            y + LIST_ROW_H + LIST_DIVIDER_H,
            cw,
            self.children_height(),
        );
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.advance(Instant::now());
        if self.width <= 0.0 {
            return;
        }
        let scale = fonts.scale;
        let px = |v: f32| v as f64 * scale as f64;

        // Header chevron plus title.
        let cy = self.y + LIST_ROW_H / 2.0;
        let chev_cx = self.x + DISCLOSURE_CHEV_PAD + DISCLOSURE_CHEV_W / 2.0;
        self.draw_chevron(scene, chev_cx, cy, scale, self.eff(self.chevron_color));
        let title_x =
            self.x + DISCLOSURE_CHEV_PAD + DISCLOSURE_CHEV_W + DISCLOSURE_CHEV_GAP;
        let layout = fonts.layout_text(
            &self.title.clone(),
            LIST_FONT_SIZE,
            self.eff(self.title_color),
            Some((self.width - (title_x - self.x)).max(0.0)),
        );
        let (_, th) = FontSystem::layout_size(&layout);
        draw_layout(
            scene,
            &layout,
            title_x,
            self.y + (LIST_ROW_H - th / scale) / 2.0,
            scale,
        );

        // Full-bleed hairline below the header.
        let hy = self.y + LIST_ROW_H;
        let head_line = Rect::new(
            px(self.x),
            px(hy),
            px(self.x + self.width),
            px(hy + LIST_DIVIDER_H),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.divider_color)),
            None,
            &head_line,
        );

        // Children reveal into a clip layer, then the closing divider
        // glides with the animated height.
        let revealed = self.progress * self.children_height();
        if revealed > 0.5 && self.children_height() > 0.0 {
            let clip = Rect::new(
                px(self.x),
                px(hy + LIST_DIVIDER_H),
                px(self.x + self.width),
                px(hy + LIST_DIVIDER_H + revealed),
            );
            scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
            self.child.draw(scene, fonts, images);
            scene.pop_layer();
        }
        let fy = hy + LIST_DIVIDER_H + revealed;
        let foot_line = Rect::new(
            px(self.x),
            px(fy),
            px(self.x + self.width),
            px(fy + LIST_DIVIDER_H),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.divider_color)),
            None,
            &foot_line,
        );
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.mouse_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    fn group() -> DisclosureGroup {
        DisclosureGroup::from_slice("Fruits", &["Apple", "Banana"])
    }

    /// Drive the animation to rest at 60 fps steps.
    fn settled(mut group: DisclosureGroup, open: bool) -> DisclosureGroup {
        group.set_open(open);
        let t0 = Instant::now();
        group.advance(t0);
        for i in 1..=60 {
            group.advance(t0 + Duration::from_secs_f32(i as f32 / 60.0));
            if group.anim.is_none() {
                break;
            }
        }
        group
    }

    #[test]
    fn closed_by_default_open_builder_snaps() {
        let closed = group();
        assert!(!closed.is_open());
        assert_eq!(closed.progress(), 0.0);
        let open = DisclosureGroup::from_slice("F", &["A"]).open(true);
        assert!(open.is_open());
        assert_eq!(open.progress(), 1.0);
    }

    #[test]
    fn animation_reaches_target() {
        let group = settled(group(), true);
        assert!(group.anim.is_none());
        assert_eq!(group.progress(), 1.0);
        let group = settled(group, false);
        assert_eq!(group.progress(), 0.0);
    }

    #[test]
    fn measured_height_follows_progress() {
        let mut fonts = FontSystem::new();
        let mut closed = group();
        let (_, h0) = closed.measure(&mut fonts);
        assert_eq!(h0, LIST_ROW_H + LIST_DIVIDER_H);
        let mut open = settled(group(), true);
        let (_, h1) = open.measure(&mut fonts);
        assert_eq!(
            h1,
            LIST_ROW_H + LIST_DIVIDER_H + open.children_height() + LIST_DIVIDER_H
        );
        assert!(h1 > h0);
    }

    #[test]
    fn only_chevron_clicks_toggle() {
        let mut group = group();
        group.place(&mut FontSystem::new(), 0.0, 0.0, 400.0, 100.0);
        // Title click: armed never set, no toggle.
        group.mouse_down(200.0, 10.0);
        group.mouse_up(200.0, 10.0);
        assert!(!group.is_open());
        // Press chevron, release on title: no toggle.
        group.mouse_down(4.0, 10.0);
        group.mouse_up(200.0, 10.0);
        assert!(!group.is_open());
        // Press plus release on the chevron: toggles.
        group.mouse_down(4.0, 10.0);
        group.mouse_up(4.0, 10.0);
        assert!(group.is_open());
        // Disabled: chevron click does nothing.
        let mut disabled = DisclosureGroup::from_slice("G", &["A"]).disabled(true);
        disabled.place(&mut FontSystem::new(), 0.0, 0.0, 400.0, 100.0);
        disabled.mouse_down(4.0, 10.0);
        disabled.mouse_up(4.0, 10.0);
        assert!(!disabled.is_open());
    }

    #[test]
    fn toggle_fires_callback_set_open_does_not() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut group = group().on_toggle(move |_| count.set(count.get() + 1));
        group.toggle();
        assert!(group.is_open());
        assert_eq!(fires.get(), 1);
        group.set_open(false);
        assert!(!group.is_open());
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn children_forwarding() {
        let mut group = group();
        group.push_child(ListRow::item("Cherry"));
        assert_eq!(group.children().len(), 3);
        group.set_children(vec![ListRow::item("Date")]);
        assert_eq!(group.children()[0].text(), "Date");
    }
}
