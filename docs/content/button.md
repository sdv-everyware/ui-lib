---
title: Button
role: A tappable text button, Material 3's filled, tonal and text button, in two widths.
rust: core/src/button.rs
types: ButtonKind, ButtonSize, Button, ButtonEvent
---
A button is the design's fixed size per variant (Google's components are drawn that way, the label centred in the box), not the label's width. `kind` is the look, `size` the width (200 regular for a sheet's actions, 116 compact in a widget's row), `state` only `default` or `disabled`: pressed is a layer the code fades, not a variant.

## In Figma
A button on a screen is a slot of the design's size: a frame 200 × 56 (`size=regular`) or 116 × 56 (`size=compact`) named `#<screen>/<name>` (for example `#wifi/connect`), with an instance of `#button` inside. The instance's label is the designer's preview; the app writes the real label into `#button/label`. Place two buttons as two slots, never one instance twice.

## In the app
```rust
// state: the component
pub connect: Button,                                    // Button::new(ButtonKind::Primary, "Connect")
pub talk: Button,                                       // Button::new(ButtonKind::Primary, "Talk").compact()

// ui_model: one Slot per place on the screen
wifi_connect: Slot = "#wifi/connect" ["SlotCustomizer"];

// presenter: the slot gets the component (variant, label, pressed fade)
model.wifi_connect = slot("#wifi/connect", &wifi.connect);

// actions: the gesture harry reported for the node goes to the button; it answers Clicked
"#intercom/talk" => {
    if d.talk.on(gesture) == Some(ButtonEvent::Clicked) {
        log::info!("smarthome: Talk");
    }
}
```

Disable with `button.press.enabled = false` (the variant becomes `state=disabled`, never animated).

## Traps
- A label changed at runtime needs the text node named `#button/label` in every variant; a renamed inner layer is lost on a variant swap, a text override is kept.
- Two buttons on one screen must sit in two slots: harry finds parts by node name, and the slot gives each clone its own scope.
