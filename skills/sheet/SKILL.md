---
name: sheet
description: Add a sheet, dialog or side panel over a screen of a harry app with the EveryWare UI library's #sheet - the content component in Figma swapped into the sheet's slot, the renamed parts, the Sheet or Form state in the app, the card that slides and the scrim that fades, the dialog that moves above the keyboard, the designer's visible frame. Use when the user asks for a new dialog, sheet, drawer or side panel, for a step inside one (networks -> password), or when a sheet opens empty, does not slide, or sits under the keyboard. For a single control inside it use use-component.
---

# A sheet, dialog or side panel on `#sheet`

Read first: `docs/content/sheet.md` (layouts, the API) and `docs/examples/sheet.rs` (the code). The library owns the container (scrim, card,
slide, the layouts); the app owns the content and when it opens.

```
Library #sheet (layout=dialog|left|right|top|bottom, state=closed|open)
  #sheet/scrim   opacity 0 ─► 35 %           ┐ display-only, moved by the app
  #sheet/card    behind the edge ─► in place  ┘ from Sheet::card_position / scrim()
  #sheet/content a slot: the app's content component swapped in (#wifi-content, #device-content)
```

## 1. Figma: the content component and the sheet on the screen

1. **The content** is a component of the app's design (`#<x>-content`), auto layout that hugs
   one inner frame of the content's size, with the app's layers and slots inside it
   (`#wifi/networks`, `#wifi/password`, the buttons' slots: use-component skill). A step or
   mode of the sheet is a variant property of this component (`step=networks|password`): the
   app sets this variant on the content, never on the sheet. A page deeper that slides in and
   Back that slides it out (the settings and the list one of their rows opens) is `Pages`
   instead (ui-lib docs/content/pages.md): the pages are frames of the content side by side,
   the app moves them by opacity and x, so the switch is display only; a variant change builds
   harry's tree again.
2. **The sheet on the screen**: an instance of the Library's `#sheet` (`layout=dialog` or an
   edge, `state=closed`) inside a plain absolute frame over the whole screen (an absolute
   instance by itself lands in the root's flow after import). Select the nested
   `#sheet/content` and swap the instance to the content component. Rename the three parts
   for this sheet: `#sheet/scrim` → `#<x>/scrim`, `#sheet/card` → `#<x>/card`,
   `#sheet/content` → `#<x>/content`.
3. Layers in the root: the screen first, the sheets after it, the keyboard dock last.
4. **For people**: outside the root, a frame that shows the sheet open (`cardOverlay()` in
   `prototype-patch.js`) with the Open overlay interaction from the control that opens it
   (figma-design skill). harry never reads the prototype; the device is right when they differ.

In the generated Smart HOme file this is `sheetInstance()` and the content sets in
smarthome-app's `design/*-patch.js` (an app's own `design/`); build and import a new version, then fetch.

## 2. The app

```rust
// state.rs: the layout is a constant, the sheet (or a Form for input fields) lives in the state
pub const SETTINGS_LAYOUT: Layout = Layout::Panel(Edge::Right);   // or Layout::Dialog
pub settings: Sheet,                                              // Sheet::new(SETTINGS_LAYOUT)

// ui_model.rs: the card moves, the scrim fades and takes taps, a tap on the card stays there
settings_card: Position = "#settings/card" ["FollowPositionCustomizer"] = Position::new(SETTINGS_LAYOUT.closed(SCREEN), None);
settings_scrim: Meter = "#settings/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
settings_scrim_taps: TapTarget = "#settings/scrim" ["TapTargetCustomizer"];
_settings_card_tap: () = "#settings/card" interactive;

// presenter.rs: a dialog is as high as its content and centred in the free height
// (the screen, or the screen above the open keyboard dock); a panel keeps its edge
let free_height = if state.osk.open() { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 };
let sheet = state.settings.with_content_height(state.sizes.height("#settings-content", "state=default"));
let slide = state.settings.transition();
model.settings_card = Position::new(sheet.card_position(SCREEN, free_height), slide);
model.settings_scrim = Meter::new(sheet.scrim(), slide);
model.settings_scrim_taps = TapTarget(state.settings.is_open());

// actions.rs: open from a tap, close from the scrim or a button; while open, route the
// gestures to the sheet's controls first
"#home/settings" => if clicked(&mut state.dash.settings, gesture) { state.settings.open() },
"#settings/scrim" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.settings.close(),
```

`Sizes::load(dcf)` (everyware_ui::design) reads the content's height per variant from the
.dcf at start, so a dialog hugs its content without a framework change. A sheet with input
fields is `Form::new(layout, [("#x/field", field)])`: it routes taps and the keyboard's
outcomes to the fields and the dock follows the focus (`state.osk.follow(...)`).

## 3. Check

- `everyware_ui_test` (the parts of `#sheet` exist in the .dcf), `m smarthome_app`, run
  (the harry-app skill): the card slides in by the `open` transition, the scrim fades, a tap on
  the scrim closes, a dialog moves up when the keyboard opens.
- `scripts/check.sh` before the commit.

## When it does not work

| Symptom | Cause | Fix |
|---|---|---|
| every sheet shows open and empty at start | the app set `Variant` on the `#sheet` instance: harry took the Library's variant (no content, the names `#sheet/*`) | never set the sheet's variant; move the card and fade the scrim |
| the card jumps instead of sliding | a parent's animation is not inherited by harry | one `Position`/`Meter` field per moving part with the transition |
| the sheet is a strip at the bottom, taps after "open" are swallowed | the instance is absolute by itself; the import dropped it into the flow | the instance inside a plain absolute frame |
| the dialog is too high or has empty space | the content component does not hug its inner frame, or a size override on the nested instance (ignored by import) | `hugBox` content; the dialog's height follows the content |
| the dialog sits under the keyboard | the presenter uses the full screen height | `free_height` from `state.osk.open()` |

A change lands through its own branch, a review by another agent and the user's word:
`vendor/sdv-everyware/everyware/AGENTS.md`, "How a change lands" (`scripts/feature.sh` there).
