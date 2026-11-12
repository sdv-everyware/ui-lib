---
title: Keyboard dock
role: The screen's one keyboard, which slides up from the bottom while the user types into a field and away when the field is done.
rust: core/src/dock.rs
types: KeyboardDock
---
The user taps a field and the keys slide up from the bottom edge; Enter, a tap on the scrim of the sheet or a closed sheet takes the focus away and the keys slide out again. The app gets one keyboard for every field on the screen and one rule: after every action, the dock follows the focus.

## When to use it
- Once per screen that has a field. It is the screen's keyboard, as a system keyboard is, not a part of each sheet: every `InputField` and every `Form` on the screen types through it.
- The keys themselves are the `Keyboard` in it (docs/content/keyboard.md); the field is an `InputField` (docs/content/input_field.md) or a field of a `Form` (docs/content/form.md).

## Quick start
**What.** The keyboard sliding in from the bottom while a field is active, and out again when it is not.

**Why.** The dock does not know the fields, only whether one is active. The app tells it after every action, so it never stays open by mistake.

<!-- example: docs/examples/keyboard_dock.rs use slot=#osk/keys field=osk -->
```rust
use everyware_ui::dock::{KeyboardDock, KEYBOARD_DOCK_H};
use everyware_ui::input_field::InputField;
use everyware_ui::keyboard::Outcome;
```

<!-- example: docs/examples/keyboard_dock.rs state slot=#osk/keys field=osk -->
```rust
/// The screen's one keyboard: closed below the screen, or docked at the bottom.
pub osk: KeyboardDock,
/// A field it types into (yours: every field of the screen).
pub password: InputField,
```

<!-- example: docs/examples/keyboard_dock.rs init slot=#osk/keys field=osk -->
```rust
osk: KeyboardDock::new(),
password: InputField::new("Password", "At least 8 characters"),
```

<!-- example: docs/examples/keyboard_dock.rs model slot=#osk/keys field=osk -->
```rust
// the keys in the dock's slot, and the slot's place: it moves, the dock's variant never changes
keys: Slot = "#osk/keys" ["SlotCustomizer"];
keys_position: Position = "#osk/keys" ["FollowPositionCustomizer"] = Position::new(KeyboardDock::new().keys_position(SCREEN), None);
password: Slot = "#wifi/password" ["SlotCustomizer"];
```

<!-- example: docs/examples/keyboard_dock.rs present slot=#osk/keys field=osk -->
```rust
model.keys = slot("#osk/keys", &state.osk.keyboard);
model.keys_position = Position::new(state.osk.keys_position(SCREEN), state.osk.transition());
model.password = slot("#wifi/password", &state.password);
```

<!-- example: docs/examples/keyboard_dock.rs act slot=#osk/keys field=osk -->
```rust
match node {
    // a tap on the field gives it the focus
    "#wifi/password" => {
        state.password.on(gesture, Instant::now());
    }
    // a key types into the field in focus; Enter takes the focus away
    _ if node.starts_with("#kbd/") && state.password.focused => {
        match state.osk.keyboard.on(node, gesture, &mut state.password.text) {
            Some(Outcome::Edited) => state.password.edited(Instant::now()),
            Some(Outcome::Done) => state.password.blur(),
            None => {}
        }
    }
    _ => {}
}
// after every action: the dock is open while a field has the focus
state.osk.follow(state.password.focused);
```

## How it behaves
- **Opening.** The field takes the focus and the keys slide up with the open spring (550 ms) to the bottom of the screen, 420 px high (`KEYBOARD_DOCK_H`).
- **Closing.** No field has the focus any more (Enter, the sheet closed, the app took it away) and the keys slide down with the close spring (400 ms), 40 px past the screen's edge, where the design draws them.
- **Another field.** The focus moves from one field to another: the dock stays open, no animation.
- **Dialogs.** While the dock is open a dialog is centred in the 660 px above it, or sits at the top when it is higher than that. A side panel keeps its place.
- **No scrim.** The dock has none: the sheet above the keys still takes taps (a button, another field).

## From the app
**What.** The app closes the keyboard without a tap: the screen goes to sleep, the sheet with the field is closed by the app. It takes the focus away and the dock follows. A dialog asks for the height above the keys.

**Why.** The dock shows the focus; it has no state of its own to keep in step. A sheet's presenter needs the free height to centre a dialog above it (docs/content/sheet.md).

<!-- example: docs/examples/keyboard_dock.rs change slot=#osk/keys field=osk -->
```rust
/// The app closes the keyboard without a tap (the screen goes to sleep, the sheet with the
/// field closes): it takes the focus away and the dock follows.
pub fn screen_off(state: &mut State) {
    state.password.blur();
    state.osk.follow(false);
}

/// The height a dialog has while the dock is open: the screen above the keys.
pub fn free_height(state: &State) -> f32 {
    if state.osk.open() { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 }
}
```

## In Figma
`#keyboard-dock` is a set (`state=closed|open`) in the app's design, the screen's size, with the slot `#osk/keys` holding `#keyboard`. On the screen it is an instance in the `state=closed` drawing, inside a plain absolute frame over the whole screen, the last layer of the root so it is above every sheet. The open variant is for the designer's view and the prototype; harry never shows it.

## How it works
`follow(focus)` opens or closes the dock's `Overlay` (docs/content/overlay.md); `open()` says which. `keys_position(screen)` is the slot's place: `(0, screen.1 - KEYBOARD_DOCK_H)` open, `(0, screen.1 + 40)` closed. The app gives it to `#osk/keys` as a `Position` with `FollowPositionCustomizer` and `transition()`, a display-only move in harry: no new tree, no layout. `variant()` still names the state for tests.

## Traps
- An instance placed absolute by itself lands in the root's flow after import: put the instance in a plain frame with absolute positioning.
- Centre a dialog in the free height while the dock is open (`SCREEN.1 - KEYBOARD_DOCK_H`), or the keys cover its lower half.
- Never switch the dock's variant: a variant switch rebuilds the tree and lays the screen out on every frame of the animation, and the dock stutters (frames of 100 to 170 ms on the RB3). Move the slot.
- With several forms on a screen, follow the focus of all of them in one call. Two calls in a row would open and close it in the same action.

## Example
[docs/examples/keyboard_dock.rs](../examples/keyboard_dock.rs): the whole path, from a tap on the field to the keys sliding in and out and the app closing them, compiled and run by `everyware_ui_recipes_test`.
