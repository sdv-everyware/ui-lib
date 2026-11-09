---
title: Tile
role: A device's card, Material 3's card for a smart home device: a symbol, a title, a subtitle, a value and the device's control; the whole card opens the device.
rust: core/src/tile.rs
types: TileKind, Tile, TileEvent
---
`kind` is the control the card shows: `toggle` (a light), `slider` (a dimmer), `segmented` (a fan's modes), `stepper` (a value with minus and plus) or `image` (a camera's picture); `state` is `default` or `disabled`. Pressed is `#tile/pressed` faded in. The parts: `#tile/title`, `#tile/subtitle`, `#tile/value`, the symbol stack `#tile/symbol/<icon>` (one lit), `#tile/image`, and the control inside: an instance of the Library's toggle, slider or segmented button, which the binding sets as its own (`#toggle/thumb`, `#slider/fill`, `#segmented/segment-<i>/selected` ...).

The card is one touch target and reports `TileEvent::Opened`; the control in it shows the device and takes no taps. Its layers have the same names in every tile of a screen, so a tap on one could not say which tile it was; the device's sheet changes the value.

## In Figma
A slot of the tile's size (438 x 248) named `#<screen>/<device>` with an instance of `#tile` of the right kind. Texts and the control's position in the preview are placeholders.

## Traps
- A glyph that is not in the stack `#tile/symbol/*` cannot be shown: add it to `#symbol` and the stack (ui-component skill).
- Control parts belong to one kind: `#toggle/*` is only in `kind=toggle`. The binding marks them optional per kind, so the contract test checks them in that kind's variants.
- A control that must take taps (a toggle you flip from the dashboard) is its own slot next to a plain card, not inside a tile.
