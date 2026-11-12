---
title: Input field
role: A one-line text field for a name, a search or a password, typed with the screen's keyboard.
rust: core/src/input_field.rs
types: InputField, InputEvent
---
The user taps the field, the screen's keyboard slides in from the bottom, the text appears as it is typed (dots for a password), and Enter hands it to the app. The app reads the text, and can show an error under the field.

## When to use it
- One line of text the user types: a password, a name, a search. For a choice from a known list use `#dropdown` or `#radio`; for a number with steps, the stepper of a `#tile`.
- Several fields in a sheet (a sign-in, a network with its password): put them in a `Form` (docs/content/form.md). It does the focus and the keys for all of them; this page shows what it does, on one field.

## Quick start
**What.** A password field: a tap brings up the keyboard, the typed characters show as dots, and Enter hands the password to the app.

**Why.** The field does not own a keyboard: the screen has one for every field. So the app opens the keyboard while a field is active and sends the keys there.

<!-- example: docs/examples/input_field.rs use slot=#wifi/password field=password -->
```rust
use everyware_ui::dock::KeyboardDock;
use everyware_ui::input_field::{InputEvent, InputField};
use everyware_ui::keyboard::Outcome;
```

<!-- example: docs/examples/input_field.rs state slot=#wifi/password field=password -->
```rust
pub password: InputField,
/// The screen's one keyboard: open while a field has the focus.
pub osk: KeyboardDock,
```

<!-- example: docs/examples/input_field.rs init slot=#wifi/password field=password -->
```rust
password: {
    let mut field = InputField::new("Password", "At least 8 characters");
    field.secure = true; // dots instead of the characters
    field
},
osk: KeyboardDock::new(),
```

<!-- example: docs/examples/input_field.rs model slot=#wifi/password field=password -->
```rust
password: Slot = "#wifi/password" ["SlotCustomizer"];
// the keyboard: its keys, and its place (it slides in from the bottom edge)
osk_keys: Slot = "#osk/keys" ["SlotCustomizer"];
osk_keys_position: Position = "#osk/keys" ["FollowPositionCustomizer"] = Position::new(KeyboardDock::new().keys_position(SCREEN), None);
```

<!-- example: docs/examples/input_field.rs present slot=#wifi/password field=password -->
```rust
model.password = slot("#wifi/password", &state.password);
model.osk_keys = slot("#osk/keys", &state.osk.keyboard);
model.osk_keys_position = Position::new(state.osk.keys_position(SCREEN), state.osk.transition());
```

<!-- example: docs/examples/input_field.rs act slot=#wifi/password field=password -->
```rust
// a tap on the field: it takes the focus, the keyboard opens
"#wifi/password" => {
    if let Some(InputEvent::Focused) = state.password.on(gesture, Instant::now()) {
        state.osk.follow(true);
    }
}
// a key of the keyboard: it types into the field in focus
_ if node.starts_with("#kbd/") && state.password.focused => {
    match state.osk.keyboard.on(node, gesture, &mut state.password.text) {
        Some(Outcome::Edited) => state.password.edited(Instant::now()),
        Some(Outcome::Done) => {
            // Enter: the text is complete, the keyboard closes
            state.password.blur();
            state.osk.follow(false);
            let password = state.password.text.clone();
            log::info!("password of {} characters entered", password.len()); // yours: connect with it
        }
        None => {}
    }
}
```

## How it behaves
- **Focus.** A tap focuses the field: a ring around it, a blinking caret. Another field or Enter takes the focus away.
- **Typing.** The text shows as it is typed; the placeholder (`Password`) only while the field is empty. With `secure` every character is a dot.
- **Helper and error.** The helper line (`At least 8 characters`) is under the field. An error replaces it, in red: the variant `state=error`. The first key the user types takes the error away.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled`; a tap does not focus it.

## From the app
**What.** The app changes the field from what it learned, without a tap: an error under the field, a cleared text, a new placeholder.

**Why.** The field shows what the app knows (the password was wrong, the network is gone). Its state is plain fields: `text`, `error`, `helper`, `placeholder`, `secure`.

<!-- example: docs/examples/input_field.rs change slot=#wifi/password field=password -->
```rust
/// The app shows what went wrong under the field (the variant state=error, in red) and clears
/// it for the next try; the error goes again when the user types.
pub fn wrong_password(state: &mut State) {
    state.password.error = Some("Wrong password".into());
    state.password.text.clear();
}
```

## In Figma
A slot of the field's size named `#<screen>/<name>` (`#wifi/password`) with an instance of `#input-field`. The text node carries at least a space as preview: an empty text may be dropped on import. The keyboard is the screen's `#osk/keys` slot, once per screen.

## How it works
`state` is `default`, `disabled` or `error`. Focus (`#input-field/focus`), the blinking caret (`#input-field/caret`) and the placeholder (`#input-field/placeholder`) are opacities the binding sets; the text is `#input-field/text`, the helper or the error `#input-field/helper`. The caret blinks by itself (the binding asks harry for the frames): nothing to animate in the app.

## Traps
- A key typed while no field has the focus goes nowhere: check `focused` before handing a key to the field, as the example does.
- Close the keyboard when the field loses the focus (`osk.follow(false)`), or it stays over the screen.

## Example
[docs/examples/input_field.rs](../examples/input_field.rs): the whole path, from a tap on the field to the text on Enter and an error from the app, compiled and run by `everyware_ui_recipes_test`.
