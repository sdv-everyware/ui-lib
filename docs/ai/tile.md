# #tile

A device's card, Material 3's card for a smart home device: a symbol, a title, a subtitle, a value and the device's control; a tap on the card opens the device, the control inside changes it.

## Figma

- set: `#tile` (node 2-2824 in the Library)
- property `kind`: `toggle` | `slider` | `segmented` | `stepper` | `image`
- property `state`: `default` | `disabled`
- variant string: `kind=toggle, state=default` (the properties in this order; `Tile::variant()` produces it, never typed by hand)
- parts the code sets: a device's card: #tile/symbol/<icon> stacked at opacity 0 (the code lights one), #tile/title, #tile/subtitle, #tile/value; the control drawn with the library: #tile/toggle, #tile/slider, #tile/segmented, #tile/minus and #tile/plus, #tile/image (a picture on a plain frame); #tile/pressed opacity; a tap on the tile opens the device, the control inside takes its own taps (#tile/toggle, #tile/slider, the segments, #tile/minus and #tile/plus)
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
- events `TileEvent`: `Opened`, `Toggled(bool)`, `Moving(f32)`, `Level(f32)`, `Mode(usize)`, `Step(i8)`
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
  - field `pub slider: Slider`: The slider's level and its track (kind slider): the track where the design draws it, in the tile's own pixels (`Tile::new`); `on_part` shifts the finger by the tile's place.
  - field `pub segmented: Segmented`: The modes and the chosen one (kind segmented).
  - field `pub image: Option<String>`: The key of a registered picture (kind image, everyware_ui::images).
  - field `pub press: Pressable`: Pressed and disabled.
  - field `pub minus: Pressable`: The stepper's minus button (kind stepper).
  - field `pub plus: Pressable`: The stepper's plus button (kind stepper).
  - field `pub selected: bool`: The chosen card of a ribbon (a horizontal ListView lifts it).
  - `pub fn value(mut self, value: impl Into<String>) -> Self`: The value text.
  - `pub fn on_part(&mut self, part: Option<&str>, gesture: Gesture, x: f32, y: f32) -> Option<TileEvent>`: A gesture on the card (`part` None) or on a part inside it, by the name harry reports (`PART_TOGGLE`, `PART_SLIDER`, `#segmented/segment-<i>`, `PART_MINUS`, `PART_PLUS`). `x`, `y` is where the tile's top left is on the display (its row's place in the list plus the slot's), so the slider can read the finger against its track.
- pub enum `TileEvent`: What a tile reports.
  - `Opened`: The card was tapped: open the device.
  - `Toggled`: The toggle was flipped (kind toggle): the new value.
  - `Moving`: The slider is being dragged (kind slider): the level under the finger.
  - `Level`: The slider was released (kind slider): the level.
  - `Mode`: A segment was chosen (kind segmented): its index.
  - `Step`: Minus (-1) or plus (+1) was tapped (kind stepper).

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
// the control inside the card, by its part: state.rooms_tile_1.on_part(Some(part), gesture, tile_x, tile_y)
// gives Toggled, Moving and Level, Mode or Step (in a list: row_of and part_of on the node)
```

A tap on the card opens the device; the control inside (`state.rooms_tile_1.toggle`, `.slider`, `.segmented`) takes its own taps through `on_part`. The app's listener returns the status `Input::feed` gives.

`kind` is the control the card shows: `toggle` (a light), `slider` (a dimmer), `segmented` (a fan's modes), `stepper` (a value with minus and plus) or `image` (a camera's picture); `state` is `default` or `disabled`. Pressed is `#tile/pressed` faded in. The parts: `#tile/title`, `#tile/subtitle`, `#tile/value`, the symbol stack `#tile/symbol/<icon>` (one lit), `#tile/image`, and the control inside: an instance of the Library's toggle, slider or segmented button, which the binding sets as its own (`#toggle/thumb`, `#slider/fill`, `#segmented/segment-<i>/selected` ...).

A tap on the card reports `TileEvent::Opened`; the control inside takes its own taps: `on_part` with the part harry reports (`PART_TOGGLE`, `PART_SLIDER`, `#segmented/segment-<i>`, `PART_MINUS`, `PART_PLUS`) and the tile's place on the display gives `Toggled`, `Moving` and `Level`, `Mode` or `Step`. The app gets the part with its slot, and in a list with the row too ("`<slot>/<part>`", "`<slot>/<row>/<part>`": `row_of`, `part_of`): the control's layers have the same names in every tile, harry hands the touch to the control first and to the tile's slot or row next, and `Input` pairs the two.

## In Figma
A slot of the tile's size (438 x 248) named `#<screen>/<device>` with an instance of `#tile` of the right kind. Texts and the control's position in the preview are placeholders.

## Traps
- A glyph that is not in the stack `#tile/symbol/*` cannot be shown: add it to `#symbol` and the stack (ui-component skill).
- Control parts belong to one kind: `#toggle/*` is only in `kind=toggle`. The binding marks them optional per kind, so the contract test checks them in that kind's variants.
- A tile in a list needs its place for the slider: `on_part(part, gesture, slot_x + list.row_offset(row, now), 0.0)`; the app's listener returns the status `Input::feed` gives, or the control's touch never reaches the row.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
