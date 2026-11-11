---
title: Keyboard dock
role: The screen's one keyboard: an overlay at the bottom with the keyboard component in its slot, open while a field has the focus.
rust: core/src/dock.rs
types: KeyboardDock
---
`state` is `closed` or `open`; the dock slides up by the `open` transition and is 420 px high (`KEYBOARD_DOCK_H`). `KeyboardDock::follow(focused)` opens and closes it with the focus; `keyboard` inside it is the `Keyboard` state machine. One dock per screen, above every sheet. The app never sets the dock's variant: it moves the slot with the keys to `keys_position(screen)` with `transition()` (a `Position` with `FollowPositionCustomizer`), as a sheet moves its card.

## In Figma
A plain absolute frame over the screen with an instance of `#keyboard-dock` (state `closed`, below the screen's edge) whose slot `#<app>/keys` holds `#keyboard`; the last layer of the root so it is above the sheets.

## Traps
- An instance placed absolute by itself lands in the root's flow after import: the instance inside a plain frame with absolute positioning.
- Dialogs centre in the screen minus the dock while it is open: give the presenter the free height (`SCREEN.1 - KEYBOARD_DOCK_H`).
- A variant switch of the dock (closed to open) rebuilds the tree and lays the screen out on every frame of the animation: the dock stutters (frames of 100 to 170 ms on the RB3). Move the slot instead.
