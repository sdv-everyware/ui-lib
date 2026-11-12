---
title: Tile
role: A device's card on a dashboard: what the device is, its state, and the one control the user needs most, right on the card.
rust: core/src/tile.rs
types: TileKind, Tile, TileEvent
---
The user sees a device at a glance (its symbol, its name, its room, its value) and changes it on the card: flips the light, drags the dimmer, picks the fan's mode, steps the temperature. A tap elsewhere on the card opens the device. The app gets one event per gesture and sends it on to the device.

## When to use it
- A device on a dashboard or a ribbon of cards: one tile per device, the kind is its main control (`toggle`, `slider`, `segmented`, `stepper`, `image` for a camera).
- The same control without a card (a setting in a sheet): the control alone, `#toggle`, `#slider` or `#segmented`. A device in a long list: a `#list-item` row.

## Quick start
**What.** A room card with a light: a tap on the card opens the device, and dragging the slider inside the card dims the lamp.

**Why.** One card holds two things the user can touch. The app hears either "open" or "a new level" and never has to work out which part was hit.

<!-- example: docs/examples/tile.rs use slot=#rooms/tile-1 field=bedroom -->
```rust
use everyware_ui::tile::{Tile, TileEvent, TileKind};
use everyware_ui::symbol::SymbolIcon;
```

<!-- example: docs/examples/tile.rs state slot=#rooms/tile-1 field=bedroom -->
```rust
pub bedroom: Tile,
/// Where the design puts the card on the display: the slider reads the finger against it.
pub tile_at: (f32, f32),
/// The device's sheet is open (the app's own sheet, docs/content/sheet.md).
pub device_open: bool,
```

<!-- example: docs/examples/tile.rs init slot=#rooms/tile-1 field=bedroom -->
```rust
bedroom: {
    let mut tile = Tile::new(TileKind::Slider, SymbolIcon::BedDouble, "Bedroom", "Lights").value("65%");
    tile.slider.value = 65.0;
    tile
},
tile_at: sizes.frame("#rooms/tile-1").map_or((0.0, 0.0), |f| (f.x, f.y)),
device_open: false,
```

<!-- example: docs/examples/tile.rs model slot=#rooms/tile-1 field=bedroom -->
```rust
bedroom: Slot = "#rooms/tile-1" ["SlotCustomizer"];
```

<!-- example: docs/examples/tile.rs present slot=#rooms/tile-1 field=bedroom -->
```rust
model.bedroom = slot("#rooms/tile-1", &state.bedroom);
```

<!-- example: docs/examples/tile.rs act slot=#rooms/tile-1 field=bedroom -->
```rust
// the card ("#rooms/tile-1") or the control inside it ("#rooms/tile-1/#tile/slider")
_ if node == "#rooms/tile-1" || part_of("#rooms/tile-1", node).is_some() => {
    let (x, y) = state.tile_at;
    match state.bedroom.on_part(part_of("#rooms/tile-1", node), gesture, x, y) {
        Some(TileEvent::Opened) => state.device_open = true,
        // the slider drags: the value text follows, the light waits
        Some(TileEvent::Moving(level)) => state.bedroom.value = format!("{level:.0}%"),
        // the finger is up: the light gets the level
        Some(TileEvent::Level(level)) => {
            state.bedroom.value = format!("{level:.0}%");
            if !send(Command::Level(level)) {
                log::warn!("tile: the light is not there, nothing was sent");
            }
        }
        // the other kinds: Toggled(on), Mode(i), Step(-1 or 1), the same way
        _ => {}
    }
}
```

## How it behaves
- **The card.** A down shows the card pressed (`#tile/pressed` fades in); an up over the card reports `TileEvent::Opened`.
- **The control.** It changes the device and does not open it: `Toggled(on)` for a toggle, `Moving(level)` while a slider drags and `Level(level)` when it lifts, `Mode(i)` for a segment, `Step(-1)` or `Step(1)` for minus and plus.
- **Texts.** `title`, `subtitle` and `value` are the app's; the value is shown for every kind but `segmented` and `image`.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled`; neither the card nor its control takes a tap.

## From the app
**What.** The device reports (another panel, its own button, it went offline): the card shows its level or that it is offline. And the level the user set goes to the device.

**Why.** The tile shows what the device has. Its state is plain fields: the texts, `toggle`, `slider`, `segmented`, `press`. A slider's level waits while a finger is on it (`slider.dragging()`).

<!-- example: docs/examples/tile.rs change slot=#rooms/tile-1 field=bedroom -->
```rust
/// The light reports (another panel or its own button changed it): the card shows the level, or
/// that the light is offline (the variant state=disabled, which takes no taps). Not while a
/// finger is on the slider, or the thumb would jump under it.
pub fn light_reported(state: &mut State, level: Option<f32>) {
    let tile = &mut state.bedroom;
    match level {
        Some(level) if !tile.slider.dragging() => {
            tile.slider.value = level.clamp(0.0, 100.0);
            tile.value = format!("{:.0}%", tile.slider.value);
            tile.subtitle = "Lights".into();
            tile.press.set_enabled(true);
        }
        Some(_) => {}
        None => {
            tile.subtitle = "Offline".into();
            tile.press.set_enabled(false);
        }
    }
}
```

<!-- example: docs/examples/tile.rs send slot=#rooms/tile-1 field=bedroom -->
```rust
/// What the card asks the light to do. An app has its own link to the device (smarthome-app:
/// a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// The light's level, 0..100.
    Level(f32),
}

/// Hands the command to the device; false when it is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## In Figma
A slot of the tile's size (438 x 248) named `#<screen>/<device>` (`#rooms/tile-1`) with an instance of `#tile` of the right kind. The texts and the control's position in the preview are placeholders.

## How it works
`kind` is the control the card shows, `state` `default` or `disabled`. The parts: `#tile/title`, `#tile/subtitle`, `#tile/value`, the symbol stack `#tile/symbol/<icon>` (one glyph lit by opacity), `#tile/pressed`, `#tile/image`, and the control inside: an instance of the Library's toggle, slider or segmented button, which the binding sets as its own (`#toggle/thumb`, `#slider/fill`, `#segmented/segment-<i>/selected`). The parts a finger can touch are `PART_TOGGLE`, `PART_SLIDER`, `#segmented/segment-<i>`, `PART_MINUS` and `PART_PLUS`. `on_part(part, gesture, x, y)` takes the part (`None` for the card) and the tile's top left on the display; the slider's track is in the tile's own pixels (`TILE_PAD` plus the slider's inset).

## Traps
- A glyph that is not in the stack `#tile/symbol/*` cannot be shown: add it to `#symbol` and the stack (ui-component skill).
- The control's parts belong to one kind: `#toggle/*` is only in `kind=toggle`. The binding marks them optional per kind, so the contract test checks them in that kind's variants.
- A tile in a list needs its place for the slider: `on_part(part, gesture, slot_x + list.row_offset(row, now), 0.0)`, with `row_of` and `part_of` on the node. The app's listener returns the status `Input::feed` gives, or the control's touch never reaches the row.

## Example
[docs/examples/tile.rs](../examples/tile.rs): the whole path, from a tap on the card and a drag on its slider to the level sent to the light and a light that goes offline, compiled and run by `everyware_ui_recipes_test`.
