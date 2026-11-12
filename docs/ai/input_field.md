# #input-field

A one-line text field: the focus ring, the caret and the placeholder are opacities, the variant is only the state.

## Figma

- set: `#input-field` (node 2-2473 in the Library)
- property `state`: `default` | `disabled` | `error`
- variant string: `state=default` (the properties in this order; `InputField::variant()` produces it, never typed by hand)
- parts the code sets: #input-field/focus opacity (focused), #input-field/caret blinks by opacity, #input-field/placeholder opacity (empty), texts: set by code, not a variant
- nodes set by code (`InputField`, harry/src/slot.rs): `#input-field/text`, `#input-field/placeholder`, `#input-field/helper`, `#input-field/focus`, `#input-field/caret`
- on the screens now: `#wifi/password` in #wifi-content (step=password) (node 3-30)
- to place one: a frame named `#<screen>/<name>` with an instance of `#input-field` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- focus: 150 ms ease-out
- blur: 150 ms ease-in

## Rust

- module: `core/src/input_field.rs`, type `InputField`; binding: `harry/src/slot.rs` (`impl Slotted for InputField`)
- `pub fn new(placeholder: impl Into<String>, helper: impl Into<String>) -> Self`
- `pub fn on(&mut self, gesture: Gesture, now: Instant) -> Option<InputEvent>`
- events `InputEvent`: `Focused`
- pub struct `InputField`: A one-line text field.
  - field `pub text: String`: The text the user entered.
  - field `pub placeholder: String`: Shown while the text is empty.
  - field `pub helper: String`: Shown under the field while there is no error.
  - field `pub error: Option<String>`: An error: the field turns red and shows it instead of the helper.
  - field `pub secure: bool`: Shows dots instead of the characters (passwords).
  - field `pub focused: bool`: Takes the keyboard's input.
  - field `pub press: Pressable`: Disabled; taps do not focus it.
  - `pub fn focus(&mut self, now: Instant)`: Takes the focus (the keyboard writes here).
  - `pub fn blur(&mut self)`: Gives the focus away.
  - `pub fn edited(&mut self, now: Instant)`: The text was changed (by the keyboard): the caret restarts on, an error goes away.
  - `pub fn shown(&self) -> String`: The text to show: dots for a secure field.
  - `pub fn helper_line(&self) -> &str`: The line under the field: the error, else the helper.
  - `pub fn focus_target(&self) -> f32`: Opacity target of the focus ring: 100 focused, 0 not.
  - `pub fn focus_transition(&self) -> Option<Transition>`: How the ring fades: focus and blur have their own curves.
  - `pub fn placeholder_target(&self) -> f32`: Opacity target of the placeholder: 100 while the text is empty.
  - `pub fn caret_since(&self) -> Option<Instant>`: Since when the caret blinks; none while not focused (no caret).
  - `pub fn caret_on(&self, now: Instant) -> bool`: Whether the caret is on at `now`: on for BLINK_MS, off for BLINK_MS, from `caret_since`.
- pub enum `InputEvent`: What an input field reports.
  - `Focused`: A tap focused it.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::input_field::{InputEvent, InputField};

// state.rs: in struct State, and its first value in State::new
pub wifi_password: InputField,
// wifi_password: InputField::new("Password", ""),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
wifi_password: Slot = "#wifi/password" ["SlotCustomizer"];

// presenter.rs: after every action
model.wifi_password = slot("#wifi/password", &state.wifi_password);

// actions.rs: in the match on the gesture's node name
"#wifi/password" => {
    if let Some(InputEvent::Focused) = state.wifi_password.on(gesture, Instant::now()) {
        // yours: open the keyboard (a Form does it for its fields)
    }
}
```

## Live: `#wifi/password` in smarthome-app (cut from the sources at build time)

```rust
// ui_model.rs:221
wifi_password: Slot = "#wifi/password" ["SlotCustomizer"];

// presenter.rs:74
model.wifi_password = slot("#wifi/password", wifi.password());
```

`state` is `default`, `disabled` or `error`. Focus (`#input-field/focus`), the blinking caret (`#input-field/caret`), the placeholder (`#input-field/placeholder`, shown while empty) are opacities; the text is `#input-field/text` (dots while `secure`). `InputField::new(placeholder, helper)`; `on(gesture, now)` answers `Focused`; typing comes through the `Form` from the keyboard.

## In Figma
A slot of the field's size named `#<screen>/<name>` (`#wifi/password`) with an instance of `#input-field`; the text node carries at least a space as preview (an empty text may be dropped on import).

## Traps
- Text wrapping one character per line, or a focus ring that never shows, is stock harry's list-item bug; the display_safety series in `everyware/patches` fixes both.
- The caret blinks through `UpdatePolicy` from the binding: nothing to animate in the app.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
