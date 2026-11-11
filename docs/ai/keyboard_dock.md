# #keyboard-dock

The screen's one keyboard: an overlay at the bottom with the keyboard component in its slot, open while a field has the focus.

## Figma

- set: `#keyboard-dock` (node 3-36 in the Library)
- property `state`: `closed` | `open`
- variant string: `state=closed` (the properties in this order; `KeyboardDock::variant()` produces it, never typed by hand)
- parts the code sets: the screen's one keyboard: an overlay at the bottom with the #keyboard component in its slot (#<app>/keys), open while a field has the focus; the instance stays closed and the app moves the slot (keys_position)
- to place one: a frame named `#<screen>/<name>` with an instance of `#keyboard-dock` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- open: 550 ms spring
- close: 400 ms spring

## Rust

- module: `core/src/dock.rs`, type `KeyboardDock`
- `pub fn new() -> Self`
- pub struct `KeyboardDock`: The screen's keyboard: an overlay with the keys.
  - field `pub overlay: Overlay`: Closed (below the screen) or open (docked at the bottom).
  - field `pub keyboard: Keyboard`: The keys: labels per mode, pressed keys, typing.
  - `pub fn follow(&mut self, focus: bool)`: Opens the dock while a field has the focus and closes it when none has: the screen calls it after every action with `form.focused().is_some()` (or several forms' or'ed).
  - `pub fn open(&self) -> bool`: The dock is open: the keys are on the screen.
  - `pub fn keys_position(&self, screen: (f32, f32)) -> (f32, f32)`: Where the slot with the keys is now, on a screen of `(w, h)`: docked at the bottom while open, `CLOSED_GAP` below the screen's edge while closed (where the design draws it). The app gives both coordinates to the slot as a position override (FollowPositionCustomizer) with `transition()`.

`state` is `closed` or `open`; the dock slides up by the `open` transition and is 420 px high (`KEYBOARD_DOCK_H`). `KeyboardDock::follow(focused)` opens and closes it with the focus; `keyboard` inside it is the `Keyboard` state machine. One dock per screen, above every sheet. The app never sets the dock's variant: it moves the slot with the keys to `keys_position(screen)` with `transition()` (a `Position` with `FollowPositionCustomizer`), as a sheet moves its card.

## In Figma
A plain absolute frame over the screen with an instance of `#keyboard-dock` (state `closed`, below the screen's edge) whose slot `#<app>/keys` holds `#keyboard`; the last layer of the root so it is above the sheets.

## Traps
- An instance placed absolute by itself lands in the root's flow after import: the instance inside a plain frame with absolute positioning.
- Dialogs centre in the screen minus the dock while it is open: give the presenter the free height (`SCREEN.1 - KEYBOARD_DOCK_H`).
- A variant switch of the dock (closed to open) rebuilds the tree and lays the screen out on every frame of the animation: the dock stutters (frames of 100 to 170 ms on the RB3). Move the slot instead.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
