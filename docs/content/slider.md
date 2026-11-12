---
title: Slider
role: A level the user sets by dragging, from 0 to 100: a light's brightness, a speaker's volume.
rust: core/src/slider.rs
types: Slider, SliderEvent
---
The user puts a finger on the track, the thumb jumps there and follows the finger, and when the finger lifts the app gets the level to send to the device. The app moves the thumb when the device reports a new level.

## When to use it
- A level on a scale where the exact number matters less than the feel: brightness, volume, a blind's position.
- An exact value in steps (a temperature): the stepper of a `#tile` or two `#icon-button`s. A few named levels (Low, Med, High): a `#segmented` button. A device's card with a level on it: a `#tile` of kind `slider`, which has this slider inside.

## Quick start
**What.** The dimmer of the bedroom light: the user drags the thumb, the fill follows the finger, and when the finger lifts the app sends the new level to the lamp.

**Why.** Sending every small move would flood the lamp, so the slider reports the level while it moves and the app sends it once, at the end.

<!-- example: docs/examples/slider.rs use slot=#bedroom/dimmer field=dimmer -->
```rust
use everyware_ui::slider::{Slider, SliderEvent};
use everyware_ui::design::Sizes;
```

<!-- example: docs/examples/slider.rs state slot=#bedroom/dimmer field=dimmer -->
```rust
pub dimmer: Slider,
```

<!-- example: docs/examples/slider.rs init slot=#bedroom/dimmer field=dimmer -->
```rust
// at 65 %, the track where the design puts the slot (the finger's x is the display's)
dimmer: Slider::in_slot(65.0, frame.x, frame.width),
```

<!-- example: docs/examples/slider.rs model slot=#bedroom/dimmer field=dimmer -->
```rust
dimmer: Slot = "#bedroom/dimmer" ["SlotCustomizer"];
```

<!-- example: docs/examples/slider.rs present slot=#bedroom/dimmer field=dimmer -->
```rust
model.dimmer = slot("#bedroom/dimmer", &state.dimmer);
```

<!-- example: docs/examples/slider.rs act slot=#bedroom/dimmer field=dimmer -->
```rust
"#bedroom/dimmer" => match state.dimmer.on(gesture) {
    // the finger drags: the thumb follows it, the light waits
    Some(SliderEvent::Moving(_)) => {}
    // the finger is up: the light gets the level
    Some(SliderEvent::Changed(level)) => {
        if !send(Command::Level(level)) {
            log::warn!("dimmer: the light is not there, nothing was sent");
        }
    }
    None => {}
},
```

## How it behaves
- **Down.** The thumb jumps under the finger and the slider shows `state=pressed`.
- **Drag.** The fill and the thumb follow the finger every frame, also when it leaves the slider; past the ends the level stays at 0 or 100.
- **Up.** The slider goes back to `state=default` and reports `Changed(level)`: the level the device gets.
- **Steps.** With `step` (say `10.0`) the level snaps to its multiples.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled`; no finger moves it.

## From the app
**What.** The device reports a new level (another panel, its own button): the thumb moves there. And the level the user set goes to the device.

**Why.** The slider shows what the device has, so a report sets `value`. Not while a finger is on it (`dragging()`), or the thumb would jump away under the finger. The app sends only `Changed`, once per drag.

<!-- example: docs/examples/slider.rs change slot=#bedroom/dimmer field=dimmer -->
```rust
/// The light reports its level (another panel or its own button changed it): the thumb moves
/// there. Not while a finger is on the slider, or the thumb would jump under it.
pub fn level_reported(state: &mut State, level: f32) {
    if !state.dimmer.dragging() {
        state.dimmer.value = level.clamp(0.0, 100.0);
    }
}
```

<!-- example: docs/examples/slider.rs send slot=#bedroom/dimmer field=dimmer -->
```rust
/// What the screen asks the light to do. An app has its own link to the device (smarthome-app:
/// a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// The light's level, 0..100.
    Level(f32),
}

/// Hands the command to the device; false when it is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## In Figma
A slot of the slider's size named `#<screen>/<name>` (`#bedroom/dimmer`) with an instance of `#slider`. The thumb marker sits at x = 0 of the track, its travel is the track's width. The app's track comes from the slot's x and width in the design (`Slider::in_slot`).

## How it works
`state` is `default`, `pressed` or `disabled`: the variant is only the state. The level moves `#slider/fill` (a progress bar) and `#slider/thumb` (a progress marker); both are meters, display only, so the thumb follows the finger without a new tree. The track is inset 20 on both sides of the component (`TRACK_INSET`); `Slider::in_slot(value, slot_x, slot_width)` takes that off, `Slider::new(value, track_x, track_width)` takes the track itself. `on(gesture)` answers `Moving(level)` while the finger is down and `Changed(level)` when it lifts.

## Traps
- A thumb that starts in the wrong place or runs past the end: the marker node is not at x = 0 of its parent, or its travel is not the track's width.
- The level changes but nothing moves: the marker must be the shape itself (a rounded rect), not a frame with children.
- A slider whose x is a number in the code does not follow the design: take the place from `Sizes::frame`, as the example does. `everyware_ui_test` cannot see a wrong place, the finger can.

## Example
[docs/examples/slider.rs](../examples/slider.rs): the whole path, from the finger on the track to the level sent to the light and a level the light reports, compiled and run by `everyware_ui_recipes_test`.
