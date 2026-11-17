---
title: Tabs
role: The screen's navigation: a tab per screen, and a tap shows that screen.
rust: core/src/tabs.rs
types: Tabs, TabsEvent
---
The user taps a tab, its pill lights and the screen behind it fades in while the other one fades out. The app gets `Selected(i)`, draws every screen from the chosen tab, and can open a screen itself.

## When to use it
- Two to four screens of the same app that the user moves between: Overview, Rooms, Automations.
- A value inside a widget (a fan's Low, Med, High): a `#segmented` button. Tabs change what the whole screen shows, a segmented button changes one setting.
- One screen over another for a moment (a device's details, a password): a `#sheet` (docs/content/sheet.md).
- Pages in one place that the user goes deeper into and back from, without a tab bar (a sheet's settings and the list a row opens): `Pages` (docs/content/pages.md). It uses the same effects as the screens behind tabs.

## Quick start
**What.** The tab bar of the panel, Overview, Rooms and Automations. A tap on Rooms fades the dashboard out and the rooms in.

**Why.** The tabs only know which tab is chosen. Each screen fades in or out from that, so another animation is one line: pick another effect.

<!-- example: docs/examples/tabs.rs use slot=#home/tabs field=tabs -->
```rust
use everyware_ui::tabs::{Tabs, TabsEvent};
use everyware_ui::screens::Effect;
use everyware_ui::part_of;
use everyware_ui::ScreenView;

/// The screens behind the tabs, in the tabs' order: the label and the screen's node.
pub const SCREENS: [(&str, &str); 3] = [("Overview", "#overview"), ("Rooms", "#rooms"), ("Automations", "#automations")];
/// How the screens change: Effect::Fade (the default), FadeThrough, Slide or None.
pub const EFFECT: Effect = Effect::Fade;
/// The distance between two screens of a slide: the screen's width.
pub const SCREEN_WIDTH: f32 = 1920.0;
```

<!-- example: docs/examples/tabs.rs state slot=#home/tabs field=tabs -->
```rust
pub tabs: Tabs,
/// Where the design puts each screen in its parent (its x): a slide starts from there.
pub screen_x: [f32; 3],
```

<!-- example: docs/examples/tabs.rs init slot=#home/tabs field=tabs -->
```rust
tabs: Tabs::new(SCREENS.map(|(label, _)| label)),
screen_x: SCREENS.map(|(_, node)| sizes.frame(node).map_or(0.0, |f| f.x)),
```

<!-- example: docs/examples/tabs.rs model slot=#home/tabs field=tabs -->
```rust
tabs: Slot = "#home/tabs" ["SlotCustomizer"];
// one field per screen behind the tabs, the first one shown at the start
overview: ScreenView = "#overview" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(100.0, None), x: None };
rooms: ScreenView = "#rooms" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(0.0, None), x: None };
automations: ScreenView = "#automations" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(0.0, None), x: None };
// a screen drawn as a set with state=hidden/shown: shown once, the effect does the rest
rooms_variant: Variant = "#rooms" = Variant("state=shown".to_string());
automations_variant: Variant = "#automations" = Variant("state=shown".to_string());
```

<!-- example: docs/examples/tabs.rs present slot=#home/tabs field=tabs -->
```rust
model.tabs = slot("#home/tabs", &state.tabs);
// each screen: its opacity, and its x for Effect::Slide, from the effect
let screen = |i: usize| ScreenView::new(state.tabs.screen(i, EFFECT, SCREEN_WIDTH), state.screen_x[i]);
model.overview = screen(0);
model.rooms = screen(1);
model.automations = screen(2);
```

<!-- example: docs/examples/tabs.rs act slot=#home/tabs field=tabs -->
```rust
// a tab reports with its tab bar: "#home/tabs/#tabs/tab-2"
_ if part_of("#home/tabs", node).is_some() => {
    let tab = part_of("#home/tabs", node).and_then(|p| p.strip_prefix("#tabs/tab-")).and_then(|n| n.parse::<usize>().ok());
    if let Some(TabsEvent::Selected(i)) = tab.and_then(|n| n.checked_sub(1)).and_then(|i| state.tabs.on(i, gesture)) {
        // the presenter already shows screen i; tell anyone else who needs it here
        log::info!("screen {}", SCREENS[i].0);
    }
}
```

## How it behaves
- **Tap.** A tap on another tab lights its pill and its label in the accent; the screen switches in `tabs`/`switch` of components.json (300 ms). A tap on the chosen tab does nothing.
- **Effects.** `Effect::Fade` (the default) crossfades the two screens. `Effect::FadeThrough` lets the old screen go in the first 30 % and brings the new one after it. `Effect::Slide` lays the screens side by side in the tabs' order: the new screen comes one width from its side, the old one leaves one width to the other side, whatever tabs lie between. `Effect::None` switches at once. To pick another one, change `EFFECT`: the tabs and the screens stay as they are.
- **Hidden screens.** A screen at opacity 0 is not drawn and takes no taps.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled`; no tab takes a tap.

## From the app
**What.** The app opens a screen itself, without a tap: a notification about a routine opens Automations.

**Why.** `select(i)` chooses the tab as a tap would, and keeps the tab before it, so the screens switch with the same effect. An index out of range or the chosen tab changes nothing.

<!-- example: docs/examples/tabs.rs change slot=#home/tabs field=tabs -->
```rust
/// The app opens a screen itself (a notification about a routine opens Automations): the tab is
/// chosen and the screens switch with the same effect as after a tap.
pub fn open_screen(state: &mut State, node: &str) {
    if let Some(i) = SCREENS.iter().position(|(_, n)| *n == node) {
        state.tabs.select(i);
    }
}
```

## In Figma
One slot in the header or the bottom bar, named `#<screen>/tabs` (`#home/tabs`), with an instance of `#tabs` with the right number of tabs. Each screen is one frame named after it (`#overview`, `#rooms`, `#automations`), all at the same place. For designers every tab can be a frame of the prototype (figma-design skill); on the device the app draws the screen.

## How it works
`tabs` is the number (2, 3, 4), `state` `default` or `disabled`. Each tab is `#tabs/tab-<i>` (a tap target) with `#tabs/tab-<i>/selected` (the pill), `#tabs/tab-<i>/label` and `#tabs/tab-<i>/label-selected`; the choice is opacities, the variant does not change. `on(index, gesture)` answers `Selected(index)` and keeps `previous` for a slide's direction. The screens (core/src/screens.rs): `tabs.screen(i, effect, width)` gives screen `i` a `ScreenTarget`, its opacity and its x offset, each with a transition; `ScreenView::new(target, design_x)` adds the offset to the screen's place in the design. A new animation is one more `Effect` with its test.

## Traps
- A tap comes with its tab bar (`#home/tabs/#tabs/tab-2`): match the slot and the part with `part_of`, so two tab bars on one screen stay apart.
- A screen drawn as a set with a variant of its own (`state=hidden`, `state=shown`) is set to shown once, then the effect shows or hides it: a variant per switch rebuilds the tree.
- `Effect::Slide` moves a screen by its x in the parent: give `design_x` from the design (`Sizes::frame`), or the screen jumps to x 0.
- The first screen's `ScreenView` starts at opacity 100 in the UI model, the others at 0: or every screen shows until the first action.

## Example
[docs/examples/tabs.rs](../examples/tabs.rs): the whole path, from a tap on a tab to the three screens it switches and a screen the app opens itself, compiled and run by `everyware_ui_recipes_test`.
