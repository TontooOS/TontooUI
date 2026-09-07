//! ViewThatFits demo — 1:1 aus dem Screenshot
//! 2 Varianten direkt auf dem Background (#1d1d1d / #ececec), nur TontooUI API, Ampeln sichtbar.

use tontooui::prelude::*;
use tontooui::ViewThatFits;
use uikit::style::{Color, Rect, Size, Padding};
use uikit::view::{View, ViewContent};
use uikit::widget::{Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

// Wrap a View as Widget so it can be used inside VStack/HStack
struct ViewWidget(View);
impl Widget for ViewWidget {
    fn id(&self) -> WidgetId { self.0.id() }
    fn to_gtk(&self) -> gtk::Widget { self.0.to_gtk() }
    fn padding(&self) -> Padding { Padding::ZERO }
}

// Simple red box Widget for demo (SF Pro)
struct RedBoxWidget { id: WidgetId, w: f32, h: f32, text: String, alpha: f32 }
impl RedBoxWidget {
    fn new(w: f32, h: f32, text: &str, alpha: f32) -> Self { Self { id: next_widget_id(), w, h, text: text.to_string(), alpha } }
}
impl Widget for RedBoxWidget {
    fn id(&self) -> WidgetId { self.id }
    fn to_gtk(&self) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_size_request(self.w as i32, self.h as i32);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        outer.add_css_class("redbox");
        let bg = format!("rgba(255,59,48,{})", self.alpha);
        uikit::widget::apply_css(&outer, &format!(".redbox {{ background: {bg}; border-radius: 4px; }}"));
        let lbl = gtk::Label::new(Some(&self.text));
        uikit::widget::apply_css(&lbl, "label { color: white; font-family: 'SF Pro Display'; font-size: 10px; font-weight: 600; }");
        lbl.set_halign(gtk::Align::Center);
        lbl.set_valign(gtk::Align::Center);
        outer.append(&lbl);
        outer.upcast()
    }
    fn padding(&self) -> Padding { Padding::ZERO }
}
impl ViewContent for RedBoxWidget {
    fn render(&self, _frame: Rect) -> gtk::Widget { self.to_gtk() }
    fn size_that_fits(&self, _a: Size) -> Size { Size::new(self.w, self.h) }
}

fn red_box(w: f32, h: f32, text: &str, alpha: f32) -> RedBoxWidget { RedBoxWidget::new(w, h, text, alpha) }
fn vtf_view(vtf: ViewThatFits, w: f32, h: f32) -> ViewWidget {
    ViewWidget(View::new(vtf).with_frame(0.0, 0.0, w, h))
}

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(HStack::new().spacing(0.0).child(Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap())))
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(200.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(200.0))
}

fn preview_vertical() -> impl Widget {
    VStack::new().spacing(4.0)
        .child(vtf_view(
            ViewThatFits::vertical()
                .child(red_box(300.0, 30.0, "Size w300 h30", 1.0))
                .child(red_box(200.0, 20.0, "Size w200 h20", 0.7))
                .child(Text::new("Fallback").font_size(11.0).color(Color::from_hex("#a1a1aa").unwrap())),
            320.0, 50.0
        ))
        .child(vtf_view(
            ViewThatFits::vertical()
                .child(red_box(300.0, 30.0, "Size w300 h30", 1.0))
                .child(red_box(200.0, 20.0, "Size w200 h20", 0.7))
                .child(Text::new("Fallback").font_size(11.0).color(Color::from_hex("#a1a1aa").unwrap())),
            320.0, 30.0
        ))
}

fn preview_regular() -> impl Widget {
    VStack::new().spacing(4.0)
        .child(Text::new("Available width 300").font_size(9.0).color(Color::from_hex("#8e8e93").unwrap()))
        .child(vtf_view(
            ViewThatFits::new()
                .child(red_box(300.0, 20.0, "Available width 300", 1.0))
                .child(red_box(200.0, 20.0, "Available width 200", 0.7))
                .child(Text::new("A…w. 100").font_size(10.0))
                .child(Text::new("Fallback").font_size(11.0).color(Color::from_hex("#a1a1aa").unwrap())),
            300.0, 24.0
        ))
        .child(Text::new("Available width 200").font_size(9.0).color(Color::from_hex("#8e8e93").unwrap()))
        .child(vtf_view(
            ViewThatFits::new()
                .child(red_box(300.0, 20.0, "Available width 300", 1.0))
                .child(red_box(200.0, 20.0, "Available width 200", 0.7))
                .child(Text::new("Fallback").font_size(11.0).color(Color::from_hex("#a1a1aa").unwrap())),
            220.0, 24.0
        ))
}

fn main() {
    let mut app = App::new("ViewThatFits", 700, 520);
    let title = Text::new("ViewThatFits").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row = HStack::new().spacing(24.0)
        .child(cell("ViewThatFits Vertical", "A view that adapts to the available vertical space by providing the first chi...", "initializer", preview_vertical()))
        .child(cell("ViewThatFits", "A view that adapts to the available space by providing the first child view...", "initializer", preview_regular()));

    let grid = VStack::new().spacing(28.0).child(row);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
