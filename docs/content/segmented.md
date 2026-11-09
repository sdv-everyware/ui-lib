---
title: Segmented button
role: One choice of two to four, Material 3's segmented button: the segments are tap targets, the selected one is lit by opacity.
rust: core/src/segmented.rs
types: Segmented, SegmentedEvent
---
`segments` is the number (2, 3, 4), `state` `default` or `disabled`. Each segment is `#segmented/segment-<i>` (a tap target) with `/selected` (the lit background) and `/label`; selection fades, the variant does not change. `on(index, gesture)` takes the segment from the node name and answers `Selected(index)`. The texts of the segments are the app's (`set_labels`).

## In Figma
A slot of the variant's size (the design's width per number of segments) named `#<screen>/<name>` (`#office/fan`) with an instance of `#segmented` of the right `segments`. Labels in the preview are placeholders.

## Traps
- `Tabs` look alike but switch screens; a segmented button picks a value. Do not use one for the other (the set names and the roles differ).
- The taps come by the segment's own name (`#segmented/segment-2`): the match arm in actions.rs starts with the prefix, not the slot's name.
