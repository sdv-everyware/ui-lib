# #slider

A horizontal slider: the fill and the thumb are meters the code moves, the variant is only the state.

## Figma

- set: `#slider` (node 2-2402 in the Library)
- property `state`: `default` | `pressed` | `disabled`
- variant string: `state=default` (the properties in this order; `Slider::variant()` produces it, never typed by hand)
- parts the code sets: #slider/fill progress bar meter, #slider/thumb progress marker meter: set by code, not a variant
- nodes set by code (`Slider`, harry/src/slot.rs): `#slider/fill`, `#slider/thumb`
- on the screens now: `#bedroom/dimmer` in Room controls (node 1-157)
- to place one: a frame named `#<screen>/<name>` with an instance of `#slider` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out
- value: 80 ms ease-out

## Rust

- module: `core/src/slider.rs`, type `Slider`; binding: `harry/src/slot.rs` (`impl Slotted for Slider`)
- `pub fn new(value: f32, track_x: f32, track_width: f32) -> Self`
- `pub fn on(&mut self, gesture: Gesture) -> Option<SliderEvent>`
- events `SliderEvent`: `Moving(f32)`, `Changed(f32)`
- pub struct `Slider`: A horizontal slider over a track at a known place on the display.
  - field `pub value: f32`: 0..100.
  - field `pub step: f32`: Steps of the value; 0 for none.
  - field `pub track_x: f32`: Display x of the track's start (where the thumb's centre is at 0).
  - field `pub track_width: f32`: Track length on the display (thumb centre travel from 0 to 100).
  - field `pub press: Pressable`: Pressed and disabled.
  - `pub fn in_slot(value: f32, slot_x: f32, slot_width: f32) -> Self`: A slider at `value` in a slot whose left edge is `slot_x` on the display and which is `slot_width` wide: the track is inset TRACK_INSET on both sides (the #slider design). The slot's place comes from the design (everyware_ui::design::Sizes::frame), so a slot moved in Figma moves the touch area with it.
  - `pub fn dragging(&self) -> bool`: A finger is on the slider: a value from outside (the device reported one) waits until it is up, or the thumb would jump under the finger.
  - `pub fn value_transition(&self) -> Option<Transition>`: How fill and thumb follow the value: short, so the thumb stays under the finger.
- pub enum `SliderEvent`: What a slider reports.
  - `Moving`: The value while dragging.
  - `Changed`: The value when the finger went up.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::slider::{Slider, SliderEvent};
use everyware_ui::design::Sizes;

// state.rs: in struct State, and its first value in State::new
pub bedroom_dimmer: Slider,
// bedroom_dimmer: Slider::in_slot(65.0, frame.x, frame.width),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
bedroom_dimmer: Slot = "#bedroom/dimmer" ["SlotCustomizer"];

// presenter.rs: after every action
model.bedroom_dimmer = slot("#bedroom/dimmer", &state.bedroom_dimmer);

// actions.rs: in the match on the gesture's node name
"#bedroom/dimmer" => {
    if let Some(SliderEvent::Changed(value)) = state.bedroom_dimmer.on(gesture) {
        let _ = value; // yours: the finger is up; Moving(value) while it drags
    }
}
```

`frame` is where the design puts the slot: `let frame = sizes.frame("#bedroom/dimmer")` with `sizes = Sizes::load(<the .dcf>)` at the start; the touch area follows the design.

## Live: `#bedroom/dimmer` in smarthome-app (cut from the sources at build time)

```rust
// state.rs:90
pub dimmer: Slider,
dimmer: Slider::new(65.0, 763.0 + 20.0, 203.0),

// ui_model.rs:172
bedroom_dimmer: Slot = "#bedroom/dimmer" ["SlotCustomizer"];

// presenter.rs:38
model.bedroom_dimmer = slot("#bedroom/dimmer", &d.dimmer);

// actions.rs:333
"#bedroom/dimmer" => {
    // the value follows the finger (SliderEvent::Moving) and is in the slider; the
    // device gets it when the finger is up
    if let Some(SliderEvent::Changed(percent)) = d.dimmer.on(gesture) {
        return Some(Control::Dimmer(percent));
    }
}
```

`state` is `default`, `pressed` or `disabled`. The value (0..100) moves `#slider/fill` (a progress bar) and `#slider/thumb` (a progress marker), display-only, so the thumb follows the finger every frame. `on(gesture)` answers `Moving(value)` while the finger drags and `Changed(value)` when it lifts; the app sends the device the `Changed` value and may show `Moving` live. `Slider::new(value, track_x, track_width)` needs the track's place on the display, because the finger's x is a display coordinate.

## In Figma
A slot of the slider's size named `#<screen>/<name>` (`#bedroom/dimmer`) with an instance of `#slider`; the thumb marker at x = 0 of the track, its travel the track's width. The app's `track_x`/`track_width` are the slot's x and width in the design.

## Traps
- A thumb that starts in the wrong place or runs past the end: the marker node is not at x = 0 of its parent, or startX..endX is not the travel.
- The value changes but nothing moves: the marker must be the shape itself (a rounded rect), not a frame with children.
- A slider moved to another x in Figma needs the new `track_x` in the app: `everyware_ui_test` cannot see it, the finger can.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
