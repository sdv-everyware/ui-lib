# <app>-sheet

A sheet with input fields: a tap focuses a field, the screen's keyboard dock opens and types into it, Enter submits.

## Figma

- set: `<app>-sheet`
- property `state`: `closed` | `open`
- variant string: `state=closed` (the properties in this order; `Form::variant()` produces it, never typed by hand)
- parts the code sets: an overlay with input fields (slots): a tap focuses a field, the screen's keyboard dock opens and types into it; one set per form in the app's design, state first
- to place one: a frame named `#<screen>/<name>` with an instance of `<app>-sheet` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- open: 550 ms spring
- close: 400 ms spring

## Rust

- module: `core/src/form.rs`, type `Form`
- `pub fn new(layout: Layout, fields: impl IntoIterator<Item = (impl Into<String>, InputField)>) -> Self`
- `pub fn on(&mut self, node: &str, gesture: Gesture, now: Instant, keyboard: &mut Keyboard) -> Option<FormEvent>`
- events `FormEvent`: `Focused(usize)`, `Edited(usize)`, `Submitted(usize)`
- pub struct `Form`: A sheet with input fields.
  - field `pub sheet: Sheet`: The sheet: closed or open over the screen, its layout, the card's place.
  - `pub fn open(&mut self)`: Opens it, nothing in focus; the change animates with the open transition.
  - `pub fn close(&mut self)`: Closes it and takes the focus away (the keyboard closes with it); the change animates with the close transition.
  - `pub fn is_open(&self) -> bool`: Whether the sheet is open.
  - `pub fn clear(&mut self)`: Empties the fields (texts and errors); placeholders and helpers stay.
  - `pub fn field(&self, index: usize) -> &InputField`: A field, by its index in `new`.
  - `pub fn field_mut(&mut self, index: usize) -> &mut InputField`: A field to change, by its index in `new`.
  - `pub fn focused(&self) -> Option<usize>`: The field with the focus; the keyboard dock follows it.
  - `pub fn focus(&mut self, index: usize, now: Instant)`: Gives a field the focus; a disabled field does not take it.
  - `pub fn blur(&mut self)`: Takes the focus away from every field.
- pub enum `FormEvent`: What the form reports; the index is the field's, in the order given to `new`.
  - `Focused`: A tap gave a field the focus (the keyboard opens).
  - `Edited`: A key changed a field's text.
  - `Submitted`: Done on the keyboard: the field's text is submitted.

`Form::new(layout, fields)` wraps a `Sheet` and the fields' `InputField`s, in the order given. `on(node, gesture, now, keyboard)` routes a tap to the field by its node name and the keyboard's outcome to the focused field; it answers `Focused(i)`, `Edited(i)`, `Submitted(i)`. The keyboard dock follows the focus: `state.osk.follow(form.focused().is_some())` after every action.

## In Figma
The fields are slots of `#input-field` inside the sheet's content component (`#wifi/password`); the form is code.

## Traps
- A dialog with a form moves above the keyboard dock (the free height); a side panel does not.
- The field's text is the form's, not the keyboard's: the keyboard only reports outcomes.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
