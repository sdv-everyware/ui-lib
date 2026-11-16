---
title: Toggle
role: An on/off switch for something that stays on or off: a light, a setting.
rust: core/src/toggle.rs
types: Toggle, ToggleEvent
---
The user taps the switch and it slides over: the thumb moves, the track turns blue. The app gets the new value and sends it to the device. When the device reports its value (someone used the wall switch), the app sets it and the switch slides by itself.

## When to use it
- One thing that is on or off and stays so: a light, Wi-Fi, a setting. The change takes effect at once, there is no Save.
- One choice of more than two values: `#segmented` (two to four, side by side) or `#radio` (a list). An action that happens once (call, connect): `#button`.
- A device with more than the switch (a name, a state line): a `#tile` of kind toggle, which draws the switch for you.

## Quick start
**What.** The light switch of a room: the user taps it, the switch moves at once and the app sends on or off to the lamp.

**Why.** The switch moves before the lamp answers, so the screen feels instant. When the lamp reports its real state, the app sets the switch to it.

<!-- example: docs/examples/toggle.rs use slot=#entryway/light field=light -->
```rust
use everyware_ui::toggle::{Toggle, ToggleEvent};
```

<!-- example: docs/examples/toggle.rs state slot=#entryway/light field=light -->
```rust
pub light: Toggle,
```

<!-- example: docs/examples/toggle.rs init slot=#entryway/light field=light -->
```rust
light: Toggle::new(false),
```

<!-- example: docs/examples/toggle.rs model slot=#entryway/light field=light -->
```rust
light: Slot = "#entryway/light" ["SlotCustomizer"];
```

<!-- example: docs/examples/toggle.rs present slot=#entryway/light field=light -->
```rust
model.light = slot("#entryway/light", &state.light);
```

<!-- example: docs/examples/toggle.rs act slot=#entryway/light field=light -->
```rust
"#entryway/light" => {
    if let Some(ToggleEvent::Toggled(on)) = state.light.on(gesture) {
        // the switch moved already; the light follows when the command arrives
        if !send(Command::Light(on)) {
            log::warn!("light: the service is not there, nothing was sent");
        }
    }
}
```

<!-- example: docs/examples/toggle.rs send slot=#entryway/light field=light -->
```rust
/// What the screen asks the light to do. An app has its own link to the service (smarthome-app:
/// screen_binder::send_matter_command, a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Light(bool),
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## How it behaves
- **Pressed.** While the finger is down the thumb stretches (the variant `state=pressed`); the value does not change yet.
- **Tap.** On the up over the switch the value flips: the thumb slides and the blue track fades in or out over 200 ms. A finger that slides off is no tap.
- **From the device.** `set(value)` moves the thumb the same way, without an event.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled` (paler) and takes no taps; the value still shows.

## From the app
**What.** The app shows what the device reported: its value, or off while the device is gone.

**Why.** The switch must show the light as it is, also when it changed elsewhere. A switch for a device that is not there would send to nobody, so the app turns it off. The state is plain fields: `value`, and `press` for on and off.

<!-- example: docs/examples/toggle.rs change slot=#entryway/light field=light -->
```rust
/// The app shows what the light reported: its value moves the thumb (no tap, no event), and no
/// light (gone from the network) turns the switch off: the variant state=disabled, no taps.
pub fn light_reported(state: &mut State, on: Option<bool>) {
    match on {
        Some(on) => {
            state.light.set(on);
            state.light.press.set_enabled(true);
        }
        None => state.light.press.set_enabled(false),
    }
}
```

## In Figma
A slot 68 × 40 named `#<screen>/<name>` (`#entryway/light`) with an instance of `#toggle` inside. The preview shows the off position; the app's value moves the thumb.

## How it works
`state` is `default`, `pressed` or `disabled`. On and off are not variants: `#toggle/thumb` is a horizontal progress marker the binding moves from 0 to 100 (`target()`), `#toggle/on` the blue track whose opacity follows the same value. Both are display-only in harry and move with the `value` transition (200 ms, ease-in-out), so a flip does not rebuild the tree. The marker sits at x = 0 of the track with its travel as startX..endX.

## Traps
- On and off as a variant would switch in a step and rebuild the tree: keep the value on the marker and the opacity, as the binding does.
- Two toggles on one screen share the part names `#toggle/thumb` and `#toggle/on`: each in its own slot, or they move together.
- A toggle inside a list row needs a harry where a list item's customizations reach the animation ([super-harry](https://github.com/sdv-everyware/super-harry) has it); the harry of android-17.0.0_r1 switches it instead of sliding.

## Example
[docs/examples/toggle.rs](../examples/toggle.rs): the whole path, from a tap on the switch to the command to the light and the value the light reports, compiled and run by `everyware_ui_recipes_test`.
