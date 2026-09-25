use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

use vello::Scene;
use vello::kurbo::{Affine, Line, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::buttons::Button;
use super::super::groupbox::{GROUP_BG_DARK, GROUP_BG_LIGHT, GROUP_RADIUS};
use super::super::images::SFSymbolImage;
use super::super::layout::View;
use super::super::menu::Menu;
use super::super::textfield::{BasicTextField, SecureField};
use super::super::toggles::Toggle;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::renderer::window::Key;
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Form row height in logical px.
pub const FORM_ROW_H: f32 = 46.0;
/// Group inner padding in logical px.
pub const FORM_PAD: f32 = 16.0;
/// Gap between the label column and the control in logical px.
pub const FORM_LABEL_GAP: f32 = 16.0;
/// Row label size in logical px.
pub const FORM_LABEL_SIZE: f32 = 15.0;
/// Section title size in logical px.
pub const FORM_TITLE_SIZE: f32 = 17.0;
/// Section title block height in logical px.
pub const FORM_TITLE_H: f32 = 26.0;
/// Gap between title and group in logical px.
pub const FORM_TITLE_GAP: f32 = 8.0;
/// Footnote size in logical px.
pub const FORM_NOTE_SIZE: f32 = 13.0;
/// Footnote block height in logical px.
pub const FORM_NOTE_H: f32 = 20.0;
/// Gap between group and footnote in logical px.
pub const FORM_NOTE_GAP: f32 = 8.0;
/// Gap between sections in logical px.
pub const FORM_SECTION_GAP: f32 = 28.0;
/// Gap between buttons in a button row in logical px.
pub const FORM_BUTTON_GAP: f32 = 12.0;
/// Leading row icon box in logical px.
pub const FORM_ICON_SIZE: f32 = 18.0;
/// Gap between icon and label in logical px.
pub const FORM_ICON_GAP: f32 = 8.0;
/// Intrinsic minimum width in logical px.
pub const FORM_MIN_W: f32 = 320.0;
/// Text control height inside a row in logical px.
const FORM_FIELD_H: f32 = 32.0;

/// Form row control kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormRowKind {
    Text,
    Secure,
    Toggle,
    Picker,
    Buttons,
}

/// One form row: a label on the left, a control on the right.
/// Button rows span the group centered; every other row keeps the
/// label column width of its section so controls align.
pub struct FormRow {
    label: String,
    icon: Option<SFSymbolImage>,
    kind: FormRowKind,
    field: Option<BasicTextField>,
    secure: Option<SecureField>,
    toggle: Option<Toggle>,
    menu: Option<Menu>,
    options: Vec<String>,
    selected: usize,
    pick_pending: Rc<Cell<Option<usize>>>,
    buttons: Vec<Button>,
    on_pick: Option<Box<dyn FnMut(usize)>>,
    x: f32,
    y: f32,
    width: f32,
    label_w: f32,
}

impl FormRow {
    /// Text row: label left, borderless right-aligned input right.
    pub fn text(label: impl Into<String>, value: impl Into<String>) -> Self {
        let mut field = BasicTextField::new("").borderless(true).align_right(true);
        field.set_text(value);
        Self {
            label: label.into(),
            icon: None,
            kind: FormRowKind::Text,
            field: Some(field),
            secure: None,
            toggle: None,
            menu: None,
            options: Vec::new(),
            selected: 0,
            pick_pending: Rc::new(Cell::new(None)),
            buttons: Vec::new(),
            on_pick: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            label_w: 0.0,
        }
    }

    /// Secure row: label left, borderless right-aligned password
    /// input right.
    pub fn secure(label: impl Into<String>, value: impl Into<String>) -> Self {
        let mut field = SecureField::new("").borderless(true).align_right(true);
        field.set_text(value);
        Self {
            label: label.into(),
            icon: None,
            kind: FormRowKind::Secure,
            field: None,
            secure: Some(field),
            toggle: None,
            menu: None,
            options: Vec::new(),
            selected: 0,
            pick_pending: Rc::new(Cell::new(None)),
            buttons: Vec::new(),
            on_pick: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            label_w: 0.0,
        }
    }

    /// Toggle row: label left, switch right.
    pub fn toggle(label: impl Into<String>, on: bool) -> Self {
        Self {
            label: label.into(),
            icon: None,
            kind: FormRowKind::Toggle,
            field: None,
            secure: None,
            toggle: Some(Toggle::new("").on(on)),
            menu: None,
            options: Vec::new(),
            selected: 0,
            pick_pending: Rc::new(Cell::new(None)),
            buttons: Vec::new(),
            on_pick: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            label_w: 0.0,
        }
    }

    /// Picker row: label left, dropdown right. The button shows the
    /// selected option with a checkmark on it; `on_pick` fires with
    /// the new index.
    pub fn picker(label: impl Into<String>, options: Vec<String>, selected: usize) -> Self {
        let selected = selected.min(options.len().saturating_sub(1));
        let pending = Rc::new(Cell::new(None));
        let capture = pending.clone();
        let button = options.get(selected).cloned().unwrap_or_default();
        let refs: Vec<&str> = options.iter().map(String::as_str).collect();
        let menu = Menu::from_slice(button, &refs)
            .checked(Some(selected))
            .on_action(move |index| capture.set(Some(index)));
        Self {
            label: label.into(),
            icon: None,
            kind: FormRowKind::Picker,
            field: None,
            secure: None,
            toggle: None,
            menu: Some(menu),
            options,
            selected,
            pick_pending: pending,
            buttons: Vec::new(),
            on_pick: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            label_w: 0.0,
        }
    }

    /// Button row: centered buttons in their own group (Cancel plus
    /// Save, built by the app with styles and callbacks).
    pub fn buttons(buttons: Vec<Button>) -> Self {
        Self {
            label: String::new(),
            icon: None,
            kind: FormRowKind::Buttons,
            field: None,
            secure: None,
            toggle: None,
            menu: None,
            options: Vec::new(),
            selected: 0,
            pick_pending: Rc::new(Cell::new(None)),
            buttons,
            on_pick: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            label_w: 0.0,
        }
    }

    /// Leading SF Symbol icon before the label (e.g. `"tag"`,
    /// `"globe"`), like the reference picker rows.
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(SFSymbolImage::new(name).size(FORM_ICON_SIZE));
        self
    }

    /// Input placeholder for text and secure rows.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        let placeholder = placeholder.into();
        if let Some(field) = self.field.as_mut() {
            field.set_placeholder(placeholder);
        } else if let Some(field) = self.secure.as_mut() {
            field.set_placeholder(placeholder);
        }
        self
    }

    /// Fires with the full text on every user edit (text rows).
    pub fn on_change(mut self, callback: impl FnMut(&str) + 'static) -> Self {
        if let Some(field) = self.field.take() {
            self.field = Some(field.on_change(callback));
        } else if let Some(field) = self.secure.take() {
            self.secure = Some(field.on_change(callback));
        }
        self
    }

    /// Fires with the new state on every toggle flip.
    pub fn on_toggle(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        if let Some(toggle) = self.toggle.take() {
            self.toggle = Some(toggle.on_toggle(callback));
        }
        self
    }

    /// Fires with the new option index on every pick.
    pub fn on_pick(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_pick = Some(Box::new(callback));
        self
    }

    pub fn kind(&self) -> FormRowKind {
        self.kind
    }

    pub fn label_text(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Field text for text and secure rows (empty otherwise).
    pub fn text_value(&self) -> &str {
        if let Some(field) = self.field.as_ref() {
            field.text_value()
        } else if let Some(field) = self.secure.as_ref() {
            field.text_value()
        } else {
            ""
        }
    }

    pub fn set_text(&mut self, value: impl Into<String>) {
        let value = value.into();
        if let Some(field) = self.field.as_mut() {
            field.set_text(value);
        } else if let Some(field) = self.secure.as_mut() {
            field.set_text(value);
        }
    }

    pub fn is_on(&self) -> bool {
        self.toggle.as_ref().is_some_and(|t| t.is_on())
    }

    pub fn set_on(&mut self, on: bool) {
        if let Some(toggle) = self.toggle.as_mut() {
            toggle.set_on(on);
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn option_text(&self) -> &str {
        self.options.get(self.selected).map(String::as_str).unwrap_or("")
    }

    /// Pick programmatically: updates the button plus checkmark and
    /// fires `on_pick`. Returns false when nothing is selectable.
    pub fn select(&mut self, index: usize) -> bool {
        if self.options.is_empty() {
            return false;
        }
        let index = index.min(self.options.len() - 1);
        if index == self.selected {
            return true;
        }
        self.selected = index;
        if let Some(menu) = self.menu.as_mut() {
            menu.set_button(self.options[index].clone());
            menu.set_checked(Some(index));
        }
        if let Some(callback) = self.on_pick.as_mut() {
            callback(index);
        }
        true
    }

    fn drain_pick(&mut self) {
        if let Some(index) = self.pick_pending.take() {
            self.select(index);
        }
    }

    fn label_content_w(&self, fonts: &mut FontSystem) -> f32 {
        let mut w = 0.0;
        if self.icon.is_some() {
            w += FORM_ICON_SIZE + FORM_ICON_GAP;
        }
        if !self.label.is_empty() {
            let layout = fonts.layout_text(&self.label, FORM_LABEL_SIZE, Color::WHITE, None);
            w += FontSystem::layout_size(&layout).0 / fonts.scale;
        }
        w
    }
}

/// One titled group of rows with an optional footnote below it,
/// like the reference Connection/Authentication/Options blocks.
pub struct FormSection {
    title: Option<String>,
    rows: Vec<FormRow>,
    footnote: Option<String>,
    title_y: f32,
    group_y: f32,
    note_y: f32,
}

impl FormSection {
    pub fn new() -> Self {
        Self {
            title: None,
            rows: Vec::new(),
            footnote: None,
            title_y: 0.0,
            group_y: 0.0,
            note_y: 0.0,
        }
    }

    pub fn titled(title: impl Into<String>) -> Self {
        Self::new().title(title)
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn row(mut self, row: FormRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Dim helper text below the group (e.g. terms notices).
    pub fn footnote(mut self, note: impl Into<String>) -> Self {
        self.footnote = Some(note.into());
        self
    }

    pub fn rows_len(&self) -> usize {
        self.rows.len()
    }

    pub fn row_mut(&mut self, index: usize) -> Option<&mut FormRow> {
        self.rows.get_mut(index)
    }

    fn height(&self) -> f32 {
        let mut h = 0.0;
        if self.title.is_some() {
            h += FORM_TITLE_H + FORM_TITLE_GAP;
        }
        h += self.rows.len() as f32 * FORM_ROW_H;
        if self.footnote.is_some() {
            h += FORM_NOTE_GAP + FORM_NOTE_H;
        }
        h
    }
}

impl Default for FormSection {
    fn default() -> Self {
        Self::new()
    }
}

/// Settings-style form: titled sections of label/control rows in
/// `GroupBox` bodies (text left, action right), dividers between
/// rows, footnotes below groups. Text rows edit inline, toggle rows
/// flip switches, picker rows open dropdowns, button rows center
/// app-built buttons. Embeds directly or in a `ScrollView` for long
/// forms (`measure` reports the full content height).
pub struct Form {
    sections: Vec<FormSection>,
    accent: Color,
    dark: bool,
    focused: bool,
    glass_mode: ThemeMode,
    glass_amount: GlassAmount,
    vp_x: f32,
    vp_y: f32,
    vp_w: f32,
    vp_h: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Form {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            accent: Color::from_rgb8(0x00, 0x7a, 0xff),
            dark: true,
            focused: true,
            glass_mode: ThemeMode::Dark,
            glass_amount: GlassAmount::Glass,
            vp_x: 0.0,
            vp_y: 0.0,
            vp_w: 0.0,
            vp_h: 0.0,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn section(mut self, section: FormSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn sections_len(&self) -> usize {
        self.sections.len()
    }

    pub fn section_mut(&mut self, index: usize) -> Option<&mut FormSection> {
        self.sections.get_mut(index)
    }

    fn row(&self, section: usize, row: usize) -> Option<&FormRow> {
        self.sections.get(section)?.rows.get(row)
    }

    fn row_mut(&mut self, section: usize, row: usize) -> Option<&mut FormRow> {
        self.sections.get_mut(section)?.rows.get_mut(row)
    }

    /// Field text of a text/secure row (empty for other rows and
    /// out-of-range indices).
    pub fn text_value(&self, section: usize, row: usize) -> &str {
        self.row(section, row).map(FormRow::text_value).unwrap_or("")
    }

    pub fn set_text(&mut self, section: usize, row: usize, value: impl Into<String>) {
        if let Some(row) = self.row_mut(section, row) {
            row.set_text(value);
        }
    }

    pub fn is_on(&self, section: usize, row: usize) -> bool {
        self.row(section, row).is_some_and(FormRow::is_on)
    }

    pub fn set_on(&mut self, section: usize, row: usize, on: bool) {
        if let Some(row) = self.row_mut(section, row) {
            row.set_on(on);
        }
    }

    pub fn selected_index(&self, section: usize, row: usize) -> usize {
        self.row(section, row).map(FormRow::selected_index).unwrap_or(0)
    }

    pub fn select(&mut self, section: usize, row: usize, index: usize) -> bool {
        self.row_mut(section, row).is_some_and(|row| row.select(index))
    }

    /// Live theme forwarded to every control plus the group bodies,
    /// labels, icons and dividers.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        self.dark = dark;
        let label = self.text_color();
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(field) = row.field.as_mut() {
                    field.set_theme(accent, dark);
                }
                if let Some(field) = row.secure.as_mut() {
                    field.set_theme(accent, dark);
                }
                if let Some(toggle) = row.toggle.as_mut() {
                    toggle.set_theme(accent, dark);
                }
                if let Some(menu) = row.menu.as_mut() {
                    menu.set_theme(accent, dark);
                }
                for button in &mut row.buttons {
                    button.set_theme(accent, dark);
                }
                if let Some(icon) = row.icon.as_mut() {
                    icon.set_theme(label, dark);
                }
            }
        }
    }

    /// Glass stage for the picker dropdown panels.
    pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount) {
        self.glass_mode = mode;
        self.glass_amount = amount;
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(menu) = row.menu.as_mut() {
                    menu.set_glass(mode, amount);
                }
            }
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        let label = self.text_color();
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(field) = row.field.as_mut() {
                    field.set_focused(focused);
                }
                if let Some(field) = row.secure.as_mut() {
                    field.set_focused(focused);
                }
                if let Some(toggle) = row.toggle.as_mut() {
                    toggle.set_focused(focused);
                }
                if let Some(menu) = row.menu.as_mut() {
                    menu.set_focused(focused);
                }
                for button in &mut row.buttons {
                    button.set_focused(focused);
                }
                if let Some(icon) = row.icon.as_mut() {
                    icon.set_theme(label, self.dark);
                }
            }
        }
    }

    /// Window bounds the picker panels clamp into. Apps must call
    /// this every frame with the current viewport.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vp_x = x;
        self.vp_y = y;
        self.vp_w = w.max(0.0);
        self.vp_h = h.max(0.0);
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(menu) = row.menu.as_mut() {
                    menu.set_viewport(x, y, w, h);
                }
            }
        }
    }

    /// Full content height in logical px (fixed rows, no fonts
    /// needed), for scroll containers and window sizing.
    pub fn content_height(&self) -> f32 {
        let mut h = 0.0;
        for (i, section) in self.sections.iter().enumerate() {
            h += section.height();
            if i + 1 < self.sections.len() {
                h += FORM_SECTION_GAP;
            }
        }
        h
    }

    /// True while a picker panel is open or a switch knob is held:
    /// return it from `App::wants_backdrop` for the blur pass.
    pub fn wants_backdrop(&self) -> bool {
        self.sections.iter().flat_map(|s| s.rows.iter()).any(|row| {
            row.menu.as_ref().is_some_and(Menu::is_open)
                || row.toggle.as_ref().is_some_and(Toggle::is_dragging)
        })
    }

    /// True while the pointer hovers an input: the app returns the
    /// I-beam cursor from `App::cursor` then.
    pub fn wants_text_cursor(&self) -> bool {
        self.sections.iter().flat_map(|s| s.rows.iter()).any(|row| {
            row.field.as_ref().is_some_and(BasicTextField::wants_text_cursor)
                || row.secure.as_ref().is_some_and(SecureField::wants_text_cursor)
        })
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(field) = row.field.as_mut() {
                    field.mouse_down(x, y);
                }
                if let Some(field) = row.secure.as_mut() {
                    field.mouse_down(x, y);
                }
                if let Some(toggle) = row.toggle.as_mut() {
                    toggle.mouse_down(x, y);
                }
                if let Some(menu) = row.menu.as_mut() {
                    menu.mouse_down(x, y);
                }
                for button in &mut row.buttons {
                    button.mouse_down(x, y);
                }
            }
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(toggle) = row.toggle.as_mut() {
                    toggle.mouse_up(x, y);
                }
                if let Some(menu) = row.menu.as_mut() {
                    menu.mouse_up(x, y);
                }
                for button in &mut row.buttons {
                    button.mouse_up(x, y);
                }
            }
        }
        self.drain_picks();
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(toggle) = row.toggle.as_mut() {
                    toggle.mouse_move(x, y);
                }
                if let Some(menu) = row.menu.as_mut() {
                    menu.mouse_move(x, y);
                }
            }
        }
    }

    /// Scroll wheel delta in logical px: open picker panels scroll.
    pub fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(menu) = row.menu.as_mut() {
                    menu.mouse_wheel(dx, dy);
                }
            }
        }
    }

    /// Type printable text into the focused input (the app forwards
    /// its `text` here).
    pub fn type_text(&mut self, content: &str) {
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(field) = row.field.as_mut() {
                    field.type_text(content);
                }
                if let Some(field) = row.secure.as_mut() {
                    field.type_text(content);
                }
            }
        }
    }

    /// Key handling for the focused input. Returns true when a
    /// field consumed the key. The app forwards its `key` here.
    pub fn key(&mut self, key: Key) -> bool {
        let mut consumed = false;
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(field) = row.field.as_mut() {
                    consumed |= field.key(key);
                }
                if let Some(field) = row.secure.as_mut() {
                    consumed |= field.key(key);
                }
            }
        }
        consumed
    }

    fn drain_picks(&mut self) {
        for section in &mut self.sections {
            for row in &mut section.rows {
                row.drain_pick();
            }
        }
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

    fn dim_color(&self) -> Color {
        if self.dark {
            Color::from_rgb8(0x9a, 0x9a, 0x9e)
        } else {
            Color::from_rgb8(0x6e, 0x6e, 0x72)
        }
    }

    fn group_fill(&self) -> Color {
        self.eff(if self.dark {
            GROUP_BG_DARK
        } else {
            GROUP_BG_LIGHT
        })
    }

    fn divider_color(&self) -> Color {
        self.eff(if self.dark {
            Color::from_rgba8(255, 255, 255, 36)
        } else {
            Color::from_rgba8(0, 0, 0, 31)
        })
    }

    fn place_row_control(&mut self, fonts: &mut FontSystem, section: usize, row: usize) {
        let Some(sec) = self.sections.get(section) else {
            return;
        };
        let Some(r) = sec.rows.get(row) else {
            return;
        };
        let (rx, ry, rw, label_w) = (r.x, r.y, r.width, r.label_w);
        let kind = r.kind;
        let row = &mut self.sections[section].rows[row];
        match kind {
            FormRowKind::Text | FormRowKind::Secure => {
                let fx = rx + FORM_PAD + label_w + FORM_LABEL_GAP;
                let fw = (rx + rw - FORM_PAD - fx).max(0.0);
                let fy = ry + (FORM_ROW_H - FORM_FIELD_H) / 2.0;
                if let Some(field) = row.field.as_mut() {
                    field.place(fonts, fx, fy, fw, FORM_FIELD_H);
                }
                if let Some(field) = row.secure.as_mut() {
                    field.place(fonts, fx, fy, fw, FORM_FIELD_H);
                }
            }
            FormRowKind::Toggle => {
                if let Some(toggle) = row.toggle.as_mut() {
                    let (tw, th) = toggle.measure(fonts);
                    toggle.place(fonts, rx + rw - FORM_PAD - tw, ry + (FORM_ROW_H - th) / 2.0, tw, th);
                }
            }
            FormRowKind::Picker => {
                if let Some(menu) = row.menu.as_mut() {
                    let (mw, mh) = menu.measure(fonts);
                    menu.place(fonts, rx + rw - FORM_PAD - mw, ry + (FORM_ROW_H - mh) / 2.0, mw, mh);
                }
            }
            FormRowKind::Buttons => {
                let n = row.buttons.len();
                if n == 0 {
                    return;
                }
                let mut widths = Vec::with_capacity(n);
                let mut heights = Vec::with_capacity(n);
                for button in row.buttons.iter_mut() {
                    let (bw, bh) = button.measure(fonts);
                    widths.push(bw);
                    heights.push(bh);
                }
                let total: f32 = widths.iter().sum::<f32>() + FORM_BUTTON_GAP * (n as f32 - 1.0);
                let mut bx = rx + (rw - total) / 2.0;
                for (i, button) in row.buttons.iter_mut().enumerate() {
                    button.place(fonts, bx, ry + (FORM_ROW_H - heights[i]) / 2.0, widths[i], heights[i]);
                    bx += widths[i] + FORM_BUTTON_GAP;
                }
            }
        }
        if let Some(icon) = row.icon.as_mut() {
            let (iw, ih) = icon.measure(fonts);
            icon.place(fonts, rx + FORM_PAD, ry + (FORM_ROW_H - ih) / 2.0, iw, ih);
        }
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.drain_picks();
        if self.width <= 0.0 || self.height <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let text = self.eff(self.text_color());
        let dim = self.eff(self.dim_color());
        let group = self.group_fill();
        let divider = self.divider_color();
        let (fx, fw) = (self.x, self.width);
        for section in &mut self.sections {
            if let Some(title) = section.title.clone() {
                let layout = fonts.layout_text_weighted(
                    &title,
                    FORM_TITLE_SIZE,
                    text,
                    600.0,
                    None,
                );
                draw_layout(scene, &layout, fx + FORM_PAD, section.title_y, fonts.scale);
            }
            let group_h = section.rows.len() as f32 * FORM_ROW_H;
            if group_h > 0.0 {
                let body = RoundedRect::new(
                    px(fx),
                    px(section.group_y),
                    px(fx + fw),
                    px(section.group_y + group_h),
                    px(GROUP_RADIUS),
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(group),
                    None,
                    &body,
                );
            }
            for (i, row) in section.rows.iter_mut().enumerate() {
                if i > 0 {
                    let dy = section.group_y + i as f32 * FORM_ROW_H;
                    let line = Line::new(
                        (px(fx), px(dy)),
                        (px(fx + fw), px(dy)),
                    );
                    scene.stroke(
                        &Stroke::new(1.0 * scale),
                        Affine::IDENTITY,
                        &Brush::Solid(divider),
                        None,
                        &line,
                    );
                }
                if !row.label.is_empty() {
                    let layout = fonts.layout_text_weighted(
                        &row.label,
                        FORM_LABEL_SIZE,
                        text,
                        400.0,
                        None,
                    );
                    let (_, th) = FontSystem::layout_size(&layout);
                    let mut lx = row.x + FORM_PAD;
                    if row.icon.is_some() {
                        lx += FORM_ICON_SIZE + FORM_ICON_GAP;
                    }
                    draw_layout(scene, &layout, lx, row.y + (FORM_ROW_H - th / fonts.scale) / 2.0, fonts.scale);
                }
                if let Some(icon) = row.icon.as_mut() {
                    icon.draw(scene, fonts, images);
                }
                if let Some(field) = row.field.as_mut() {
                    field.draw(scene, fonts, images);
                }
                if let Some(field) = row.secure.as_mut() {
                    field.draw(scene, fonts, images);
                }
                if let Some(toggle) = row.toggle.as_mut() {
                    toggle.draw(scene, fonts, images);
                }
                if let Some(menu) = row.menu.as_mut() {
                    menu.draw(scene, fonts, images);
                }
                for button in row.buttons.iter_mut() {
                    button.draw(scene, fonts, images);
                }
            }
            if let Some(note) = section.footnote.clone() {
                let layout = fonts.layout_text(&note, FORM_NOTE_SIZE, dim, None);
                draw_layout(scene, &layout, fx + FORM_PAD, section.note_y, fonts.scale);
            }
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Form {
    /// Intrinsic size: minimum width with the full content height.
    /// Place with the window width (or inside a `ScrollView`, which
    /// reports the whole height through here).
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (FORM_MIN_W, self.content_height())
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(0.0);
        self.height = h.max(0.0);
        let mut cy = y;
        for s in 0..self.sections.len() {
            let (has_title, has_note, rows) = match self.sections.get(s) {
                Some(sec) => (sec.title.is_some(), sec.footnote.is_some(), sec.rows.len()),
                None => (false, false, 0),
            };
            if has_title {
                if let Some(sec) = self.sections.get_mut(s) {
                    sec.title_y = cy;
                }
                cy += FORM_TITLE_H + FORM_TITLE_GAP;
            }
            if let Some(sec) = self.sections.get_mut(s) {
                sec.group_y = cy;
            }
            // Label column: widest label (plus icon) in the section.
            let mut label_w = 0.0f32;
            for r in 0..rows {
                if let Some(row) = self.sections.get(s).and_then(|sec| sec.rows.get(r)) {
                    label_w = label_w.max(row.label_content_w(fonts));
                }
            }
            for r in 0..rows {
                if let Some(row) = self.sections.get_mut(s).and_then(|sec| sec.rows.get_mut(r)) {
                    row.x = x;
                    row.y = cy + r as f32 * FORM_ROW_H;
                    row.width = w.max(0.0);
                    row.label_w = label_w;
                }
                self.place_row_control(fonts, s, r);
            }
            cy += rows as f32 * FORM_ROW_H;
            if has_note {
                cy += FORM_NOTE_GAP;
                if let Some(sec) = self.sections.get_mut(s) {
                    sec.note_y = cy;
                }
                cy += FORM_NOTE_H;
            }
            if s + 1 < self.sections.len() {
                cy += FORM_SECTION_GAP;
            }
        }
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.mouse_up(x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        for section in &mut self.sections {
            for row in &mut section.rows {
                if let Some(field) = row.field.as_mut() {
                    field.set_hover(x, y);
                }
                if let Some(field) = row.secure.as_mut() {
                    field.set_hover(x, y);
                }
                if let Some(toggle) = row.toggle.as_mut() {
                    toggle.set_hover(x, y);
                }
                for button in &mut row.buttons {
                    button.set_hover(x, y);
                }
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::buttons::ButtonStyle;

    fn basic_form() -> Form {
        Form::new().section(
            FormSection::titled("Profile")
                .row(FormRow::text("Name", "Ada"))
                .row(FormRow::secure("Password", "")),
        )
    }

    fn placed(form: &mut Form) {
        let mut fonts = FontSystem::new();
        let w = 600.0;
        form.place(&mut fonts, 0.0, 0.0, w, form.content_height());
    }

    #[test]
    fn content_height_counts_titles_rows_and_notes() {
        let form = basic_form();
        // Title block + 2 rows, no note, one section.
        assert_eq!(
            form.content_height(),
            FORM_TITLE_H + FORM_TITLE_GAP + 2.0 * FORM_ROW_H
        );
        let noted = Form::new().section(
            FormSection::titled("Options")
                .row(FormRow::toggle("I agree", false))
                .footnote("Please read the terms."),
        );
        assert_eq!(
            noted.content_height(),
            FORM_TITLE_H + FORM_TITLE_GAP + FORM_ROW_H + FORM_NOTE_GAP + FORM_NOTE_H
        );
    }

    #[test]
    fn text_rows_read_and_write() {
        let mut form = basic_form();
        assert_eq!(form.text_value(0, 0), "Ada");
        form.set_text(0, 0, "Grace");
        assert_eq!(form.text_value(0, 0), "Grace");
        assert_eq!(form.text_value(0, 1), "");
        assert_eq!(form.text_value(9, 9), "");
    }

    #[test]
    fn toggle_rows_flip_and_report() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let seen: Rc<RefCell<Option<bool>>> = Rc::new(RefCell::new(None));
        let capture = seen.clone();
        let mut form = Form::new().section(
            FormSection::titled("Options").row(
                FormRow::toggle("Use SSH Key", false).on_toggle(move |on| {
                    *capture.borrow_mut() = Some(on);
                }),
            ),
        );
        assert!(!form.is_on(0, 0));
        form.set_on(0, 0, true);
        assert!(form.is_on(0, 0));
        assert_eq!(*seen.borrow(), Some(true));
    }

    #[test]
    fn picker_select_updates_button_and_fires() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let seen: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
        let capture = seen.clone();
        let mut form = Form::new().section(
            FormSection::titled("Options").row(
                FormRow::picker("Protocol", vec!["FTP".into(), "SFTP".into()], 0)
                    .on_pick(move |i| capture.borrow_mut().push(i)),
            ),
        );
        assert_eq!(form.selected_index(0, 0), 0);
        assert!(form.select(0, 0, 1));
        assert_eq!(form.selected_index(0, 0), 1);
        assert_eq!(form.section_mut(0).unwrap().row_mut(0).unwrap().option_text(), "SFTP");
        assert_eq!(*seen.borrow(), vec![1]);
        // Same index again: stays put, no second fire.
        assert!(form.select(0, 0, 5));
        assert_eq!(*seen.borrow(), vec![1]);
    }

    #[test]
    fn pending_menu_pick_drains_on_mouse_up() {
        let mut form = Form::new().section(
            FormSection::titled("Options")
                .row(FormRow::picker("Protocol", vec!["FTP".into(), "SFTP".into()], 0)),
        );
        placed(&mut form);
        // Simulate the menu action firing (as Menu does on row click).
        form.section_mut(0).unwrap().row_mut(0).unwrap().pick_pending.set(Some(1));
        form.mouse_up(0.0, 0.0);
        assert_eq!(form.selected_index(0, 0), 1);
    }

    #[test]
    fn button_row_press_fires() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let taps = count.clone();
        let mut form = Form::new().section(
            FormSection::new().row(FormRow::buttons(vec![
                Button::new("Cancel").style(ButtonStyle::Bordered),
                Button::new("Save")
                    .style(ButtonStyle::BorderedProminent)
                    .on_press(move || *taps.borrow_mut() += 1),
            ])),
        );
        placed(&mut form);
        // Press the Save button: find its rect through the row.
        let rect = {
            let sec = form.section_mut(0).unwrap();
            let row = sec.row_mut(0).unwrap();
            assert_eq!(row.kind(), FormRowKind::Buttons);
            row.buttons[1].rect()
        };
        let (bx, by, bw, bh) = rect;
        form.mouse_down((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        form.mouse_up((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        assert_eq!(*count.borrow(), 1);
    }

    #[test]
    fn toggle_click_flips_state() {
        let mut form = Form::new()
            .section(FormSection::titled("Options").row(FormRow::toggle("Use SSH Key", false)));
        placed(&mut form);
        // Click the switch: right side of the first row.
        let (sx, sy) = (
            (600.0 - FORM_PAD - 24.0) as f64,
            (FORM_TITLE_H + FORM_TITLE_GAP + FORM_ROW_H / 2.0) as f64,
        );
        form.mouse_down(sx, sy);
        form.mouse_up(sx, sy);
        assert!(form.is_on(0, 0));
    }

    #[test]
    fn rows_align_controls_to_widest_label() {
        let mut form = Form::new().section(
            FormSection::titled("Connection")
                .row(FormRow::text("Username", ""))
                .row(FormRow::text("Host", "")),
        );
        placed(&mut form);
        let sec = form.section_mut(0).unwrap();
        let w0 = sec.row_mut(0).unwrap().label_w;
        let w1 = sec.row_mut(1).unwrap().label_w;
        assert!(w0 > 0.0);
        assert_eq!(w0, w1);
    }
}
