# <app>-sheet

The state machine of anything shown over the screen: closed or open, with the open and close transitions; the base of Sheet and Form.

## Figma

- set: `<app>-sheet`
- property `state`: `closed` | `open`
- variant string: `state=closed` (the properties in this order; `Overlay::variant()` produces it, never typed by hand)
- parts the code sets: an overlay over the screen: the variant is the state, harry animates the change with the open/close transitions; one set per overlay in the app's design. The moving parts carry the same animation as the instance: harry gives a variant change's children no animation of their own
- to place one: a frame named `#<screen>/<name>` with an instance of `<app>-sheet` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- open: 550 ms spring
- close: 400 ms spring

## Rust

- module: `core/src/overlay.rs`, type `Overlay`
- `pub fn new() -> Self`
- pub struct `Overlay`: An overlay: closed or open.
  - field `pub open: bool`: Shown over the dashboard.
  - `pub fn open(&mut self)`: Opens it; the change animates with the open transition.
  - `pub fn close(&mut self)`: Closes it; the change animates with the close transition.
  - `pub fn slide(&self, open: f32, closed: f32) -> f32`: Where a part that moves with the overlay goes: `open` while it is open, `closed` while it is closed (a card's y in the open drawing and below the screen, a scrim's opacity).
  - `pub fn transition(&self) -> Option<Transition>`: The transition of the last change (open or close), for harry's animation of the variant.

`Overlay` is not a Figma set of its own: it is the Rust side of `#sheet` and of the keyboard dock. `open()`, `close()`, `is_open()`, `transition()` (the `open` or `close` spec from components.json for the change that just happened). A sheet or a dialog is `Sheet` (sheet.rs), which adds the layout; a sheet with input fields is `Form` (form.rs).

## In Figma
Nothing: see `#sheet`.

## Traps
- The app moves the card and fades the scrim from the overlay's state (display-only); it never sets the sheet instance's variant.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
