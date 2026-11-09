---
title: Toggle
role: An on/off switch, Material 3's switch: the value moves the thumb and fades the track, the variant is only the state.
rust: core/src/toggle.rs
types: Toggle, ToggleEvent
---
`state` is `default`, `pressed` (the thumb stretches under the finger) or `disabled`. On and off are not variants: `#toggle/thumb` is a horizontal progress marker the code moves from 0 to 100, `#toggle/on` the blue track the code fades in, both display-only and animated by the `value` transition (200 ms). `Toggle::set(value)` follows the device without a gesture; `on(gesture)` answers `Toggled(bool)` on a completed tap.

## In Figma
A slot 52 × 32 named `#<screen>/<name>` (`#entryway/light`) with an instance of `#toggle` inside. The marker node sits at x = 0 of the track with its travel as startX..endX; the preview shows the off position.

## Traps
- On/off as a variant switches in steps and rebuilds the tree: keep the value on the marker and the opacity.
- Two toggles on one screen share the part names `#toggle/thumb`, `#toggle/on`: each in its own slot, or they move together.
- A toggle inside a list row switches instead of fading on stock harry (the animation pass read only the global scope): the display_safety series in `everyware/patches` fixes it.
