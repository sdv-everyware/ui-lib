---
title: Tabs
role: The screen's navigation, Material 3's tabs: the active tab is lit by opacity, a tap switches the screen.
---
`tabs` is the number, `state` `default` or `disabled`. Each tab is `#tabs/tab-<i>` with `/active` (the indicator and the lit label) and `/label`. The app lights the active tab and shows the screen it stands for; switching is the app's (the tabs report a tap on `#tabs/tab-<i>`). There is no Rust state machine yet: the app sets the opacities directly through the parts.

## In Figma
One slot in the header or the bottom bar, named `#<screen>/tabs` (`#home/tabs`), with an instance of `#tabs` of the right number. Every tab is a screen or a frame of the prototype for designers (figma-design skill); on the device the app draws the screen.

## Traps
- Tabs are not a segmented button: a segmented button picks a value in a widget, tabs change what the whole screen shows.
- A tab's label is a text of the component (`#tabs/tab-<i>/label`): the app writes it, the design's text is a placeholder.
