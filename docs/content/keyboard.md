---
title: Keyboard
role: The on-screen keys the user types with: letters, Shift, digits and symbols, Delete, space and Enter.
rust: core/src/keyboard.rs
types: Row, Key, Mode, Keyboard, Outcome
---
The user taps keys and the text grows under the finger's key, Shift gives one capital, 123 switches to digits and symbols, Enter says the text is complete. The app gets an `Outcome` per tap (`Edited` or `Done`) and owns the text: the keyboard only changes the `String` it is handed.

## When to use it
- Inside the keyboard dock (`#keyboard-dock`, docs/content/keyboard_dock.md): the screen's one keyboard, which slides in while a field has the focus. That is where it lives on a screen.
- With an `InputField` (docs/content/input_field.md) or a `Form` (docs/content/form.md): they give it the text of the field in focus. This page shows the keyboard alone, typing into a text the app keeps.
- Not for a choice from a list (`#dropdown`, `#radio`) or a number with steps (the stepper of a `#tile`).

## Quick start
**What.** The keyboard of the screen typing into a text of the app: letters, Shift, the digits page, Delete and Enter.

**Why.** The keyboard does not know what it types into. The app hands it the text on every key, so the same keyboard serves every field.

<!-- example: docs/examples/keyboard.rs use slot=#osk/keys field=keys -->
```rust
use everyware_ui::keyboard::{Keyboard, Mode, Outcome};
```

<!-- example: docs/examples/keyboard.rs state slot=#osk/keys field=keys -->
```rust
/// The keys: the mode (letters, Shift, symbols) and which key is down.
pub keys: Keyboard,
/// What the keys type into: the keyboard keeps no text of its own.
pub typed: String,
```

<!-- example: docs/examples/keyboard.rs init slot=#osk/keys field=keys -->
```rust
keys: Default::default(), // the letters, no key down
typed: String::new(),
```

<!-- example: docs/examples/keyboard.rs model slot=#osk/keys field=keys -->
```rust
keys: Slot = "#osk/keys" ["SlotCustomizer"];
```

<!-- example: docs/examples/keyboard.rs present slot=#osk/keys field=keys -->
```rust
// the labels of the mode, the pressed keys, a tap target per key
model.keys = slot("#osk/keys", &state.keys);
```

<!-- example: docs/examples/keyboard.rs act slot=#osk/keys field=keys -->
```rust
// the keys report by their own names: #kbd/r1/3, #kbd/shift, #kbd/done
_ if node.starts_with("#kbd/") => match state.keys.on(node, gesture, &mut state.typed) {
    Some(Outcome::Edited) => {} // a character, a space or a delete: the text changed
    Some(Outcome::Done) => {
        let text = std::mem::take(&mut state.typed);
        log::info!("{} characters typed", text.len()); // yours: hand it on
    }
    None => {} // the key is still down, or Shift or 123 changed the labels
},
```

## How it behaves
- **A tap.** A key fades its pressed layer in while the finger is on it (80 ms) and out when it lifts (160 ms). It types when the finger lifts on it; a finger that slides off types nothing.
- **Shift.** One capital, then back to lower case, as on a phone. In the symbols Shift shows `#+=` and switches to the other symbol page.
- **123 and abc.** The mode key switches between the letters and the digits with symbols. Only the labels change: no `Outcome`, the text stays.
- **Delete, space.** Delete removes the last character, space adds one. A text stops at 63 characters (`MAX_LEN`, the longest WPA password).
- **Enter.** `Outcome::Done`: the text is complete. The keyboard itself does not close: the dock follows the focus, so the app takes the focus away (docs/content/keyboard_dock.md).

## From the app
**What.** The app picks what the keys show for the next text: the digits first for a PIN, the letters for a name. It sets `mode` and starts the text empty.

**Why.** The mode is a plain field of the keyboard (`Mode::Lower`, `Mode::Upper`, `Mode::Symbols`, `Mode::MoreSymbols`); the presenter writes the labels of that mode at the next frame.

<!-- example: docs/examples/keyboard.rs change slot=#osk/keys field=keys -->
```rust
/// The app picks what the keys show for the next text: digits first for a PIN, the letters for
/// a name. The labels change at the next frame; Shift and 123 still switch.
pub fn start_text(state: &mut State, digits: bool) {
    state.keys.mode = if digits { Mode::Symbols } else { Mode::Lower };
    state.typed.clear();
}
```

## In Figma
`#keyboard` is one component, not a set: rows of 10, 9 and 7 character keys, Shift, Delete, 123, space and Done. On the screen it sits in the dock's slot `#osk/keys`. Every key is a node `#kbd/<row>/<i>` (`#kbd/r1/3`) or `#kbd/shift`, `#kbd/delete`, `#kbd/mode`, `#kbd/space`, `#kbd/done`, with a `/pressed` layer and a `/label` text.

## How it works
The binding (`harry/src/slot.rs`, `impl Slotted for Keyboard`) sets, per key, the label of the current mode (`label_of`), the opacity of `/pressed` (`pressed_target`, with the press or release transition) and a tap target under the key's own name. `on(node, gesture, text)` finds the key from the node name (`key_of_node`), tracks the press, and on the lift applies the key to the text (`press`). The variant is always `state=default`.

## Traps
- The keys report by their own names (`#kbd/r1/3`): one keyboard on a screen, or the names collide. That is why the screen has one dock for every field.
- A key typed while no field has the focus must go nowhere: check the focus before handing the text in (`InputField`), or let a `Form` do it.
- The labels in the design are a preview: the app writes every character label from the mode. A key whose label is fixed (Delete, space, Done) keeps the design's text.

## Example
[docs/examples/keyboard.rs](../examples/keyboard.rs): the whole path, from a pressed key to Shift, the digits from the app and the text on Enter, compiled and run by `everyware_ui_recipes_test`.
