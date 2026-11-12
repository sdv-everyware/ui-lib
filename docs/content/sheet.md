---
title: Sheet
role: A card the user opens over the screen, a dialog in the middle or a panel at an edge, with the app's content in it.
rust: core/src/sheet.rs
types: Edge, Layout, Sheet
---
The user taps something and a card slides in over the screen, the screen behind it darkens; a tap on the dark scrim or on the card's close button slides it away. The app gets the card's place for every layout (`dialog` in the middle, `left`, `right`, `top`, `bottom` panels) and the scrim's opacity, and fills the card with its own content component.

## When to use it
- A dialog or a side panel with the app's own content: a device's details, a settings panel, a web page beside the screen. Material 3's dialog and side sheet.
- With input fields in it, use a `Form` (docs/content/form.md): a sheet that also does the focus and the keyboard.
- For something that slides in and is not a card, the `Overlay` under it (docs/content/overlay.md). A choice from a short list is a `#dropdown`, not a sheet.

## Quick start
**What.** The device dialog: it rises from the bottom to the middle of the screen with the device's details, and closes with its X button or a tap outside.

**Why.** Every dialog and side panel of the app looks and moves the same way: the app puts its own content into the sheet and says open or close.

<!-- example: docs/examples/sheet.rs use slot=#device field=device -->
```rust
use everyware_ui::button::{ButtonEvent, Icon, IconButton, IconButtonKind};
use everyware_ui::dock::KEYBOARD_DOCK_H;
use everyware_ui::sheet::{Layout, Sheet};
```

<!-- example: docs/examples/sheet.rs state slot=#device field=device -->
```rust
/// The sheet: closed or open, its layout, the height of its content.
pub device: Sheet,
/// The content's own controls and texts.
pub close: IconButton,
pub title: String,
```

<!-- example: docs/examples/sheet.rs init slot=#device field=device -->
```rust
// the card hugs the content: its height comes from the design (the .dcf)
device: Sheet::new(DEVICE_LAYOUT).with_content_height(sizes.height("#device-content", "state=default")),
close: IconButton::new(IconButtonKind::Tonal, Icon::Close),
title: String::new(),
```

<!-- example: docs/examples/sheet.rs model slot=#device field=device -->
```rust
// the card moves on both axes, the scrim fades and takes taps while open
device_card: Position = "#device/card" ["FollowPositionCustomizer"] = Position::new(DEVICE_LAYOUT.closed(SCREEN), None);
device_scrim: Meter = "#device/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
device_scrim_taps: TapTarget = "#device/scrim" ["TapTargetCustomizer"];
// a tap on the card between its parts stays on the card
_device_card_tap: () = "#device/card" interactive;
// the content: the app's own slots and texts in the card
device_close: Slot = "#device/close" ["SlotCustomizer"];
device_title: String = "#device/title";
```

<!-- example: docs/examples/sheet.rs present slot=#device field=device -->
```rust
// a dialog is centred in the screen above the keyboard while the keyboard is open
let free_height = if state.keyboard_open { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 };
let slide = state.device.transition();
model.device_card = Position::new(state.device.card_position(SCREEN, free_height), slide);
model.device_scrim = Meter::new(state.device.scrim(), slide);
model.device_scrim_taps = TapTarget(state.device.is_open());
model.device_close = slot("#device/close", &state.close);
model.device_title = state.title.clone();
```

<!-- example: docs/examples/sheet.rs act slot=#device field=device -->
```rust
// the close button and a tap on the scrim close the sheet
"#device/close" => {
    if let Some(ButtonEvent::Clicked) = state.close.on(gesture) {
        state.device.close();
    }
}
"#device/scrim" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.device.close(),
```

## How it behaves
- **Opening.** The card slides in from 40 px behind its edge with the open spring (550 ms): a dialog from below the screen, a panel from its side. The scrim darkens to 35 % with it.
- **Closing.** The close button, a tap on the scrim or the app: the card slides back with the close spring (400 ms), the scrim clears.
- **The scrim.** It takes taps only while the sheet is open; closed, taps go through to the screen. A tap on the card between its controls stays on the card.
- **The layouts.** A dialog is 1120 wide, as high as its content plus 40 around it, in the middle. A side panel is half the width and the full height; a top or bottom panel the full width and half the height.
- **The keyboard.** While the keyboard dock is open a dialog is centred in the screen above it, or at the top when it is higher than that. A panel keeps its edge.

## From the app
**What.** The app opens the sheet with what it shows (a tile tapped, a device that needs the user) and closes it when there is nothing to show any more.

**Why.** The sheet is the state; the presenter draws the card and the scrim from it at the next frame. The content's texts are the app's, written before it opens.

<!-- example: docs/examples/sheet.rs change slot=#device field=device -->
```rust
/// The app opens the sheet with what it shows (a tap on a tile, a device that needs the user)
/// and closes it when there is nothing to show any more (the device is gone).
pub fn show_device(state: &mut State, name: Option<&str>) {
    match name {
        Some(name) => {
            state.title = name.to_string();
            state.device.open();
        }
        None => state.device.close(),
    }
}
```

## In Figma
Three things make a sheet in the design:

1. **The content**: a component of the app's design (`#wifi-content`, `#device-content`), set to auto layout that **hugs** one inner frame of the content's size, with the app's layers inside that frame (`#device/title`, `#device/close`, the lists). A step or mode of the sheet is a variant property of this component (`step=networks|password`): the app sets that variant on the content, never on the sheet.
2. **The sheet on the screen**: an instance of the library's `#sheet` (`layout=dialog`, or a panel edge; `state=closed`) in a plain absolute frame over the whole screen. Select its nested `#sheet/content` and **swap the instance** to the content component. Rename the three parts in the instance for this sheet: `#sheet/scrim` to `#<x>/scrim`, `#sheet/card` to `#<x>/card`, `#sheet/content` to `#<x>/content`, so the app drives each sheet by its own names.
3. **Nothing hidden and nothing resized by hand**: the card hugs the content (a dialog's height follows it), the closed variant keeps the card behind the screen's edge; the app slides it in.

The layers in the root: the screen first, the sheets after it, the keyboard dock last. The sheets of the smarthome panel (`#wifi`, `#device`, `#panel`) are built exactly like this.

## How it works
`Sheet` is an `Overlay` (docs/content/overlay.md), a `Layout` and the content's height. `card_position(screen, free_height)` is the card's top left: in place while open (a dialog centred in `free_height`), behind its edge while closed (`Layout::closed`). `scrim()` is 35 open, 0 closed. The app gives the card a `Position` with `FollowPositionCustomizer` and the scrim a `Meter` with `FollowOpacityCustomizer`, both with `transition()`: display-only moves in harry, no new tree, no layout. `variant()` names the layout and the state (`layout=dialog, state=open`) for tests and the contract check; `Layout` in the code must match the instance's `layout` in the design.

## Traps
- Never set a `Variant` on the sheet instance: harry takes the Library's variant, which has no content and the names `#sheet/*`, and every sheet shows open and empty.
- A size override on the nested content instance is not honoured by Figma's import; the content component hugs a box instead (`hugBox` in the generator).
- An instance placed absolute by itself lands in the root's flow after import: the sheet is a strip at the bottom. Put the instance in a plain absolute frame.
- Without the content's height (no design loaded, `Sizes::none()`) a dialog has the default 840 px card and sits off centre for a smaller content.
- Use the free height while the keyboard is open (`SCREEN.1 - KEYBOARD_DOCK_H`), or the dialog sits under the keys.

## Example
[docs/examples/sheet.rs](../examples/sheet.rs): the whole path, from opening the device sheet to the scrim and the close button that close it and the dialog above the keyboard, compiled and run by `everyware_ui_recipes_test`.
