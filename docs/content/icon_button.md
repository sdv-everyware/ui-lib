---
title: Icon button
role: A tappable button with one glyph and no label, Material 3's icon button, in two sizes.
rust: core/src/button.rs
types: IconButtonKind, Icon, IconButton, ButtonEvent
---
`kind` is the look (filled, tonal, plain), `icon` the design's glyph (Lucide names), `size` 56 or 48. `state` only `default` or `disabled`; pressed is `#icon-button/pressed`, a layer the code fades in. The glyph is part of the variant: an icon that changes at runtime is a variant change (rebuild), so pick a stable glyph per place, or stack glyphs in a slot of your own and light one by opacity.

## In Figma
A slot of the variant's size (56 × 56 or 48 × 48) named `#<screen>/<name>` (`#climate/plus`) with an instance of `#icon-button` inside; the instance's `icon` is the preview, the app's `IconButton::new(kind, icon)` picks the real one. A row of icon buttons is a row of slots, 8 px apart.

## Traps
- A glyph missing in the `icon` property of the set cannot be shown: `everyware_ui_test` names the variant; add the glyph to `components.json` and the generator (ui-component skill), do not draw it on the screen.
- The pressed layer is in the variant, not in the slot: a slot drawn by hand without an instance shows no press.
