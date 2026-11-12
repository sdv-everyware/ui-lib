---
title: Button
role: A button with a text that the user taps to start one action: connect, cancel, talk.
rust: core/src/button.rs
types: ButtonKind, ButtonSize, Button, ButtonEvent
---
The user taps a button with a word on it (Connect, Cancel, Talk). It darkens under the finger and does nothing if the finger slides off. The app gets one `Clicked` per tap and decides what it means, usually a command to a service. The app can relabel the button and turn it off while it cannot act.

## When to use it
- One action with a word for it: the action of a sheet (Connect, Cancel), a widget's own action (Talk). For an action that has a well-known glyph (plus, minus, close) use `#icon-button`.
- Not for a state that stays on or off (use `#toggle`) and not for one choice of a few (use `#segmented` or `#radio`).

## Quick start
**What.** The Talk button of the panel: the user taps it, the app gets one click and sends a command to the intercom.

**Why.** The button only knows that it was tapped. What the tap means is the app's decision, so the app turns the click into a command.

<!-- example: docs/examples/button.rs use slot=#intercom/talk field=talk -->
```rust
use everyware_ui::button::{Button, ButtonEvent, ButtonKind};
```

<!-- example: docs/examples/button.rs state slot=#intercom/talk field=talk -->
```rust
pub talk: Button,
```

<!-- example: docs/examples/button.rs init slot=#intercom/talk field=talk -->
```rust
talk: Button::new(ButtonKind::Primary, "Talk").compact(),
```

<!-- example: docs/examples/button.rs model slot=#intercom/talk field=talk -->
```rust
talk: Slot = "#intercom/talk" ["SlotCustomizer"];
```

<!-- example: docs/examples/button.rs present slot=#intercom/talk field=talk -->
```rust
model.talk = slot("#intercom/talk", &state.talk);
```

<!-- example: docs/examples/button.rs act slot=#intercom/talk field=talk -->
```rust
"#intercom/talk" => {
    if let Some(ButtonEvent::Clicked) = state.talk.on(gesture) {
        // the tap is done: send it on to the service that acts on it
        if !send(Command::Talk) {
            log::warn!("talk: the service is not there, nothing was sent");
        }
    }
}
```

<!-- example: docs/examples/button.rs send slot=#intercom/talk field=talk -->
```rust
/// What the screen asks a service to do. An app has its own link to the service (smarthome-app:
/// screen_binder::send_matter_command, a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Talk,
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## How it behaves
- **Pressed.** While the finger is down the button darkens: a layer fades in over 80 ms and out over 160 ms. The variant stays.
- **Tap.** A down and an up on the button is one `Clicked`. A finger that slides off undoes the press; coming back does not restore it, and the up is no tap.
- **Disabled.** `press.set_enabled(false)` shows `state=disabled` (paler, no press) and the button takes no taps. It switches without an animation.
- **Label.** The label is the app's and is centred; a longer label does not make the button wider.

## From the app
**What.** The app changes the button from what it knows, without a tap: another label (Talk becomes End during a call), or off while the service is not there.

**Why.** The button shows what the app can do now. A tap on a button that cannot work would do nothing, so the app turns it off. Its state is plain fields: `label`, `kind`, `size`, and `press` for on and off.

<!-- example: docs/examples/button.rs change slot=#intercom/talk field=talk -->
```rust
/// The app changes the button from its own state, outside a tap: a new label, or off while it
/// cannot act (no service). The presenter shows it at the next frame: the label as text, off as
/// the variant state=disabled, which takes no taps.
pub fn on_call(state: &mut State, in_call: bool, service_up: bool) {
    state.talk.label = if in_call { "End".into() } else { "Talk".into() };
    state.talk.press.set_enabled(service_up);
}
```

## In Figma
A slot of the variant's size named `#<screen>/<name>` (`#intercom/talk`, `#wifi/connect`): 200 × 56 for `size=regular` (a sheet's actions), 116 × 56 for `size=compact` (a widget's row). Inside it an instance of `#button`. The instance's label is the designer's preview; the app writes the real one. Two buttons are two slots, never one instance twice.

## How it works
`kind` is the look (`primary` filled, `secondary` soft, `ghost` text only), `size` the width, `state` only `default` or `disabled`. Pressed is not a variant: `#button/pressed` is a layer whose opacity the binding fades (display-only in harry, no tree rebuild). The label is `#button/label`. `Button::new(kind, label)` makes a regular one, `.compact()` the narrow one.

## Traps
- A label changed at runtime needs the text node named `#button/label` in every variant: a renamed inner layer is lost on a variant swap, a text override is kept.
- Two buttons on one screen must sit in two slots: harry finds parts by node name, and the slot gives each clone its own scope.

## Example
[docs/examples/button.rs](../examples/button.rs): the whole path, from a tap on the button to the command to the service and a new label from the app, compiled and run by `everyware_ui_recipes_test`.
