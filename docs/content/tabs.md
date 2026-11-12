---
title: Tabs
role: The screen's navigation, Material 3's tabs: the chosen tab is lit by opacity, a tap switches the screen with the screens' effect.
rust: core/src/tabs.rs
types: Tabs, TabsEvent
---
`tabs` is the number (2, 3, 4), `state` `default` or `disabled`. Each tab is `#tabs/tab-<i>` (a tap target) with `/selected` (the pill), `/label` and `/label-selected`; the choice fades, the variant does not change. `on(index, gesture)` answers `Selected(index)`; `Tabs` keeps the tab before it (`previous`) for the direction of a switch. The labels are the app's.

The screens (core/src/screens.rs): `tabs.screen(i, effect, width)` gives screen `i` its opacity and its x offset, each with a transition. The time of a switch is `tabs`/`switch` in components.json; the effect splits it: `Fade` (the default), `FadeThrough`, `Slide`, `None`. The app binds each screen with `ScreenCustomizer` and `ScreenView::new(target, design_x)`, so a switch is display only. A new animation is one more `Effect` with its test.

## In Figma
One slot in the header or the bottom bar, named `#<screen>/tabs` (`#home/tabs`), with an instance of `#tabs` of the right number. Every tab is a screen or a frame of the prototype for designers (figma-design skill); on the device the app draws the screen.

## Traps
- Tabs are not a segmented button: a segmented button picks a value in a widget, tabs change what the whole screen shows.
- A tap comes with its tab bar (`#home/tabs/#tabs/tab-2`): match the slot and the part, two tab bars on one screen stay apart.
- A screen at opacity 0 is not drawn and takes no taps; a screen with a variant of its own (shown, hidden) is set to shown once, then the effect shows or hides it.
- `Slide` moves a screen by its x in the parent: give `design_x` from the design (`Sizes::frame`), or the screen jumps.
