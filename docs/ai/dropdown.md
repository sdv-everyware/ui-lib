# #dropdown

A closed pill that opens a list of items below it; the open list and the selected item are opacities, the variant only the state.

## Figma

- set: `#dropdown` (node 2-924 in the Library)
- property `state`: `default` | `disabled`
- variant string: `state=default` (the properties in this order; `Dropdown::variant()` produces it, never typed by hand)
- parts the code sets: #dropdown/list opacity (open), #dropdown/item/N/selected opacity: set by code, not a variant
- nodes set by code (`Dropdown`, harry/src/slot.rs): `#dropdown/value`, `#dropdown/list`, `#dropdown/item/{}`, `#dropdown/item/<…>`
- on the screens now: `#status/presence` in #smarthome-root (node 1-18)
- to place one: a frame named `#<screen>/<name>` with an instance of `#dropdown` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- open: 200 ms ease-out
- close: 150 ms ease-in
- select: 150 ms ease-in

## Rust

- module: `core/src/dropdown.rs`, type `Dropdown`; binding: `harry/src/slot.rs` (`impl Slotted for Dropdown`)
- `pub fn new<S: Into<String>>(items: impl IntoIterator<Item = S>) -> Self`
- events `DropdownEvent`: `Selected(usize)`
- pub struct `Dropdown`: A dropdown with its items.
  - field `pub items: Vec<String>`: The items of the list.
  - field `pub selected: usize`: Index of the selected item.
  - field `pub open: bool`: The list is shown.
  - field `pub press: Pressable`: Disabled (the field opens on tap, pressed is not shown).
  - `pub fn value(&self) -> &str`: The text shown in the field.
  - `pub fn on_field(&mut self, gesture: Gesture)`: Applies a gesture on the field: a tap opens or closes the list.
  - `pub fn on_item(&mut self, index: usize, gesture: Gesture) -> Option<DropdownEvent>`: Applies a gesture on item `index`: a tap on the open list selects it and closes. Taps on the closed (transparent) list are ignored.
  - `pub fn close(&mut self)`: Closes the list without a choice (a tap outside).
  - `pub fn list_target(&self) -> f32`: Opacity target of the list: 100 open, 0 closed.
  - `pub fn mark_target(&self, index: usize) -> f32`: Opacity target of the selection mark of item `index`.
  - `pub fn list_transition(&self) -> Option<Transition>`: How the list fades: open and close have their own curves.
- pub enum `DropdownEvent`: What a dropdown reports.
  - `Selected`: The user picked the item with this index.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::dropdown::{Dropdown, DropdownEvent};

// state.rs: in struct State, and its first value in State::new
pub status_presence: Dropdown,
// status_presence: Dropdown::new(["At home", "Away", "Asleep"]),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
status_presence: Slot = "#status/presence" ["SlotCustomizer"];

// presenter.rs: after every action
model.status_presence = slot("#status/presence", &state.status_presence);

// actions.rs: in the match on the gesture's node name
"#status/presence" => state.status_presence.on_field(gesture),
// the open list's items report by their own names, #dropdown/item/1 ...
_ if node.starts_with("#dropdown/item/") => {
    let n = node["#dropdown/item/".len()..].parse::<usize>().ok().and_then(|n| n.checked_sub(1));
    if let Some(DropdownEvent::Selected(i)) = n.and_then(|n| state.status_presence.on_item(n, gesture)) {
        let _ = i; // yours
    }
}
```

The field opens and closes the list (`on_field`); an item reports by its own name, one open list at a time.

`state` is `default` or `disabled`. Open is `#dropdown/list` faded in, the chosen row `#dropdown/item/N/selected` lit, both display-only. The pill shows the selected item's text (`#dropdown/value`); the items report taps by their own names (`#dropdown/item/2`), unambiguous while one dropdown is open on the screen. `on_field(gesture)` opens and closes, `on_item(i, gesture)` answers `Selected(i)`.

## In Figma
The slot keeps the closed pill's height (48) and lets the list hang below it (counter-align MIN, no clipping on the slot), so a row of the dashboard does not grow by the open list. The component is drawn as tall as its open list.

## Traps
- A slot as tall as the component centres the pill half above the row (seen after a regenerate): slot height = the pill, `frameMaskDisabled` on the slot.
- Items are a fixed number in the set (the list is drawn); more items than rows are not shown. The app fills the texts, it cannot add rows.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
