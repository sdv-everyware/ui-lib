---
title: List item
role: One row of a list (a story, a network, a setting), which the user scrolls with the others and taps to choose.
rust: core/src/list_item.rs
types: ListItem, ListItemKind, ListItemLines, ListItemSize, ListItemEvent
---
The user sees rows one under the other: a headline, a line of supporting text, at the start an icon, a picture or a short text of two lines with a coloured bar, at the end a glyph, a toggle, a chevron or a short text. A subheader can stand over a group of rows. The finger drags the rows, a fast lift flings them, a tap chooses one. The app gives the rows from its own data and learns which row was tapped.

## When to use it
- Several items of the same kind, more than fit or of a number that changes: stories, networks, the values of a device. For a few fixed choices use `#radio` or `#segmented`; for a choice that opens from one field, `#dropdown`.
- A control on a dashboard (a light with its switch) is a `#tile`, not a row.
- The rows never stand alone: a `ListView` holds them, scrolls them and reports the tapped row. Beside a list that scrolls put a `#scrollbar` (docs/content/scrollbar.md).
- Rows in groups (by day, by letter): a row of kind `subheader` over each group, with its own pitch. A short value that leads the row (a time and a duration, a score) is kind `leading-text`; its bar takes a colour the app gives.

## Quick start
**What.** A list of stories (`#x/stories`, a slot of the app's screen): rows with a picture, a headline and a short text. The user scrolls it with a finger, and a tap on a row tells the app which story to open.

**Why.** The list scrolls and highlights by itself. The app only says which rows there are and what a tap on one means.

<!-- example: docs/examples/list_item.rs use slot=#x/stories field=stories -->
```rust
use everyware_ui::images;
use everyware_ui::list_item::{ListItem, ListItemKind, ListItemLines, ListItemSize};
use everyware_ui::list_view::{ListEvent, ListView};
```

<!-- example: docs/examples/list_item.rs state slot=#x/stories field=stories -->
```rust
/// The rows and where they are scrolled to: one `ListItem` per item of the app's data.
pub stories: ListView,
```

<!-- example: docs/examples/list_item.rs init slot=#x/stories field=stories -->
```rust
// empty until the first data comes; the slot's height and the row pitch
stories: ListView::new([], LIST_H, ROW_PITCH),
```

<!-- example: docs/examples/list_item.rs model slot=#x/stories field=stories -->
```rust
stories: Slot = "#x/stories" interactive ["SlotCustomizer"];
```

<!-- example: docs/examples/list_item.rs present slot=#x/stories field=stories -->
```rust
model.stories = list("#x/stories", &state.stories);
```

<!-- example: docs/examples/list_item.rs act slot=#x/stories field=stories -->
```rust
// a row ("#x/stories/<row>") or the list between its rows (None): a drag scrolls,
// a fast lift flings, a tap chooses the row
_ if node == "#x/stories" || node.starts_with("#x/stories/") => {
    let row = row_of("#x/stories", node);
    if let Some(ListEvent::Chosen(row)) = state.stories.on(row, gesture, Instant::now()) {
        state.stories.select(Some(row));
        log::info!("row {row} chosen"); // yours: open it
    }
}
```

## How it behaves
- **Scrolling.** The rows follow the finger once it moved past the touch slop; a fast lift flings them and they slow down as on Android. The list stops at its first and its last row.
- **Catching a fling.** A finger on a running fling stops it where it is. That press is never a tap.
- **Tap.** Down and up on the same row without a drag: the row was chosen. While the finger is down the row shows its pressed layer. A row scrolled partly out of the list takes a tap on the part that shows, however high it is; a touch beside the list never reaches a row.
- **Selected.** `select(Some(row))` marks one row (the variant `state=selected`) and clears the others.
- **Scroll to a row.** `scroll_to(row)` puts the row at the top at once, or as near as the content lets it, as `RecyclerView.scrollToPosition` does: a list that opens on the chosen row.
- **Disabled.** `row.press.set_enabled(false)` shows `state=disabled`; a tap does not choose it.
- **Picture.** A row of kind `image` shows the picture registered under its key; until it comes, the design's placeholder.
- **Leading text and its bar.** A row of kind `leading-text` shows `leading_text(title, subtitle)` before the headline, the first line strong, the second dim, and a thin bar between them and the headline. `accent(0xRRGGBB)` colours the bar; without it the bar has the design's accent. Its headline is shown whole: `lines(ListItemLines::with_headline(n))` gives it n lines (one to six; the supporting text keeps one), the app counts the lines of its text, and `height()` says how high the row is, for its `pitch`.
- **Subheaders.** A row of kind `subheader` is the label of the rows below it, with a trailing text at the end: no card, 48 high. In a list of taller rows it takes `pitch(48.0 + gap)`: a row with a pitch of its own moves the rows after it by that much, and the scroll, `scroll_to` and the scrollbar count it. It takes no tap (`takes_taps`): it is never pressed or chosen, a finger on it drags the list.
- **Passive rows.** `passive()` makes a row of any kind a line that only tells something ("No stories", "Nothing scheduled"): it looks as a row, takes no tap and shows no pressed layer, as a subheader.

## From the app
**What.** The app turns its data into rows: here one row per story with the story's picture when it arrives, or rows in groups under a subheader each (days, letters). It calls this whenever the service sends new data, also while the user scrolls.

**Why.** The rows show what the app knows. `set_rows` replaces them and keeps the pressed and the selected row by its key, so a tap that started on the old rows still lands on the same row. The key is the row's `id(...)` (the id of the item in the app's data), or its headline when it has none.

<!-- example: docs/examples/list_item.rs change slot=#x/stories field=stories -->
```rust
/// New stories from the service: one row each, the headline, the summary, the age at the end and
/// the picture by its key. A row keeps its pressed and selected state by its id (the story's: two
/// stories may have the same headline), so a tap in progress still lands; the scroll stays inside
/// the new content.
pub fn show_stories(state: &mut State, stories: &[Story]) {
    state.stories.set_rows(stories.iter().map(|story| {
        ListItem::new(ListItemKind::Image, story.title.clone(), story.summary.clone())
            .id(story.id.to_string())
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

/// A subheader is 48 high, a row of two lines 72; 8 between them.
const SUBHEADER_PITCH: f32 = 48.0 + 8.0;
const TWO_LINES_PITCH: f32 = 72.0 + 8.0;

/// Rows in groups: a subheader over each group (its label and a trailing text), then rows that
/// lead with a short text of two lines and a bar in the group's colour (from a palette: each
/// colour stays a picture). Each row takes its own pitch, so the rows after a subheader move up.
/// A line that only tells something ("No stories") is passive: no tap, no pressed look.
pub fn show_groups(state: &mut State, groups: &[Group]) {
    state.stories.set_rows(groups.iter().flat_map(|group| {
        let head = ListItem::new(ListItemKind::Subheader, group.label.clone(), "")
            .trailing_text(group.count.clone())
            .size(ListItemSize::Column)
            .pitch(SUBHEADER_PITCH);
        let empty = group.stories.is_empty().then(|| {
            ListItem::new(ListItemKind::Plain, "No stories", "").lines(ListItemLines::One).size(ListItemSize::Column).pitch(TWO_LINES_PITCH).passive()
        });
        let rows = group.stories.iter().map(|story| {
            ListItem::new(ListItemKind::LeadingText, story.title.clone(), story.summary.clone())
                .id(story.id.to_string())
                .leading_text(story.age.clone(), "ago")
                .accent(group.color)
                .size(ListItemSize::Column)
                .pitch(TWO_LINES_PITCH)
        });
        std::iter::once(head).chain(empty).chain(rows)
    }));
}
```

## In Figma
One slot for all the rows: a frame named `#<screen>/<name>` (the panel's agenda: `#agenda/list`) as wide as a row (`regular` 440, `column` 490, `panel` 856 in a left or right sheet, `wide` 960) and as high as the rows it shows, with **Clip content** on. Inside it as many preview instances of `#list-item` as fit, one row pitch apart (a row of 72 with 8 between is a pitch of 80; three lines are 128; a subheader is 48). harry places the real rows at their scrolled place; the preview rows only show the designer the list. The scrollbar is a slot of its own beside the list.

## How it works
The variant is `state` (`default`, `selected`, `disabled`), `kind` (`plain`, `icon`, `image`, `toggle`, `chevron`, `text`, `leading-text`, `subheader`), `lines` (`one`, `two`, `three`) and `size`. The texts are `#list-item/title`, `#list-item/subtitle` and `#list-item/trailing-text`, with a leading text `#list-item/leading-title` and `#list-item/leading-subtitle`; the picture is `#list-item/image`. The bar of a leading text is `#list-item/accent`, a plain frame in a clip with the corners: harry has no colour to set on a node, so `everyware_ui::images::color` makes the colour a picture of one pixel and the row sets it there (harry draws a picture on a plain frame only). Each colour stays registered, so the colours of the bars come from a palette, not from free data. Pressed is not a variant: the opacity of `#list-item/pressed`. The glyphs at the start and the end are stacks of `#icon` (`#list-item/leading/<glyph>`, `#list-item/trailing/<glyph>`), every glyph at opacity 0 but the chosen one (docs/content/icon.md); a row without a glyph switches the whole stack off. A row sets only the parts its kind and lines have. `list(...)` puts every row at its y minus the scroll and steps a fling frame by frame itself: no ticks reach the app. harry hits a node by its bounds, without clipping, so a row scrolled partly out of the slot lies over what is beside the list. A row takes taps while some of it is in the slot, and `Input` takes a touch on a row only when the list's slot is under the finger too (harry reports the slot after the row); a touch beside the list goes on to what is drawn there. A row that is out of the slot is not drawn: at opacity 0 harry leaves it out of its animation pass and its display list. Every row still has its customizations and its nodes in the tree, so a long list costs more than its visible rows: build the rows once, not on the tap that shows them.

## Traps
- A gesture comes on a row (`#x/stories/<row>`) or on the list itself (a move or a lift between the rows): match both, as the example does, or a drag that leaves the rows never ends.
- The slot needs `interactive` in the UI model: without it a drag that starts between two rows is lost, and no row takes a tap (a row's touch counts only with the slot under it).
- The key of a row is its `id`, or its headline without one. Rows that can repeat a headline (a daily event, two stories of the same title) need `id(...)`, or they share their state across `set_rows`.
- A subheader without its own `pitch` takes the list's, as high as a row: give it `pitch(48.0 + gap)`; a row of more lines likewise `pitch(row.height() + gap)`.
- Every colour given to `accent` stays a picture for the life of the process: map the data's colours to a palette first.
- A glyph is not a variant (harry does not swap a variant inside a list item): it is lit by opacity in the row's stack.
- The scrollbar is its own slot beside the list; inside the list it would scroll away with the rows.

## Example
[docs/examples/list_item.rs](../examples/list_item.rs): the whole path, from new data to a drag, a fling and a tap that selects a row, compiled and run by `everyware_ui_recipes_test`.
