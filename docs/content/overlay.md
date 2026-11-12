---
title: Overlay (code)
role: Anything that comes in over the screen and goes away again with the same spring: the state under every sheet and the keyboard.
rust: core/src/overlay.rs
types: Overlay
---
The user sees something slide in over the screen (a card, a panel, the keys) and go away again: a tap on the scrim, a close button, or the app closes it. The app gets two values from the overlay: whether it is open and the transition of the last change, and with `slide(open, closed)` the value of each moving part.

## When to use it
- Mostly not directly: a card over the screen is a `Sheet` (docs/content/sheet.md), a sheet with fields a `Form` (docs/content/form.md), the keyboard the `KeyboardDock` (docs/content/keyboard_dock.md). Each is an overlay with the places worked out.
- Directly for a part of your own that slides in and out with the library's spring and is not one of those: a banner, a panel you place by hand. This page moves the right panel (`#panel`) by hand, to show what a `Sheet` does for you.

## Quick start
**What.** A side panel that slides in from the right over a darkened screen, and slides away when the user taps the dark part.

**Why.** Opening and closing is the same for every panel and dialog, so it lives in one place and the app only says open or close.

<!-- example: docs/examples/overlay.rs use slot=#panel field=panel -->
```rust
use everyware_ui::overlay::Overlay;
```

<!-- example: docs/examples/overlay.rs state slot=#panel field=panel -->
```rust
/// The panel over the screen: closed or open, and the transition of the last change.
pub panel: Overlay,
```

<!-- example: docs/examples/overlay.rs init slot=#panel field=panel -->
```rust
panel: Overlay::new(),
```

<!-- example: docs/examples/overlay.rs model slot=#panel field=panel -->
```rust
// the card moves, the scrim fades and takes taps while open
card: Position = "#panel/card" ["FollowPositionCustomizer"] = Position::new((CLOSED_X, 0.0), None);
scrim: Meter = "#panel/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
scrim_taps: TapTarget = "#panel/scrim" ["TapTargetCustomizer"];
// a tap on the card stays on the card
_card_tap: () = "#panel/card" interactive;
```

<!-- example: docs/examples/overlay.rs present slot=#panel field=panel -->
```rust
// both parts move with the overlay's transition: 550 ms in, 400 ms out, a spring
let slide = state.panel.transition();
model.card = Position::new((state.panel.slide(OPEN_X, CLOSED_X), 0.0), slide);
model.scrim = Meter::new(state.panel.slide(35.0, 0.0), slide);
model.scrim_taps = TapTarget(state.panel.open);
```

<!-- example: docs/examples/overlay.rs act slot=#panel field=panel -->
```rust
// a tap on the scrim closes it; the finger has to lift on the scrim
"#panel/scrim" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.panel.close(),
```

## How it behaves
- **Opening.** `open()`: the parts move to their open values with the open spring (550 ms). Here the card comes in from the right edge and the scrim darkens to 35 %.
- **Closing.** `close()`: back to the closed values with the close spring (400 ms).
- **The scrim.** It takes taps only while open (`TapTarget`); a tap that lifts on it closes. A finger that slides off before it lifts does nothing.
- **Twice.** Opening an open overlay (or closing a closed one) changes nothing, not even the transition.

## From the app
**What.** The app opens and closes the overlay from what happened, outside a tap: a page to show, the page gone.

**Why.** The overlay is the state; the presenter draws the parts from it at the next frame, so the app only says open or closed.

<!-- example: docs/examples/overlay.rs change slot=#panel field=panel -->
```rust
/// The app opens and closes it from what happened: a story to read, the page is gone. Open
/// twice is open once: the transition stays the one of the first change.
pub fn show_page(state: &mut State, page: Option<&str>) {
    match page {
        Some(_) => state.panel.open(),
        None => state.panel.close(),
    }
}
```

## In Figma
Nothing of its own: the parts it moves are the app's (here `#panel/card` and `#panel/scrim` of a `#sheet` instance, see docs/content/sheet.md). They are drawn in their closed place: the card behind the screen's edge, the scrim clear.

## How it works
`Overlay` is `open: bool` and the last change (`Change::Open` or `Change::Close`). `transition()` looks that change up in components.json (`overlay`: open 550 ms, close 400 ms, spring); it is None before the first change. `slide(open, closed)` returns one of the two values. `variant()` is `state=open` or `state=closed`, for tests and the contract check.

## Traps
- The app moves the parts from the overlay's state (display-only); it never sets the variant of an instance in the design. A variant switch rebuilds the tree and lays the screen out on every frame of the animation.
- Give every moving part its own field with the transition (`Position`, `Meter`): harry does not hand a parent's animation to its children.
- The scrim takes taps only while open: closed, it lies transparent over the whole screen and would swallow every tap.

## Example
[docs/examples/overlay.rs](../examples/overlay.rs): the whole path, from opening the panel to the scrim tap that closes it, compiled and run by `everyware_ui_recipes_test`.
