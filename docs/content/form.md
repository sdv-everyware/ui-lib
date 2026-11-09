---
title: Form (code)
role: A sheet with input fields: a tap focuses a field, the screen's keyboard dock opens and types into it, Enter submits.
rust: core/src/form.rs
types: Form, FormEvent
---
`Form::new(layout, fields)` wraps a `Sheet` and the fields' `InputField`s, in the order given. `on(node, gesture, now, keyboard)` routes a tap to the field by its node name and the keyboard's outcome to the focused field; it answers `Focused(i)`, `Edited(i)`, `Submitted(i)`. The keyboard dock follows the focus: `state.osk.follow(form.focused().is_some())` after every action.

## In Figma
The fields are slots of `#input-field` inside the sheet's content component (`#wifi/password`); the form is code.

## Traps
- A dialog with a form moves above the keyboard dock (the free height); a side panel does not.
- The field's text is the form's, not the keyboard's: the keyboard only reports outcomes.
