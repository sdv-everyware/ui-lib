# #toggle

An on/off switch, Material 3's switch: the value moves the thumb and fades the track, the variant is only the state.

## Figma

- set: `#toggle` (node 2-882 in the Library)
- property `state`: `default` | `pressed` | `disabled`
- variant string: `state=default` (the properties in this order; `Toggle::variant()` produces it, never typed by hand)
- parts the code sets: #toggle/thumb progress marker meter, #toggle/on opacity: set by code, not a variant
- nodes set by code (`Toggle`, harry/src/slot.rs): `#toggle/thumb`, `#toggle/on`
- on the screens now: `#wifi/reveal` in #wifi-content (step=password) (node 3-32); `#entryway/light` in Room controls (node 1-138)
- to place one: a frame named `#<screen>/<name>` with an instance of `#toggle` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out
- value: 200 ms ease-in-out

## Rust

- module: `core/src/toggle.rs`, type `Toggle`; binding: `harry/src/slot.rs` (`impl Slotted for Toggle`)
- `pub fn new(value: bool) -> Self`
- `pub fn on(&mut self, gesture: Gesture) -> Option<ToggleEvent>`
- events `ToggleEvent`: `Toggled(bool)`
- pub struct `Toggle`: An on/off switch.
  - field `pub value: bool`: On or off.
  - field `pub press: Pressable`: Pressed and disabled.
  - `pub fn set(&mut self, value: bool)`: Sets the value from outside (the device reported it).
  - `pub fn target(&self) -> f32`: Where the thumb and the blue track go: 0 (off) or 100 (on), for meter and opacity.
  - `pub fn value_transition(&self) -> Option<Transition>`: How the thumb and the track move to a new target.
- pub enum `ToggleEvent`: What a toggle reports.
  - `Toggled`: The user switched it; the new value.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::toggle::{Toggle, ToggleEvent};

// state.rs: in struct State, and its first value in State::new
pub wifi_reveal: Toggle,
// wifi_reveal: Toggle::new(false),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
wifi_reveal: Slot = "#wifi/reveal" ["SlotCustomizer"];

// presenter.rs: after every action
model.wifi_reveal = slot("#wifi/reveal", &state.wifi_reveal);

// actions.rs: in the match on the gesture's node name
"#wifi/reveal" => {
    if let Some(ToggleEvent::Toggled(on)) = state.wifi_reveal.on(gesture) {
        let _ = on; // yours: state.wifi_reveal.value is the same
    }
}
```

Read the value any time as `state.wifi_reveal.value` (a pub field); `state.wifi_reveal.set(on)` follows the device without a gesture.

`state` is `default`, `pressed` (the thumb stretches under the finger) or `disabled`. On and off are not variants: `#toggle/thumb` is a horizontal progress marker the code moves from 0 to 100, `#toggle/on` the blue track the code fades in, both display-only and animated by the `value` transition (200 ms). `Toggle::set(value)` follows the device without a gesture; `on(gesture)` answers `Toggled(bool)` on a completed tap.

## In Figma
A slot 52 × 32 named `#<screen>/<name>` (`#entryway/light`) with an instance of `#toggle` inside. The marker node sits at x = 0 of the track with its travel as startX..endX; the preview shows the off position.

## Traps
- On/off as a variant switches in steps and rebuilds the tree: keep the value on the marker and the opacity.
- Two toggles on one screen share the part names `#toggle/thumb`, `#toggle/on`: each in its own slot, or they move together.
- A toggle inside a list row switches instead of fading on stock harry (the animation pass read only the global scope): the display_safety series in `everyware/patches` fixes it.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
