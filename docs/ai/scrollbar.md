# #scrollbar

The thumb of a scrolling list: a vertical progress marker along a track, faded in while the list moves.

## Figma

- set: `#scrollbar` (node 2-1121 in the Library)
- property `length`: `472` | `744`
- variant string: `length=472` (the properties in this order; `ListView::variant()` produces it, never typed by hand)
- parts the code sets: the thumb of a scrolling list: #scrollbar/thumb vertical progress marker (0..100 along the track), the clone fades out 300 ms after the content stopped (transition hide); one value of length per list height in the design
- on the screens now: `#wifi/scrollbar` in #wifi-content (step=networks) (node 3-16); `#news/scrollbar` in #overview (node 4-92); `#automations/routines-scrollbar` in #automations-screen (state=hidden) (node 5-118); `#device/state-list-scrollbar` in #device-content (state=default) (node 6-22)
- to place one: a frame named `#<screen>/<name>` with an instance of `#scrollbar` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- hide: 250 ms ease-in

## Rust

- module: `core/src/list_view.rs`, type `ListView`
- `pub fn new(rows: impl IntoIterator<Item = R>, view_h: f32, pitch: f32) -> Self`
- `pub fn on(&mut self, row: Option<usize>, gesture: Gesture, now: Instant) -> Option<ListEvent>`
- events `ListEvent`: `Chosen(usize)`
- pub struct `ListView`: A scrolling list of rows (`#list-item` by default, any `Row`).
  - `pub fn horizontal(mut self) -> Self`: The same list, scrolling sideways: rows left to right, the finger's x moves it, `view_h` is the slot's width and `pitch` a row's width plus gap.
  - `pub fn is_horizontal(&self) -> bool`: Whether the rows lie side by side.
  - `pub fn rows(&self) -> &[R]`: The rows, in order.
  - `pub fn row_mut(&mut self, row: usize) -> Option<&mut R>`: A row to change.
  - `pub fn set_rows(&mut self, rows: impl IntoIterator<Item = R>)`: Replaces the rows (a new scan). A row with the same key keeps its pressed and selected state, so a tap in progress survives a refresh between its down and its up (a list fed by a topic refreshes at any moment); the scroll stays within the content.
  - `pub fn pitch(&self) -> f32`: The row pitch (height plus gap).
  - `pub fn view_h(&self) -> f32`: The viewport's height.
  - `pub fn scroll(&self) -> Scroll`: Where the content is (for the harry glue, which steps a fling itself).
  - `pub fn scroll_at(&self, now: Instant) -> f32`: The scroll at `now`.
  - `pub fn content_h(&self) -> f32`: The content's height (rows times pitch; the gap after the last row stays).
  - `pub fn max_scroll(&self) -> f32`: How far the content scrolls at most.
  - `pub fn thumb(&self, now: Instant) -> f32`: The scrollbar's thumb: where (0..100) along the track; 0 when nothing scrolls.
  - `pub fn selected(&self) -> Option<usize>`: The selected row.
  - `pub fn select(&mut self, row: Option<usize>)`: Marks a row as the selected one (none: clears the mark).

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::list_view::ListView;

// state.rs: in struct State, and its first value in State::new
// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
wifi_scrollbar: Slot = "#wifi/scrollbar" ["SlotCustomizer"];

// presenter.rs: after every action
model.wifi_scrollbar = scrollbar("#wifi/scrollbar", &state.list);

// actions.rs: nothing, display only
```

Display only: it follows the `ListView` of its list (`list`) and fades out when the list stops.

## Live: `#wifi/scrollbar` in smarthome-app (cut from the sources at build time)

```rust
// state.rs:141
/// The list #wifi/networks: one row (ui-lib list item) per network, scrolling.
pub list: ListView,
list: ListView::new([], LIST_H, ROW_PITCH),

// ui_model.rs:221
wifi_scrollbar: Slot = "#wifi/scrollbar" ["SlotCustomizer"];

// presenter.rs:62
model.wifi_scrollbar = scrollbar("#wifi/scrollbar", &wifi.list);
```

`length` is the thumb's share of the track, as the design draws it; the position `#scrollbar/thumb` is a vertical progress marker (0..100) the code moves with the list's scroll. The scrollbar has no state of its own: `scrollbar("#x", &list)` reads the `ListView` of the rows and fades the thumb out after the list settles.

## In Figma
A slot of the list's height, 8 wide, beside the list's slot (not inside it: inside, it would scroll away with the rows), named `#<screen>/<name>-scrollbar`, with an instance of `#scrollbar` of that height.

## Traps
- A scrollbar inside the list's slot scrolls with the content: a slot of its own, next to the list.
- The marker is the shape itself (a rounded rect with the meter), not a frame with children: harry moves a marker's own shape.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
