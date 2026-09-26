## Repo

Only Englisch in TontooOS Repos

## Finish

Always Compile it with "wsl -d archlinux" or if there is no WSL use cargo check but first build it with cargo

Make Sure to Run / Test new stuff

### AND MAKE SURE TO ALWAYS UPDATE THE WIKI READ THE RULE.md AND MAIN.md!


## IMPORTANT All TontooUI elements should have if there like movable or something a shadow  not evry but most should habe a small shadow


AND The Colors Should be: Light Mode:

Background: R: 255, G: 255, B: 255
Text: R: 039, G: 039, B: 039

Dark Mode:

Background: R: 027, G: 032, B: 034
Text: R: 216, G: 217, B: 217

THIS COLORS ARE FROM MACOS 27 NEWEST VERSION,
The Colors are for Default Text / App Backgrounds 

## Crisp Text Rules (CoreText)

- All text goes through CoreText (`coretext` crate, re-exported from
  `src/renderer/text.rs`). Draw text only via `draw_layout` (solid),
  `draw_with_brush` (gradient) or `draw_frame_mapped` (mixed).
  Never hand-roll a `Scene::draw_glyphs` loop: snapping + hinting
  live in CoreText (`src/render.rs` there).
- Build layouts with logical px (`size`, `max_width`); `FontSystem`
  applies the display scale internally. `layout_size()` returns
  physical px, so divide by `fonts.scale` for logical units.
- Any element caching an `Option<CTFrame>` must store `layout_scale: f32`
  (init `0.0`), rebuild when `layout_scale != fonts.scale`, and store
  `fonts.scale` after building. Otherwise text goes stale/blurry on
  DPI or monitor changes.
- Never pre-multiply draw positions by scale; pass logical coordinates
  and let the draw helpers snap to physical pixels.
- Hit testing and decorations use CoreText (`hit_byte`,
  `decorations()`), never Parley directly. TontooUI must not depend
  on `parley`: it only sees `CTFrame`/`CTLine`.