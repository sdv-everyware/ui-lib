---
title: Pages (code)
role: Pages that take turns in one place without tabs: one page deeper slides in, Back slides it out.
rust: core/src/screens.rs
types: Pages, Effect, ScreenTarget
kind: code
---
The user taps a row and the page slides away to the left while the next one comes in from the right. Back slides it out again. This is how Android's Settings goes one page deeper. The app says which page shows; `Pages` remembers the one before it and places every page with the screens' effect, so the switch is display only: harry draws it from its cache and builds no new tree.

## When to use it
- One place on the screen that shows one of a few pages, and the user goes deeper and back: a sheet's settings and the list one of its rows opens, a device and its details, the steps of a setup.
- Screens the user picks from a tab bar: `#tabs` (docs/content/tabs.md). Tabs and pages move with the same effects; tabs add the bar.
- Something over the screen for a moment: a `#sheet` (docs/content/sheet.md). Pages often live inside a sheet's content.

## Quick start
**What.** A settings page with a row that opens a list. A tap on the row slides the list in from the right, Back slides it out.

**Why.** The pages only know which page shows and which showed before. Each page moves from that, so another animation is one line: pick another effect.

<!-- example: docs/examples/pages.rs use slot=#prefs field=pages -->
```rust
use everyware_ui::screens::{Effect, Pages};
use everyware_ui::ScreenView;

/// The pages in their order: the settings, then the list one of its rows opens (the panel's
/// settings sheet and its time zones).
pub const PAGES: [&str; 2] = ["#prefs/page-main", "#prefs/page-timezone"];
/// The component the pages are drawn in (a sheet's content): the pages' places are read from its
/// variant, where the frame that clips them stands at x 0.
pub const CONTENT: (&str, &str) = ("#prefs-content", "state=default");
/// One page deeper slides in from the right, Back slides it out (Material 3's shared axis). With
/// the pages side by side only Slide brings the next page into view.
pub const EFFECT: Effect = Effect::Slide;
```

<!-- example: docs/examples/pages.rs state slot=#prefs field=pages -->
```rust
/// Which page shows and which showed before.
pub pages: Pages,
/// The first page's x in the frame that clips the pages, and its width: the next page lies
/// beside it.
pub first_page: Frame,
```

<!-- example: docs/examples/pages.rs init slot=#prefs field=pages -->
```rust
pages: Pages::default(),
first_page: sizes.part(CONTENT.0, CONTENT.1, PAGES[0]).unwrap_or(Frame { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }),
```

<!-- example: docs/examples/pages.rs model slot=#prefs field=pages -->
```rust
// one field per page, the first one shown at the start
main_page: ScreenView = "#prefs/page-main" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(100.0, None), x: None };
list_page: ScreenView = "#prefs/page-timezone" ["ScreenCustomizer"] = ScreenView::default();
// the row that opens the list, and Back (a list item and a button on a real page)
_row: () = "#prefs/timezone" interactive;
_back: () = "#prefs/back" interactive;
```

<!-- example: docs/examples/pages.rs present slot=#prefs field=pages -->
```rust
// every page from the first page's place: the one that shows is there, a slide moves the
// others by the width of a page
let page = |i: usize| ScreenView::new(state.pages.page(i, EFFECT, state.first_page.width), state.first_page.x);
model.main_page = page(0);
model.list_page = page(1);
```

<!-- example: docs/examples/pages.rs act slot=#prefs field=pages -->
```rust
// the row goes one page deeper, Back comes back; the finger has to lift on it
"#prefs/timezone" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.pages.show(1),
"#prefs/back" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.pages.show(0),
```

## How it behaves
- **Deeper.** `show(1)`: with `Effect::Slide` the new page comes from the right to the first page's place and the old one leaves to the left, both in 300 ms (components.json, `pages`/`switch`, the tabs' time). The old page disappears when the slide ends.
- **Back.** `show(0)`: the same the other way. The direction follows the order of the pages, so a page with a lower number always comes from the left.
- **The page that shows.** `show` of the page that shows changes nothing.
- **Again.** `reset()` when the sheet opens again: the first page shows and nothing slides.
- **Other effects.** With the pages side by side only `Effect::Slide` brings the next page into view. `Effect::Fade`, `Effect::FadeThrough` and `Effect::None` move nothing: each page stays where the design draws it. Use them only for pages drawn on top of each other at one place, as the tabs' screens are.

## From the app
**What.** The app goes back to the first page from what happened, outside a tap: the sheet opens again.

**Why.** A sheet that opens on the page the user left it on is a surprise. `reset()` starts it on the first page without a slide.

<!-- example: docs/examples/pages.rs change slot=#prefs field=pages -->
```rust
/// The sheet opens again: it starts on its first page, and nothing slides.
pub fn reopen(state: &mut State) {
    state.pages.reset();
}
```

## In Figma
Each page is a frame of the app's design with its own name (`#prefs/page-main`, `#prefs/page-timezone`), all of the same size, side by side in the order of the pages, in one frame that clips them. The designer sees every page at once. The pages need no component set of their own.

- Draw every page at full opacity: harry takes no taps on what the design draws transparent. The app hides the pages that do not show.
- Put the frame that clips the pages at x 0 of the component the pages are in (a sheet's content, `#prefs-content`). The app reads the first page's place from that component's variant (`Sizes::part`), and harry takes a page's x in the clipping frame: the two agree only at x 0.

## How it works
`Pages` is `selected` and `previous`. `page(i, effect, width)` asks the effect where page `i` goes (`Effect::target`): an opacity and an offset from the first page's place, each with its transition (none before the first switch and after `reset()`: the pages take their places at once). `ScreenView::new(target, design_x)` adds the offset to the first page's x, and `ScreenCustomizer` moves and fades the page frame by frame. A page at opacity 0 is not drawn and takes no taps.

## Traps
- Give `ScreenView::new` the first page's x for every page, not each page's own x: the page that shows stands where the first page stands.
- That x is the page's x in the frame that clips the pages. `Sizes::frame` measures from the screen: in a sheet that waits behind the screen's edge it gives about 2000, and the sheet opens empty. Read it with `Sizes::part` from the component's variant, as the example does.
- `width` is the distance between two pages in the design: the first page's width when they lie side by side without a gap. With 0 nothing slides.
- Build what a page shows before it slides in, not on the tap that opens it: a new list in that frame makes harry build its tree again and the slide starts late (docs/content/list_item.md).
- Each page needs its own `ScreenView` field: harry does not hand a parent's animation to its children.

## Example
[docs/examples/pages.rs](../examples/pages.rs): the whole path, from the tap on the row to Back and a sheet that opens again, compiled and run by `everyware_ui_recipes_test`.
