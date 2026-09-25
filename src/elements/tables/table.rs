use std::any::Any;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use parley::Layout;
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::gestures::GESTURE_DOUBLE_TAP_SECONDS;
use super::super::layout::View;
use super::super::scrollbar::{SCROLLBAR_GRAY, SCROLLBAR_W_HOVER, Scrollbar};
use super::super::textfield::BasicTextField;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
use crate::renderer::window::Key;
use crate::theme::desaturate;

/// Header height in logical px.
pub const TABLE_HEADER_H: f32 = 30.0;
/// Data row pill height in logical px.
pub const TABLE_ROW_H: f32 = 34.0;
/// Vertical gap between row pills in logical px.
pub const TABLE_ROW_GAP: f32 = 6.0;
/// Row pill corner radius in logical px.
pub const TABLE_RADIUS: f32 = 8.0;
/// Cell text size in logical px.
pub const TABLE_FONT_SIZE: f32 = 13.0;
/// Header title size in logical px.
pub const TABLE_HEADER_SIZE: f32 = 12.0;
/// Horizontal text inset inside pills in logical px.
pub const TABLE_PAD_X: f32 = 12.0;
/// Gap between columns in logical px.
pub const TABLE_COL_GAP: f32 = 16.0;
/// Overlay bar width in logical px (same as the scroll views).
pub const TABLE_BAR_W: f32 = SCROLLBAR_W_HOVER;
/// Horizontal bar strip height in logical px.
pub const TABLE_HBAR_H: f32 = 10.0;
/// Selection fill alpha (0-1 of the theme accent).
pub const TABLE_SELECTED_ALPHA: f32 = 0.35;

/// Data row pill fill in dark mode.
pub const TABLE_ROW_FILL_DARK: Color = Color::from_rgba8(255, 255, 255, 10);
/// Data row pill fill in light mode.
pub const TABLE_ROW_FILL_LIGHT: Color = Color::from_rgba8(0, 0, 0, 10);
/// Header divider and empty-pill stroke in dark mode.
pub const TABLE_DIVIDER_DARK: Color = Color::from_rgba8(255, 255, 255, 36);
/// Header divider and empty-pill stroke in light mode.
pub const TABLE_DIVIDER_LIGHT: Color = Color::from_rgba8(0, 0, 0, 31);

/// Custom column comparator: compares two cell strings, like
/// `Ord::cmp`. Returning `Ordering::Equal` falls through to the row
/// position so sorts stay stable.
pub type TableSortCmp = Box<dyn Fn(&str, &str) -> Ordering>;

/// One table column: title, flex weight, flags and an optional
/// custom comparator (otherwise the smart sort applies).
pub struct TableColumn {
    title: String,
    weight: f32,
    min_width: f32,
    sortable: bool,
    editable: bool,
    sort_cmp: Option<TableSortCmp>,
}

impl TableColumn {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            weight: 1.0,
            min_width: 80.0,
            sortable: true,
            editable: false,
            sort_cmp: None,
        }
    }

    /// Flex share of the table width (default `1.0`).
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = weight.max(0.1);
        self
    }

    /// Minimum width in logical px (default `80.0`). When the summed
    /// minima exceed the table width the body scrolls horizontally.
    pub fn min_width(mut self, px: f32) -> Self {
        self.min_width = px.max(20.0);
        self
    }

    /// Header click sorts this column (default `true`).
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Cells can be edited inline (default `false`). Editing starts
    /// on double-click (when enabled on the table) or through
    /// `begin_edit`, e.g. from a context menu action.
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Custom comparator for this column. It replaces the smart sort
    /// (numbers numeric, otherwise case-insensitive text).
    pub fn sort_by(mut self, cmp: impl Fn(&str, &str) -> Ordering + 'static) -> Self {
        self.sort_cmp = Some(Box::new(cmp));
        self
    }
}

/// Hit position inside the table. Row indices are display positions
/// (post-sort), column indices match the column order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableHit {
    Header(usize),
    Cell(usize, usize),
}

struct Editing {
    row: usize,
    col: usize,
    field: BasicTextField,
}

/// Basic table: header with click-to-sort, scrollable row pills,
/// optional Ctrl/Shift multi-select and inline cell editing.
///
/// Row indices in the public API are display positions. Selection
/// tracks content across sorts; sorting keeps the selected rows
/// selected. The body always fills the placed rect: columns stretch
/// by weight, and leftover height draws empty outline pills like the
/// reference. Overflow scrolls: rows through the integrated
/// `Scrollbar` (wheel, thumb drag, track page jump), wide columns
/// through a slim horizontal bar at the bottom.
pub struct BasicTable {
    columns: Vec<TableColumn>,
    rows: Vec<Vec<String>>,
    order: Vec<usize>,
    sort_col: Option<usize>,
    ascending: bool,
    selectable: bool,
    selected: HashSet<usize>,
    anchor: Option<usize>,
    ctrl: bool,
    shift: bool,
    vbar: Scrollbar,
    hoff: f32,
    content_w: f32,
    h_drag: Option<f32>,
    h_hover: bool,
    col_widths: Vec<f32>,
    col_offsets: Vec<f32>,
    editing: Option<Editing>,
    edit_on_double_click: bool,
    on_sort: Option<Box<dyn FnMut(usize, bool)>>,
    on_select: Option<Box<dyn FnMut(Vec<usize>)>>,
    on_edit_request: Option<Box<dyn FnMut(usize, usize)>>,
    on_edit_commit: Option<Box<dyn FnMut(usize, usize, String)>>,
    accent: Color,
    dark: bool,
    focused: bool,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    last_click: Option<(usize, usize, Instant)>,
    hover: Option<TableHit>,
    cell_layouts: HashMap<(usize, usize), Layout<SolidBrush>>,
    header_layouts: Vec<Option<Layout<SolidBrush>>>,
    layout_scale: f32,
    layout_dark: bool,
    last_vmodel: (f32, f32),
    last_hmodel: (f32, f32),
}

impl BasicTable {
    pub fn new(columns: Vec<TableColumn>, rows: Vec<Vec<String>>) -> Self {
        let mut table = Self {
            columns,
            rows: Vec::new(),
            order: Vec::new(),
            sort_col: None,
            ascending: true,
            selectable: false,
            selected: HashSet::new(),
            anchor: None,
            ctrl: false,
            shift: false,
            vbar: Scrollbar::new(),
            hoff: 0.0,
            content_w: 0.0,
            h_drag: None,
            h_hover: false,
            col_widths: Vec::new(),
            col_offsets: Vec::new(),
            editing: None,
            edit_on_double_click: true,
            on_sort: None,
            on_select: None,
            on_edit_request: None,
            on_edit_commit: None,
            accent: Color::from_rgb8(0x00, 0x7a, 0xff),
            dark: true,
            focused: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            last_click: None,
            hover: None,
            cell_layouts: HashMap::new(),
            header_layouts: Vec::new(),
            layout_scale: 0.0,
            layout_dark: true,
            last_vmodel: (-1.0, -1.0),
            last_hmodel: (-1.0, -1.0),
        };
        table.set_rows(rows);
        table
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        if !selectable {
            self.selected.clear();
            self.anchor = None;
        }
        self
    }

    /// Double-click starts inline editing on editable columns
    /// (default `true`). Manual `begin_edit` always works.
    pub fn edit_on_double_click(mut self, enabled: bool) -> Self {
        self.edit_on_double_click = enabled;
        self
    }

    /// Fires with `(column, ascending)` whenever the sort changes
    /// through a header click or `sort_by_column`.
    pub fn on_sort(mut self, callback: impl FnMut(usize, bool) + 'static) -> Self {
        self.on_sort = Some(Box::new(callback));
        self
    }

    /// Fires with the sorted selected display positions whenever the
    /// selection changes.
    pub fn on_select(mut self, callback: impl FnMut(Vec<usize>) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Fires with `(row, col)` when inline editing begins.
    pub fn on_edit_request(mut self, callback: impl FnMut(usize, usize) + 'static) -> Self {
        self.on_edit_request = Some(Box::new(callback));
        self
    }

    /// Fires with `(row, col, value)` when an edit commits.
    pub fn on_edit_commit(mut self, callback: impl FnMut(usize, usize, String) + 'static) -> Self {
        self.on_edit_commit = Some(Box::new(callback));
        self
    }

    /// Live theme: selection tint (accent) plus mode grays. Forwarded
    /// to the scroll bar and the inline editor.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        self.dark = dark;
        self.vbar.set_theme(accent, dark);
        if let Some(edit) = self.editing.as_mut() {
            edit.field.set_theme(accent, dark);
        }
        self.invalidate_layouts();
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.vbar.set_focused(focused);
        if let Some(edit) = self.editing.as_mut() {
            edit.field.set_focused(focused);
        }
        self.invalidate_layouts();
    }

    /// Modifier state from the shell (`App::set_modifiers`): Ctrl
    /// toggles single rows, Shift selects the range from the anchor.
    pub fn set_modifiers(&mut self, ctrl: bool, shift: bool) {
        self.ctrl = ctrl;
        self.shift = shift;
    }

    pub fn set_selectable(&mut self, selectable: bool) {
        self.selectable = selectable;
        if !selectable {
            self.selected.clear();
            self.anchor = None;
        }
    }

    /// Replace all rows (ragged rows are padded/truncated to the
    /// column count). Clears selection and editing, keeps and
    /// reapplies the current sort silently.
    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        let cols = self.columns.len();
        self.rows = rows
            .into_iter()
            .map(|mut row| {
                row.resize_with(cols, String::new);
                row.truncate(cols);
                row
            })
            .collect();
        self.order = (0..self.rows.len()).collect();
        self.selected.clear();
        self.anchor = None;
        self.editing = None;
        self.last_click = None;
        self.apply_sort();
        self.clamp_offsets();
        self.invalidate_layouts();
    }

    /// Write one cell directly (no callback). Out-of-range indices
    /// are ignored.
    pub fn set_cell(&mut self, row: usize, col: usize, value: impl Into<String>) {
        if let Some(data) = self.order.get(row).and_then(|&d| self.rows.get_mut(d)) {
            if let Some(cell) = data.get_mut(col) {
                *cell = value.into();
                self.cell_layouts.remove(&(row, col));
            }
        }
    }

    pub fn cell_value(&self, row: usize, col: usize) -> &str {
        self.order
            .get(row)
            .and_then(|&d| self.rows.get(d))
            .and_then(|r| r.get(col))
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn row_count(&self) -> usize {
        self.order.len()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn is_cell_editable(&self, row: usize, col: usize) -> bool {
        row < self.order.len() && self.columns.get(col).is_some_and(|c| c.editable)
    }

    /// Selected display positions, ascending.
    pub fn selected_rows(&self) -> Vec<usize> {
        let mut out: Vec<usize> = self
            .order
            .iter()
            .enumerate()
            .filter(|(_, &d)| self.selected.contains(&d))
            .map(|(pos, _)| pos)
            .collect();
        out.sort_unstable();
        out
    }

    pub fn select_rows(&mut self, rows: &[usize]) {
        if !self.selectable {
            return;
        }
        let next: HashSet<usize> = rows
            .iter()
            .filter_map(|&pos| self.order.get(pos).copied())
            .collect();
        if next != self.selected {
            self.selected = next;
            self.anchor = rows.last().copied();
            self.notify_select();
        }
    }

    pub fn clear_selection(&mut self) {
        if !self.selected.is_empty() {
            self.selected.clear();
            self.anchor = None;
            self.notify_select();
        }
    }

    pub fn sort_state(&self) -> Option<(usize, bool)> {
        self.sort_col.map(|col| (col, self.ascending))
    }

    /// Sort programmatically (fires `on_sort`). Returns false when
    /// the column is missing or not sortable.
    pub fn sort_by_column(&mut self, col: usize, ascending: bool) -> bool {
        if self.columns.get(col).is_none_or(|c| !c.sortable) {
            return false;
        }
        self.sort_col = Some(col);
        self.ascending = ascending;
        self.apply_sort();
        self.notify_sort(col, ascending);
        true
    }

    pub fn clear_sort(&mut self) {
        if self.sort_col.is_some() {
            self.sort_col = None;
            self.order = (0..self.rows.len()).collect();
            self.clamp_offsets();
            self.invalidate_layouts();
        }
    }

    /// Start inline editing (fires `on_edit_request`). Works on
    /// editable columns only; commits a running edit first. Returns
    /// false when the cell cannot be edited.
    pub fn begin_edit(&mut self, row: usize, col: usize) -> bool {
        if !self.is_cell_editable(row, col) {
            return false;
        }
        if let Some(edit) = self.editing.as_ref() {
            if edit.row == row && edit.col == col {
                return true;
            }
            self.commit_edit();
        }
        let mut field = BasicTextField::new("");
        field.set_text(self.cell_value(row, col));
        field.set_theme(self.accent, self.dark);
        field.set_focused(self.focused);
        let (fx, fy, fw, fh) = self.edit_rect(row, col);
        field.place(&mut FontSystem::new(), fx, fy, fw, fh);
        field.mouse_down(((fx + fw / 2.0) as f64), ((fy + fh / 2.0) as f64));
        self.editing = Some(Editing { row, col, field });
        self.notify_edit_request(row, col);
        true
    }

    /// Commit the running edit into the cell (fires
    /// `on_edit_commit`). Returns false when not editing.
    pub fn commit_edit(&mut self) -> bool {
        let Some(edit) = self.editing.take() else {
            return false;
        };
        let value = edit.field.text_value().to_string();
        if let Some(data) = self.order.get(edit.row).and_then(|&d| self.rows.get_mut(d)) {
            if let Some(cell) = data.get_mut(edit.col) {
                if *cell != value {
                    *cell = value.clone();
                }
            }
        }
        self.cell_layouts.remove(&(edit.row, edit.col));
        self.last_click = None;
        self.notify_edit_commit(edit.row, edit.col, value);
        true
    }

    /// Discard the running edit. Returns false when not editing.
    pub fn cancel_edit(&mut self) -> bool {
        if self.editing.is_some() {
            self.editing = None;
            self.last_click = None;
            true
        } else {
            false
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    pub fn editing_cell(&self) -> Option<(usize, usize)> {
        self.editing.as_ref().map(|e| (e.row, e.col))
    }

    /// Type printable text into the running edit (the app forwards
    /// its `text` here while editing).
    pub fn type_text(&mut self, content: &str) {
        if let Some(edit) = self.editing.as_mut() {
            edit.field.type_text(content);
            self.cell_layouts.remove(&(edit.row, edit.col));
        }
    }

    /// Key handling for the running edit: Enter commits, Escape
    /// cancels, the rest goes to the field. Returns true when
    /// consumed. The app forwards its `key` here.
    pub fn key(&mut self, key: Key) -> bool {
        if self.editing.is_none() {
            return false;
        }
        match key {
            Key::Enter => {
                self.commit_edit();
                true
            }
            Key::Escape => {
                self.cancel_edit();
                true
            }
            _ => {
                if let Some(edit) = self.editing.as_mut() {
                    edit.field.key(key);
                    self.cell_layouts.remove(&(edit.row, edit.col));
                }
                true
            }
        }
    }

    /// Hit position for a point in logical px (header, cell or
    /// `None`). Used by apps to wire context menus: right-click,
    /// map with `cell_at`, open the menu, call `begin_edit` from
    /// the action.
    pub fn cell_at(&self, x: f64, y: f64) -> Option<TableHit> {
        let (x, y) = (x as f32, y as f32);
        if x < self.x || x > self.x + self.width || y < self.y || y > self.y + self.height {
            return None;
        }
        if y < self.y + TABLE_HEADER_H {
            return self.col_at(x).map(TableHit::Header);
        }
        let (row, _) = self.row_at(y)?;
        self.col_at(x).map(|col| TableHit::Cell(row, col))
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn scroll_offset(&self) -> f32 {
        self.vbar.offset()
    }

    pub fn horizontal_offset(&self) -> f32 {
        self.hoff
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let (x32, y32) = (x as f32, y as f32);
        self.vbar.mouse_down(x, y);
        if self.hbar_hit(x32, y32) {
            self.h_drag = Some(x32 - self.hbar_thumb_x());
            self.last_click = None;
            return;
        }
        if self.over_vbar(x32, y32) {
            self.last_click = None;
            return;
        }
        if let Some(edit) = self.editing.as_ref() {
            let (fx, fy, fw, fh) = edit.field.rect();
            if x32 >= fx && x32 <= fx + fw && y32 >= fy && y32 <= fy + fh {
                if let Some(e) = self.editing.as_mut() {
                    e.field.mouse_down(x, y);
                }
                return;
            }
            self.commit_edit();
        }
        match self.cell_at(x, y) {
            Some(TableHit::Header(col)) => {
                self.last_click = None;
                self.click_header(col);
            }
            Some(TableHit::Cell(row, col)) => self.click_cell(row, col),
            None => {
                self.last_click = None;
                if self.selectable && self.in_body(x32, y32) {
                    self.clear_selection();
                }
            }
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.vbar.mouse_up(x, y);
        self.h_drag = None;
    }

    /// Scroll wheel delta in logical px (right/down positive, like
    /// the shell): vertical through the bar, horizontal through the
    /// bottom bar. The app forwards its `mouse_wheel` here.
    pub fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        self.vbar.mouse_wheel(dx, dy);
        if self.hscrollable() {
            self.hoff = (self.hoff + dx as f32).clamp(0.0, self.max_hoff());
        }
    }

    fn click_header(&mut self, col: usize) {
        if self.columns.get(col).is_none_or(|c| !c.sortable) {
            return;
        }
        let ascending = if self.sort_col == Some(col) {
            !self.ascending
        } else {
            true
        };
        self.sort_col = Some(col);
        self.ascending = ascending;
        self.apply_sort();
        self.notify_sort(col, ascending);
    }

    fn click_cell(&mut self, row: usize, col: usize) {
        if self.selectable {
            if self.ctrl {
                let data = self.order[row];
                if self.selected.contains(&data) {
                    self.selected.remove(&data);
                } else {
                    self.selected.insert(data);
                }
                self.anchor = Some(row);
                self.notify_select();
            } else if self.shift {
                let anchor = self.anchor.unwrap_or(row);
                let (lo, hi) = if anchor <= row {
                    (anchor, row)
                } else {
                    (row, anchor)
                };
                let next: HashSet<usize> =
                    (lo..=hi).filter_map(|pos| self.order.get(pos).copied()).collect();
                if next != self.selected {
                    self.selected = next;
                    self.notify_select();
                }
            } else {
                let data = self.order[row];
                let single: HashSet<usize> = [data].into_iter().collect();
                if self.selected != single {
                    self.selected = single;
                    self.notify_select();
                }
                self.anchor = Some(row);
            }
        }
        let now = Instant::now();
        let double = matches!(self.last_click, Some((r, c, t))
            if r == row && c == col && now.duration_since(t).as_secs_f64() < GESTURE_DOUBLE_TAP_SECONDS);
        if double {
            self.last_click = None;
            if self.edit_on_double_click {
                self.begin_edit(row, col);
            }
        } else {
            self.last_click = Some((row, col, now));
        }
    }

    fn apply_sort(&mut self) {
        let Some(col) = self.sort_col else {
            return;
        };
        if col >= self.columns.len() {
            self.sort_col = None;
            return;
        }
        let ascending = self.ascending;
        let custom = self.columns[col].sort_cmp.as_ref();
        let mut order = std::mem::take(&mut self.order);
        if let Some(cmp) = custom {
            order.sort_by(|&a, &b| {
                let ra = self.rows.get(a).and_then(|r| r.get(col)).map(String::as_str).unwrap_or("");
                let rb = self.rows.get(b).and_then(|r| r.get(col)).map(String::as_str).unwrap_or("");
                let ord = cmp(ra, rb);
                if ascending { ord } else { ord.reverse() }
            });
        } else {
            order.sort_by(|&a, &b| {
                let ra = self.rows.get(a).and_then(|r| r.get(col)).map(String::as_str).unwrap_or("");
                let rb = self.rows.get(b).and_then(|r| r.get(col)).map(String::as_str).unwrap_or("");
                let ord = smart_cmp(ra, rb);
                if ascending { ord } else { ord.reverse() }
            });
        }
        // Stable ties keep insertion order: `sort_by` is stable and
        // `order` starts in insertion order after `set_rows`.
        self.order = order;
        self.clamp_offsets();
        self.invalidate_layouts();
    }

    fn notify_sort(&mut self, col: usize, ascending: bool) {
        if let Some(callback) = self.on_sort.as_mut() {
            callback(col, ascending);
        }
    }

    fn notify_select(&mut self) {
        let rows = self.selected_rows();
        if let Some(callback) = self.on_select.as_mut() {
            callback(rows);
        }
    }

    fn notify_edit_request(&mut self, row: usize, col: usize) {
        if let Some(callback) = self.on_edit_request.as_mut() {
            callback(row, col);
        }
    }

    fn notify_edit_commit(&mut self, row: usize, col: usize, value: String) {
        if let Some(callback) = self.on_edit_commit.as_mut() {
            callback(row, col, value);
        }
    }

    fn invalidate_layouts(&mut self) {
        self.cell_layouts.clear();
        self.header_layouts.clear();
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

    fn row_fill(&self) -> Color {
        self.eff(if self.dark {
            TABLE_ROW_FILL_DARK
        } else {
            TABLE_ROW_FILL_LIGHT
        })
    }

    fn divider_color(&self) -> Color {
        self.eff(if self.dark {
            TABLE_DIVIDER_DARK
        } else {
            TABLE_DIVIDER_LIGHT
        })
    }

    fn rows_y(&self) -> f32 {
        self.y + TABLE_HEADER_H + 4.0
    }

    fn rows_h(&self) -> f32 {
        let hbar = if self.hscrollable() { TABLE_HBAR_H + 4.0 } else { 0.0 };
        (self.height - TABLE_HEADER_H - 4.0 - hbar).max(0.0)
    }

    fn advance(&self) -> f32 {
        TABLE_ROW_H + TABLE_ROW_GAP
    }

    fn total_rows_h(&self) -> f32 {
        if self.order.is_empty() {
            0.0
        } else {
            self.order.len() as f32 * self.advance() - TABLE_ROW_GAP
        }
    }

    fn layout_columns(&mut self) {
        let n = self.columns.len();
        self.col_widths = vec![0.0; n];
        self.col_offsets = vec![0.0; n];
        if n == 0 {
            self.content_w = 0.0;
            return;
        }
        let gaps = TABLE_COL_GAP * (n as f32 - 1.0);
        let total_weight: f32 = self.columns.iter().map(|c| c.weight).sum::<f32>().max(0.1);
        let mut x = 0.0;
        for (i, col) in self.columns.iter().enumerate() {
            let w = ((self.width - gaps) * col.weight / total_weight).max(col.min_width);
            self.col_widths[i] = w;
            self.col_offsets[i] = x;
            x += w + TABLE_COL_GAP;
        }
        self.content_w = x - TABLE_COL_GAP;
        self.hoff = self.hoff.clamp(0.0, self.max_hoff());
    }

    fn max_hoff(&self) -> f32 {
        (self.content_w - self.width).max(0.0)
    }

    fn hscrollable(&self) -> bool {
        self.content_w > self.width + 0.5 && self.width > 0.0
    }

    fn clamp_offsets(&mut self) {
        self.hoff = self.hoff.clamp(0.0, self.max_hoff());
    }

    fn in_body(&self, x: f32, y: f32) -> bool {
        x >= self.x
            && x <= self.x + self.width
            && y >= self.rows_y()
            && y <= self.rows_y() + self.rows_h()
    }

    fn over_vbar(&self, x: f32, y: f32) -> bool {
        if !self.vbar.scrollable() {
            return false;
        }
        let bx = self.x + self.width - TABLE_BAR_W;
        x >= bx - 2.0
            && x <= self.x + self.width + 2.0
            && y >= self.rows_y()
            && y <= self.rows_y() + self.rows_h()
    }

    fn col_at(&self, x: f32) -> Option<usize> {
        let cx = x - self.x + self.hoff;
        for (i, (&off, &w)) in self.col_offsets.iter().zip(self.col_widths.iter()).enumerate() {
            if cx >= off && cx <= off + w {
                return Some(i);
            }
        }
        None
    }

    fn row_at(&self, y: f32) -> Option<(usize, f32)> {
        let ry = self.rows_y();
        if y < ry || y > ry + self.rows_h() {
            return None;
        }
        let voff = self.vbar.offset();
        let idx = ((y - ry + voff) / self.advance()).floor() as usize;
        if idx >= self.order.len() {
            return None;
        }
        let top = ry + idx as f32 * self.advance() - voff;
        // Clicks in the gap between pills miss.
        if y > top + TABLE_ROW_H {
            return None;
        }
        Some((idx, top))
    }

    fn edit_rect(&self, row: usize, col: usize) -> (f32, f32, f32, f32) {
        let voff = self.vbar.offset();
        let top = self.rows_y() + row as f32 * self.advance() - voff;
        let left = self.x + self.col_offsets.get(col).copied().unwrap_or(0.0) - self.hoff;
        let w = self.col_widths.get(col).copied().unwrap_or(0.0);
        (left + 2.0, top + 2.0, (w - 4.0).max(20.0), TABLE_ROW_H - 4.0)
    }

    fn hbar_rect(&self) -> Rect {
        Rect::new(
            self.x as f64,
            (self.y + self.height - TABLE_HBAR_H) as f64,
            (self.x + self.width) as f64,
            (self.y + self.height) as f64,
        )
    }

    fn hbar_thumb_w(&self) -> f32 {
        if !self.hscrollable() {
            return 0.0;
        }
        (self.width * self.width / self.content_w).clamp(24.0, self.width)
    }

    fn hbar_thumb_x(&self) -> f32 {
        let max = self.max_hoff();
        if max <= 0.0 {
            return self.x;
        }
        self.x + (self.width - self.hbar_thumb_w()) * (self.hoff / max)
    }

    fn hbar_hit(&self, x: f32, y: f32) -> bool {
        if !self.hscrollable() {
            return false;
        }
        let r = self.hbar_rect();
        x as f64 >= r.x0 - 2.0
            && x as f64 <= r.x1 + 2.0
            && y as f64 >= r.y0 - 2.0
            && y as f64 <= r.y1 + 2.0
    }

    fn sync_bars(&mut self) {
        let vmodel = (self.total_rows_h(), self.rows_h());
        if vmodel != self.last_vmodel {
            self.last_vmodel = vmodel;
            self.vbar.set_content(vmodel.0, vmodel.1);
        }
        self.vbar.set_rect(
            self.x + (self.width - TABLE_BAR_W).max(0.0),
            self.rows_y(),
            TABLE_BAR_W.min(self.width).max(0.0),
            self.rows_h(),
        );
        let hmodel = (self.content_w, self.width);
        if hmodel != self.last_hmodel {
            self.last_hmodel = hmodel;
            self.clamp_offsets();
        }
    }

    fn ensure_header_layout(&mut self, fonts: &mut FontSystem, col: usize) {
        let rebuild = self.layout_scale != fonts.scale
            || self.layout_dark != self.dark
            || self.header_layouts.get(col).and_then(|o| o.as_ref()).is_none();
        if !rebuild {
            return;
        }
        if self.layout_scale != fonts.scale || self.layout_dark != self.dark {
            self.cell_layouts.clear();
            self.header_layouts.clear();
            self.layout_scale = fonts.scale;
            self.layout_dark = self.dark;
        }
        if self.header_layouts.len() < self.columns.len() {
            self.header_layouts.resize_with(self.columns.len(), || None);
        }
        let title = self.columns.get(col).map(|c| c.title.clone()).unwrap_or_default();
        let layout = fonts.layout_text_weighted(&title, TABLE_HEADER_SIZE, self.eff(self.dim_color()), 600.0, None);
        self.header_layouts[col] = Some(layout);
    }

    fn ensure_cell_layout(&mut self, fonts: &mut FontSystem, row: usize, col: usize, avail: f32) {
        if self.layout_scale != fonts.scale || self.layout_dark != self.dark {
            self.cell_layouts.clear();
            self.header_layouts.clear();
            self.layout_scale = fonts.scale;
            self.layout_dark = self.dark;
        }
        if self.cell_layouts.contains_key(&(row, col)) {
            return;
        }
        let text = self.cell_value(row, col).to_string();
        let color = self.eff(self.text_color());
        let full = fonts.layout_text(&text, TABLE_FONT_SIZE, color, None);
        let (tw, _) = FontSystem::layout_size(&full);
        if tw / fonts.scale <= avail {
            self.cell_layouts.insert((row, col), full);
            return;
        }
        // Truncate with ellipsis to the column width.
        let chars: Vec<char> = text.chars().collect();
        let mut lo = 0usize;
        let mut hi = chars.len();
        let mut best = 0usize;
        while lo <= hi {
            let mid = (lo + hi) / 2;
            let probe: String = chars[..mid].iter().collect::<String>() + "…";
            let layout = fonts.layout_text(&probe, TABLE_FONT_SIZE, color, None);
            let (pw, _) = FontSystem::layout_size(&layout);
            if pw / fonts.scale <= avail {
                best = mid;
                if mid == chars.len() {
                    break;
                }
                lo = mid + 1;
            } else if mid == 0 {
                break;
            } else {
                hi = mid - 1;
            }
        }
        let final_text: String = chars[..best].iter().collect::<String>() + "…";
        let layout = fonts.layout_text(&final_text, TABLE_FONT_SIZE, color, None);
        self.cell_layouts.insert((row, col), layout);
    }

    fn draw_sort_chevron(&self, scene: &mut Scene, cx: f32, cy: f32, up: bool, scale: f32) {
        let w = 7.0;
        let h = 4.5;
        let mut path = BezPath::new();
        if up {
            path.move_to(((cx - w / 2.0) as f64 * scale as f64, (cy + h / 2.0) as f64 * scale as f64));
            path.line_to((cx as f64 * scale as f64, (cy - h / 2.0) as f64 * scale as f64));
            path.line_to(((cx + w / 2.0) as f64 * scale as f64, (cy + h / 2.0) as f64 * scale as f64));
        } else {
            path.move_to(((cx - w / 2.0) as f64 * scale as f64, (cy - h / 2.0) as f64 * scale as f64));
            path.line_to((cx as f64 * scale as f64, (cy + h / 2.0) as f64 * scale as f64));
            path.line_to(((cx + w / 2.0) as f64 * scale as f64, (cy - h / 2.0) as f64 * scale as f64));
        }
        let mut stroke = Stroke::new(1.5 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(self.eff(self.accent)), None, &path);
    }

    fn render(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        self.sync_bars();
        if self.width <= 0.0 || self.height <= 0.0 || self.columns.is_empty() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let voff = self.vbar.offset();

        // Header (fixed, scrolls horizontally with the columns).
        let header_clip = Rect::new(px(self.x), px(self.y), px(self.x + self.width), px(self.y + TABLE_HEADER_H));
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &header_clip);
        for col in 0..self.columns.len() {
            self.ensure_header_layout(fonts, col);
            let left = self.x + self.col_offsets[col] - self.hoff;
            let w = self.col_widths[col];
            if let Some(TableHit::Header(h)) = self.hover {
                if h == col && self.columns[col].sortable {
                    let hover_rect = RoundedRect::new(px(left), px(self.y), px(left + w), px(self.y + TABLE_HEADER_H), px(6.0));
                    let hover = if self.dark {
                        Color::from_rgba8(255, 255, 255, 14)
                    } else {
                        Color::from_rgba8(0, 0, 0, 10)
                    };
                    scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(self.eff(hover)), None, &hover_rect);
                }
            }
            let layout = self.header_layouts[col].as_ref().expect("header layout built");
            let (tw, th) = FontSystem::layout_size(layout);
            let tx = left + TABLE_PAD_X.min(w / 2.0);
            draw_layout(scene, layout, tx, self.y + (TABLE_HEADER_H - th / fonts.scale) / 2.0, fonts.scale);
            if self.sort_col == Some(col) {
                self.draw_sort_chevron(scene, tx + tw / fonts.scale + 10.0, self.y + TABLE_HEADER_H / 2.0, self.ascending, fonts.scale);
            }
            if col + 1 < self.columns.len() {
                let dx = left + w + TABLE_COL_GAP / 2.0;
                let line = Line::new((px(dx), px(self.y + 7.0)), (px(dx), px(self.y + TABLE_HEADER_H - 7.0)));
                scene.stroke(&Stroke::new(1.0 * scale), Affine::IDENTITY, &Brush::Solid(self.divider_color()), None, &line);
            }
        }
        scene.pop_layer();

        // Rows (clipped, scrolled vertically and horizontally).
        let ry = self.rows_y();
        let rh = self.rows_h();
        let rows_clip = Rect::new(px(self.x), px(ry), px(self.x + self.width), px(ry + rh));
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &rows_clip);
        let advance = self.advance();
        let first = (voff / advance).floor().max(0.0) as usize;
        let editing_pos = self.editing.as_ref().map(|e| (e.row, e.col));
        let selected_accent = self.eff(with_alpha(self.accent, TABLE_SELECTED_ALPHA));
        let hover_fill = self.eff(if self.dark {
            Color::from_rgba8(255, 255, 255, 12)
        } else {
            Color::from_rgba8(0, 0, 0, 8)
        });
        let visible: Vec<(usize, usize, f32)> = self
            .order
            .iter()
            .enumerate()
            .skip(first)
            .map(|(pos, &data)| (pos, data, ry + pos as f32 * advance - voff))
            .take_while(|&(_, _, top)| top < ry + rh)
            .filter(|&(_, _, top)| top + TABLE_ROW_H > ry)
            .collect();
        for (pos, data, top) in visible {
            let pill = RoundedRect::new(px(self.x), px(top), px(self.x + self.width), px(top + TABLE_ROW_H), px(TABLE_RADIUS));
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(self.row_fill()), None, &pill);
            if self.selected.contains(&data) {
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(selected_accent), None, &pill);
            }
            if matches!(self.hover, Some(TableHit::Cell(r, _)) if r == pos) {
                scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(hover_fill), None, &pill);
            }
            for col in 0..self.columns.len() {
                if editing_pos == Some((pos, col)) {
                    continue;
                }
                let left = self.x + self.col_offsets[col] - self.hoff;
                let w = self.col_widths[col];
                if left + w < self.x || left > self.x + self.width {
                    continue;
                }
                let avail = (w - TABLE_PAD_X * 2.0).max(10.0);
                self.ensure_cell_layout(fonts, pos, col, avail);
                if let Some(layout) = self.cell_layouts.get(&(pos, col)) {
                    let (_, th) = FontSystem::layout_size(layout);
                    draw_layout(scene, layout, left + TABLE_PAD_X, top + (TABLE_ROW_H - th / fonts.scale) / 2.0, fonts.scale);
                }
            }
        }
        // Empty outline pills fill the leftover space.
        let mut top = ry + self.order.len() as f32 * advance - voff;
        if self.order.is_empty() {
            top = ry;
        } else {
            top += TABLE_ROW_GAP;
            // `total_rows_h` has no trailing gap; next pill starts after one gap.
        }
        while top + TABLE_ROW_H <= ry + rh + 1.0 {
            let pill = RoundedRect::new(px(self.x), px(top), px(self.x + self.width), px(top + TABLE_ROW_H), px(TABLE_RADIUS));
            scene.stroke(&Stroke::new(1.0 * scale), Affine::IDENTITY, &Brush::Solid(self.divider_color()), None, &pill);
            top += advance;
        }
        // Inline editor floats above the edited cell.
        if let Some((erow, ecol)) = editing_pos {
            let (fx, fy, fw, fh) = self.edit_rect(erow, ecol);
            if let Some(edit) = self.editing.as_mut() {
                edit.field.place(fonts, fx, fy, fw, fh);
                edit.field.draw(scene, fonts, images);
            }
        }
        scene.pop_layer();

        self.vbar.draw(scene, fonts, images);

        // Slim horizontal bar, only when the columns overflow.
        if self.hscrollable() {
            let r = self.hbar_rect();
            let track = RoundedRect::new(r.x0, r.y0 + 2.0 * scale, r.x1, r.y1 - 2.0 * scale, 3.0 * scale);
            let track_color = if self.dark {
                Color::from_rgba8(255, 255, 255, 26)
            } else {
                Color::from_rgba8(0, 0, 0, 20)
            };
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(self.eff(track_color)), None, &track);
            let tx = self.hbar_thumb_x();
            let thumb = RoundedRect::new(
                px(tx),
                r.y0 + 2.0 * scale,
                px(tx + self.hbar_thumb_w()),
                r.y1 - 2.0 * scale,
                3.0 * scale,
            );
            let mut base = SCROLLBAR_GRAY;
            if self.h_hover || self.h_drag.is_some() {
                base = self.eff(self.accent);
            } else {
                base = self.eff(base);
            }
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(base), None, &thumb);
        }
    }
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let c = color.to_rgba8();
    Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * alpha).round() as u8)
}

/// Smart sort: both sides numeric compares numerically (`9` before
/// `28`); otherwise case-insensitive text order.
fn smart_cmp(a: &str, b: &str) -> Ordering {
    match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

impl View for BasicTable {
    /// Intrinsic content size (columns by weight/minima, all rows).
    /// Place with bounds and `flex` takes the remaining stack space
    /// instead, so the table fills the window and scrolls inside.
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        let gaps = TABLE_COL_GAP * (self.columns.len() as f32 - 1.0).max(0.0);
        let w: f32 = self.columns.iter().map(|c| c.min_width).sum::<f32>() + gaps;
        (w, TABLE_HEADER_H + 4.0 + self.total_rows_h())
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(0.0);
        self.height = h.max(0.0);
        self.layout_columns();
        self.sync_bars();
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.layout_columns();
        self.render(scene, fonts, images);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.mouse_up(x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.vbar.mouse_move(x as f64, y as f64);
        if let Some(grab) = self.h_drag {
            let max = self.max_hoff();
            let travel = (self.width - self.hbar_thumb_w()).max(1.0);
            self.hoff = ((x - grab - self.x) / travel * max).clamp(0.0, max);
            return;
        }
        self.h_hover = self.hbar_hit(x, y);
        self.hover = match self.cell_at(x as f64, y as f64) {
            hit @ (Some(TableHit::Header(_)) | Some(TableHit::Cell(_, _))) => hit,
            None => None,
        };
    }

    fn flex(&self) -> f32 {
        1.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cols() -> Vec<TableColumn> {
        vec![
            TableColumn::new("Name").weight(1.4),
            TableColumn::new("Age").weight(0.6).min_width(60.0),
            TableColumn::new("City").weight(1.0).editable(true),
        ]
    }

    fn rows() -> Vec<Vec<String>> {
        vec![
            vec!["Alice".into(), "28".into(), "Berlin".into()],
            vec!["Bob".into(), "9".into(), "Munich".into()],
            vec!["Charlie".into(), "34".into(), "Hamburg".into()],
        ]
    }

    fn table() -> BasicTable {
        let mut table = BasicTable::new(cols(), rows()).selectable(true);
        table.place(&mut FontSystem::new(), 0.0, 0.0, 600.0, 400.0);
        table
    }

    #[test]
    fn ragged_rows_normalize_to_column_count() {
        let table = BasicTable::new(
            cols(),
            vec![vec!["Only".into()], vec!["a".into(), "b".into(), "c".into(), "d".into()]],
        );
        assert_eq!(table.cell_value(0, 1), "");
        assert_eq!(table.cell_value(0, 2), "");
        assert_eq!(table.cell_value(1, 2), "c");
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn smart_sort_is_numeric_for_numbers() {
        let mut table = table();
        assert!(table.sort_by_column(1, true));
        assert_eq!(table.cell_value(0, 0), "Bob");
        assert_eq!(table.cell_value(1, 0), "Alice");
        assert_eq!(table.cell_value(2, 0), "Charlie");
    }

    #[test]
    fn header_click_toggles_direction() {
        let mut table = table();
        // Header middle: Name column.
        table.mouse_down(100.0, 15.0);
        assert_eq!(table.sort_state(), Some((0, true)));
        assert_eq!(table.cell_value(0, 0), "Alice");
        table.mouse_down(100.0, 15.0);
        assert_eq!(table.sort_state(), Some((0, false)));
        assert_eq!(table.cell_value(0, 0), "Charlie");
    }

    #[test]
    fn custom_comparator_replaces_smart_sort() {
        let mut table = BasicTable::new(
            vec![TableColumn::new("City").sort_by(|a, b| a.len().cmp(&b.len()))],
            vec![
                vec!["Hamburg".into()],
                vec!["Berlin".into()],
                vec!["Ulm".into()],
            ],
        );
        table.place(&mut FontSystem::new(), 0.0, 0.0, 400.0, 300.0);
        assert!(table.sort_by_column(0, true));
        assert_eq!(table.cell_value(0, 0), "Ulm");
        assert_eq!(table.cell_value(2, 0), "Hamburg");
    }

    #[test]
    fn unsortable_column_ignores_sort() {
        let mut table = BasicTable::new(
            vec![TableColumn::new("Fixed").sortable(false)],
            vec![vec!["b".into()], vec!["a".into()]],
        );
        table.place(&mut FontSystem::new(), 0.0, 0.0, 400.0, 300.0);
        assert!(!table.sort_by_column(0, true));
        assert_eq!(table.sort_state(), None);
        table.mouse_down(50.0, 15.0);
        assert_eq!(table.sort_state(), None);
        assert_eq!(table.cell_value(0, 0), "b");
    }

    #[test]
    fn single_click_selects_one_row() {
        let mut table = table();
        // Second row pill: rows start at y=34, advance 40.
        table.mouse_down(100.0, 34.0 + 40.0 + 10.0);
        assert_eq!(table.selected_rows(), vec![1]);
        table.mouse_down(100.0, 34.0 + 10.0);
        assert_eq!(table.selected_rows(), vec![0]);
    }

    #[test]
    fn ctrl_click_toggles_rows() {
        let mut table = table();
        table.set_modifiers(true, false);
        table.mouse_down(100.0, 34.0 + 10.0);
        table.mouse_down(100.0, 34.0 + 40.0 + 10.0);
        assert_eq!(table.selected_rows(), vec![0, 1]);
        table.mouse_down(100.0, 34.0 + 10.0);
        assert_eq!(table.selected_rows(), vec![1]);
    }

    #[test]
    fn shift_click_selects_range() {
        let mut table = table();
        table.mouse_down(100.0, 34.0 + 10.0);
        table.set_modifiers(false, true);
        table.mouse_down(100.0, 34.0 + 80.0 + 10.0);
        assert_eq!(table.selected_rows(), vec![0, 1, 2]);
    }

    #[test]
    fn selection_tracks_content_across_sorts() {
        let mut table = table();
        table.mouse_down(100.0, 34.0 + 10.0);
        assert_eq!(table.cell_value(0, 0), "Alice");
        table.sort_by_column(1, true);
        // Alice moved to position 1 but stays selected.
        assert_eq!(table.selected_rows(), vec![1]);
        assert_eq!(table.cell_value(1, 0), "Alice");
    }

    #[test]
    fn set_rows_clears_selection() {
        let mut table = table();
        table.mouse_down(100.0, 34.0 + 10.0);
        assert_eq!(table.selected_rows(), vec![0]);
        table.set_rows(rows());
        assert!(table.selected_rows().is_empty());
    }

    #[test]
    fn unselectable_table_ignores_selection() {
        let mut table = BasicTable::new(cols(), rows());
        table.place(&mut FontSystem::new(), 0.0, 0.0, 600.0, 400.0);
        table.mouse_down(100.0, 34.0 + 10.0);
        assert!(table.selected_rows().is_empty());
    }

    #[test]
    fn edit_flow_commits_and_cancels() {
        let mut table = table();
        assert!(!table.begin_edit(0, 0));
        assert!(!table.begin_edit(5, 2));
        assert!(table.begin_edit(0, 2));
        assert_eq!(table.editing_cell(), Some((0, 2)));
        table.type_text("X");
        assert!(table.commit_edit());
        assert_eq!(table.cell_value(0, 2), "BerlinX");
        assert!(!table.is_editing());
        assert!(table.begin_edit(1, 2));
        table.type_text("zzz");
        assert!(table.cancel_edit());
        assert_eq!(table.cell_value(1, 2), "Munich");
    }

    #[test]
    fn enter_commits_and_escape_cancels() {
        let mut table = table();
        assert!(table.begin_edit(0, 2));
        table.type_text("!");
        assert!(table.key(Key::Enter));
        assert_eq!(table.cell_value(0, 2), "Berlin!");
        assert!(table.begin_edit(0, 2));
        table.type_text("?");
        assert!(table.key(Key::Escape));
        assert_eq!(table.cell_value(0, 2), "Berlin!");
    }

    #[test]
    fn double_click_starts_edit_on_editable_cells() {
        let mut table = table();
        // City cell of the first row (third column, x ~ 420).
        table.mouse_down(420.0, 34.0 + 10.0);
        assert!(!table.is_editing());
        table.mouse_down(420.0, 34.0 + 10.0);
        assert_eq!(table.editing_cell(), Some((0, 2)));
    }

    #[test]
    fn cell_at_maps_header_cells_and_misses() {
        let table = table();
        assert!(matches!(table.cell_at(100.0, 15.0), Some(TableHit::Header(0))));
        assert!(matches!(table.cell_at(100.0, 44.0), Some(TableHit::Cell(0, 0))));
        assert_eq!(table.cell_at(100.0, 399.0), None);
        assert_eq!(table.cell_at(700.0, 44.0), None);
    }

    #[test]
    fn wheel_scroll_clamps_and_pages_content() {
        let mut table = BasicTable::new(cols(), (0..50).map(|i| vec![format!("n{i}"), i.to_string(), "x".into()]).collect());
        table.place(&mut FontSystem::new(), 0.0, 0.0, 600.0, 200.0);
        assert!(table.vbar.scrollable());
        table.mouse_wheel(0.0, -100.0);
        assert!(table.scroll_offset() > 0.0);
        table.mouse_wheel(0.0, -100000.0);
        assert_eq!(table.scroll_offset(), table.vbar.max_offset());
        table.mouse_wheel(0.0, 100000.0);
        assert_eq!(table.scroll_offset(), 0.0);
    }

    #[test]
    fn scrolled_cell_at_follows_offset() {
        let mut table = BasicTable::new(cols(), (0..50).map(|i| vec![format!("n{i}"), i.to_string(), "x".into()]).collect());
        table.place(&mut FontSystem::new(), 0.0, 0.0, 600.0, 200.0);
        table.mouse_wheel(0.0, -40.0);
        // One advance scrolled: the top visible pill is row 1.
        assert!(matches!(table.cell_at(100.0, 44.0), Some(TableHit::Cell(1, 0))));
    }
}
