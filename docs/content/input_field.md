---
title: Input field
role: A one-line text field: the focus ring, the caret and the placeholder are opacities, the variant is only the state.
rust: core/src/input_field.rs
types: InputField, InputEvent
---
`state` is `default`, `disabled` or `error`. Focus (`#input-field/focus`), the blinking caret (`#input-field/caret`), the placeholder (`#input-field/placeholder`, shown while empty) are opacities; the text is `#input-field/text` (dots while `secure`). `InputField::new(placeholder, helper)`; `on(gesture, now)` answers `Focused`; typing comes through the `Form` from the keyboard.

## In Figma
A slot of the field's size named `#<screen>/<name>` (`#wifi/password`) with an instance of `#input-field`; the text node carries at least a space as preview (an empty text may be dropped on import).

## Traps
- Text wrapping one character per line, or a focus ring that never shows, is stock harry's list-item bug; the display_safety series in `everyware/patches` fixes both.
- The caret blinks through `UpdatePolicy` from the binding: nothing to animate in the app.
