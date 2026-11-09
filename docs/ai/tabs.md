# #tabs

The screen's navigation, Material 3's tabs: the active tab is lit by opacity, a tap switches the screen.

## Figma

- set: `#tabs` (node 2-2736 in the Library)
- property `tabs`: `2` | `3` | `4`
- property `state`: `default` | `disabled`
- variant string: `tabs=2, state=default` (the properties in this order; `Component::variant()` produces it, never typed by hand)
- parts the code sets: Material 3's tabs, the screen's navigation (a segmented button picks a value, tabs switch screens): per tab #tabs/tab-<i> (a touch target) with #tabs/tab-<i>/selected (the white pill), /label (ink2) and /label-selected (accent, semi bold) lit by opacity, set by code, not a variant; the labels set by code
- on the screens now: `#home/tabs` in #smarthome-root (node 1-11); `#rooms/tabs` in #rooms-screen (state=hidden) (node 5-6)
- to place one: a frame named `#<screen>/<name>` with an instance of `#tabs` inside (the slot); texts in it are placeholders, nothing hidden

## Rust

- no Rust state machine: the app sets the parts directly (texts, opacities, meters)

## The app

No Rust binding yet: the app cannot put `#tabs` in a slot (no `impl Slotted` in harry/src/slot.rs).

`tabs` is the number, `state` `default` or `disabled`. Each tab is `#tabs/tab-<i>` with `/active` (the indicator and the lit label) and `/label`. The app lights the active tab and shows the screen it stands for; switching is the app's (the tabs report a tap on `#tabs/tab-<i>`). There is no Rust state machine yet: the app sets the opacities directly through the parts.

## In Figma
One slot in the header or the bottom bar, named `#<screen>/tabs` (`#home/tabs`), with an instance of `#tabs` of the right number. Every tab is a screen or a frame of the prototype for designers (figma-design skill); on the device the app draws the screen.

## Traps
- Tabs are not a segmented button: a segmented button picks a value in a widget, tabs change what the whole screen shows.
- A tab's label is a text of the component (`#tabs/tab-<i>/label`): the app writes it, the design's text is a placeholder.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
