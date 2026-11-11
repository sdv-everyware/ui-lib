# #button

A tappable text button, Material 3's filled, tonal and text button, in two widths.

## Figma

- set: `#button` (node 2-9 in the Library)
- property `kind`: `primary` | `secondary` | `ghost`
- property `state`: `default` | `disabled`
- property `size`: `regular` | `compact`
- variant string: `kind=primary, state=default, size=regular` (the properties in this order; `Button::variant()` produces it, never typed by hand)
- parts the code sets: #button/pressed opacity (pressed): set by code, not a variant; size: the design's width (200, 116), the label centred
- nodes set by code (`Button`, harry/src/slot.rs): `#button/label`, `#button/pressed`
- on the screens now: `#wifi/cancel` in #wifi-content (step=networks) (node 3-6); `#wifi/rescan` in #wifi-content (step=networks) (node 3-18); `#wifi/connect` in #wifi-content (step=password) (node 3-28); `#intercom/talk` in #smarthome-root (node 1-251)
- to place one: a frame named `#<screen>/<name>` with an instance of `#button` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out

## Rust

- module: `core/src/button.rs`, type `Button`; binding: `harry/src/slot.rs` (`impl Slotted for Button`)
- `pub fn new(kind: ButtonKind, label: impl Into<String>) -> Self`
- `pub fn on(&mut self, gesture: Gesture) -> Option<ButtonEvent>`
- events `ButtonEvent`: `Clicked`
- pub enum `ButtonKind`: The kind property of #button.
  - `Primary`: Filled with the accent colour: the main action.
  - `Secondary`: Soft fill.
  - `Ghost`: Text only.
- pub enum `ButtonSize`: The size property of #button and #icon-button: the size is the design's, as Google's components are drawn (a fixed size per variant, the label centred in it), not the label's.
  - `Regular`: #button 200 wide (a sheet's Connect, Cancel); #icon-button 56.
  - `Compact`: #button 116 wide (Talk in a widget's row); #icon-button 48.
- pub struct `Button`: A button with text.
  - field `pub kind: ButtonKind`: Which look.
  - field `pub size: ButtonSize`: Which width.
  - field `pub label: String`: The text on it.
  - field `pub press: Pressable`: Pressed and disabled.
  - `pub fn compact(mut self) -> Self`: The same button in the compact size.
- pub enum `ButtonEvent`: What a button reports.
  - `Clicked`: The button was tapped.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::button::{Button, ButtonEvent, ButtonKind};

// state.rs: in struct State, and its first value in State::new
pub wifi_cancel: Button,
// wifi_cancel: Button::new(ButtonKind::Primary, "Talk").compact(),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
wifi_cancel: Slot = "#wifi/cancel" ["SlotCustomizer"];

// presenter.rs: after every action
model.wifi_cancel = slot("#wifi/cancel", &state.wifi_cancel);

// actions.rs: in the match on the gesture's node name
"#wifi/cancel" => {
    if let Some(ButtonEvent::Clicked) = state.wifi_cancel.on(gesture) {
        // yours
    }
}
```

## Live: `#wifi/cancel` in smarthome-app (cut from the sources at build time)

```rust
// state.rs:144
pub cancel: Button,
cancel: Button::new(ButtonKind::Secondary, "Cancel"),

// ui_model.rs:213
wifi_cancel: Slot = "#wifi/cancel" ["SlotCustomizer"];

// presenter.rs:66
model.wifi_cancel = slot("#wifi/cancel", &wifi.cancel);

// actions.rs:429
"#wifi/cancel" => {
    if wifi.cancel.on(gesture) == Some(ButtonEvent::Clicked) {
        match wifi.step {
            // from the password back to the list (as Android does)
            Step::Password => wifi.back_to_networks(),
            // the form closes with its transition
            Step::Networks => wifi.form.close(),
        }
    }
}
```

A button is the design's fixed size per variant (Google's components are drawn that way, the label centred in the box), not the label's width. `kind` is the look, `size` the width (200 regular for a sheet's actions, 116 compact in a widget's row), `state` only `default` or `disabled`: pressed is a layer the code fades, not a variant.

## In Figma
A button on a screen is a slot of the design's size: a frame 200 × 56 (`size=regular`) or 116 × 56 (`size=compact`) named `#<screen>/<name>` (for example `#wifi/connect`), with an instance of `#button` inside. The instance's label is the designer's preview; the app writes the real label into `#button/label`. Place two buttons as two slots, never one instance twice.

## In the app
```rust
// state: the component
pub connect: Button,                                    // Button::new(ButtonKind::Primary, "Connect")
pub talk: Button,                                       // Button::new(ButtonKind::Primary, "Talk").compact()

// ui_model: one Slot per place on the screen
wifi_connect: Slot = "#wifi/connect" ["SlotCustomizer"];

// presenter: the slot gets the component (variant, label, pressed fade)
model.wifi_connect = slot("#wifi/connect", &wifi.connect);

// actions: the gesture harry reported for the node goes to the button; it answers Clicked
"#intercom/talk" => {
    if d.talk.on(gesture) == Some(ButtonEvent::Clicked) {
        log::info!("smarthome: Talk");
    }
}
```

Disable with `button.press.enabled = false` (the variant becomes `state=disabled`, never animated).

## Traps
- A label changed at runtime needs the text node named `#button/label` in every variant; a renamed inner layer is lost on a variant swap, a text override is kept.
- Two buttons on one screen must sit in two slots: harry finds parts by node name, and the slot gives each clone its own scope.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
