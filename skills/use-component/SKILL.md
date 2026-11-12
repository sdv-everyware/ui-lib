---
name: use-component
description: Put a component of the EveryWare UI library (button, icon button, toggle, slider, segmented, dropdown, list, input field, symbol ...) on a screen of a harry app, end to end - the slot in Figma, the fetch, the four lines in the app (state, ui_model, presenter, actions), the contract test, the check before the commit. Use when the user asks to add a control or a list to a screen or a sheet, to bind a Figma layer to a library component, or when a placed component does not show, does not react or shows the wrong variant. For a new or changed component use ui-component; for a new sheet or dialog use sheet.
---

# Put a library component on a screen

Read first: `docs/content/<component>.md` and `docs/examples/<component>.rs` (the guide and the code; variants in `components.json`, the API in the Rust file the guide names; events,
the four lines with a real slot name). Paths below are relative to
`vendor/sdv-everyware/ui-lib`.

```
Figma: a slot                 .dcf                 the app: four lines                harry
#home/night  ──fetch──►  smarthome.dcf  ──►  state.rs      pub night: Toggle          clones the variant
 └ instance #toggle                           ui_model.rs   night: Slot = "#home/night" into the slot,
                                              presenter.rs  model.night = slot(...)    moves the parts
                                              actions.rs    "#home/night" => night.on  reports taps by name
```

## 1. The slot in Figma (the contract)

1. In the design file, on the screen or in the sheet's content component: drag the component
   from the Library (`#toggle`, `#button` ...), pick any variant (the app sets the real one).
2. Wrap it in a frame of the same size (Ctrl/Cmd + Alt + G) and name the frame
   `#<screen>/<name>` (`#home/night`). The frame is the slot; its name is the contract with
   the app. A list is a clipping frame with as many preview rows as it shows, its scrollbar a
   slot of its own beside it (`docs/examples/list_item.rs`, `docs/examples/scrollbar.rs`).
3. Texts and glyphs in the preview are placeholders. Nothing hidden: a hidden layer is
   `display: none` forever in harry.
4. If the file is generated (the Smart HOme file is: `everyware/design/figma/*-patch.js`), the
   slot goes into the generator with `slotted(...)` and the file is rebuilt and imported
   (figma-design skill); never edit a generated .fig by hand.

## 2. Fetch

The user exports `FIGMA_TOKEN` and runs the fetch (the token never goes on a command line or
into a file; the file key is the local `scripts/figma-key`, not in git):

```bash
# WHAT: the design into the app's .dcf (root frame + Library); keeps the previous .dcf next to it
# WHY: the app reads nodes by name from this file; a slot that is not in it does not exist for harry
FIGMA_TOKEN=... DCF_CHECK='#smarthome-root #home/title' vendor/sdv-everyware/everyware/scripts/fetch-dcf.sh packages/services/display_safety/utils/smarthome-app/data/smarthome.dcf
```

## 3. The four lines in the app

From `docs/examples/<component>.rs`, the parts between the doc marks, with your slot and field names. One component value per
place on the screen; the field name is yours.

| File | Line | What it is |
|---|---|---|
| `state.rs` | `pub night: Toggle,` and `night: Toggle::new(false)` in the constructor | the truth: the component's state lives in the app |
| `ui_model.rs` | `night: Slot = "#home/night" ["SlotCustomizer"];` (`interactive` for a list) | the contract: one field per `#name` |
| `presenter.rs` | `model.night = slot("#home/night", &state.night);` | after every action: variant, texts, parts, transition into the slot |
| `actions.rs` | the arm `"#home/night" => { if let Some(ToggleEvent::Toggled(on)) = state.night.on(gesture) { ... } }` in the match on the node name | the component takes the gesture, the app acts on the event |

Components with inner tap targets (segmented, dropdown items, keyboard keys, list rows) report
taps by the inner name (`#segmented/segment-2`, `#dropdown/item/1`, `#home/list/3`): the arm
matches the prefix (`node.starts_with(...)`) and reads the index from the name. The gesture
reaches `apply` as `HomeAction::Gesture(node, gesture)`, sent by the app's
`ui_event_listener.rs` (written once per app).

## 4. Check

```bash
# WHAT: the contract: every variant and part the code uses exists in the fetched .dcf
# WHY: a missing variant or a renamed layer fails here, not on the device
m everyware_ui_test
out_rb3/host/linux-x86/nativetest64/everyware_ui_test/everyware_ui_test
# the same check against the .dcf an app ships (what the device reads)
LIBRARY_DCF=packages/services/display_safety/utils/<name>-app/data/<name>.dcf \
  out_rb3/host/linux-x86/nativetest64/everyware_ui_test/everyware_ui_test
```

Then on the device (the harry-app skill: build, run, verify): press and release it, the
pressed layer fades, the value moves, the event reaches the app (`log::info!` in the arm).
Before the commit: `scripts/check.sh`.

## When it does not work

| Symptom | Cause | Fix |
|---|---|---|
| the slot shows the preview, the app's values do nothing | the field is missing in `ui_model.rs` or the name differs from the frame's | the names must be equal; `everyware_ui_test` only checks the library, not the slot names: compare by hand |
| a tap does nothing | the slot is not `interactive` (lists), or the arm matches the wrong name (inner targets report their own names) | see step 3 |
| two instances move or show the same | both share the part names and are not in slots | one slot per place |
| the variant is right but the picture is the Library's (empty sheet, default texts) | the app set `Variant` on the instance | never set a variant on an instance of the library; the component's `variant()` goes through the slot |
| `everyware_ui_test` names a variant or a part | the code and the design disagree | the message says which side to change (ui-component skill) |

A change lands through its own branch, a review by another agent and the user's word:
`vendor/sdv-everyware/everyware/AGENTS.md`, "How a change lands" (`scripts/feature.sh` there).
