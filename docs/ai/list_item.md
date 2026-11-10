# #list-item

One row of a list, Material 3's list item, with a leading icon or image, a headline, supporting text and a trailing end; rows live in a ListView.

## Figma

- set: `#list-item` (node 2-1133 in the Library)
- property `state`: `default` | `selected` | `disabled`
- property `kind`: `plain` | `icon` | `image` | `toggle` | `chevron` | `text`
- property `lines`: `one` | `two` | `three`
- property `size`: `regular` | `column` | `wide`
- variant string: `state=default, kind=plain, lines=one, size=regular` (the properties in this order; `ListItem::variant()` produces it, never typed by hand)
- parts the code sets: the roles of Material 3's list item: #list-item/title (headline), /subtitle (supporting text, not in lines one), #list-item/pressed opacity (pressed): set by code. Slots #list-item/leading (kind icon) and #list-item/trailing (kinds plain, icon): instances of #icon, the glyph set by code, the trailing one shown or hidden. Kind image: #list-item/image (a picture the app registered, set by code) and #list-item/trailing-text. Kind text: a label (#list-item/title) and its value (#list-item/trailing-text, right aligned), lines one only. lines: one and two (72) for the kinds plain, icon, toggle and chevron, two and three (128, headline and supporting text up to two lines each) for image. Rows of a scrolling list (ListView, harry glue list()): a clipping slot, rows pitch apart, drag/fling as Android
- nodes set by code (`ListRow`, harry/src/slot.rs): `#list-item/title`, `#list-item/subtitle`, `#list-item/trailing-text`, `#list-item/image`, `#list-item/{slot}/{glyph}`, `#list-item/pressed`, `#list-item/{slot}/<…>`
- on the screens now: `#wifi/networks` in #wifi-content (step=networks) (node 3-9); `#news/list` in #overview (node 4-85); `#automations/routines` in #automations-screen (state=hidden) (node 5-111); `#device/state-list` in #device-content (state=default) (node 6-15)
- to place one: a frame named `#<screen>/<name>` with an instance of `#list-item` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out
- select: 160 ms ease-out

## Rust

- module: `core/src/list_item.rs`, type `ListItem`; binding: `harry/src/slot.rs` (`impl Slotted for ListRow`)
- `pub fn new(kind: ListItemKind, title: impl Into<String>, subtitle: impl Into<String>) -> Self`
- `pub fn on(&mut self, gesture: Gesture) -> Option<ListItemEvent>`
- events `ListItemEvent`: `Clicked`
- pub enum `ListItemKind`: The kind property of #list-item: what is at the ends of the row.
  - `Plain`: Text only.
  - `Icon`: An icon before the text.
  - `Toggle`: A toggle at the end.
  - `Chevron`: A chevron at the end: opens something.
  - `Image`: An image before the text (a registered picture, everyware_ui::images) and a trailing text at the end.
  - `Text`: A label and its value (the trailing supporting text of Material 3) at the end, in one line: "Firmware · 0.1.0".
- pub enum `ListItemLines`: The lines property of #list-item: the height of the row.
  - `One`: The headline only (72, the touch target).
  - `Two`: Headline and one line of supporting text (72).
  - `Three`: Headline and supporting text up to two lines each (128); with an image only.
- pub enum `ListItemSize`: The size property of #list-item: the width of the row (a variant has a fixed size).
  - `Regular`: 440 wide: a column of a card.
  - `Column`: 490 wide: a column of a dashboard.
  - `Wide`: 960 wide: a sheet's list.
- pub struct `ListItem`: A row of a list.
  - field `pub kind: ListItemKind`: What is at the ends of the row.
  - field `pub title: String`: First line.
  - field `pub subtitle: String`: Second line.
  - field `pub selected: bool`: Marked as the current one.
  - field `pub press: Pressable`: Pressed and disabled.
  - field `pub leading: Option<&'static str>`: The glyph of `#icon` in the leading slot (kind icon), by its `icon` value.
  - field `pub trailing: Option<&'static str>`: The glyph of `#icon` in the trailing slot (kinds plain and icon); None hides it.
  - field `pub size: ListItemSize`: The width.
  - field `pub lines: ListItemLines`: The height.
  - field `pub image: Option<String>`: The picture of kind image: the key of a registered image; None keeps the placeholder.
  - field `pub trailing_text: String`: The trailing text of kinds image (a source, an age) and text (the value).
  - `pub fn leading(mut self, icon: &'static str) -> Self`: The leading icon (kind icon): a glyph of `#icon`.
  - `pub fn trailing(mut self, icon: Option<&'static str>) -> Self`: The trailing icon: a glyph of `#icon`, or none.
  - `pub fn wide(mut self) -> Self`: The wide row (960).
  - `pub fn size(mut self, size: ListItemSize) -> Self`: The width.
  - `pub fn lines(mut self, lines: ListItemLines) -> Self`: The height.
  - `pub fn image(mut self, key: impl Into<String>) -> Self`: The picture (kind image): the key of a registered image.
  - `pub fn trailing_text(mut self, text: impl Into<String>) -> Self`: The trailing text (kinds image and text).
  - `pub fn set_selected(&mut self, selected: bool)`: Marks or unmarks the row as the selected one.
- pub enum `ListItemEvent`: What a list item reports.
  - `Clicked`: The row was tapped.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::list_item::{ListItem, ListItemKind};
use everyware_ui::list_view::{ListEvent, ListView};

// state.rs: in struct State, and its first value in State::new
pub wifi_networks: ListView,
// wifi_networks: ListView::new([ListItem::new(ListItemKind::Plain, "Title", "Supporting text")], 744.0, 72.0 + 8.0),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
wifi_networks: Slot = "#wifi/networks" interactive ["SlotCustomizer"];

// presenter.rs: after every action
model.wifi_networks = list("#wifi/networks", &state.wifi_networks);

// actions.rs: in the match on the gesture's node name
// the list itself (a drag between rows) or a row, "#wifi/networks/<row>"
_ if node == "#wifi/networks" || node.starts_with("#wifi/networks/") => {
    if let Some(ListEvent::Chosen(row)) = state.wifi_networks.on(row_of("#wifi/networks", node), gesture, Instant::now()) {
        let _ = row; // yours
    }
}
```

A list is a `ListView` of rows (`ListItem`), its slot `interactive`; new rows with `state.wifi_networks.set_rows(...)` keep a tap in progress.

## Live: `#wifi/networks` in smarthome-app (cut from the sources at build time)

```rust
// state.rs:141
/// The list #wifi/networks: one row (ui-lib list item) per network, scrolling.
pub list: ListView,
list: ListView::new([], LIST_H, ROW_PITCH),

// ui_model.rs:216
wifi_networks: Slot = "#wifi/networks" interactive ["SlotCustomizer"];

// presenter.rs:61
model.wifi_networks = list("#wifi/networks", &wifi.list);
```

`kind` is what the row carries: `plain`, `icon` (a leading glyph), `image` (a leading picture), `toggle`, `chevron`, `text` (a value on the right, for label and value rows). `lines` one to three, `size` the width (`regular`, `column` 490 for two columns in a dialog, `wide` 960). `state` only `default`, `selected`, `disabled`: pressed is a layer.

Rows do not stand alone on a screen: a **ListView** (`core/src/list_view.rs`) holds them, scrolls them (drag, fling, Android's physics), selects one and reports a tap; the slot of the list is a clipping frame with preview rows, harry places the rows at their scrolled y.

## In Figma
A list is one slot for all its rows: a frame named `#<screen>/<name>` (for example `#wifi/networks`) as wide as a row (`size=wide` 960, `column` 490) and as high as the rows it shows, with **Clip content** on, and inside it as many preview instances of `#list-item` as fit, one row pitch apart (72 high, 8 between: 80). harry places the real rows at their scrolled position; the preview rows only show the designer the list. The scrollbar is a slot of its own beside the list (`#<screen>/<name>-scrollbar`, an instance of `#scrollbar` of the list's height), not inside it.

## In the app
```rust
// state: the rows and the view (height of the slot, pitch between rows)
pub list: ListView,                                     // ListView::new([], LIST_H, ROW_PITCH)
fn row(network: &Network) -> ListItem {
    ListItem::new(ListItemKind::Icon, network.ssid.clone(), network.security.name()).wide().leading("wifi").trailing(Some("lock"))
}
list.set_rows(networks.iter().map(row));                // keeps pressed and selected by key

// ui_model: the list slot is interactive (a drag may start between the rows), the scrollbar is not
wifi_networks: Slot = "#wifi/networks" interactive ["SlotCustomizer"];
wifi_scrollbar: Slot = "#wifi/scrollbar" ["SlotCustomizer"];

// presenter
model.wifi_networks = list("#wifi/networks", &wifi.list);
model.wifi_scrollbar = scrollbar("#wifi/scrollbar", &wifi.list);

// actions: a gesture on a row ("#wifi/networks/3") or on the list itself (None: between the rows)
if node == "#wifi/networks" || node.starts_with("#wifi/networks/") {
    let row = node.strip_prefix("#wifi/networks/").and_then(|r| r.parse().ok());
    if let Some(ListEvent::Chosen(row)) = wifi.list.on(row, gesture, Instant::now()) {
        // the user tapped row `row`
    }
}
```

## Traps
- A glyph per row is not a variant (harry does not swap a variant inside a list item): every glyph of the stack is in the row at opacity 0 and `leading(icon)` lights one.
- Rows replaced while a finger is down: `set_rows` carries pressed and selected over by the row's key, so the tap still lands.
- The scrollbar is its own slot beside the list; inside the list it would scroll away with the rows.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
