---
title: Radio
role: A short list of options where exactly one is on, each with its label: a mode, a level.
rust: core/src/radio.rs
types: Radio, RadioGroup, RadioEvent
---
The user taps one option of a short list (Off, Eco, Comfort) and its circle fills; the one that was on empties. The app gets the index of the new option and sends it on. The app can also set the option the device reported, and turn off an option that cannot be taken now.

## When to use it
- One choice of a few options in a list, each with a label, in a sheet or a settings page.
- Two to four short options side by side in a widget: `#segmented`. Many options or little room: `#dropdown`. On or off: `#toggle`.

## Quick start
**What.** Three heating modes, Off, Eco and Comfort, as three round buttons. A tap on one turns it on and the other two off, and the app sends the mode.

**Why.** Only one mode can be on, so the group decides that for all three buttons. The app only hears which one was chosen.

<!-- example: docs/examples/radio.rs use slot=#x/mode field=mode -->
```rust
use everyware_ui::radio::{RadioEvent, RadioGroup};
```

<!-- example: docs/examples/radio.rs state slot=#x/mode field=mode -->
```rust
pub mode: RadioGroup,
```

<!-- example: docs/examples/radio.rs init slot=#x/mode field=mode -->
```rust
// the labels are drawn in each option (#radio/label); the first is on
mode: RadioGroup::new(["Off", "Eco", "Comfort"]),
```

<!-- example: docs/examples/radio.rs model slot=#x/mode field=mode -->
```rust
mode_1: Slot = "#x/mode/1" ["SlotCustomizer"];
mode_2: Slot = "#x/mode/2" ["SlotCustomizer"];
mode_3: Slot = "#x/mode/3" ["SlotCustomizer"];
```

<!-- example: docs/examples/radio.rs present slot=#x/mode field=mode -->
```rust
model.mode_1 = slot("#x/mode/1", &state.mode.options[0]);
model.mode_2 = slot("#x/mode/2", &state.mode.options[1]);
model.mode_3 = slot("#x/mode/3", &state.mode.options[2]);
```

<!-- example: docs/examples/radio.rs act slot=#x/mode field=mode -->
```rust
_ if node.starts_with("#x/mode/") => {
    // the option from its slot's name: #x/mode/1 is index 0
    if let Some(i) = row_of("#x/mode", node).and_then(|n| n.checked_sub(1)) {
        if let Some(RadioEvent::Selected(i)) = state.mode.on(i, gesture) {
            let mode = state.mode.options[i].label.clone();
            if !send(Command::Mode(mode)) {
                log::warn!("mode: the service is not there, nothing was sent");
            }
        }
    }
}
```

<!-- example: docs/examples/radio.rs send slot=#x/mode field=mode -->
```rust
/// What the screen asks the heating to do. An app has its own link to the service
/// (smarthome-app: screen_binder, a binder call); this one stands in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Mode(String),
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## How it behaves
- **Pressed.** While the finger is down a soft halo shows around the circle (the variant `state=pressed`).
- **Tap.** On the up the tapped option is on (a filled circle), the one before off: `Selected(index)`. A tap on the option that is on already does nothing.
- **Disabled.** `press.set_enabled(false)` on one option shows it pale (`state=disabled`), and a tap does not pick it.
- **Label.** Each option draws its own label (`#radio/label`), from the texts given to `RadioGroup::new`.

## From the app
**What.** The app shows the option the device reported, and turns off an option the device cannot take now.

**Why.** The list must show what is, not only what was tapped. The group's state is plain fields: `options`, each with `label`, `on` and `press`.

<!-- example: docs/examples/radio.rs change slot=#x/mode field=mode -->
```rust
/// The app shows what the heating reported: the mode it is in is on, the others off (no tap, no
/// event), and a mode it cannot take now is off (the variant state=disabled, no taps).
pub fn heating_reported(state: &mut State, mode: usize, can_comfort: bool) {
    for (i, option) in state.mode.options.iter_mut().enumerate() {
        option.on = i == mode;
    }
    state.mode.options[2].press.set_enabled(can_comfort);
}
```

## In Figma
One slot per option, 240 × 48, named `#<screen>/<group>/<i>` from 1 (`#x/mode/1`, `#x/mode/2`, `#x/mode/3`), each with an instance of `#radio`. The label is part of the component; the instance's text is a preview. The slots share nothing in Figma: the group is the app's.

## How it works
`value` is `on` or `off`, `state` `default`, `pressed` or `disabled`: both are variants (a tap on a radio is rare, a variant change is fine), with the `value` transition (160 ms). `RadioGroup` holds the options in order and answers `Selected(index)`; `selected()` is the index of the one that is on. The binding writes the label into `#radio/label`.

## Traps
- The index the app passes to `on(index, gesture)` comes from the node name (`#x/mode/2` is index 1), not from the layer order: name the options in order, from 1.
- `RadioGroup` has no `select` from outside (as `Segmented` has): the change sets `on` of every option itself. Set exactly one, or the group shows two on.

## Example
[docs/examples/radio.rs](../examples/radio.rs): the whole path, from a tap on an option to the command to the service and the mode the device reports, compiled and run by `everyware_ui_recipes_test`.
