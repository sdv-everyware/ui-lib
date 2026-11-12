---
title: Dropdown
role: A choice of one from a short list that stays closed until the user needs it: the house's mode, a room, a scene.
rust: core/src/dropdown.rs
types: Dropdown, DropdownEvent
---
The user taps the pill, a list opens below it, a tap on an item picks it and closes the list. The pill shows the chosen item; the app gets `Selected(i)` and can show a choice it learned elsewhere.

## When to use it
- One of a few items (up to the rows drawn in the set) where the screen has no room for all of them at once.
- Two to four choices that the user should see side by side: a `#segmented` button. A longer list or one with details per item: a list of `#list-item` rows in a sheet (docs/content/sheet.md). On or off: a `#toggle`.

## Quick start
**What.** The presence choice in the header, At home, Away or Asleep. A tap opens the list, a tap on an item picks it and closes the list, and the app gets the new mode.

**Why.** The closed pill and the open list are one component, so the app asks it what a tap means: open the list, or pick an item.

<!-- example: docs/examples/dropdown.rs use slot=#status/presence field=presence -->
```rust
use everyware_ui::dropdown::{Dropdown, DropdownEvent};
```

<!-- example: docs/examples/dropdown.rs state slot=#status/presence field=presence -->
```rust
pub presence: Dropdown,
```

<!-- example: docs/examples/dropdown.rs init slot=#status/presence field=presence -->
```rust
presence: Dropdown::new(["At home", "Away", "Asleep"]),
```

<!-- example: docs/examples/dropdown.rs model slot=#status/presence field=presence -->
```rust
presence: Slot = "#status/presence" ["SlotCustomizer"];
```

<!-- example: docs/examples/dropdown.rs present slot=#status/presence field=presence -->
```rust
model.presence = slot("#status/presence", &state.presence);
```

<!-- example: docs/examples/dropdown.rs act slot=#status/presence field=presence -->
```rust
// a tap on the pill opens the list, the next one closes it
"#status/presence" => state.presence.on_field(gesture),
// an item of the open list, by its name in the component: #dropdown/item/1, /2, /3
_ if node.starts_with("#dropdown/item/") => {
    let n = node["#dropdown/item/".len()..].parse::<usize>().ok().and_then(|n| n.checked_sub(1));
    if let Some(DropdownEvent::Selected(i)) = n.and_then(|n| state.presence.on_item(n, gesture)) {
        // the list is closed and the pill shows the item
        log::info!("mode {}", state.presence.items[i]); // yours: tell the house
    }
}
```

## How it behaves
- **Open and close.** A tap on the pill opens the list, the next tap on the pill closes it. The list fades in and out; the pill does not change.
- **Pick.** A tap on an item marks it, closes the list and shows the item in the pill. A tap on the item that is already chosen closes the list and reports nothing.
- **Closed list.** The closed list is transparent and takes no taps: a tap where it would be goes to what is below.
- **Outside.** A tap elsewhere does not close the list by itself: the app calls `close()` if it should.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled`; the pill does not open.

## From the app
**What.** The app shows a choice it learned without a tap: the house reports its mode, another panel changed it.

**Why.** The dropdown shows what the app knows. Its state is plain fields: `items`, `selected`, `open`. Leave `selected` alone while the list is open: the user is choosing.

<!-- example: docs/examples/dropdown.rs change slot=#status/presence field=presence -->
```rust
/// The house reports its mode (another panel, a schedule): the pill shows it. Not while the
/// list is open, the user is choosing; a name that is not an item changes nothing.
pub fn mode_reported(state: &mut State, mode: &str) {
    if let Some(i) = state.presence.items.iter().position(|item| item == mode) {
        if !state.presence.open {
            state.presence.selected = i;
        }
    }
}
```

## In Figma
A slot named `#<screen>/<name>` (`#status/presence`) with an instance of `#dropdown`. The slot keeps the closed pill's height (48) and lets the list hang below it (counter-align MIN, no clipping on the slot), so a row of the screen does not grow by the open list. The component is drawn as tall as its open list.

## How it works
`state` is `default` or `disabled`: the variant is only the state. The open list is `#dropdown/list` faded in, the chosen row `#dropdown/item/N/selected` lit; both are opacities, so opening and picking change no variant. The pill's text is `#dropdown/value`, an item's text `#dropdown/item/N/label`. The items take taps only while the list is open. `on_field(gesture)` opens and closes, `on_item(i, gesture)` answers `Selected(i)` when the item changed.

## Traps
- A slot as tall as the component centres the pill half above the row: slot height = the pill, `frameMaskDisabled` on the slot.
- The items are a fixed number of rows in the set: the app fills their texts, it cannot add rows. Items beyond the drawn rows are not shown.
- `#dropdown/item/N` counts from 1, `on_item` from 0: subtract one, as the example does.

## Example
[docs/examples/dropdown.rs](../examples/dropdown.rs): the whole path, from a tap on the pill to the picked item and a mode the house reports, compiled and run by `everyware_ui_recipes_test`.
