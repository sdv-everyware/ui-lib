---
title: Radio
role: A group of radio buttons where one option is on; the group owns the selection, each button is a variant.
rust: core/src/radio.rs
types: Radio, RadioGroup, RadioEvent
---
`value` is `on` or `off`, `state` `default`, `pressed` or `disabled`. The group (`RadioGroup`) holds which index is selected and answers `Selected(index)`; each option is a `Radio` whose variant follows the group. Selection is a variant change (rare, a tap), so there is no meter here.

## In Figma
One slot per option, 24 × 24 (or the row that holds it) named `#<screen>/<group>/<i>`, each with an instance of `#radio`; the label is a text of the screen next to it, not part of the component. The group is the app's: the slots share nothing in Figma.

## Traps
- The index the app passes to `on(index, gesture)` comes from the node name (`#x/2`), not from the layer order: name the options in order.
- A label inside the component would need `#radio/label` in every variant; the design keeps labels on the screen.
