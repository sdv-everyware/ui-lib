---
title: List item
role: One row of a list (a story, a network, a setting), which the user scrolls with the others and taps to choose.
rust: core/src/list_item.rs
types: ListItem, ListItemKind, ListItemLines, ListItemSize, ListItemEvent
---
The user sees rows one under the other: a headline, a line of supporting text, at the start an icon or a picture, at the end a glyph, a toggle, a chevron or a short text. The finger drags the rows, a fast lift flings them, a tap chooses one. The app gives the rows from its own data and learns which row was tapped.

## When to use it
- Several items of the same kind, more than fit or of a number that changes: stories, networks, the values of a device. For a few fixed choices use `#radio` or `#segmented`; for a choice that opens from one field, `#dropdown`.
- A control on a dashboard (a light with its switch) is a `#tile`, not a row.
- The rows never stand alone: a `ListView` holds them, scrolls them and reports the tapped row. Beside a list that scrolls put a `#scrollbar` (docs/content/scrollbar.md).

## Quick start
**What.** The news list of the panel: rows with a picture, a headline and a short text. The user scrolls it with a finger, and a tap on a row tells the app which story to open.

**Why.** The list scrolls and highlights by itself. The app only says which rows there are and what a tap on one means.

<!-- example: docs/examples/list_item.rs use slot=#news/list field=news -->
```rust
use everyware_ui::images;
use everyware_ui::list_item::{ListItem, ListItemKind, ListItemLines, ListItemSize};
use everyware_ui::list_view::{ListEvent, ListView};
```

<!-- example: docs/examples/list_item.rs state slot=#news/list field=news -->
```rust
/// The rows and where they are scrolled to: one `ListItem` per story.
pub news: ListView,
```

<!-- example: docs/examples/list_item.rs init slot=#news/list field=news -->
```rust
// empty until the first stories come; the slot's height and the row pitch
news: ListView::new([], LIST_H, ROW_PITCH),
```

<!-- example: docs/examples/list_item.rs model slot=#news/list field=news -->
```rust
news: Slot = "#news/list" interactive ["SlotCustomizer"];
```

<!-- example: docs/examples/list_item.rs present slot=#news/list field=news -->
```rust
model.news = list("#news/list", &state.news);
```

<!-- example: docs/examples/list_item.rs act slot=#news/list field=news -->
```rust
// a row ("#news/list/<row>") or the list between its rows (None): a drag scrolls,
// a fast lift flings, a tap chooses the row
_ if node == "#news/list" || node.starts_with("#news/list/") => {
    let row = row_of("#news/list", node);
    if let Some(ListEvent::Chosen(row)) = state.news.on(row, gesture, Instant::now()) {
        state.news.select(Some(row));
        log::info!("story {row} chosen"); // yours: open it
    }
}
```

## How it behaves
- **Scrolling.** The rows follow the finger once it moved past the touch slop; a fast lift flings them and they slow down as on Android. The list stops at its first and its last row.
- **Catching a fling.** A finger on a running fling stops it where it is. That press is never a tap.
- **Tap.** Down and up on the same row without a drag: the row was chosen. While the finger is down the row shows its pressed layer.
- **Selected.** `select(Some(row))` marks one row (the variant `state=selected`) and clears the others.
- **Scroll to a row.** `scroll_to(row)` puts the row at the top at once, or as near as the content lets it, as `RecyclerView.scrollToPosition` does: a list that opens on the chosen row.
- **Disabled.** `row.press.set_enabled(false)` shows `state=disabled`; a tap does not choose it.
- **Picture.** A row of kind `image` shows the picture registered under its key; until it comes, the design's placeholder.

## From the app
**What.** The app turns its data into rows: here one row per story, and the story's picture when it arrives. It calls this whenever the service sends new data, also while the user scrolls.

**Why.** The rows show what the app knows. `set_rows` replaces them and keeps the pressed and the selected row by its key (the headline), so a tap that started on the old rows still lands on the same story.

<!-- example: docs/examples/list_item.rs change slot=#news/list field=news -->
```rust
/// New stories from the service: one row each, the headline, the summary, the age at the end and
/// the picture by its key. A row keeps its pressed and selected state by its key (the headline),
/// so a tap in progress still lands; the scroll stays inside the new content.
pub fn show_stories(state: &mut State, stories: &[Story]) {
    state.news.set_rows(stories.iter().map(|story| {
        ListItem::new(ListItemKind::Image, story.title.clone(), story.summary.clone())
            .lines(ListItemLines::Three)
            .size(ListItemSize::Column)
            .image(format!("news/{}", story.id))
            .trailing_text(story.age.clone())
    }));
}

/// A story's picture came: registered under its row's key, it shows at the next frame. Until
/// then the row shows the design's placeholder.
pub fn show_picture(id: u64, rgb: &[u8], width: u32, height: u32) {
    if let Err(err) = images::register_rgb(format!("news/{id}"), rgb, width, height) {
        log::warn!("picture of story {id}: {err}");
    }
}
```

## In Figma
One slot for all the rows: a frame named `#<screen>/<name>` (`#news/list`) as wide as a row (`regular` 440, `column` 490, `panel` 856 in a left or right sheet, `wide` 960) and as high as the rows it shows, with **Clip content** on. Inside it as many preview instances of `#list-item` as fit, one row pitch apart (a row of 72 with 8 between is a pitch of 80; three lines are 128). harry places the real rows at their scrolled place; the preview rows only show the designer the list. The scrollbar is a slot of its own beside the list.

## How it works
The variant is `state` (`default`, `selected`, `disabled`), `kind` (`plain`, `icon`, `image`, `toggle`, `chevron`, `text`), `lines` (`one`, `two`, `three`) and `size`. The texts are `#list-item/title`, `#list-item/subtitle` and `#list-item/trailing-text`, the picture `#list-item/image`. Pressed is not a variant: the opacity of `#list-item/pressed`. The glyphs at the start and the end are stacks of `#icon` (`#list-item/leading/<glyph>`, `#list-item/trailing/<glyph>`), every glyph at opacity 0 but the chosen one (docs/content/icon.md); a row without a glyph switches the whole stack off. A row sets only the parts its kind and lines have. `list(...)` puts every row at its y minus the scroll and steps a fling frame by frame itself: no ticks reach the app. A row that is out of the slot is not drawn: at opacity 0 harry leaves it out of its animation pass and its display list. Every row still has its customizations and its nodes in the tree, so a long list costs more than its visible rows: build the rows once, not on the tap that shows them.

## Traps
- A gesture comes on a row (`#news/list/<row>`) or on the list itself (a move or a lift between the rows): match both, as the example does, or a drag that leaves the rows never ends.
- The slot needs `interactive` in the UI model: without it a drag that starts between two rows is lost.
- The key of a row is its headline. Two rows with the same headline share their state across `set_rows`.
- A glyph is not a variant (harry does not swap a variant inside a list item): it is lit by opacity in the row's stack.
- The scrollbar is its own slot beside the list; inside the list it would scroll away with the rows.

## Example
[docs/examples/list_item.rs](../examples/list_item.rs): the whole path, from new stories to a drag, a fling and a tap that selects one, compiled and run by `everyware_ui_recipes_test`.
