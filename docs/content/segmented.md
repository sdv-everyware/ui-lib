---
title: Segmented button
role: One choice of two to four short options side by side, such as a fan's speed: one tap picks it.
rust: core/src/segmented.rs
types: Segmented, SegmentedEvent
---
The user taps one segment of a pill (Low, Med, High) and it lights up; the one before goes dark. The app gets the index of the new segment, reads its label and sends it to the device. When the device reports its speed, the app lights that segment, and it turns the control off while the device is gone.

## When to use it
- One value of two to four short options, side by side in a widget or a card: a fan's speed, a mode.
- More options, or long labels: `#radio` (a list) or `#dropdown`. On or off: `#toggle`. Switching between screens looks alike but is `#tabs`.

## Quick start
**What.** The fan speed of the office, Low, Med and High, in one bar. A tap on Med lights Med, and the app sends the speed to the fan.

**Why.** The bar keeps which speed is chosen and lights it. The app learns the new speed from the tap and sends it on.

<!-- example: docs/examples/segmented.rs use slot=#office/fan field=fan -->
```rust
use everyware_ui::segmented::{Segmented, SegmentedEvent};
```

<!-- example: docs/examples/segmented.rs state slot=#office/fan field=fan -->
```rust
pub fan: Segmented,
```

<!-- example: docs/examples/segmented.rs init slot=#office/fan field=fan -->
```rust
// 2 to 4 labels, the first chosen; the number picks the variant (segments=3)
fan: Segmented::new(["Low", "Med", "High"]),
```

<!-- example: docs/examples/segmented.rs model slot=#office/fan field=fan -->
```rust
fan: Slot = "#office/fan" ["SlotCustomizer"];
```

<!-- example: docs/examples/segmented.rs present slot=#office/fan field=fan -->
```rust
model.fan = slot("#office/fan", &state.fan);
```

<!-- example: docs/examples/segmented.rs act slot=#office/fan field=fan -->
```rust
// a segment reports with its slot: "#office/fan/#segmented/segment-2"
_ if part_of("#office/fan", node).is_some() => {
    let segment = part_of("#office/fan", node).and_then(|p| p.strip_prefix("#segmented/segment-")).and_then(|n| n.parse::<usize>().ok());
    if let Some(SegmentedEvent::Selected(_)) = segment.and_then(|n| n.checked_sub(1)).and_then(|i| state.fan.on(i, gesture)) {
        if !send(Command::Fan(state.fan.value().to_string())) {
            log::warn!("fan: the service is not there, nothing was sent");
        }
    }
}
```

<!-- example: docs/examples/segmented.rs send slot=#office/fan field=fan -->
```rust
/// What the screen asks the fan to do. An app has its own link to the service (smarthome-app:
/// screen_binder::send_matter_command, a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Fan(String),
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## How it behaves
- **Tap.** A down and an up on the same segment picks it: its accent highlight and white label fade in over 160 ms, the old one's fade out. `Selected(index)`; `value()` is its label.
- **The chosen segment.** A tap on the segment that is chosen already does nothing. A finger that slides off is no tap.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled` (paler); the segments take no taps, the choice still shows.
- **Labels.** The texts are the app's, from `Segmented::new`; a long label is cut with an ellipsis.

## From the app
**What.** The app lights the segment the device reported (`select`), and turns the control off while the device is gone.

**Why.** The control must show the fan as it is, also when it changed elsewhere. The state is plain fields: `labels`, `selected`, and `press` for on and off; `select(index)` lights a segment with the transition and ignores an index out of range.

<!-- example: docs/examples/segmented.rs change slot=#office/fan field=fan -->
```rust
/// The app shows what the fan reported: its speed lights that segment (no tap, no event), and no
/// fan turns the control off (the variant state=disabled, the segments take no taps).
pub fn fan_reported(state: &mut State, speed: Option<&str>) {
    state.fan.press.set_enabled(speed.is_some());
    if let Some(i) = speed.and_then(|s| state.fan.labels.iter().position(|l| l == s)) {
        state.fan.select(i);
    }
}
```

## In Figma
A slot 243 × 56 named `#<screen>/<name>` (`#office/fan`) with an instance of `#segmented` of the right `segments` (2, 3 or 4). Labels in the preview are placeholders; the app writes the real ones.

## How it works
The variant is `segments` (the number) and `state` (`default` or `disabled`). The choice is not a variant: each segment is `#segmented/segment-<i>` (a tap target) with `/selected` (the accent highlight), `/label` (ink2) and `/label-selected` (white). The binding lights the chosen one by opacity with the `value` transition (160 ms), so a choice does not rebuild the tree. Segments 3 and 4 are optional parts: `segments=2` has two.

## Traps
- Taps come by the segment's name with the slot in front (`#office/fan/#segmented/segment-2`): match the slot with `part_of`, as the example does, not with `#segmented/segment-` alone, or two segmented buttons on one screen answer each other's taps.
- `Segmented::new` takes 2 to 4 labels and panics on any other number: the set has no variant for it.
- Tabs look alike but switch screens; a segmented button picks a value. Do not use one for the other.

## Example
[docs/examples/segmented.rs](../examples/segmented.rs): the whole path, from a tap on a segment to the command to the fan and the speed the fan reports, compiled and run by `everyware_ui_recipes_test`.
