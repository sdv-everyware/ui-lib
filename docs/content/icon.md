---
title: Icon
role: A 24 px glyph (Lucide) for the slots of other components; display only, no state.
---
`icon` names the glyph. The set exists so that a list row, a tile or a symbol can show one of many glyphs by stacking instances at opacity 0 and lighting one: harry does not swap a variant inside a list item, so a row's leading icon is a stack of `#list-item/leading/<glyph>`.

## In Figma
Inside another component's layer, one instance per glyph the place can show, all at opacity 0 but the preview's; named `<part>/<glyph>`. Alone on a screen an icon is a plain picture of the design, not a slot.

## Traps
- A glyph that is not in the stack cannot be shown at runtime; add it to the stack (and to `components.json` for the icon buttons) rather than drawing it on the screen.
- An instance swap at runtime is a variant change in a list item and is ignored by harry: opacity only.
