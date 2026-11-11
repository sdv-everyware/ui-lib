# #keyboard

The on-screen keyboard: rows of keys with a character each, pressed keys fade a layer; one component, not a set.

## Figma

- set: `#keyboard` (node 2-2500 in the Library)
- variant string: `` (the properties in this order; `Keyboard::variant()` produces it, never typed by hand)
- parts the code sets: #kbd/<key>/pressed opacity (pressed): set by code; one component, not a set
- nodes set by code (`Keyboard`, harry/src/slot.rs): 
- on the screens now: `#osk/keys` in #keyboard-dock (state=closed) (node 3-38)
- to place one: a frame named `#<screen>/<name>` with an instance of `#keyboard` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out

## Rust

- module: `core/src/keyboard.rs`, type `Keyboard`; binding: `harry/src/slot.rs` (`impl Slotted for Keyboard`)
- `pub fn on(&mut self, node: &str, gesture: Gesture, text: &mut String) -> Option<Outcome>`
- pub enum `Row`: The key rows with a character on them, top to bottom.
  - `One`: 10 keys.
  - `Two`: 9 keys.
  - `Three`: 7 keys, between Shift and Delete.
- pub enum `Key`: A key of the keyboard.
  - `Char`: A character key: row and index from 0.
  - `Shift`: Upper case for one letter, or the other symbol page.
  - `Delete`: Removes the last character.
  - `Mode`: 123 / abc.
  - `Space`: A space.
  - `Done`: Finished with the field.
- pub enum `Mode`: What the keys show.
  - `Lower`: Lower-case letters.
  - `Upper`: After Shift, for one character.
  - `Symbols`: Digits and the common symbols.
  - `MoreSymbols`: The other symbols (Shift in Symbols).
- pub struct `Keyboard`: Keyboard state: the mode and which keys are pressed; the text belongs to the field being edited.
  - field `pub mode: Mode`: What the keys show.
  - `pub fn on_key(&mut self, node: &str, gesture: Gesture) -> Option<Key>`: Applies a gesture on a key node: the key shows pressed while the finger is on it and is returned when it is released over it.
  - `pub fn key_nodes() -> Vec<String>`: The key nodes of the component, in the order of the layout.
  - `pub fn label_of(&self, node: &str) -> Option<String>`: The label the code sets on a key node (the character keys, Shift and mode); None for keys whose label is fixed in the design (Delete, space, Done).
  - `pub fn pressed_target(&self, node: &str) -> f32`: Opacity target of the pressed layer of a key node: 100 pressed, 0 not.
  - `pub fn press_transition(&self, node: &str) -> Option<Transition>`: How the pressed layer of a key node fades.
  - `pub fn char_at(&self, row: Row, index: usize) -> Option<char>`: The character on a key in the current mode.
  - `pub fn label(&self, row: Row, index: usize) -> String`: The label of a character key.
  - `pub fn shift_label(&self) -> &'static str`: The label of Shift: what pressing it switches to.
  - `pub fn mode_label(&self) -> &'static str`: The label of the mode key: what pressing it switches to.
  - `pub fn press(&mut self, key: Key, text: &mut String) -> Outcome`: Applies a key to `text`.
- pub enum `Outcome`: What a key press means for the screen around the keyboard.
  - `Edited`: The text or the mode changed.
  - `Done`: Done: the user is finished with the field.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::keyboard::Keyboard;

// state.rs: in struct State, and its first value in State::new
pub osk_keys: Keyboard,
// osk_keys: Keyboard::default(),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
osk_keys: Slot = "#osk/keys" ["SlotCustomizer"];

// presenter.rs: after every action
model.osk_keys = slot("#osk/keys", &state.osk_keys);

// actions.rs: in the match on the gesture's node name
// the keys report by their own names, #kbd/<row>/<i>
_ if node.starts_with("#kbd/") => {
    let mut text = String::new(); // yours: the text of the field being edited
    if let Some(outcome) = state.osk_keys.on(node, gesture, &mut text) {
        let _ = outcome;
    }
}
```

Usually inside the keyboard dock with a form (`Form`), which routes the keys to the focused field.

## Live: `#osk/keys` in smarthome-app (cut from the sources at build time)

```rust
// ui_model.rs:267
osk_keys: Slot = "#osk/keys" ["SlotCustomizer"];

// presenter.rs:113
model.osk_keys = slot("#osk/keys", &state.osk.keyboard);
```

`#keyboard` has no variants: the keys are `#kbd/<row>/<i>` with `/pressed` (faded in while down) and `/label` (the character of the current mode: letters, shift, symbols). `Keyboard::on(gesture)` takes the key from the node name and answers an `Outcome` (a character, backspace, done, a mode change); the text itself belongs to the field being edited (`Form`, `InputField`). The keyboard sits in the keyboard dock (`#keyboard-dock`), one per screen.

## In Figma
Inside the dock's slot (`#<app>/keys`), an instance of `#keyboard`; every key's label is a text override (static labels survive a rebuild), the pressed layer is in the component.

## Traps
- Keys report by their own names (`#kbd/r1/3`): one keyboard on the screen at a time, or the names collide.
- A label as a renamed inner layer is lost on a variant swap of the dock; labels are text overrides (`labelled()` in the generator).

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
