---
title: Overlay (code)
role: The state machine of anything shown over the screen: closed or open, with the open and close transitions; the base of Sheet and Form.
rust: core/src/overlay.rs
types: Overlay
---
`Overlay` is not a Figma set of its own: it is the Rust side of `#sheet` and of the keyboard dock. `open()`, `close()`, `is_open()`, `transition()` (the `open` or `close` spec from components.json for the change that just happened). A sheet or a dialog is `Sheet` (sheet.rs), which adds the layout; a sheet with input fields is `Form` (form.rs).

## In Figma
Nothing: see `#sheet`.

## Traps
- The app moves the card and fades the scrim from the overlay's state (display-only); it never sets the sheet instance's variant.
