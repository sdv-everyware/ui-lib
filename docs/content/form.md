---
title: Form (code)
role: A sheet the user fills in: a tap on a field brings up the keyboard, Enter hands the text to the app.
rust: core/src/form.rs
types: Form, FormEvent
---
The user opens a sheet with fields (a network's password, a name), taps a field, types with the screen's keyboard and presses Enter. The app gets `FormEvent::Submitted` with the field's index and the text in it, sends it on, and shows the answer under the field or closes the sheet.

## When to use it
- A sheet with one or more `InputField`s: a sign-in, a network with its password, a rename. The form does the focus and sends the keys to the field in focus.
- A sheet without fields (a device's details, a settings panel) is a plain `Sheet` (docs/content/sheet.md).
- One field on the screen itself, not in a sheet, is an `InputField` with the keyboard dock (docs/content/input_field.md).
- Something that slides in and is not a card is an `Overlay` (docs/content/overlay.md).

## Quick start
**What.** The Wi-Fi password dialog: it slides in, the user types the password with the screen's keyboard, Enter joins the network, and a tap outside closes it.

**Why.** The dialog and its fields work as one: it knows which field is active, so the keys go there, and it hands the text to the app on Enter.

<!-- example: docs/examples/form.rs use slot=#wifi field=wifi -->
```rust
use everyware_ui::dock::{KeyboardDock, KEYBOARD_DOCK_H};
use everyware_ui::form::{Form, FormEvent};
use everyware_ui::input_field::InputField;
use everyware_ui::sheet::Layout;
```

<!-- example: docs/examples/form.rs state slot=#wifi field=wifi -->
```rust
/// The sheet with its fields, in the order given: 0 is the password.
pub wifi: Form,
/// The screen's one keyboard: open while a field of any form has the focus.
pub osk: KeyboardDock,
```

<!-- example: docs/examples/form.rs init slot=#wifi field=wifi -->
```rust
wifi: {
    let mut password = InputField::new("Password", "At least 8 characters");
    password.secure = true;
    let mut form = Form::new(WIFI_LAYOUT, [("#wifi/password", password)]);
    // the card hugs the content: its height comes from the design (the .dcf)
    form.sheet = form.sheet.with_content_height(sizes.height("#wifi-content", "step=password"));
    form
},
osk: KeyboardDock::new(),
```

<!-- example: docs/examples/form.rs model slot=#wifi field=wifi -->
```rust
// the sheet: the card moves, the scrim fades and takes taps while open
wifi_card: Position = "#wifi/card" ["FollowPositionCustomizer"] = Position::new(WIFI_LAYOUT.closed(SCREEN), None);
wifi_scrim: Meter = "#wifi/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
wifi_scrim_taps: TapTarget = "#wifi/scrim" ["TapTargetCustomizer"];
_wifi_card_tap: () = "#wifi/card" interactive;
// the content shows its password step; the field in it
wifi_step: Variant = "#wifi/content" = Variant("step=password".to_string());
wifi_password: Slot = "#wifi/password" ["SlotCustomizer"];
// the screen's keyboard
osk_keys: Slot = "#osk/keys" ["SlotCustomizer"];
osk_keys_position: Position = "#osk/keys" ["FollowPositionCustomizer"] = Position::new(KeyboardDock::new().keys_position(SCREEN), None);
```

<!-- example: docs/examples/form.rs present slot=#wifi field=wifi -->
```rust
// a dialog is centred in the screen above the keyboard while the keyboard is open
let free_height = if state.osk.open() { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 };
let slide = state.wifi.transition();
model.wifi_card = Position::new(state.wifi.sheet.card_position(SCREEN, free_height), slide);
model.wifi_scrim = Meter::new(state.wifi.sheet.scrim(), slide);
model.wifi_scrim_taps = TapTarget(state.wifi.is_open());
model.wifi_password = slot("#wifi/password", state.wifi.field(0));
model.osk_keys = slot("#osk/keys", &state.osk.keyboard);
model.osk_keys_position = Position::new(state.osk.keys_position(SCREEN), state.osk.transition());
```

<!-- example: docs/examples/form.rs act slot=#wifi field=wifi -->
```rust
if state.wifi.is_open() {
    match node {
        // a tap on the scrim closes the sheet and takes the focus away
        "#wifi/scrim" => {
            if matches!(gesture, Gesture::Up { inside: true, .. }) {
                state.wifi.close();
            }
        }
        // a tap on a field, a key of the keyboard: the form routes them
        _ => match state.wifi.on(node, gesture, Instant::now(), &mut state.osk.keyboard) {
            Some(FormEvent::Submitted(_)) => submit(state),
            Some(FormEvent::Focused(_)) | Some(FormEvent::Edited(_)) | None => {}
        },
    }
}
// after every action: the keyboard is open while a field has the focus
state.osk.follow(state.wifi.focused().is_some());
```

## How it behaves
- **Opening and closing.** The card slides in from below the screen with the open spring (550 ms) and the scrim darkens to 35 %; closing slides it back with the close spring (400 ms). `close()` also takes the focus away, so the keyboard goes with it.
- **The scrim.** A tap on the scrim closes the form. It takes taps only while the form is open; closed, taps go through to the screen.
- **Focus.** A tap on a field gives it the focus and takes it from the others; the keyboard slides in. A tap on another field moves the focus, the keyboard stays.
- **The keyboard and the card.** While the keyboard is open the dialog is centred in the screen above it, or at the top when it is higher than that. A side panel stays where it is.
- **Enter.** `Submitted(i)`. The focus stays on the field: the app decides. Here a short password turns the field red and the keyboard stays for the next try; a long one takes the focus away (the keyboard slides out) and goes to the service.

## From the app
**What.** Enter checks the text and sends it on; the app opens the form empty and shows the service's answer: joined closes the sheet, a failure shows under the field with the text cleared.

**Why.** The form shows what the app knows, outside a gesture. Its fields are plain `InputField`s (`field_mut(i).error`, `.helper`, `.text`); `clear()` empties all of them; `blur()` takes the focus away.

<!-- example: docs/examples/form.rs change slot=#wifi field=wifi -->
```rust
/// Enter: a short password turns the field red at once; a long enough one goes to the
/// service, and the keyboard closes while the app waits for the answer.
fn submit(state: &mut State) {
    let len = state.wifi.field(0).text.chars().count();
    if len < MIN_LEN {
        state.wifi.field_mut(0).error = Some(format!("At least {MIN_LEN} characters ({len} so far)"));
        return;
    }
    let password = state.wifi.field(0).text.clone();
    state.wifi.blur();
    if send(Command::Join { password }) {
        state.wifi.field_mut(0).helper = "Joining...".into();
    }
}

/// The app opens the form empty (a tap on the Wi-Fi indicator) and shows the service's answer:
/// joined closes it, a failure shows under the field.
pub fn open_wifi(state: &mut State) {
    state.wifi.clear();
    state.wifi.open();
}

pub fn joined(state: &mut State, result: Result<(), String>) {
    match result {
        Ok(()) => state.wifi.close(),
        Err(error) => {
            state.wifi.field_mut(0).error = Some(format!("{error}: try again"));
            state.wifi.field_mut(0).text.clear();
        }
    }
}
```

The submitted text goes to the service through the app's own link to it:

<!-- example: docs/examples/form.rs send slot=#wifi field=wifi -->
```rust
/// What the screen asks the Wi-Fi service to do. An app has its own link to the service
/// (smarthome-app: screen_binder::send_wifi_command, a binder call to the Wi-Fi bundle); this
/// one stands in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Join { password: String },
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    match &command {
        Command::Join { password } => log::info!("join with a password of {} characters", password.len()),
    }
    true
}
```

## In Figma
A form is a sheet (docs/content/sheet.md): an instance of `#sheet` with the app's content component swapped in (`#wifi-content`) and its parts renamed (`#wifi/scrim`, `#wifi/card`, `#wifi/content`). The fields are slots of `#input-field` inside the content (`#wifi/password`). The keyboard is the screen's `#osk/keys`. The form itself is code.

## How it works
`Form::new(layout, fields)` makes a closed `Sheet` and keeps the fields in the order given; an index in `FormEvent` is that order. `on(node, gesture, now, keyboard)`: a key node (`#kbd/...`) goes to the field in focus through the keyboard and answers `Edited(i)` or `Submitted(i)`; a field's node goes to that field and answers `Focused(i)` when it took the focus; any other node answers `None`, so the app's own arms (the scrim, the buttons) come next. `variant()` and `transition()` are the sheet's.

## Traps
- Hand the form gestures only while it is open: `on` does not check, and a tap on a field of a closed form would focus it and bring up the keyboard.
- `Submitted` does not take the focus away: call `blur()` when the keyboard should go.
- The field's text is the form's, not the keyboard's: read it with `field(i).text`.
- Follow the focus after every action (`osk.follow(form.focused().is_some())`), also after `close()`, or the keyboard stays over the screen.

## Example
[docs/examples/form.rs](../examples/form.rs): the whole path, from opening the sheet to the typed password on Enter, an error from the app and the scrim that closes it, compiled and run by `everyware_ui_recipes_test`.
