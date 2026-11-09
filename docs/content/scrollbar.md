---
title: Scrollbar
role: The thumb of a scrolling list: a vertical progress marker along a track, faded in while the list moves.
rust: core/src/list_view.rs
types: ListView
---
`length` is the thumb's share of the track, as the design draws it; the position `#scrollbar/thumb` is a vertical progress marker (0..100) the code moves with the list's scroll. The scrollbar has no state of its own: `scrollbar("#x", &list)` reads the `ListView` of the rows and fades the thumb out after the list settles.

## In Figma
A slot of the list's height, 8 wide, beside the list's slot (not inside it: inside, it would scroll away with the rows), named `#<screen>/<name>-scrollbar`, with an instance of `#scrollbar` of that height.

## Traps
- A scrollbar inside the list's slot scrolls with the content: a slot of its own, next to the list.
- The marker is the shape itself (a rounded rect with the meter), not a frame with children: harry moves a marker's own shape.
