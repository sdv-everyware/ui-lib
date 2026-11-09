---
title: Keyboard
role: The on-screen keyboard: rows of keys with a character each, pressed keys fade a layer; one component, not a set.
rust: core/src/keyboard.rs
types: Row, Key, Mode, Keyboard, Outcome
---
`#keyboard` has no variants: the keys are `#kbd/<row>/<i>` with `/pressed` (faded in while down) and `/label` (the character of the current mode: letters, shift, symbols). `Keyboard::on(gesture)` takes the key from the node name and answers an `Outcome` (a character, backspace, done, a mode change); the text itself belongs to the field being edited (`Form`, `InputField`). The keyboard sits in the keyboard dock (`#keyboard-dock`), one per screen.

## In Figma
Inside the dock's slot (`#<app>/keys`), an instance of `#keyboard`; every key's label is a text override (static labels survive a rebuild), the pressed layer is in the component.

## Traps
- Keys report by their own names (`#kbd/r1/3`): one keyboard on the screen at a time, or the names collide.
- A label as a renamed inner layer is lost on a variant swap of the dock; labels are text overrides (`labelled()` in the generator).
