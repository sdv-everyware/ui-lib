# #sheet

The container for anything over the screen, Material 3's dialog and side sheet, with the app's content component in its slot.

## Figma

- set: `#sheet` (node 2-2963 in the Library)
- property `layout`: `dialog` | `left` | `right` | `top` | `bottom`
- property `state`: `closed` | `open`
- variant string: `layout=dialog, state=closed` (the properties in this order; `Sheet::variant()` produces it, never typed by hand)
- parts the code sets: the library's container for anything over the screen: #sheet/scrim (opacity 0 to 35 %) and #sheet/card, a dialog 1120 wide in the middle, as high as its content plus 40 around it (the card hugs the content; the app reads the content's height from the .dcf, design.rs), or a panel along an edge, half the screen; #sheet/content is an empty slot for the app's content. The app gives the card its position (FollowPositionCustomizer, ui-lib sheet.rs: the closed card waits 40 px behind its edge, a dialog is centred by its content's height) and the scrim its opacity, so open and close are display-only; the app never sets the instance's variant (the Library's variant has no content)
- to place one: a frame named `#<screen>/<name>` with an instance of `#sheet` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- open: 550 ms spring
- close: 400 ms spring

## Rust

- module: `core/src/sheet.rs`, type `Sheet`
- `pub fn new(layout: Layout) -> Self`
- pub enum `Edge`: The screen's edge a panel sits at and slides in from.
  - `Left`: 
  - `Right`: 
  - `Top`: 
  - `Bottom`: 
- pub enum `Layout`: How a sheet shows its card.
  - `Dialog`: A card in the middle of the screen, in from the bottom.
  - `Panel`: A panel along `Edge`, half the screen, in from behind that edge.
  - `pub fn name(self) -> &'static str`: The value of the Figma property `layout`.
  - `pub fn card(self, screen: (f32, f32), content_height: Option<f32>) -> Rect`: The card open, on a screen of `(w, h)`, with the content `content_height` high (a dialog's card hugs it; None: the default height).
  - `pub fn closed(self, screen: (f32, f32)) -> (f32, f32)`: Where the card waits closed: behind the edge it comes from, `CLOSED_GAP` past the screen.
- pub struct `Sheet`: A sheet: an overlay with the library's card, in one of the layouts.
  - field `pub overlay: Overlay`: 
  - field `pub layout: Layout`: 
  - field `pub content_height: Option<f32>`: The height of the content in the card (design.rs Sizes): the card hugs it, so the middle of the screen depends on it. None until known: the default dialog height.
  - `pub fn with_content_height(mut self, height: Option<f32>) -> Self`: The same sheet with this content height (the card's height follows the content).
  - `pub fn open(&mut self)`: 
  - `pub fn close(&mut self)`: 
  - `pub fn is_open(&self) -> bool`: 
  - `pub fn card_position(&self, screen: (f32, f32), free_height: f32) -> (f32, f32)`: Where the card is now: in place while open, behind its edge while closed; the app gives both coordinates to the card as a position override (FollowPositionCustomizer). `free_height`: the screen's height without what covers its bottom (the keyboard dock while it is open): a dialog is centred in that, or at the top when it is higher than that, as a phone moves a dialog above its keyboard. A panel keeps its edge.
  - `pub fn scrim(&self) -> f32`: The scrim's opacity, 0..100.
  - `pub fn transition(&self) -> Option<Transition>`: 

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

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
