# #radio

A group of radio buttons where one option is on; the group owns the selection, each button is a variant.

## Figma

- set: `#radio` (node 2-898 in the Library)
- property `value`: `off` | `on`
- property `state`: `default` | `pressed` | `disabled`
- variant string: `value=off, state=default` (the properties in this order; `Radio::variant()` produces it, never typed by hand)
- nodes set by code (`Radio`, harry/src/slot.rs): `#radio/label`
- on the screens now: not used yet
- to place one: a frame named `#<screen>/<name>` with an instance of `#radio` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out
- value: 160 ms ease-out

## Rust

- module: `core/src/radio.rs`, type `Radio`; binding: `harry/src/slot.rs` (`impl Slotted for Radio`)
- events `RadioEvent`: `Selected(usize)`
- pub struct `Radio`: One radio button; part of a RadioGroup.
  - field `pub label: String`: The text next to it.
  - field `pub on: bool`: This option is the selected one.
  - field `pub press: Pressable`: Pressed and disabled.
- pub struct `RadioGroup`: A group of radio buttons with one selected.
  - field `pub options: Vec<Radio>`: The options, in order.
  - `pub fn selected(&self) -> Option<usize>`: The index of the selected option.
- pub enum `RadioEvent`: What a radio group reports.
  - `Selected`: The user picked the option with this index.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::radio::{RadioEvent, RadioGroup};

// state.rs: in struct State, and its first value in State::new
pub home_radio: RadioGroup,
// home_radio: RadioGroup::new(["Off", "Low", "High"]),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
home_radio_1: Slot = "#home/radio/1" ["SlotCustomizer"];
home_radio_2: Slot = "#home/radio/2" ["SlotCustomizer"];
home_radio_3: Slot = "#home/radio/3" ["SlotCustomizer"];

// presenter.rs: after every action
model.home_radio_1 = slot("#home/radio/1", &state.home_radio.options[0]);
model.home_radio_2 = slot("#home/radio/2", &state.home_radio.options[1]);
model.home_radio_3 = slot("#home/radio/3", &state.home_radio.options[2]);

// actions.rs: in the match on the gesture's node name
_ if node.starts_with("#home/radio/") => {
    // the option from its slot's name: #home/radio/1 is index 0
    if let Some(i) = row_of("#home/radio", node).and_then(|n| n.checked_sub(1)) {
        if let Some(RadioEvent::Selected(i)) = state.home_radio.on(i, gesture) {
            let _ = i; // yours
        }
    }
}
```

One slot per option, `#home/radio/1`, `#home/radio/2` ...; the group decides which is on.

`value` is `on` or `off`, `state` `default`, `pressed` or `disabled`. The group (`RadioGroup`) holds which index is selected and answers `Selected(index)`; each option is a `Radio` whose variant follows the group. Selection is a variant change (rare, a tap), so there is no meter here.

## In Figma
One slot per option, 24 × 24 (or the row that holds it) named `#<screen>/<group>/<i>`, each with an instance of `#radio`; the label is a text of the screen next to it, not part of the component. The group is the app's: the slots share nothing in Figma.

## Traps
- The index the app passes to `on(index, gesture)` comes from the node name (`#x/2`), not from the layer order: name the options in order.
- A label inside the component would need `#radio/label` in every variant; the design keeps labels on the screen.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
