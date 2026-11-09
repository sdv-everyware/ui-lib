---
title: Sheet
role: The container for anything over the screen, Material 3's dialog and side sheet, with the app's content component in its slot.
rust: core/src/sheet.rs
types: Edge, Layout, Sheet
---
`layout` is where the card is: `dialog` (1120 wide in the middle, in from the bottom, as high as its content) or a panel along an edge (`left`, `right`: half the width and the full height; `top`, `bottom`: the full width and half the height). `state` is `closed` (the card 40 px behind its edge, the scrim clear) or `open`. The parts: `#sheet/scrim`, `#sheet/card`, `#sheet/content`, a slot that holds the content component.

The app **never sets a sheet's variant**: harry would take the variant from the Library and the instance's own part names and content would be lost. The instance stays the closed variant of the design; the app slides the card from behind its edge and fades the scrim, both display-only.

## In Figma
Three things make a sheet in the design:

1. **The content**: a component of the app's design (`#wifi-content`, `#device-content`, …), set to auto layout that **hugs** one inner frame of the content's size, with the app's layers inside that frame (`#wifi/networks`, `#wifi/password`, the buttons' slots). A step or mode of the sheet is a variant property of this component (`step=networks|password`).
2. **The sheet on the screen**: an instance of the library's `#sheet` (`layout=dialog`, or a panel edge; `state=closed`) in a plain frame over the whole screen. Select its nested `#sheet/content` and **swap the instance** to the content component. Rename the three parts in the instance for this sheet: `#sheet/scrim` → `#<x>/scrim`, `#sheet/card` → `#<x>/card`, `#sheet/content` → `#<x>/content`, so the app can drive each sheet by its own names.
3. **Nothing hidden and nothing resized by hand**: the card hugs the content (a dialog's height follows it), the closed variant keeps the card behind the screen's edge; the app slides it in.

The sheets the app has now (#wifi, #device, #panel) are built exactly like this; the designer frames below the dashboard show each one open.

## In the app
```rust
// state: the sheet (or a Form: a sheet with input fields) and its layout
pub const WIFI_LAYOUT: Layout = Layout::Dialog;
pub const PANEL_LAYOUT: Layout = Layout::Panel(Edge::Right);
pub panel: Sheet,                                        // Sheet::new(PANEL_LAYOUT)
form: Form::new(WIFI_LAYOUT, [("#wifi/password", password)]),

// ui_model: the card moves on both axes, the scrim fades and takes taps while open
panel_card: Position = "#panel/card" ["FollowPositionCustomizer"] = Position::new(PANEL_LAYOUT.closed(SCREEN), None);
panel_scrim: Meter = "#panel/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
panel_scrim_taps: TapTarget = "#panel/scrim" ["TapTargetCustomizer"];
_panel_card_tap: () = "#panel/card" interactive;         // a tap on the card stays on the card
wifi_step: Variant = "#wifi/content" = Variant("step=networks".to_string());   // a step is the content's variant

// presenter: the content's height from the design (Sizes, read from the .dcf at the start)
// centres a dialog; while the keyboard dock is open, in the screen above it
let free_height = if state.osk.open() { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 };
let sheet = wifi.form.sheet.with_content_height(state.sizes.height("#wifi-content", &wifi.step_variant()));
let slide = wifi.form.transition();
model.wifi_card = Position::new(sheet.card_position(SCREEN, free_height), slide);
model.wifi_scrim = Meter::new(sheet.scrim(), slide);
model.wifi_scrim_taps = TapTarget(wifi.form.is_open());

// actions: open from a tap, close from the scrim or a button
state.panel.open();
"#wifi/scrim" if matches!(gesture, Gesture::Up { inside: true, .. }) => wifi.form.close(),
```

## Traps
- Setting `Variant` on the sheet instance shows every sheet open and empty: the Library's variant has no content and the names `#sheet/*`.
- A size override on the nested content instance is not honoured by Figma's import; the content component hugs a box instead (`hugBox`).
- A dialog higher than the screen above the keyboard sits at the top; a panel never moves for the keyboard.
