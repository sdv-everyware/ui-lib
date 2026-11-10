---
title: Tile
role: A device's card, Material 3's card for a smart home device: a symbol, a title, a subtitle, a value and the device's control; a tap on the card opens the device, the control inside changes it.
rust: core/src/tile.rs
types: TileKind, Tile, TileEvent
---
`kind` is the control the card shows: `toggle` (a light), `slider` (a dimmer), `segmented` (a fan's modes), `stepper` (a value with minus and plus) or `image` (a camera's picture); `state` is `default` or `disabled`. Pressed is `#tile/pressed` faded in. The parts: `#tile/title`, `#tile/subtitle`, `#tile/value`, the symbol stack `#tile/symbol/<icon>` (one lit), `#tile/image`, and the control inside: an instance of the Library's toggle, slider or segmented button, which the binding sets as its own (`#toggle/thumb`, `#slider/fill`, `#segmented/segment-<i>/selected` ...).

A tap on the card reports `TileEvent::Opened`; the control inside takes its own taps: `on_part` with the part harry reports (`PART_TOGGLE`, `PART_SLIDER`, `#segmented/segment-<i>`, `PART_MINUS`, `PART_PLUS`) and the tile's place on the display gives `Toggled`, `Moving` and `Level`, `Mode` or `Step`. In a list the app gets the part with the row ("`<slot>/<row>/<part>`", `row_of` and `part_of`): the control's layers have the same names in every tile, harry hands the touch to the control first and to the card's row next, and `Input` pairs the two.

## In Figma
A slot of the tile's size (438 x 248) named `#<screen>/<device>` with an instance of `#tile` of the right kind. Texts and the control's position in the preview are placeholders.

## Traps
- A glyph that is not in the stack `#tile/symbol/*` cannot be shown: add it to `#symbol` and the stack (ui-component skill).
- Control parts belong to one kind: `#toggle/*` is only in `kind=toggle`. The binding marks them optional per kind, so the contract test checks them in that kind's variants.
- A tile in a list needs its place for the slider: `on_part(part, gesture, slot_x + list.row_offset(row, now), 0.0)`; the app's listener returns the status `Input::feed` gives, or the control's touch never reaches the row.
