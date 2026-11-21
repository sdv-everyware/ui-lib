---
title: Icon button
role: A round button with one glyph and no text, for an action everyone knows by its sign: plus, minus, close, settings.
rust: core/src/button.rs
types: IconButtonKind, Icon, IconButton, ButtonEvent
---
The user taps a round button with a sign on it (minus to lower the heating, x to close a sheet). It darkens under the finger. The app gets one `Clicked` per tap, changes what the button stands for and sends it on, and turns the button off where it cannot go further.

## When to use it
- An action with a sign everyone reads: plus and minus of a value, close, settings, open a device. For an action that needs a word use `#button`.
- Plus and minus around a value: two icon buttons next to a text the app writes. A value the user drags is a `#slider`.

## Quick start
**What.** The minus button next to the temperature: each tap lowers the target by one degree and sends it to the thermostat. At the lowest value the button turns grey and takes no taps.

**Why.** The button has no number of its own. The app keeps the temperature and decides when the button can do nothing more.

<!-- example: docs/examples/icon_button.rs use slot=#climate/minus field=minus -->
```rust
use everyware_ui::button::{ButtonEvent, Icon, IconButton, IconButtonKind};
```

<!-- example: docs/examples/icon_button.rs state slot=#climate/minus field=minus -->
```rust
pub minus: IconButton,
/// What the button changes: the heating's setpoint in degrees.
pub setpoint: i32,
```

<!-- example: docs/examples/icon_button.rs init slot=#climate/minus field=minus -->
```rust
minus: IconButton::new(IconButtonKind::Tonal, Icon::Minus),
setpoint: 21,
```

<!-- example: docs/examples/icon_button.rs model slot=#climate/minus field=minus -->
```rust
minus: Slot = "#climate/minus" ["SlotCustomizer"];
```

<!-- example: docs/examples/icon_button.rs present slot=#climate/minus field=minus -->
```rust
model.minus = slot("#climate/minus", &state.minus);
```

<!-- example: docs/examples/icon_button.rs act slot=#climate/minus field=minus -->
```rust
"#climate/minus" => {
    if let Some(ButtonEvent::Clicked) = state.minus.on(gesture) {
        // one degree down, sent on; at the lowest setpoint the button goes off
        setpoint_reported(state, state.setpoint - 1);
        if !send(Command::Setpoint(state.setpoint)) {
            log::warn!("setpoint: the service is not there, nothing was sent");
        }
    }
}
```

<!-- example: docs/examples/icon_button.rs send slot=#climate/minus field=minus -->
```rust
/// What the screen asks the heating to do. An app has its own link to the service
/// (smarthome-app: screen_binder, a binder call); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Setpoint(i32),
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## How it behaves
- **Pressed.** While the finger is down the circle darkens: a layer fades in over 80 ms and out over 160 ms. The variant stays.
- **Tap.** A down and an up on the button is one `Clicked`; a finger that slides off is no tap.
- **Held.** `press.pressed` is true from the finger's down until it lifts or slides off: a control that acts while it is held (a direction pad, a key that repeats) reads it after every gesture instead of waiting for `Clicked`. Two fingers on two buttons hold both, as far as each finger reaches the app as a touch of its own (harry's evdev reader reports one finger per input frame: two that land in the same frame are not both seen).
- **Disabled.** `press.set_enabled(false)` shows `state=disabled`: a grey glyph on a tonal button, a paler one on glass and scrim. It takes no taps.
- **Touch target.** The button takes taps on 72 × 72 around a circle of 56 (or 48 for `.compact()`), so a small circle is still easy to hit.

## From the app
**What.** The app sets the value from its own state (a tap, or the device reported one), and the button follows: off at the end of the range, on again when the value moves away.

**Why.** A minus that cannot lower anything should look so. The button's state is plain fields: `kind`, `icon`, `size`, and `press` for on and off.

<!-- example: docs/examples/icon_button.rs change slot=#climate/minus field=minus -->
```rust
/// The app sets the setpoint from its own state (a tap, or the device reported one) and the
/// button follows: off at the lowest setpoint (the variant state=disabled, which takes no taps).
pub fn setpoint_reported(state: &mut State, setpoint: i32) {
    state.setpoint = setpoint.clamp(*SETPOINT.start(), *SETPOINT.end());
    state.minus.press.set_enabled(state.setpoint > *SETPOINT.start());
}
```

## In Figma
A slot of 72 × 72 named `#<screen>/<name>` (`#climate/minus`, `#climate/plus`) with an instance of `#icon-button` inside. The instance's `icon` is the preview; the app's `IconButton::new(kind, icon)` picks the real one. A row of icon buttons is a row of slots.

## How it works
`kind` is the look (`tonal` a soft accent circle for the widgets, `glass` white 44 % with a white line for the status row, `scrim` white 15 % over the camera), `icon` the glyph (Lucide names: `minus`, `plus`, `x`, `settings-2`, the four chevrons, `gamepad`, `pencil` ...), `size` the circle (`regular` 56, `compact` 48). `state` is only `default` or `disabled`. Pressed is `#icon-button/pressed`, a layer the binding fades (display-only, no rebuild).

## Traps
- The glyph is part of the variant: another `icon` at runtime is a variant change, a tree rebuild. Pick one glyph per place.
- A glyph that is not in the set's `icon` property cannot be shown: add it to `components.json`, the `Icon` enum and the generator (ui-component skill), do not draw it on the screen.
- The pressed layer is in the variant, not in the slot: a slot drawn by hand without an instance shows no press.

## Example
[docs/examples/icon_button.rs](../examples/icon_button.rs): the whole path, from a tap on minus to the new setpoint sent to the service and the button off at the lowest setpoint, compiled and run by `everyware_ui_recipes_test`.
