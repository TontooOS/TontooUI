# Review

Element Review is a QA tool (not a widget): it shows every TontooUI gallery element one at a time so each can be approved or rejected. Decisions persist in `temp/review/`, so restarting the app resumes at the same element — after a fix the same element is re-shown for re-review.

## Run

```bash
cargo run --example review
```

On Windows loop it (rebuild + relaunch after every decision):

```bash
powershell -ExecutionPolicy Bypass -File review.ps1
```

One command via WSL (path included, GUI over WSLg):

```bash
wsl -d archlinux -- bash -lc "cd /mnt/c/Users/arlo1/Documents/TontooLibs/TontooUI && bash review.sh"
```

## Flow

| Button | Effect | Exit code |
|---|---|---|
| `Ja` | Records approval in `temp/review/approved/<id>.txt`, advances | `0` |
| `Nein` | Records rejection in `temp/review/rejected/<id>.txt`, stays on the element | `4` (loop stops) |
| `Skip` | Advances without recording | `0` |
| `Zurück` | Goes back one element | `0` |
| Window close | No record, loop stops | `0` (state unchanged) |

## State Files

| Path | Content |
|---|---|
| `temp/review/state.json` | `{"index":N,"done":false}` — current position |
| `temp/review/approved/<id>.txt` | `Title \| Category \| badge \| desc` |
| `temp/review/rejected/<id>.txt` | Same format for rejected elements |

`temp/` is git-ignored: review data is personal QA state, never committed.

## Registry

```rust
pub struct ReviewItem {
    pub id: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub badge: &'static str,
    pub make: fn() -> WidgetNode,
}
```

Items live in `examples/review/items_*.rs`, one entry per gallery card, in wiki order. `make` is a non-capturing closure so it coerces to a `fn` pointer; `WidgetNode::new` wraps any `Widget` (single elements and `VStack` composites alike).

The window uses `App::force_size(820, 1840)` (UIKit opt-in): it bypasses the default content-size/half-monitor sizing so the review screen always opens tall.

## Fix Loop

1. Press `Nein` on a broken element (rejection saved, app stops).
2. Report `fix <number>: <what>` — the number is the `N / total` counter in the app, mapped in `temp/review/element_list.txt` (regenerate with `python3 temp/gen_list.py` after registry changes). The rejected file in `temp/review/rejected/` carries the same id and title as backup.
3. Fix lands, restart via `review.ps1` — the same element is shown again.
4. Press `Ja` to approve and continue.

## Cross References

- [MAIN.md](MAIN.md) -- full element index the registry mirrors
