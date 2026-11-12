---
title: Icon
role: A small glyph in a row or a tile that tells the user at a glance what it is: the signal of a network, a lock, a chevron.
rust: core/src/list_item.rs
types: ListItem
---
The user sees a 24 px Lucide glyph at the start or the end of a list row: four steps of the Wi-Fi signal, a lock for a network with a password. The app picks the glyph by its name from its own data, and changes it when the data changes (a stronger signal).

## When to use it
- A glyph inside another component: the leading or trailing end of a `#list-item`. A tile shows its symbol the same way, from its own stack (`#tile/symbol/<icon>`).
- A glyph the user taps is an `#icon-button`: there the icon is a variant of the button.
- The large symbol of a room or a device, on its soft tile, is a `#symbol`.
- An icon is never a slot of its own: alone on a screen it is a plain picture of the design.

## Quick start
**What.** The small glyphs in a row of the Wi-Fi list: the signal bars on the left, a lock on the right of a protected network.

**Why.** A row cannot swap its picture, so every glyph is already in it and the app says which one to show.

<!-- example: docs/examples/icon.rs use slot=#wifi/networks field=networks -->
```rust
use everyware_ui::list_item::{ListItem, ListItemKind};
use everyware_ui::list_view::ListView;
```

<!-- example: docs/examples/icon.rs state slot=#wifi/networks field=networks -->
```rust
/// The rows of the list; each row lights its glyphs.
pub networks: ListView,
```

<!-- example: docs/examples/icon.rs init slot=#wifi/networks field=networks -->
```rust
networks: ListView::new([], LIST_H, ROW_PITCH),
```

<!-- example: docs/examples/icon.rs model slot=#wifi/networks field=networks -->
```rust
networks: Slot = "#wifi/networks" interactive ["SlotCustomizer"];
```

<!-- example: docs/examples/icon.rs present slot=#wifi/networks field=networks -->
```rust
// the rows set their glyphs: the chosen one at opacity 1, every other one of the stack at 0
model.networks = list("#wifi/networks", &state.networks);
```

## How it behaves
- **One glyph per end.** `leading` lights one glyph at the start (kind `icon`), `trailing` one at the end (kinds `plain` and `icon`); `trailing(None)` shows nothing there.
- **A new glyph.** A row with another glyph shows it at the next frame, without a fade: the old glyph goes to opacity 0, the new one to 1.
- **The row stays.** A new glyph does not change the row's variant, its place, or whether it is selected.

## From the app
**What.** The app turns its data into glyph names: the signal's bars into one of four `wifi` glyphs, a password into a `lock`. A new scan gives new rows with their glyphs.

**Why.** The glyph is a fact of the app's data, so it is chosen where the row is made. `set_rows` keeps each row's state by its name, so a stronger signal only changes the glyph.

<!-- example: docs/examples/icon.rs change slot=#wifi/networks field=networks -->
```rust
/// The glyph of a signal: four steps of `wifi`, as Android shows them.
pub fn signal_glyph(bars: u8) -> &'static str {
    match bars {
        0 | 1 => "wifi-zero",
        2 => "wifi-low",
        3 => "wifi-high",
        _ => "wifi",
    }
}

/// A new scan: one row per network, the signal before the name, a lock after it when the
/// network has a password (None: no glyph at the end). A stronger signal is a new glyph on
/// the same row: the row keeps its place and its selection by its name.
pub fn show_networks(state: &mut State, networks: &[Network]) {
    state.networks.set_rows(networks.iter().map(|n| {
        let detail = if n.secured { "WPA2" } else { "Open" };
        ListItem::new(ListItemKind::Icon, n.ssid.clone(), detail)
            .leading(signal_glyph(n.bars))
            .trailing(n.secured.then_some("lock"))
            .wide()
    }));
}
```

## In Figma
Inside the component's layer, one instance of `#icon` per glyph the place can show, all at opacity 0 but the preview's, each named `<part>/<glyph>` (`#list-item/leading/wifi`, `#list-item/trailing/lock`). The library's `#list-item` has the stacks already; a screen only places rows.

## How it works
The set `#icon` has one variant per glyph (`icon=wifi`, `icon=lock`, ...). `list_item::GLYPHS` is the list of glyph names, the same as the `icon` values in `components.json` (a test checks it). The binding of a row (`impl Slotted for ListRow` in harry/src/slot.rs) sets the opacity of every `#list-item/leading/<glyph>` and `#list-item/trailing/<glyph>` to 0, and 1 for the chosen one.

## Traps
- A name that is not in `GLYPHS` lights nothing: the end stays empty. To add a glyph, add it to the `icon` values of `components.json`, to `GLYPHS`, and to the stacks of the design, then fetch.
- An instance swap at runtime is a variant change inside a list item, and harry ignores it: opacity only.
- The icon buttons have their own icons (`Icon`, a variant per icon of `#icon-button`): a glyph of a row is not one of them by itself.

## Example
[docs/examples/icon.rs](../examples/icon.rs): the whole path, from a scan to the signal and lock glyphs of each row and a stronger signal on the selected row, compiled and run by `everyware_ui_recipes_test`.
