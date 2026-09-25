use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, Rect, RoundedRect, Stroke};
use vello::peniko::{BlendMode, Brush, Color, Fill};

use super::super::images::SFSymbolImage;
use super::super::layout::View;
use crate::animation::{Easing, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Row height in logical px.
pub const OUTLINE_ROW_H: f32 = 32.0;
/// Indent per tree level in logical px.
pub const OUTLINE_INDENT: f32 = 26.0;
/// Reserved chevron slot per row in logical px (files leave it empty
/// so icons align with folder icons of the same level).
pub const OUTLINE_CHEV_SLOT: f32 = 22.0;
/// Chevron glyph width/height in logical px.
pub const OUTLINE_CHEV_W: f32 = 7.0;
pub const OUTLINE_CHEV_H: f32 = 10.0;
/// Chevron stroke width in logical px.
pub const OUTLINE_CHEV_STROKE: f32 = 1.8;
/// Label size in logical px.
pub const OUTLINE_LABEL_SIZE: f32 = 15.0;
/// Row icon box in logical px.
pub const OUTLINE_ICON_SIZE: f32 = 20.0;
/// Gap between icon and label in logical px.
pub const OUTLINE_ICON_GAP: f32 = 8.0;
/// Expand/collapse time in seconds (height plus fade).
pub const OUTLINE_ANIM_SECONDS: f32 = 0.25;
/// Selection fill alpha (0-1 of the theme accent).
pub const OUTLINE_SELECTED_ALPHA: f32 = 0.35;
/// Default icon tint (theme blue, like the reference folders).
pub const OUTLINE_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Row icon: folders and files resolve to SF Symbols, custom names
/// pass through verbatim (e.g. `"tag"`, `"globe"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutlineIcon {
    Folder,
    File,
    Symbol(String),
}

impl OutlineIcon {
    fn symbol(&self) -> &str {
        match self {
            OutlineIcon::Folder => "folder.fill",
            OutlineIcon::File => "doc",
            OutlineIcon::Symbol(name) => name,
        }
    }
}

/// One tree node: a folder (expandable while it has children) or a
/// file (leaf). Nodes nest arbitrarily deep, like the reference file
/// tree.
#[derive(Debug)]
pub struct OutlineNode {
    label: String,
    icon: OutlineIcon,
    children: Vec<OutlineNode>,
    expanded: bool,
    start_open: bool,
    progress: f32,
    anim: Option<TweenAnim<f32>>,
    anim_time: f32,
}

impl OutlineNode {
    /// Folder node, collapsed by default.
    pub fn folder(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: OutlineIcon::Folder,
            children: Vec::new(),
            expanded: false,
            start_open: false,
            progress: 0.0,
            anim: None,
            anim_time: 0.0,
        }
    }

    /// File node (leaf, never expandable).
    pub fn file(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: OutlineIcon::File,
            children: Vec::new(),
            expanded: false,
            start_open: false,
            progress: 0.0,
            anim: None,
            anim_time: 0.0,
        }
    }

    /// Custom SF Symbol for this node (e.g. `"tag"`). Custom icons
    /// still expand when the node has children.
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = OutlineIcon::Symbol(name.into());
        self
    }

    /// Initial expanded state (collapsed by default, snapped without
    /// animation; childless nodes never expand). Builder order does
    /// not matter: children added later still open the node.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.start_open = expanded;
        self.apply_start_open();
        self
    }

    /// Append one child (makes the node a folder row).
    pub fn child(mut self, node: OutlineNode) -> Self {
        self.children.push(node);
        self.apply_start_open();
        self
    }

    /// Append several children at once.
    pub fn children(mut self, nodes: Vec<OutlineNode>) -> Self {
        self.children.extend(nodes);
        self.apply_start_open();
        self
    }

    fn apply_start_open(&mut self) {
        self.expanded = self.start_open && !self.children.is_empty();
        self.progress = if self.expanded { 1.0 } else { 0.0 };
        self.anim = None;
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// True for folder and custom icons (files never expand).
    pub fn is_folder(&self) -> bool {
        !matches!(self.icon, OutlineIcon::File)
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Animated progress: 0.0 closed, 1.0 open, between mid-tween.
    /// Children fade and reveal with this value.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn child_nodes(&self) -> &[OutlineNode] {
        &self.children
    }

    fn advance(&mut self, dt: f32) {
        if let Some(mut anim) = self.anim.take() {
            self.anim_time += dt;
            if anim.update(self.anim_time) {
                self.progress = *anim.value();
            } else {
                self.progress = *anim.value();
                self.anim = Some(anim);
            }
        } else {
            self.progress = if self.expanded { 1.0 } else { 0.0 };
        }
        for child in &mut self.children {
            child.advance(dt);
        }
    }

    fn set_expanded_animated(&mut self, expanded: bool) {
        let expanded = expanded && self.has_children();
        if self.expanded == expanded {
            return;
        }
        self.expanded = expanded;
        let target = if expanded { 1.0 } else { 0.0 };
        self.anim = Some(TweenAnim::new(
            Tween::new(self.progress, target, OUTLINE_ANIM_SECONDS).easing(Easing::CubicOut),
        ));
        self.anim_time = 0.0;
    }

    /// Visible height: the row plus the revealed children block.
    /// Rows below glide as this animates.
    fn visible_h(&self) -> f32 {
        let mut children_h = 0.0;
        for child in &self.children {
            children_h += child.visible_h();
        }
        OUTLINE_ROW_H + self.progress.clamp(0.0, 1.0) * children_h
    }

    /// Full height with every descendant open (clip size for fading).
    fn full_h(&self) -> f32 {
        let mut h = OUTLINE_ROW_H;
        for child in &self.children {
            h += child.full_h();
        }
        h
    }
}

fn set_all(node: &mut OutlineNode, expanded: bool) {
    node.expanded = expanded && node.has_children();
    node.progress = if node.expanded { 1.0 } else { 0.0 };
    node.anim = None;
    for child in &mut node.children {
        set_all(child, expanded);
    }
}

struct FlatRow {
    path: Vec<usize>,
    y: f32,
}

/// Basic outline group: a file-tree of `OutlineNode`s with chevron
/// folders, tinted icons, single-select rows and animated
/// expand/collapse. Toggling tweens the children height
/// (`OUTLINE_ANIM_SECONDS`, `CubicOut`) while the block fades
/// in/out through a clip layer and the chevron morphs from `>` to
/// `v`; rows below glide down. Node paths are index vectors from
/// the roots (e.g. `vec![0, 1]` is the second child of the first
/// root).
pub struct BasicOutlineGroup {
    roots: Vec<OutlineNode>,
    selected: Option<Vec<usize>>,
    selectable: bool,
    on_toggle: Option<Box<dyn FnMut(Vec<usize>, bool)>>,
    on_select: Option<Box<dyn FnMut(Vec<usize>)>>,
    accent: Color,
    icon_color: Color,
    icon_manual: bool,
    dark: bool,
    focused: bool,
    disabled: bool,
    last_draw: Option<Instant>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rows: Vec<FlatRow>,
}

impl BasicOutlineGroup {
    pub fn new(roots: Vec<OutlineNode>) -> Self {
        Self {
            roots,
            selected: None,
            selectable: true,
            on_toggle: None,
            on_select: None,
            accent: OUTLINE_ACCENT,
            icon_color: OUTLINE_ACCENT,
            icon_manual: false,
            dark: true,
            focused: true,
            disabled: false,
            last_draw: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rows: Vec::new(),
        }
    }

    /// Row clicks select (default `true`). Folders still toggle when
    /// selection is off.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        if !selectable {
            self.selected = None;
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Manual icon tint: wins over the theme accent until cleared.
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = color;
        self.icon_manual = true;
        self
    }

    /// Fires with `(path, expanded)` on every user toggle.
    /// Programmatic `set_expanded` does not fire.
    pub fn on_toggle(mut self, callback: impl FnMut(Vec<usize>, bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(callback));
        self
    }

    /// Fires with the newly selected path on every selection change
    /// (empty when cleared).
    pub fn on_select(mut self, callback: impl FnMut(Vec<usize>) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Live theme: icon tint (accent unless set manually), label and
    /// chevron grays.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        if !self.icon_manual {
            self.icon_color = accent;
        }
        self.dark = dark;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Selected path, if any.
    pub fn selected_path(&self) -> Option<Vec<usize>> {
        self.selected.clone()
    }

    /// Select programmatically (fires `on_select` on change).
    /// Returns false for unknown paths.
    pub fn select(&mut self, path: Vec<usize>) -> bool {
        if self.node(&path).is_none() {
            return false;
        }
        if self.selected != Some(path.clone()) {
            self.selected = Some(path.clone());
            self.notify_select(path);
        }
        true
    }

    pub fn clear_selection(&mut self) {
        if self.selected.is_some() {
            self.selected = None;
            self.notify_select(Vec::new());
        }
    }

    /// Read one node by path, if it exists.
    pub fn node(&self, path: &[usize]) -> Option<&OutlineNode> {
        let mut level = self.roots.as_slice();
        let mut current = None;
        for &index in path {
            current = level.get(index);
            level = current?.children.as_slice();
        }
        current
    }

    fn node_mut(&mut self, path: &[usize]) -> Option<&mut OutlineNode> {
        fn descend<'a>(level: &'a mut [OutlineNode], path: &[usize]) -> Option<&'a mut OutlineNode> {
            let (head, tail) = path.split_first()?;
            let node = level.get_mut(*head)?;
            if tail.is_empty() {
                Some(node)
            } else {
                descend(node.children.as_mut_slice(), tail)
            }
        }
        descend(self.roots.as_mut_slice(), path)
    }

    /// User toggle at a path: flips with animation and fires
    /// `on_toggle`. Files and childless folders do nothing.
    /// Returns false for unknown or untoggleable paths.
    pub fn toggle(&mut self, path: Vec<usize>) -> bool {
        if self.disabled {
            return false;
        }
        let Some(node) = self.node_mut(&path) else {
            return false;
        };
        if !node.has_children() {
            return false;
        }
        let open = !node.expanded;
        node.set_expanded_animated(open);
        self.notify_toggle(path, open);
        true
    }

    /// Open or close with animation. No callback (use `toggle` for
    /// the user path).
    pub fn set_expanded(&mut self, path: &[usize], expanded: bool) -> bool {
        let Some(node) = self.node_mut(path) else {
            return false;
        };
        if !node.has_children() {
            return false;
        }
        node.set_expanded_animated(expanded);
        true
    }

    pub fn is_expanded(&self, path: &[usize]) -> bool {
        self.node(path).is_some_and(|node| node.expanded)
    }

    /// Open everything instantly (no animation, no callbacks).
    pub fn expand_all(&mut self) {
        for root in &mut self.roots {
            set_all(root, true);
        }
    }

    /// Close everything instantly (no animation, no callbacks).
    pub fn collapse_all(&mut self) {
        for root in &mut self.roots {
            set_all(root, false);
        }
    }

    /// Full content height in logical px (animated: rows below glide
    /// while blocks fade).
    pub fn content_height(&self) -> f32 {
        self.roots.iter().map(OutlineNode::visible_h).sum()
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        self.layout();
        let (x, y) = (x as f32, y as f32);
        if x < self.x || x > self.x + self.width {
            return;
        }
        let hit = self
            .rows
            .iter()
            .find(|row| y >= row.y && y < row.y + OUTLINE_ROW_H)
            .map(|row| row.path.clone());
        let Some(path) = hit else {
            return;
        };
        if self.selectable && self.selected != Some(path.clone()) {
            self.selected = Some(path.clone());
            self.notify_select(path.clone());
        }
        if self.node(&path).is_some_and(|node| node.has_children()) {
            self.toggle(path);
        }
    }

    pub fn mouse_up(&mut self, _x: f64, _y: f64) {}

    fn notify_select(&mut self, path: Vec<usize>) {
        if let Some(callback) = self.on_select.as_mut() {
            callback(path);
        }
    }

    fn notify_toggle(&mut self, path: Vec<usize>, open: bool) {
        if let Some(callback) = self.on_toggle.as_mut() {
            callback(path, open);
        }
    }

    fn advance(&mut self, now: Instant) {
        let dt = match self.last_draw {
            Some(last) => now.saturating_duration_since(last).as_secs_f32().min(0.5),
            None => 0.0,
        };
        self.last_draw = Some(now);
        for root in &mut self.roots {
            root.advance(dt);
        }
    }

    /// Flatten visible rows at full layout (children blocks clip and
    /// fade at draw time; the group height glides instead).
    fn layout(&mut self) {
        self.rows.clear();
        let mut y = self.y;
        for index in 0..self.roots.len() {
            y = self.layout_subtree(vec![index], y);
        }
    }

    fn layout_subtree(&mut self, path: Vec<usize>, y: f32) -> f32 {
        let Some(count) = self.node(&path).map(|node| node.children.len()) else {
            return y;
        };
        let progress = self.node(&path).map(|node| node.progress).unwrap_or(0.0);
        self.rows.push(FlatRow { path: path.clone(), y });
        let mut y = y + OUTLINE_ROW_H;
        if progress > 0.0 {
            for i in 0..count {
                let mut child_path = path.clone();
                child_path.push(i);
                y = self.layout_subtree(child_path, y);
            }
        }
        y
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }

    fn text_color(&self) -> Color {
        if self.dark {
            Color::from_rgb8(0xd8, 0xd9, 0xd9)
        } else {
            Color::from_rgb8(0x27, 0x27, 0x27)
        }
    }

    fn chevron_color(&self) -> Color {
        if self.dark {
            Color::from_rgb8(0x9a, 0x9a, 0x9e)
        } else {
            Color::from_rgb8(0x6e, 0x6e, 0x72)
        }
    }

    fn draw_chevron(
        &self,
        scene: &mut Scene,
        cx: f32,
        cy: f32,
        progress: f32,
        scale: f32,
        color: Color,
    ) {
        // Morph from ">" (progress 0) to "v" (progress 1) by
        // interpolating the three stroke points.
        let p = progress.clamp(0.0, 1.0);
        let hw = OUTLINE_CHEV_W / 2.0;
        let hh = OUTLINE_CHEV_H / 2.0;
        let lerp = |a: f32, b: f32| a + (b - a) * p;
        let px = |v: f32| v as f64 * scale as f64;
        let mut path = BezPath::new();
        path.move_to((px(lerp(cx - hw, cx - hw)), px(lerp(cy - hh, cy - hh))));
        path.line_to((px(lerp(cx + hw, cx)), px(lerp(cy, cy + hh))));
        path.line_to((px(lerp(cx - hw, cx + hw)), px(lerp(cy + hh, cy - hh))));
        let mut stroke = Stroke::new(OUTLINE_CHEV_STROKE as f64 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }

    /// Row snapshot without holding a borrow: drawing needs `&mut
    /// self` for layouts and images while recursing by path.
    fn row_info(&self, path: &[usize]) -> Option<RowInfo> {
        let node = self.node(path)?;
        Some(RowInfo {
            label: node.label.clone(),
            icon: node.icon.clone(),
            progress: node.progress,
            has_children: node.has_children(),
            child_count: node.children.len(),
            full_h: node.full_h() - OUTLINE_ROW_H,
        })
    }

    fn render(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        self.layout();
        if self.width <= 0.0 || self.height <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let text = self.eff(self.text_color());
        let chevron = self.eff(self.chevron_color());
        let tint = self.eff(self.icon_color);
        let accent = self.accent;
        let focused = self.focused;
        // Siblings inside a block keep full layout (revealed by the
        // clip); only the group height glides, so rows below move.
        let mut cursor = self.y;
        for index in 0..self.roots.len() {
            cursor = self.render_subtree(scene, fonts, images, vec![index], 0, cursor, text, chevron, tint, accent, focused, px);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_subtree(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        path: Vec<usize>,
        depth: usize,
        mut y: f32,
        text: Color,
        chevron: Color,
        tint: Color,
        accent: Color,
        focused: bool,
        px: impl Fn(f32) -> f64 + Copy,
    ) -> f32 {
        let Some(info) = self.row_info(&path) else {
            return y;
        };
        // Selection wash under the content.
        if self.selected.as_ref() == Some(&path) {
            let wash = RoundedRect::new(
                px(self.x),
                px(y),
                px(self.x + self.width),
                px(y + OUTLINE_ROW_H),
                6.0 * fonts.scale as f64,
            );
            let c = accent.to_rgba8();
            let fill = if focused {
                Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * OUTLINE_SELECTED_ALPHA).round() as u8)
            } else {
                desaturate(Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * OUTLINE_SELECTED_ALPHA).round() as u8))
            };
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(fill), None, &wash);
        }
        let indent_x = self.x + depth as f32 * OUTLINE_INDENT;
        if info.has_children {
            self.draw_chevron(
                scene,
                indent_x + OUTLINE_CHEV_SLOT / 2.0,
                y + OUTLINE_ROW_H / 2.0,
                info.progress,
                fonts.scale,
                chevron,
            );
        }
        let icon_x = indent_x + OUTLINE_CHEV_SLOT + 2.0;
        let mut icon = SFSymbolImage::new(info.icon.symbol())
            .size(OUTLINE_ICON_SIZE)
            .color(tint);
        icon.place(
            fonts,
            icon_x,
            y + (OUTLINE_ROW_H - OUTLINE_ICON_SIZE) / 2.0,
            OUTLINE_ICON_SIZE,
            OUTLINE_ICON_SIZE,
        );
        icon.draw(scene, fonts, images);
        let layout = fonts.layout_text(&info.label, OUTLINE_LABEL_SIZE, text, None);
        let (_, th) = FontSystem::layout_size(&layout);
        draw_layout(
            scene,
            &layout,
            icon_x + OUTLINE_ICON_SIZE + OUTLINE_ICON_GAP,
            y + (OUTLINE_ROW_H - th / fonts.scale) / 2.0,
            fonts.scale,
        );
        // Children block: full layout faded and clipped by progress.
        // Siblings keep full positions (revealed through the clip);
        // the returned end threads full layout for the next sibling.
        y += OUTLINE_ROW_H;
        let progress = info.progress.clamp(0.0, 1.0);
        if progress > 0.001 && info.child_count > 0 {
            let layered = progress < 0.999;
            if layered {
                let clip = Rect::new(
                    px(self.x),
                    px(y),
                    px(self.x + self.width),
                    px(y + info.full_h * progress),
                );
                scene.push_layer(Fill::NonZero, BlendMode::default(), progress, Affine::IDENTITY, &clip);
            }
            for i in 0..info.child_count {
                let mut child_path = path.clone();
                child_path.push(i);
                y = self.render_subtree(scene, fonts, images, child_path, depth + 1, y, text, chevron, tint, accent, focused, px);
            }
            if layered {
                scene.pop_layer();
            }
        }
        y
    }
}

struct RowInfo {
    label: String,
    icon: OutlineIcon,
    progress: f32,
    has_children: bool,
    child_count: usize,
    full_h: f32,
}

impl Default for BasicOutlineGroup {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl View for BasicOutlineGroup {
    /// Intrinsic size: content width with the animated height, so
    /// rows below glide while blocks fade.
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        self.layout();
        let mut w: f32 = 160.0;
        for row in &self.rows {
            if let Some(node) = self.node(&row.path) {
                let layout = fonts.layout_text(&node.label, OUTLINE_LABEL_SIZE, Color::WHITE, None);
                let (tw, _) = FontSystem::layout_size(&layout);
                // Depth is the path length minus one (roots sit at 0).
                let depth = row.path.len().saturating_sub(1) as f32;
                w = w.max(
                    depth * OUTLINE_INDENT
                        + OUTLINE_CHEV_SLOT
                        + OUTLINE_ICON_SIZE
                        + OUTLINE_ICON_GAP
                        + tw / fonts.scale
                        + 8.0,
                );
            }
        }
        (w, self.content_height())
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(0.0);
        self.height = h.max(0.0);
        self.layout();
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.advance(Instant::now());
        self.render(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.mouse_down(x, y);
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

    fn tree() -> BasicOutlineGroup {
        BasicOutlineGroup::new(vec![
            OutlineNode::folder("Documents")
                .expanded(true)
                .child(OutlineNode::file("Resume.pdf"))
                .child(
                    OutlineNode::folder("Projects")
                        .expanded(true)
                        .child(OutlineNode::file("App.swift"))
                        .child(OutlineNode::file("Notes.txt")),
                ),
            OutlineNode::folder("Downloads"),
            OutlineNode::file("README.md"),
        ])
    }

    fn placed() -> BasicOutlineGroup {
        let mut group = tree();
        group.place(&mut FontSystem::new(), 0.0, 0.0, 400.0, 600.0);
        group
    }

    /// Drive animations to rest at 60 fps steps.
    fn settled(mut group: BasicOutlineGroup) -> BasicOutlineGroup {
        let t0 = Instant::now();
        group.advance(t0);
        for i in 1..=60 {
            group.advance(t0 + Duration::from_secs_f32(i as f32 / 60.0));
            fn resting(node: &OutlineNode) -> bool {
                node.anim.is_none() && node.children.iter().all(resting)
            }
            if group.roots.iter().all(resting) {
                break;
            }
        }
        group
    }

    #[test]
    fn toggle_animates_progress_to_rest() {
        let mut group = placed();
        assert!(group.is_expanded(&[0]));
        group.toggle(vec![0]);
        assert!(!group.is_expanded(&[0]));
        // Mid-animation the block is half revealed and fading.
        let t0 = Instant::now();
        group.advance(t0);
        group.advance(t0 + Duration::from_millis(120));
        let mid = group.node(&[0]).expect("node").progress();
        assert!(mid > 0.0 && mid < 1.0);
        let mut group = settled(group);
        assert_eq!(group.node(&[0]).expect("node").progress(), 0.0);
        group.toggle(vec![0]);
        let group = settled(group);
        assert_eq!(group.node(&[0]).expect("node").progress(), 1.0);
    }

    #[test]
    fn toggle_fires_callback_set_expanded_does_not() {
        let fires: Rc<Cell<Vec<(Vec<usize>, bool)>>> = Rc::new(Cell::new(Vec::new()));
        let seen = fires.clone();
        let mut group = placed().on_toggle(move |path, open| {
            let mut current = seen.take();
            current.push((path, open));
            seen.set(current);
        });
        group.toggle(vec![0]);
        assert_eq!(fires.take(), vec![(vec![0], false)]);
        group.set_expanded(&[0], true);
        assert_eq!(fires.take(), vec![]);
    }

    #[test]
    fn files_and_unknown_paths_never_toggle() {
        let mut group = placed();
        assert!(!group.toggle(vec![2]));
        assert!(!group.toggle(vec![9]));
        assert!(!group.set_expanded(&[2], true));
    }

    #[test]
    fn click_selects_and_folder_click_toggles() {
        let mut group = placed();
        // File row: Resume.pdf is the second visible row.
        group.mouse_down(100.0, (OUTLINE_ROW_H + 10.0) as f64);
        assert_eq!(group.selected_path(), Some(vec![0, 0]));
        // Projects header toggles closed and selects.
        group.mouse_down(100.0, (2.0 * OUTLINE_ROW_H + 10.0) as f64);
        assert_eq!(group.selected_path(), Some(vec![0, 1]));
        assert!(!group.is_expanded(&[0, 1]));
    }

    #[test]
    fn select_programmatic_and_clear_fire() {
        let seen: Rc<Cell<Vec<Vec<usize>>>> = Rc::new(Cell::new(Vec::new()));
        let capture = seen.clone();
        let mut group = placed().on_select(move |path| {
            let mut current = capture.take();
            current.push(path);
            capture.set(current);
        });
        assert!(group.select(vec![0, 1, 0]));
        assert_eq!(group.selected_path(), Some(vec![0, 1, 0]));
        assert!(!group.select(vec![5]));
        group.clear_selection();
        assert_eq!(group.selected_path(), None);
        assert_eq!(seen.take(), vec![vec![0, 1, 0], vec![]]);
    }

    #[test]
    fn expand_and_collapse_all_snap_instantly() {
        let mut group = placed();
        group.collapse_all();
        assert!(!group.is_expanded(&[0]));
        assert_eq!(group.content_height(), 3.0 * OUTLINE_ROW_H);
        group.expand_all();
        assert!(group.is_expanded(&[0, 1]));
        // 3 roots plus Resume, Projects, App and Notes: 7 rows.
        assert_eq!(group.content_height(), 7.0 * OUTLINE_ROW_H);
    }

    #[test]
    fn disabled_group_ignores_clicks() {
        let mut group = BasicOutlineGroup::new(vec![
            OutlineNode::folder("D").child(OutlineNode::file("f")),
        ])
        .disabled(true);
        group.place(&mut FontSystem::new(), 0.0, 0.0, 400.0, 600.0);
        group.mouse_down(100.0, 10.0);
        assert_eq!(group.selected_path(), None);
        assert!(!group.is_expanded(&[0]));
    }

    #[test]
    fn unselectable_group_still_toggles() {
        let mut group = BasicOutlineGroup::new(vec![
            OutlineNode::folder("D").child(OutlineNode::file("f")),
        ])
        .selectable(false);
        group.place(&mut FontSystem::new(), 0.0, 0.0, 400.0, 600.0);
        group.mouse_down(100.0, 10.0);
        assert_eq!(group.selected_path(), None);
        assert!(group.is_expanded(&[0]));
    }

    #[test]
    fn miss_click_keeps_selection() {
        let mut group = placed();
        group.mouse_down(100.0, (OUTLINE_ROW_H + 10.0) as f64);
        assert_eq!(group.selected_path(), Some(vec![0, 0]));
        group.mouse_down(100.0, 590.0);
        assert_eq!(group.selected_path(), Some(vec![0, 0]));
    }
}
