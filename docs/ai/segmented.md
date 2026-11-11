# #segmented

One choice of two to four, Material 3's segmented button: the segments are tap targets, the selected one is lit by opacity.

## Figma

- set: `#segmented` (node 2-1034 in the Library)
- property `segments`: `2` | `3` | `4`
- property `state`: `default` | `disabled`
- variant string: `segments=2, state=default` (the properties in this order; `Segmented::variant()` produces it, never typed by hand)
- parts the code sets: Material 3's segmented button, one choice of 2 to 4: per segment #segmented/segment-<i> (a touch target), /selected (the accent highlight), /label (ink2) and /label-selected (white); the choice lights a segment by opacity, the labels set by code; not a variant
- nodes set by code (`Segmented`, harry/src/slot.rs): `#segmented/segment-{}`, `#segmented/segment-<…>`
- on the screens now: `#office/fan` in Room controls (node 1-178)
- to place one: a frame named `#<screen>/<name>` with an instance of `#segmented` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- value: 160 ms ease-out

## Rust

- module: `core/src/segmented.rs`, type `Segmented`; binding: `harry/src/slot.rs` (`impl Slotted for Segmented`)
- `pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self`
- `pub fn on(&mut self, index: usize, gesture: Gesture) -> Option<SegmentedEvent>`
- events `SegmentedEvent`: `Selected(usize)`
- pub struct `Segmented`: A segmented button.
  - field `pub labels: Vec<String>`: The segments' labels, in order (2 to 4).
  - field `pub selected: usize`: Index of the chosen segment.
  - field `pub press: Pressable`: Enabled, and the last change; a segment's press is in `segments`.
  - `pub fn value(&self) -> &str`: The label of the chosen segment.
  - `pub fn select(&mut self, index: usize)`: Chooses a segment from outside (the device reported it); out of range is ignored.
  - `pub fn target(&self, index: usize) -> f32`: Opacity target of segment `index`'s highlight and white label: 100 chosen, 0 not (the ink2 label takes the rest).
  - `pub fn value_transition(&self) -> Option<Transition>`: How the highlight and the labels move to a new choice.
- pub enum `SegmentedEvent`: What a segmented button reports.
  - `Selected`: The user chose the segment with this index.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::segmented::{Segmented, SegmentedEvent};

// state.rs: in struct State, and its first value in State::new
pub office_fan: Segmented,
// office_fan: Segmented::new(["Low", "Med", "High"]),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
office_fan: Slot = "#office/fan" ["SlotCustomizer"];

// presenter.rs: after every action
model.office_fan = slot("#office/fan", &state.office_fan);

// actions.rs: in the match on the gesture's node name
// the segments report by their own names, #segmented/segment-1 ...
_ if node.starts_with("#segmented/segment-") => {
    let n = node["#segmented/segment-".len()..].parse::<usize>().ok().and_then(|n| n.checked_sub(1));
    if let Some(SegmentedEvent::Selected(i)) = n.and_then(|n| state.office_fan.on(n, gesture)) {
        let _ = i; // yours: state.office_fan.value() is the label
    }
}
```

## Live: `#office/fan` in smarthome-app (cut from the sources at build time)

```rust
// state.rs:89
/// The Office fan's mode.
pub fan: Segmented,
fan: Segmented::new(["Low", "Med", "High"]),

// ui_model.rs:174
office_fan: Slot = "#office/fan" ["SlotCustomizer"];

// presenter.rs:40
model.office_fan = slot("#office/fan", &d.fan);

// actions.rs:340
_ if part_of("#office/fan", node).is_some() => {
    // the fan's segments, "#office/fan/#segmented/segment-<i>" (the part with its slot)
    if let Some(n) = part_of("#office/fan", node)
        .and_then(|p| p.strip_prefix("#segmented/segment-"))
        .and_then(|n| n.parse::<usize>().ok())
    {
        if let Some(SegmentedEvent::Selected(_)) = n.checked_sub(1).and_then(|i| d.fan.on(i, gesture)) {
            log::info!("smarthome: office fan {}", d.fan.value());
            return Some(Control::Fan(d.fan.value().to_string()));
        }
    }
}
```

`segments` is the number (2, 3, 4), `state` `default` or `disabled`. Each segment is `#segmented/segment-<i>` (a tap target) with `/selected` (the lit background) and `/label`; selection fades, the variant does not change. `on(index, gesture)` takes the segment from the node name and answers `Selected(index)`. The texts of the segments are the app's (`set_labels`).

## In Figma
A slot of the variant's size (the design's width per number of segments) named `#<screen>/<name>` (`#office/fan`) with an instance of `#segmented` of the right `segments`. Labels in the preview are placeholders.

## Traps
- `Tabs` look alike but switch screens; a segmented button picks a value. Do not use one for the other (the set names and the roles differ).
- The taps come by the segment's own name (`#segmented/segment-2`): the match arm in actions.rs starts with the prefix, not the slot's name.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
