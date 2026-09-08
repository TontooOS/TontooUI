//! Element Review — one TontooUI element at a time.
//!
//! Clicks through every gallery element piece by piece. Answer Ja (approve)
//! or Nein (reject), Skip forward or go Zurück. Every decision is stored in
//! `temp/review/` (`state.json`, `approved/`, `rejected/`), so restarting the
//! app resumes exactly where you stopped — after a fix you re-review the same
//! element.
//!
//! ```bash
//! cargo run --example review
//! ```
//!
//! On Windows loop it with `review.ps1` (rebuilds, relaunches after every
//! decision, stops on Nein/Done/close).
//!
//! Exit codes: `0` = continue (Ja/Skip/Zurueck/close), `4` = stop (Nein).

mod items_a;
mod items_b;
mod items_c;
mod items_d;
mod items_e;

use tontooui::prelude::*;

/// One reviewable gallery element.
pub struct ReviewItem {
    pub id: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub badge: &'static str,
    pub make: fn() -> WidgetNode,
}

/// Stop exit code: tells `review.ps1` to stop looping (Nein).
const EXIT_STOP: i32 = 4;

const STATE_PATH: &str = "temp/review/state.json";
const APPROVED_DIR: &str = "temp/review/approved";
const REJECTED_DIR: &str = "temp/review/rejected";

fn ensure_dirs() {
    for dir in ["temp/review", APPROVED_DIR, REJECTED_DIR] {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::read_to_string(STATE_PATH).is_err() {
        save_state(0, false);
    }
}

/// Parse `"index":N` out of the tiny state file. Missing/corrupt → 0.
fn load_index() -> usize {
    let text = std::fs::read_to_string(STATE_PATH).unwrap_or_default();
    let mut digits = String::new();
    let mut in_number = false;
    let mut seen_key = false;
    // Find `"index"` then read the first number after it.
    if let Some(pos) = text.find("\"index\"") {
        for ch in text[pos..].chars() {
            if ch.is_ascii_digit() {
                digits.push(ch);
                seen_key = true;
                in_number = true;
            } else if in_number {
                break;
            } else if seen_key {
                let _ = seen_key;
            }
        }
    }
    digits.parse::<usize>().unwrap_or(0)
}

fn save_state(index: usize, done: bool) {
    let _ = std::fs::write(
        STATE_PATH,
        format!("{{\"index\":{},\"done\":{}}}", index, done),
    );
}

fn count_files(dir: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}

fn record(dir: &str, id: &str, title: &str, category: &str, badge: &str, desc: &str) {
    let _ = std::fs::write(
        format!("{}/{}.txt", dir, id),
        format!("{} | {} | {} | {}", title, category, badge, desc),
    );
}

fn header(counter: &str) -> impl Widget {
    VStack::new()
        .spacing(2.0)
        .child(Text::new("Element Review").font_size(20.0).bold())
        .child(
            Text::new(counter)
                .font_size(12.0)
                .color(Color::from_rgb(142, 142, 147)),
        )
}

fn done_screen(approved: usize, rejected: usize, total: usize) -> impl Widget {
    let reset = Button::new("Von vorne")
        .style(ButtonStyle::BorderedProminent)
        .on_click(|| {
            save_state(0, false);
            std::process::exit(0);
        });
    VStack::new()
        .spacing(14.0)
        .child(Text::new("Fertig!").font_size(26.0).bold())
        .child(Text::new(format!(
            "{} Elemente reviewed — yes: {}   no: {}",
            total, approved, rejected
        )))
        .child(reset)
}

fn review_screen(item: &ReviewItem, index: usize, total: usize, approved: usize, rejected: usize) -> impl Widget {
    let green = Color::from_rgb(48, 209, 88);
    let red = Color::from_rgb(255, 69, 58);

    let id = item.id;
    let title = item.title;
    let category = item.category;
    let desc = item.desc;
    let badge = item.badge;

    let yes = Button::new("Ja")
        .style(ButtonStyle::BorderedProminent)
        .role(ButtonRole::Confirm)
        .tint(green)
        .on_click(move || {
            record(APPROVED_DIR, id, title, category, badge, desc);
            let next = index + 1;
            save_state(next, next >= total);
            std::process::exit(0);
        });
    let no = Button::new("Nein")
        .style(ButtonStyle::BorderedProminent)
        .role(ButtonRole::Destructive)
        .tint(red)
        .on_click(move || {
            // Stay on this element: after the fix, a restart re-shows it.
            record(REJECTED_DIR, id, title, category, badge, desc);
            save_state(index, false);
            std::process::exit(EXIT_STOP);
        });
    let skip = Button::new("Skip")
        .style(ButtonStyle::Bordered)
        .on_click(move || {
            let next = index + 1;
            save_state(next, next >= total);
            std::process::exit(0);
        });
    let back = Button::new("Zurück")
        .style(ButtonStyle::Bordered)
        .on_click(move || {
            save_state(index.saturating_sub(1), false);
            std::process::exit(0);
        });

    // Fresh preview instance for display (closures are non-capturing fn pointers).
    let preview = (item.make)();

    VStack::new()
        .spacing(12.0)
        .child(header(&format!(
            "{} / {}   |   yes: {}   no: {}",
            index + 1,
            total,
            approved,
            rejected
        )))
        .child(
            Text::new(format!("{}  •  {}", item.category, item.badge))
                .font_size(11.0)
                .color(Color::from_hex("#0A84FF").unwrap()),
        )
        .child(preview)
        .child(Text::new(item.title).font_size(16.0).bold())
        .child(
            Text::new(item.desc)
                .font_size(12.0)
                .color(Color::from_rgb(142, 142, 147))
                .max_width(520.0),
        )
        .child(
            HStack::new()
                .spacing(12.0)
                .child(yes)
                .child(no)
                .child(skip)
                .child(back),
        )
}

fn main() {
    ensure_dirs();

    let mut items: Vec<ReviewItem> = Vec::new();
    items_a::items(&mut items);
    items_b::items(&mut items);
    items_c::items(&mut items);
    items_d::items(&mut items);
    items_e::items(&mut items);
    let total = items.len();

    let approved = count_files(APPROVED_DIR);
    let rejected = count_files(REJECTED_DIR);

    let mut app = App::new("TontooUI Review", 820, 1840);
    app.force_size(820, 1840);
    let index = load_index().min(total);
    if index >= total {
        save_state(total, true);
        app.set_root(done_screen(approved, rejected, total));
    } else {
        app.set_root(review_screen(&items[index], index, total, approved, rejected));
    }
    app.run();
}
