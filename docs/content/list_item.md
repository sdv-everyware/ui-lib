---
title: List item
role: One row of a list, Material 3's list item, with a leading icon or image, a headline, supporting text and a trailing end; rows live in a ListView.
rust: core/src/list_item.rs
types: ListItemKind, ListItemLines, ListItemSize, ListItem, ListItemEvent
---
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
