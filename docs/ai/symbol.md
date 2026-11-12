# #symbol

The tile of a room's or a device's symbol: 48 px, the soft accent background, one glyph in ink; display only.

## Figma

- set: `#symbol` (node 2-1010 in the Library)
- property `icon`: `house` | `door-open` | `bed-double` | `armchair` | `droplets`
- variant string: `icon=house` (the properties in this order; `Symbol::variant()` produces it, never typed by hand)
- parts the code sets: display only: the tile of a room's or a device's symbol (48, the soft accent, the glyph #symbol/icon in ink)
- nodes set by code (`Symbol`, harry/src/slot.rs): 
- on the screens now: `#device/symbol` in #device-content (state=default) (node 6-5)
- to place one: a frame named `#<screen>/<name>` with an instance of `#symbol` inside (the slot); texts in it are placeholders, nothing hidden

## Rust

- module: `core/src/symbol.rs`, type `Symbol`; binding: `harry/src/slot.rs` (`impl Slotted for Symbol`)
- `pub fn new(icon: SymbolIcon) -> Self`
- pub enum `SymbolIcon`: The icon property of #symbol: the design's icons (lucide names).
  - `House`: `house`
  - `DoorOpen`: `door-open`
  - `BedDouble`: `bed-double`
  - `Armchair`: `armchair`
  - `Droplets`: `droplets`
  - `pub fn name(self) -> &'static str`: The value of the set's `icon` property.
- pub struct `Symbol`: A symbol tile.
  - field `pub icon: SymbolIcon`: Which symbol.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::symbol::{Symbol, SymbolIcon};

// state.rs: in struct State, and its first value in State::new
pub device_symbol: Symbol,
// device_symbol: Symbol::new(SymbolIcon::House),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
device_symbol: Slot = "#device/symbol" ["SlotCustomizer"];

// presenter.rs: after every action
model.device_symbol = slot("#device/symbol", &state.device_symbol);

// actions.rs: nothing, display only
```

## Live: `#device/symbol` in smarthome-app (cut from the sources at build time)

```rust
// ui_model.rs:242
device_symbol: Slot = "#device/symbol" ["SlotCustomizer"];

// presenter.rs:87
model.device_symbol = slot("#device/symbol", &Symbol::new(device.view.symbol));
```

`icon` names the glyph (the design's icons). `Symbol::new(icon)` gives the variant; there is nothing to tap. A symbol is a variant per glyph, so it changes rarely (a sheet opens for another device); in a list row use the stacked `#icon` instead.

## In Figma
A slot 48 × 48 named `#<screen>/symbol` (`#device/symbol`) with an instance of `#symbol`.

## Traps
- A glyph the set does not have cannot be shown; `everyware_ui_test` names the variant.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
