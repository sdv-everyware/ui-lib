---
title: Slider
role: A horizontal slider: the fill and the thumb are meters the code moves, the variant is only the state.
rust: core/src/slider.rs
types: Slider, SliderEvent
---
`state` is `default`, `pressed` or `disabled`. The value (0..100) moves `#slider/fill` (a progress bar) and `#slider/thumb` (a progress marker), display-only, so the thumb follows the finger every frame. `on(gesture)` answers `Moving(value)` while the finger drags and `Changed(value)` when it lifts; the app sends the device the `Changed` value and may show `Moving` live. `Slider::new(value, track_x, track_width)` needs the track's place on the display, because the finger's x is a display coordinate.

## In Figma
A slot of the slider's size named `#<screen>/<name>` (`#bedroom/dimmer`) with an instance of `#slider`; the thumb marker at x = 0 of the track, its travel the track's width. The app's `track_x`/`track_width` are the slot's x and width in the design.

## Traps
- A thumb that starts in the wrong place or runs past the end: the marker node is not at x = 0 of its parent, or startX..endX is not the travel.
- The value changes but nothing moves: the marker must be the shape itself (a rounded rect), not a frame with children.
- A slider moved to another x in Figma needs the new `track_x` in the app: `everyware_ui_test` cannot see it, the finger can.
