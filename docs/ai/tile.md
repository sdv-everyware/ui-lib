# #tile

A device's card, Material 3's card for a smart home device: a symbol, a title, a subtitle, a value and the device's control; the whole card opens the device.

## Figma

- set: `#tile` (node 2-2824 in the Library)
- property `kind`: `toggle` | `slider` | `segmented` | `stepper` | `image`
- property `state`: `default` | `disabled`
- variant string: `kind=toggle, state=default` (the properties in this order; `Tile::variant()` produces it, never typed by hand)
- parts the code sets: a device's card: #tile/symbol/<icon> stacked at opacity 0 (the code lights one), #tile/title, #tile/subtitle, #tile/value; the control drawn with the library: #tile/toggle, #tile/slider, #tile/segmented, #tile/minus and #tile/plus, #tile/image (a picture on a plain frame); #tile/pressed opacity; the whole tile a touch target (opens the device)
- nodes set by code (`Tile`, harry/src/slot.rs): `#tile/title`, `#tile/subtitle`, `#tile/value`, `#tile/pressed`, `#tile/symbol/{}`, `#segmented/segment-{i}`, `#tile/image`, `#tile/symbol/<…>`, `#segmented/segment-<…>`
- on the screens now: `#rooms/tile-1` in #rooms-screen (state=hidden) (node 5-10); `#rooms/tile-2` in #rooms-screen (state=hidden) (node 5-12); `#rooms/tile-3` in #rooms-screen (state=hidden) (node 5-14)
- to place one: a frame named `#<screen>/<name>` with an instance of `#tile` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out

## Rust

- module: `core/src/tile.rs`, type `Tile`; binding: `harry/src/slot.rs` (`impl Slotted for Tile`)
- `pub fn new(kind: TileKind, symbol: SymbolIcon, title: impl Into<String>, subtitle: impl Into<String>) -> Self`
- `pub fn on(&mut self, gesture: Gesture) -> Option<TileEvent>`
- events `TileEvent`: `Opened`
- pub enum `TileKind`: The kind property of #tile: the control the card shows.
  - `Toggle`: `toggle`: an on/off device (a light).
  - `Slider`: `slider`: a level (a dimmer).
  - `Segmented`: `segmented`: one of 2 to 4 modes (a fan).
  - `Stepper`: `stepper`: a value with minus and plus (a thermostat), the value in #tile/value.
  - `Image`: `image`: a picture (a camera's last frame) in #tile/image.
  - `pub fn name(self) -> &'static str`: The value of the kind property.
- pub struct `Tile`: A device's card.
  - field `pub kind: TileKind`: 
  - field `pub symbol: SymbolIcon`: The device's symbol: the glyph of the stack that is lit.
  - field `pub title: String`: 
  - field `pub subtitle: String`: 
  - field `pub value: String`: The value text (a slider's level, a stepper's value; empty for the others).
  - field `pub toggle: Toggle`: The toggle's state (kind toggle).
  - field `pub slider: Slider`: The slider's level (kind slider; its track is not touched, any x will do).
  - field `pub segmented: Segmented`: The modes and the chosen one (kind segmented).
  - field `pub image: Option<String>`: The key of a registered picture (kind image, everyware_ui::images).
  - field `pub press: Pressable`: Pressed and disabled.
  - field `pub selected: bool`: The chosen card of a ribbon (a horizontal ListView lifts it).
  - `pub fn value(mut self, value: impl Into<String>) -> Self`: The value text.
- pub enum `TileEvent`: What a tile reports.
  - `Opened`: The card was tapped: open the device.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::tile::{Tile, TileEvent, TileKind};
use everyware_ui::symbol::SymbolIcon;

// state.rs: in struct State, and its first value in State::new
pub rooms_tile_1: Tile,
// rooms_tile_1: Tile::new(TileKind::Slider, SymbolIcon::BedDouble, "Bedroom", "Lights").value("65%"),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
rooms_tile_1: Slot = "#rooms/tile-1" ["SlotCustomizer"];

// presenter.rs: after every action
model.rooms_tile_1 = slot("#rooms/tile-1", &state.rooms_tile_1);

// actions.rs: in the match on the gesture's node name
"#rooms/tile-1" => {
    if let Some(TileEvent::Opened) = state.rooms_tile_1.on(gesture) {
        // yours: open the device's sheet
    }
}
```

The whole card is one touch target; the control inside shows the device (`state.rooms_tile_1.toggle`, `.slider`, `.segmented`, `.value`) and takes no taps: the device's sheet changes it.

`kind` is the control the card shows: `toggle` (a light), `slider` (a dimmer), `segmented` (a fan's modes), `stepper` (a value with minus and plus) or `image` (a camera's picture); `state` is `default` or `disabled`. Pressed is `#tile/pressed` faded in. The parts: `#tile/title`, `#tile/subtitle`, `#tile/value`, the symbol stack `#tile/symbol/<icon>` (one lit), `#tile/image`, and the control inside: an instance of the Library's toggle, slider or segmented button, which the binding sets as its own (`#toggle/thumb`, `#slider/fill`, `#segmented/segment-<i>/selected` ...).

The card is one touch target and reports `TileEvent::Opened`; the control in it shows the device and takes no taps. Its layers have the same names in every tile of a screen, so a tap on one could not say which tile it was; the device's sheet changes the value.

## In Figma
A slot of the tile's size (438 x 248) named `#<screen>/<device>` with an instance of `#tile` of the right kind. Texts and the control's position in the preview are placeholders.

## Traps
- A glyph that is not in the stack `#tile/symbol/*` cannot be shown: add it to `#symbol` and the stack (ui-component skill).
- Control parts belong to one kind: `#toggle/*` is only in `kind=toggle`. The binding marks them optional per kind, so the contract test checks them in that kind's variants.
- A control that must take taps (a toggle you flip from the dashboard) is its own slot next to a plain card, not inside a tile.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
