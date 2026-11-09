---
title: Symbol
role: The tile of a room's or a device's symbol: 48 px, the soft accent background, one glyph in ink; display only.
rust: core/src/symbol.rs
types: SymbolIcon, Symbol
---
`icon` names the glyph (the design's icons). `Symbol::new(icon)` gives the variant; there is nothing to tap. A symbol is a variant per glyph, so it changes rarely (a sheet opens for another device); in a list row use the stacked `#icon` instead.

## In Figma
A slot 48 × 48 named `#<screen>/symbol` (`#device/symbol`) with an instance of `#symbol`.

## Traps
- A glyph the set does not have cannot be shown; `everyware_ui_test` names the variant.
