# #icon-button

A tappable button with one glyph and no label, Material 3's icon button, in two sizes.

## Figma

- set: `#icon-button` (node 2-51 in the Library)
- property `kind`: `tonal` | `glass` | `scrim`
- property `state`: `default` | `disabled`
- property `icon`: `minus` | `plus` | `sliders-horizontal` | `maximize-2` | `chevron-right` | `settings-2` | `wifi` | `x` | `house` | `door-open` | `bed-double` | `armchair` | `droplets`
- property `size`: `regular` | `compact`
- variant string: `kind=tonal, state=default, icon=minus, size=regular` (the properties in this order; `IconButton::variant()` produces it, never typed by hand)
- parts the code sets: #icon-button/pressed opacity (pressed): set by code, not a variant; icon: the design's icons; size: 56, 48
- nodes set by code (`IconButton`, harry/src/slot.rs): `#icon-button/pressed`
- on the screens now: `#status/wifi` in #smarthome-root (node 4-64); `#climate/minus` in #overview (node 1-37); `#climate/plus` in #overview (node 1-43); `#climate/humidity-adjust` in #overview (node 1-55)
- to place one: a frame named `#<screen>/<name>` with an instance of `#icon-button` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- press: 80 ms ease-out
- release: 160 ms ease-out

## Rust

- module: `core/src/button.rs`, type `IconButton`; binding: `harry/src/slot.rs` (`impl Slotted for IconButton`)
- `pub fn new(kind: IconButtonKind, icon: Icon) -> Self`
- `pub fn on(&mut self, gesture: Gesture) -> Option<ButtonEvent>`
- events `ButtonEvent`: `Clicked`
- pub enum `IconButtonKind`: The kind property of #icon-button: the circle, as the dashboard draws it.
  - `Tonal`: Soft accent circle: the widgets' buttons.
  - `Glass`: White 44 % with a white line: the status row.
  - `Scrim`: White 15 %: over the camera.
- pub enum `Icon`: The icon property of #icon-button: the design's icons (lucide names).
  - `Minus`: `minus`
  - `Plus`: `plus`
  - `Sliders`: `sliders-horizontal`
  - `Maximize`: `maximize-2`
  - `ChevronRight`: `chevron-right`
  - `Settings`: `settings-2`
  - `Wifi`: `wifi`
  - `Close`: `x`: closes (a sheet)
  - `House`: `house`: home (a hub's rail)
  - `DoorOpen`: `door-open`: a door, the entrance
  - `BedDouble`: `bed-double`: a bedroom
  - `Armchair`: `armchair`: a living room
  - `Droplets`: `droplets`: humidity, water
- pub struct `IconButton`: A button with an icon. Its size is the design's (regular 56, compact 48).
  - field `pub kind: IconButtonKind`: Which look.
  - field `pub icon: Icon`: Which icon.
  - field `pub size: ButtonSize`: Which size.
  - field `pub press: Pressable`: Pressed and disabled.
  - `pub fn compact(mut self) -> Self`: The same button in the compact size.
- pub enum `ButtonEvent`: What a button reports.
  - `Clicked`: The button was tapped.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::button::{ButtonEvent, Icon, IconButton, IconButtonKind};

// state.rs: in struct State, and its first value in State::new
pub status_wifi: IconButton,
// status_wifi: IconButton::new(IconButtonKind::Tonal, Icon::Plus),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
status_wifi: Slot = "#status/wifi" ["SlotCustomizer"];

// presenter.rs: after every action
model.status_wifi = slot("#status/wifi", &state.status_wifi);

// actions.rs: in the match on the gesture's node name
"#status/wifi" => {
    if let Some(ButtonEvent::Clicked) = state.status_wifi.on(gesture) {
        // yours
    }
}
```

## Live: `#status/wifi` in smarthome-app (cut from the sources at build time)

```rust
// state.rs:28
pub wifi: WifiSheet,
wifi: IconButton::new(IconButtonKind::Glass, Icon::Wifi).compact(),

// ui_model.rs:160
wifi_indicator: Slot = "#status/wifi" ["SlotCustomizer"];

// presenter.rs:29
model.wifi_indicator = slot("#status/wifi", &d.wifi);

// actions.rs:130
HomeAction::Gesture(node, gesture) if node == "#status/wifi" => {
    // the Wi-Fi indicator opens the sheet; the sheet starts over every time
    if state.dash.wifi.on(*gesture) == Some(ButtonEvent::Clicked) && !state.wifi.form.is_open() {
        state.wifi = WifiSheet::opened(state.wifi.networks.clone(), state.wifi.state.clone());
        house_link::send_wifi_command(wifi_scan());
    }
}
```

`kind` is the look (filled, tonal, plain), `icon` the design's glyph (Lucide names), `size` 56 or 48. `state` only `default` or `disabled`; pressed is `#icon-button/pressed`, a layer the code fades in. The glyph is part of the variant: an icon that changes at runtime is a variant change (rebuild), so pick a stable glyph per place, or stack glyphs in a slot of your own and light one by opacity.

## In Figma
A slot of the variant's size (56 × 56 or 48 × 48) named `#<screen>/<name>` (`#climate/plus`) with an instance of `#icon-button` inside; the instance's `icon` is the preview, the app's `IconButton::new(kind, icon)` picks the real one. A row of icon buttons is a row of slots, 8 px apart.

## Traps
- A glyph missing in the `icon` property of the set cannot be shown: `everyware_ui_test` names the variant; add the glyph to `components.json` and the generator (ui-component skill), do not draw it on the screen.
- The pressed layer is in the variant, not in the slot: a slot drawn by hand without an instance shows no press.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
