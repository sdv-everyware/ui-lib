---
title: Scrollbar
role: Shows the user where they are in a long list while it moves, and goes away when it stops.
rust: core/src/list_view.rs
types: ListView, Scroll
---
While the user drags or flings a list, a thin thumb shows beside it: at the top at the first row, at the bottom at the last. 300 ms after the list stopped the thumb fades out. The app does nothing for it but one line in the presenter: the scrollbar reads the list's `ListView`.

## When to use it
- Beside every list that can hold more rows than its slot shows (`#list-item` rows in a `ListView`).
- Not for a list whose rows always fit: it would never show. Not for a value the user sets (that is a `#slider`).

## Quick start
**What.** The thin bar next to the news list that shows where in the list the user is. It appears while the list moves and fades out after.

**Why.** The bar has nothing to remember: it is drawn from the same list as the rows, so it always shows the list's real position.

<!-- example: docs/examples/scrollbar.rs use slot=#news/scrollbar field=news_scrollbar -->
```rust
use everyware_ui::list_view::ListView;
```

<!-- example: docs/examples/scrollbar.rs state slot=#news/scrollbar field=news_scrollbar -->
```rust
/// The list the scrollbar follows (the list item's guide shows its slot and its gestures).
pub list: ListView,
```

<!-- example: docs/examples/scrollbar.rs init slot=#news/scrollbar field=news_scrollbar -->
```rust
list: ListView::new([], LIST_H, ROW_PITCH),
```

<!-- example: docs/examples/scrollbar.rs model slot=#news/scrollbar field=news_scrollbar -->
```rust
// not interactive: it takes no taps
news_scrollbar: Slot = "#news/scrollbar" ["SlotCustomizer"];
```

<!-- example: docs/examples/scrollbar.rs present slot=#news/scrollbar field=news_scrollbar -->
```rust
// the same ListView as the list's line: the thumb moves with its scroll, frame by frame
model.news_scrollbar = scrollbar("#news/scrollbar", &state.list);
```

## How it behaves
- **At rest.** Hidden. A list that was never moved shows no scrollbar.
- **Moving.** While the finger drags the list, or a fling runs, the thumb shows and moves with the content, frame by frame.
- **Fading.** 300 ms after the content stopped (Android's scroll delay) the thumb fades out in 250 ms.
- **Position.** At the first row the thumb is at the top of the track, at the last row at the bottom.
- **Rows that fit.** Nothing scrolls: the thumb stays at the top and never shows.

## From the app
**What.** The app changes the list (new rows), never the scrollbar. The thumb follows at the next frame.

**Why.** New rows give the content a new length: `set_rows` keeps the scroll inside it, so with fewer rows the thumb moves up, and with rows that fit it goes back to the top.

<!-- example: docs/examples/scrollbar.rs change slot=#news/scrollbar field=news_scrollbar -->
```rust
/// The app changes the list, never the scrollbar: new rows give the content a new length and
/// the thumb follows at the next frame. Rows that fit in the slot scroll nowhere: the thumb
/// stays at the top and the scrollbar does not show.
pub fn show_titles(state: &mut State, titles: &[&str]) {
    state.list.set_rows(titles.iter().map(|title| {
        ListItem::new(ListItemKind::Image, *title, "").lines(ListItemLines::Three).size(ListItemSize::Column)
    }));
}
```

## In Figma
A slot of the list's height, 8 wide, beside the list's slot (`#news/scrollbar` next to `#news/list`), with an instance of `#scrollbar` of that `length`. Not inside the list's slot: there it would scroll away with the rows.

## How it works
The variant is `length`, the list's height in pixels (`472`, `744`): `scrollbar(...)` takes it from the `ListView`'s height. The thumb `#scrollbar/thumb` is a vertical progress marker, 0 at the top and 100 at the bottom, set from the scroll every frame. The whole clone fades: opacity 1 while the content moves and `SCROLLBAR_DELAY` (300 ms) after, then 0 with the transition `hide`. harry wakes up when the delay ends; the app sends nothing.

## Traps
- A list height that is not a `length` of the set has no variant: add the length to `components.json` and the design, or give the list a height the set has.
- A scrollbar inside the list's slot scrolls with the content: a slot of its own, next to the list.
- The marker is the shape itself (a rounded rect with the meter), not a frame with children: harry moves a marker's own shape.
- Give `scrollbar(...)` the same `ListView` as `list(...)`, not a copy: a copy does not scroll.

## Example
[docs/examples/scrollbar.rs](../examples/scrollbar.rs): the whole path, from new rows to a drag, a fling to the end and the fade, compiled and run by `everyware_ui_recipes_test`.
