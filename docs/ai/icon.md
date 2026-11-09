# #icon

A 24 px glyph (Lucide) for the slots of other components; display only, no state.

## Figma

- set: `#icon` (node 2-961 in the Library)
- property `icon`: `wifi` | `wifi-high` | `wifi-low` | `wifi-zero` | `lock` | `chevron-right` | `minus` | `plus` | `settings-2` | `sliders-horizontal` | `maximize-2`
- variant string: `icon=wifi` (the properties in this order; `Component::variant()` produces it, never typed by hand)
- parts the code sets: a glyph, 24 px (Lucide): for the slots of other components; no state
- on the screens now: not used yet
- to place one: a frame named `#<screen>/<name>` with an instance of `#icon` inside (the slot); texts in it are placeholders, nothing hidden

## Rust

- no Rust state machine: the app sets the parts directly (texts, opacities, meters)

## The app

No Rust binding yet: the app cannot put `#icon` in a slot (no `impl Slotted` in harry/src/slot.rs).

`icon` names the glyph. The set exists so that a list row, a tile or a symbol can show one of many glyphs by stacking instances at opacity 0 and lighting one: harry does not swap a variant inside a list item, so a row's leading icon is a stack of `#list-item/leading/<glyph>`.

## In Figma
Inside another component's layer, one instance per glyph the place can show, all at opacity 0 but the preview's; named `<part>/<glyph>`. Alone on a screen an icon is a plain picture of the design, not a slot.

## Traps
- A glyph that is not in the stack cannot be shown at runtime; add it to the stack (and to `components.json` for the icon buttons) rather than drawing it on the screen.
- An instance swap at runtime is a variant change in a list item and is ignored by harry: opacity only.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
