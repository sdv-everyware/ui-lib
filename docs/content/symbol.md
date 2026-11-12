---
title: Symbol
role: The large symbol of a room or a device (a door, a bed, a house) that tells the user which one a sheet or a card is about.
rust: core/src/symbol.rs
types: Symbol, SymbolIcon
---
The user sees a 48 px tile in the soft accent colour with one glyph in ink, next to the name of a device or a room. The app picks the symbol from what the sheet shows, and gives a new one when the sheet opens for another device. Nothing to tap.

## When to use it
- The symbol of one room or device on a sheet or a card header: `#device/symbol` on the device sheet.
- A small glyph in a list row is an `#icon` (a symbol is a variant per glyph, which a list item cannot change); a tile has its own symbol stack.
- A symbol the user taps is an `#icon-button` with that icon.

## Quick start
**What.** The large round symbol at the top of the device sheet: a house for the climate, a door for the entry.

**Why.** The symbol changes rarely, when another device is opened, so the app just names the one to show.

<!-- example: docs/examples/symbol.rs use slot=#device/symbol field=device_symbol -->
```rust
use everyware_ui::symbol::{Symbol, SymbolIcon};
```

<!-- example: docs/examples/symbol.rs state slot=#device/symbol field=device_symbol -->
```rust
pub device_symbol: Symbol,
```

<!-- example: docs/examples/symbol.rs init slot=#device/symbol field=device_symbol -->
```rust
device_symbol: Symbol::new(SymbolIcon::House),
```

<!-- example: docs/examples/symbol.rs model slot=#device/symbol field=device_symbol -->
```rust
device_symbol: Slot = "#device/symbol" ["SlotCustomizer"];
```

<!-- example: docs/examples/symbol.rs present slot=#device/symbol field=device_symbol -->
```rust
model.device_symbol = slot("#device/symbol", &state.device_symbol);
```

## How it behaves
- **Still.** The tile does not move and takes no taps; a tap reaches what lies under it.
- **Another symbol.** A new `Symbol` shows its variant at the next frame.

## From the app
**What.** The app gives the symbol of what the screen shows: here the device the sheet opens for.

**Why.** The symbol belongs to the device in the app's data (`SymbolIcon`), so it comes with the device and not from a tap on the symbol.

<!-- example: docs/examples/symbol.rs change slot=#device/symbol field=device_symbol -->
```rust
/// The sheet opens for another device: its symbol comes with it. The next frame shows the
/// variant of that symbol.
pub fn open_device(state: &mut State, icon: SymbolIcon) {
    state.device_symbol = Symbol::new(icon);
}
```

## In Figma
A slot 48 x 48 named `#<screen>/<name>` (`#device/symbol`) with an instance of `#symbol` as the preview.

## How it works
The variant is `icon`, one per symbol: `house`, `door-open`, `bed-double`, `armchair`, `droplets` (`SymbolIcon::ALL`, in the order of `components.json`). The glyph inside is `#symbol/icon`. There are no parts the code sets and no transitions: the slot clones the variant.

## Traps
- A symbol the set does not have cannot be shown: add it to `SymbolIcon`, to the `icon` values of `components.json` and to the design (`everyware_ui_test` names a missing variant).
- A symbol inside a list row does not change (harry does not swap a variant inside a list item): use the row's `#icon` stack there.

## Example
[docs/examples/symbol.rs](../examples/symbol.rs): the whole path, from the first symbol to another device's, compiled and run by `everyware_ui_recipes_test`.
