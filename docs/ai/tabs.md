# #tabs

The screen's navigation, Material 3's tabs: the chosen tab is lit by opacity, a tap switches the screen with the screens' effect.

## Figma

- set: `#tabs` (node 2-2736 in the Library)
- property `tabs`: `2` | `3` | `4`
- property `state`: `default` | `disabled`
- variant string: `tabs=2, state=default` (the properties in this order; `Tabs::variant()` produces it, never typed by hand)
- parts the code sets: Material 3's tabs, the screen's navigation (a segmented button picks a value, tabs switch screens): per tab #tabs/tab-<i> (a touch target) with #tabs/tab-<i>/selected (the white pill), /label (ink2) and /label-selected (accent, semi bold) lit by opacity, set by code, not a variant; the labels set by code
- nodes set by code (`Tabs`, harry/src/slot.rs): `#tabs/tab-{}`, `#tabs/tab-<…>`
- on the screens now: `#home/tabs` in #smarthome-root (node 1-11); `#rooms/tabs` in #rooms-screen (state=hidden) (node 5-6)
- to place one: a frame named `#<screen>/<name>` with an instance of `#tabs` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- value: 160 ms ease-out
- switch: 300 ms ease-out

## Rust

- module: `core/src/tabs.rs`, type `Tabs`; binding: `harry/src/slot.rs` (`impl Slotted for Tabs`)
- `pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self`
- `pub fn on(&mut self, index: usize, gesture: Gesture) -> Option<TabsEvent>`
- events `TabsEvent`: `Selected(usize)`
- pub struct `Tabs`: A tab bar.
  - field `pub labels: Vec<String>`: The tabs' labels, in order (2 to 4).
  - field `pub selected: usize`: Index of the chosen tab.
  - field `pub previous: Option<usize>`: The tab chosen before the last switch: the screen that leaves (a slide's direction).
  - field `pub press: Pressable`: Enabled, and the last change; a tab's press is in `tabs`.
  - `pub fn select(&mut self, index: usize)`: Chooses a tab from outside (the app opens a screen itself); out of range or the chosen one is ignored.
  - `pub fn target(&self, index: usize) -> f32`: Opacity target of tab `index`'s pill and accent label: 100 chosen, 0 not (the ink2 label takes the rest).
  - `pub fn value_transition(&self) -> Option<Transition>`: How the pill and the labels move to a new tab.
  - `pub fn switch_transition(&self) -> Option<Transition>`: The time of a screen switch, which the effect splits (components.json, tabs/switch).
  - `pub fn screen(&self, screen: usize, effect: Effect, width: f32) -> ScreenTarget`: Where the screen of tab `screen` goes with this effect: its opacity and its offset from where the design puts it. `width` is the distance between two screens of a slide.
- pub enum `TabsEvent`: What tabs report.
  - `Selected`: The user chose the tab with this index.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::tabs::{Tabs, TabsEvent};
use everyware_ui::screens::Effect;
use everyware_ui::part_of;
use everyware_ui::ScreenView;

// state.rs: in struct State, and its first value in State::new
pub home_tabs: Tabs,
// home_tabs: Tabs::new(["Overview", "Rooms", "Automations"]),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
home_tabs: Slot = "#home/tabs" ["SlotCustomizer"];
// one field per screen behind the tabs, the first one shown
home_tabs_1: ScreenView = "#overview" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(100.0, None), x: None };

// presenter.rs: after every action
model.home_tabs = slot("#home/tabs", &state.home_tabs);
// each screen: its opacity (and x for Effect::Slide) from the effect; 0.0 is its x in the design
model.home_tabs_1 = ScreenView::new(state.home_tabs.screen(0, Effect::Fade, 1920.0), 0.0);

// actions.rs: in the match on the gesture's node name
// a tab reports with its tab bar: "#home/tabs/#tabs/tab-2"
_ if part_of("#home/tabs", node).is_some() => {
    let tab = part_of("#home/tabs", node).and_then(|p| p.strip_prefix("#tabs/tab-")).and_then(|n| n.parse::<usize>().ok());
    if let Some(TabsEvent::Selected(i)) = tab.and_then(|n| n.checked_sub(1)).and_then(|i| state.home_tabs.on(i, gesture)) {
        let _ = i; // yours: the presenter already shows screen i; send it on here if anyone else needs it
    }
}
```

The presenter shows the chosen screen: one `ScreenView` per screen, `state.home_tabs.screen(i, effect, width)`. `Effect::Fade` (the default), `FadeThrough`, `Slide` and `None` change how; the time is `tabs`/`switch` in components.json.

## Live: `#home/tabs` in smarthome-app (cut from the sources at build time)

```rust
// state.rs:56
/// The header's tabs (#home/tabs): which screen the panel shows (SCREENS).
pub tabs: Tabs,

// ui_model.rs:187
home_tabs: Slot = "#home/tabs" ["SlotCustomizer"];

// presenter.rs:47
model.home_tabs = slot("#home/tabs", &state.tabs);

// actions.rs:149
HomeAction::Gesture(node, gesture) if part_of("#home/tabs", node).is_some() => {
    on_tabs(state, node, *gesture);
}
```

`tabs` is the number (2, 3, 4), `state` `default` or `disabled`. Each tab is `#tabs/tab-<i>` (a tap target) with `/selected` (the pill), `/label` and `/label-selected`; the choice fades, the variant does not change. `on(index, gesture)` answers `Selected(index)`; `Tabs` keeps the tab before it (`previous`) for the direction of a switch. The labels are the app's.

The screens (core/src/screens.rs): `tabs.screen(i, effect, width)` gives screen `i` its opacity and its x offset, each with a transition. The time of a switch is `tabs`/`switch` in components.json; the effect splits it: `Fade` (the default), `FadeThrough`, `Slide`, `None`. The app binds each screen with `ScreenCustomizer` and `ScreenView::new(target, design_x)`, so a switch is display only. A new animation is one more `Effect` with its test.

## In Figma
One slot in the header or the bottom bar, named `#<screen>/tabs` (`#home/tabs`), with an instance of `#tabs` of the right number. Every tab is a screen or a frame of the prototype for designers (figma-design skill); on the device the app draws the screen.

## Traps
- Tabs are not a segmented button: a segmented button picks a value in a widget, tabs change what the whole screen shows.
- A tap comes with its tab bar (`#home/tabs/#tabs/tab-2`): match the slot and the part, two tab bars on one screen stay apart.
- A screen at opacity 0 is not drawn and takes no taps; a screen with a variant of its own (shown, hidden) is set to shown once, then the effect shows or hides it.
- `Slide` moves a screen by its x in the parent: give `design_x` from the design (`Sizes::frame`), or the screen jumps.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
