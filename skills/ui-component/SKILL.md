---
name: ui-component
description: Add a new UI component (or a new variant or transition of an existing one) to the Smart HOme component library - components.json spec, Figma component set via the kiwi generator, Rust state machine in everyware_ui_core, harry binding, use in a harry app, fetch and check on the RB3. Use when the user asks to add a button/slider/chip/stepper/... to the UI library, add a state or variant to a component, change a transition, or when a library component does not switch variant or animate on the device.
---

# Add a component to the Smart HOme UI library

A whole screen, a sheet or an overlay, and how designers see and click it in Figma (visible
frames, prototype): the figma-design skill (`vendor/sdv-everyware/everyware/skills/figma-design`).

A component lives in four places that must agree. One file is the source for the first two:

```
ui-lib/components.json ──┬──► Figma: ui-lib/design/components-patch.js  (component set, variants)
  (set name, properties, │
   transitions)          └──► Rust:  ui-lib/core/src/transitions.rs (include_str!)
ui-lib/core/src/<component>.rs   state machine: gestures → variant (state) + values → events
ui-lib/harry/src/slot.rs         Slotted: component set + parts (texts, meters, opacities)
the app (ui_model / presenter / actions)   one Slot field per place on the screen
```

On the screen a component sits in a **slot**: a frame named for the app (`#wifi/reveal`) with
a preview instance inside. harry replaces the preview with a clone of the variant and gives
the clone its own customizations (harry's list items), so one component can be on a screen
many times. A variant change rebuilds the tree (~50 ms on the RB3), so the variant carries only
what rarely changes (disabled, selected, error, a kind). Everything that changes under the
finger or moves (pressed, thumb, fill, fade, caret) is a meter value or an opacity:
display-only in harry, one cached frame per step, smooth.

**Components by role, never by use case.** The library serves every app on every device: a
component is named and shaped by its generic role, as Material 3 does it (a list item with a
leading slot that holds an icon or an image, a headline, supporting text, a trailing end), and
a new need extends an existing component by a property or a part (`#list-item` `kind=image`,
`lines`) before a new set is made. No `MediaItem`, `NewsCard`, `WifiRow` in ui-lib; the app
fills the generic component with its content.

Before step 1, name the role. The request names a use ("a weather icon", "a Wi-Fi row", "a
chart of the temperature"); the component is what it is without that use:

| Asked for | The component | What stays in the app |
|---|---|---|
| a weather icon | a pictogram: one picture of a named set | which weather code is which picture |
| a chart of the next hours' temperature | a bar chart: bars of 0..100, an accent per bar | temperatures to heights, "rain" to the accent |
| a Wi-Fi network row | a list item with a leading icon and a trailing icon | signal strength to the icon |
| a room card | a tile with a symbol, a title and a value | the room and its devices |

No data of a domain, no mapping from data to a look, no domain text and no data source in the
library (`AGENTS.md`, "The library knows no use case"). `scripts/check-roles.py` fails on a
use-case word in a module's name, a public item or components.json. If the role has no obvious
name, ask the user before writing anything.

Paths below are relative to the tree root. Read `traps.md` before step 2.

## 1. Spec: `vendor/sdv-everyware/ui-lib/components.json`

- Add `"<snake_name>": { "set": "#<kebab-name>", "props": { "<prop>": ["<value>", ...], ... } }`.
  The order of `props` is the order of the variant name: `"kind=primary, state=pressed"`.
- Every tappable component has a `state` with `default` and `disabled` (more if needed:
  `selected`, `error`, ...). Pressed is a layer `#<name>/pressed` the code fades in, not a
  variant (button, icon button, list item, keyboard). Toggle, slider and radio still have a
  `pressed` variant (the toggle's thumb stretches); move them to a layer when they jump.
- Add the component to the `transitions` it uses (`press`, `release`, `value`, `open`, `close`,
  `select`), or a new entry with `ms` and `easing` (`linear`, `ease-in`, `ease-out`,
  `ease-in-out`). `disable` is never animated.
- Values that are not variants (on/off, a slider value, open/closed) are not in `props`: they
  move parts by meter value or opacity (note them in a `"value"` field).

## 2. Figma: `vendor/sdv-everyware/ui-lib/design/components-patch.js`

Every component has one owner, the code or Figma (`design/WORKFLOW.md` has the pictures):

| | Drawn by code (most components) | Drawn in Figma (`"figma": true`) |
|---|---|---|
| Source | a `componentSet(...)` call in components-patch.js (steps 1 to 6 below) | the designers' set in Figma, fetched into `design/source/library.json` |
| Who changes the look | the agent, in the patch | a designer in Figma, then `design/fetch-library.sh`; the agent with a tweak on top |
| components.json | `set`, `props` | `set`, `props`, `"figma": true`, `"parts"`: the `#<name>/<part>` layers the code sets, in every variant |

### A component drawn in Figma

`design/fetch-library.sh` (the user exports `FIGMA_TOKEN`) reports `++ #<name>: new in Figma, not
in components.json`: a designer added a set. Adopt it:

1. Read the set in `design/source/library.json` (`sets["#<name>"]`): its variants' names are
   `prop=value, ...`: they give `props`, in that order. The layers named `#<name>/<part>` that
   are in every variant give `parts`. A layer the code must set and that is missing or hidden in
   a variant: tell the user what the designer has to change; do not draw it.
2. components.json: the entry with `set`, `props`, `"figma": true`, `"parts"`.
3. Ask the user what the component does (a tap, a value, what it reports); then steps 3 to 5
   of this skill (Rust core, harry binding, the app) as for any component.
4. `design/build-library.sh`: the set is built from the JSON and checked (variants fit `props`,
   every part in every variant, nothing hidden). `everyware_ui_test` fails on a variant the code
   produces and the set lacks.

Handing a code-drawn component over to the designers (the fetch said `!! #<name>: drawn by code,
and CHANGED in Figma`, and the user chose Figma): `"figma": true` and `"parts"` in its entry,
fetch again, build. Its `componentSet(...)` call in components-patch.js stays (it gives the set
its place in the Library); the draw function in it is not run any more and can be emptied.
Keeping the code instead: `design/fetch-library.sh --accept '#<name>'`.

Changing the look of a component drawn in Figma (the user asks for a colour, a layer the code
needs, a variant): a tweak in `design/figma-tweaks.js`, keyed by the set's name; its header has
the tools and an example. Write it so that applying it to its own result changes nothing (`t.set`,
`t.fill`, `t.addVariant` do; add a layer only when `t.find` does not find it). The build then
says whether the designers still lack the change (tell the user to paste the set from
`design/build/library.fig` into the Figma file) or have it (delete the tweak). Never edit
`design/source/library.json`: the next fetch replaces it.

A component you drew by code reaches the designers by hand: tell the user to copy the set from
`design/build/library.fig` into the Figma file.

Drawn by code:

1. Add a `heading(...)` and a `componentSet(lib, '#<name>', x, y, props, w, h, gapX, gapY, draw)`
   call. `props` as in components.json; `draw(symbol, values)` draws one variant with
   `rect`, `ellipse`, `text`, `frame`, `chevron`, `plus` from everyware's `design/figma/fig-helpers.js`
   (roundtrip.js hands it to the patch).
2. Names: the set `#<name>`; layers the code changes `#<name>/<layer>` (`#slider/fill`); other
   layers plain (`bg`, `track`). Same layer names in every variant.
3. Colours from `C` in `fig-helpers.js` (the dashboard palette), opaque fills, no hidden layers.
   A part that moves gets a DesignCompose meter: `meter(node, 'progressMarkerData', { startX,
   endX })` (moves x) or `meter(node, 'progressBarData', { endX })` (width). A marker node
   stands at **x = 0** of its parent, startX..endX are its travel (see traps.md). A part that
   fades is a separate layer the code sets the opacity of (`#toggle/on`). The pressed look is
   a layer `#<name>/pressed` in the pressed colour, opacity 0, above the background and below
   the label, in every variant.
4. Place it in `Library` without overlapping (grow the frame and move `TY`, the transitions
   table, if needed). Add `NAMES[<snake_name>]` for the table.
5. Add one or more instances to `#ui-gallery` with `g('#<name>', '<variant name>', '#ui/<name>', x, y)`.
   On a screen (smarthome-app's `design/keyboard-patch.js`) place it with `slotted(parent, '#<screen>/<name>', x, y, variant)`.
6. Build the Library alone (what every app's design starts from) and the panel that uses it
   (from the designer's export in smarthome-app's `design/source/`):
   ```bash
   # from the tree's top; both write design/build/ (build outputs, not in git)
   vendor/sdv-everyware/ui-lib/design/build-library.sh             # -> ui-lib design/build/library.fig
   packages/services/display_safety/utils/smarthome-app/design/build.sh   # -> design/build/smarthome.fig
   ```
   The log must list the new set with the expected number of variants. A new icon of the
   design: it must be in `design/library-base.fig` too (`build-library.sh --base` from the
   export keeps every icon frame of the drawing).
7. The user imports the file into Figma (a new document, a new key) and checks in Layers that
   the slots and their `…/preview` instances are there.
8. Fetch with the new key and store it:
   ```bash
   FIGMA_TOKEN=... vendor/sdv-everyware/everyware/scripts/fetch-dcf.sh packages/services/display_safety/utils/smarthome-app/data/smarthome.dcf <key>
   echo <key> > vendor/sdv-everyware/everyware/scripts/figma-key
   ```
   Check the `.dcf` (`utils/smarthome-app/data/smarthome.dcf`):
   `grep -c -a -F '<prop>=<value>, state=pressed' smarthome.dcf` > 0, and the slot names > 0.
   Meter data is binary in the .dcf (no JSON to grep): check it on the device.

## 3. Rust core: `vendor/sdv-everyware/ui-lib/core/src/<name>.rs`

- `ComponentKind::<Name>` in `transitions.rs` (serde `snake_case` must match the json key).
- A struct with its state and a `press: Pressable` (enabled, pressed, last change), a
  `pub fn on(&mut self, gesture: Gesture) -> Option<<Name>Event>`, and
  `impl Component { const KIND; fn variant(); fn last_change() }`. `variant()` returns exactly
  the Figma variant name (state only). Values as targets 0..100 (`target()`, `list_target()`)
  with their transition (`value_transition()`), as in toggle.rs, slider.rs, dropdown.rs.
- Gestures: `Down { x }`, `Move { x, inside }`, `Up { x, inside }`, `Cancel`; a tap is
  `Up { inside: true }` after `Down` (pressable.rs). Events stay with the node of `Down`, so a
  drag (slider) keeps going outside the component.
- Every public item documented (libraries build with `-D missing-docs`).
- Tests in the module: variant names, events, transitions read from the spec.
- `pub mod <name>;` in `core/src/lib.rs`.
- Time-based behaviour (a caret's blink) takes `now: Instant` as an argument, so tests are
  deterministic (input_field.rs).
- Run on the host:
  ```bash
  m everyware_ui_core_test && out_rb3/host/linux-x86/nativetest64/everyware_ui_core_test/everyware_ui_core_test
  ```

## 4. harry binding: `vendor/sdv-everyware/ui-lib/harry/src/slot.rs`

`impl Slotted for <Name>`: `const SET = "#<name>"` and `parts()`, which sets on the item what
is not the variant:
```rust
item.text("#button/label", self.label.clone())                      // text
    .meter("#toggle/thumb", Meter::new(self.target(), self.value_transition()))  // moves
    .opacity("#toggle/on", Meter::new(self.target(), self.value_transition()))   // fades
    .interactive("#dropdown/item/1", self.open)                     // inner tap target
```
The pressed layer: `pressed(item, "#<name>/pressed", &self.press, Self::KIND)` (opacity from
`Pressable::pressed_target`, 80 ms in, 160 ms out from components.json).

`SlotCustomizer` moves every meter and opacity to its target frame by frame (`Follow`, the
curves of components.json) and asks for the next frame while something moves; a blinking part
(`.blink(node, since)`, the caret) wakes harry with `UpdateIn` at the next flip instead.

Tests (`m everyware_ui_test`, host): add the component to `library()` in the tests of slot.rs.
`every_part_the_code_sets_is_in_every_variant_of_the_figma_set` loads the Library's own
`data/library.dcf` (`design/build-library.sh --dcf` after a change of the Library) and fails when
a part name of `parts()` is missing in any variant of the set (a typo, a renamed layer, a variant
drawn without the part). Run it after every change of the Library; `LIBRARY_DCF=<path>` checks
another file, for example the `.dcf` an app ships.

Slots rely on harry patch 0005 (`smarthome/patches/`): without it a hug text in a slot is
measured with the design's text and an opacity set by the item never reaches the drawing.

## 5. In an app (example: `utils/smarthome-app`)

- `State`: the component (`ui::<name>::<Name>`), created with its initial value.
- `ui_model!`: one field per slot, nothing on the inner parts:
  ```
  wifi_reveal: Slot = "#wifi/reveal" ["SlotCustomizer"];
  ```
- Presenter: `model.wifi_reveal = everyware_ui::slot("#wifi/reveal", &state.wifi.reveal);`
- Actions: `HomeAction::Gesture(node, g)` from `everyware_ui::Input` (ui_event_listener.rs);
  taps on the clone come with the slot's name (`slot_of`); the reducer calls `state.x.on(g)`
  for its node and acts on the event.
- A component with inner tap targets (keyboard keys, dropdown items) reports taps by the inner
  names (`#kbd/r1/3`, `#dropdown/item/2`); they are unambiguous while one such component is on
  the screen (one keyboard, one open list).
- Build `m smarthome_app`, run `vendor/sdv-everyware/everyware/scripts/run-smarthome-rb3.sh`,
  screenshot (see the harry-app skill), press and release the component on the touch screen.

## Example: a composite component (the keyboard)

The keyboard shows how a component with many parts fits the same scheme:

| Layer | What |
|-------|------|
| components.json | `"keyboard": { "set": "#keyboard", "props": {} }` plus press/release transitions |
| Figma (components-patch.js) | set `#keyboard`, one variant `state=default`; 31 keys, each a frame `#kbd/<id>` with `bg`, `#kbd/<id>/pressed`, `#kbd/<id>/label` |
| core (keyboard.rs) | modes and labels per mode, `on(node, gesture, &mut text) -> Option<Outcome>` (pressed, Shift and mode inside; `Done` / `Edited` out), `key_nodes()`, `label_of()`; `impl Component` with the one variant |
| harry (slot.rs) | `Slotted for Keyboard`: for every key node its label, its pressed layer and `interactive(node, true)` |
| app | `osk_keys: Slot = "#osk/keys" ["SlotCustomizer"]` in the keyboard dock's slot, `model.osk_keys = slot("#osk/keys", &state.osk.keyboard)`; typing goes through a form: `form.on(node, g, now, &mut state.osk.keyboard)` |

Labels are texts the code sets (they change per mode), so one Figma variant serves every
mode; a second layout would be a second value of a property (`layout=letters|symbols`). What
the outcome means (what Done does) stays in the app; which field gets the text is the form's
(below).

## Example: the sheets (sheet, form, keyboard dock)

A sheet over the screen is the library's `#sheet` (components.json `sheet`: `layout` dialog, left,
right, top, bottom x `state` closed, open) with the app's **content component** swapped into its
`#sheet/content` slot (figma-design skill, `sheetInstance`). The library gives the geometry, the
state machine and the transitions; the app's generator draws only the content:

| Component | Set | core | What the app does |
|-----------|-----|------|-------------------|
| `Sheet` | `#sheet`; the instance's parts renamed `#x/scrim`, `#x/card`, `#x/content` | `Sheet::new(Layout)`, `open()`, `close()`, `is_open()`, `card_position(screen)`, `scrim()`, `transition()` | never sets the sheet's variant (harry would take the Library's variant and lose the instance's names and content); `model.x_card = Position::new(sheet.card_position(SCREEN, free_height), t)` with `FollowPositionCustomizer` (`free_height`: the screen above the keyboard dock while it is open, `KEYBOARD_DOCK_H`; a dialog centres there), `model.x_scrim = Meter::new(sheet.scrim(), t)` with `FollowOpacityCustomizer`, a `TapTarget` on the scrim |
| `Form` | a `Sheet` with input-field slots in the content (`#wifi/password`) | fields, focus, `on(node, g, now, &mut keyboard) -> Focused \| Edited \| Submitted`, `sheet` | `Form::new(Layout::Dialog, [("#wifi/password", field)])`, acts on `Submitted`; a step is the content's variant (`#wifi/content`, `step=`) |
| `KeyboardDock` | `#keyboard-dock`, `state=closed\|open`, the slot `#<app>/keys` with `#keyboard` inside, y = 1080 + 40 or 1080 - 420 | `follow(focus)` | one per screen: `state.osk.follow(a.focused().is_some() \|\| b.focused().is_some())` |

How harry animates the switch: the instance's variant changes, harry rebuilds its subtree
from the new variant and, for every node that exists in both (same name), interpolates
position, size, opacity and colour from the old drawing to the new one; a node only in the
new variant fades in. Two rules follow (traps.md):

- every part that moves needs its own `AnimationOverride` field in the UI model: harry takes
  a node's own override, a parent's is not inherited (`squoosh/src/view.rs` animate_changes),
  so the presenter copies the instance's transition onto `#wifi/scrim`, `#wifi/card`,
  `#osk/keys`;
- the instance sits in a plain frame with "absolute position" (`Wi-Fi sheet layer`,
  `Keyboard layer`): Figma's import keeps the flag on a frame and drops it on an instance.

The keyboard is the screen's (as a system keyboard), not a part of each sheet: a second form
on the screen gets it for free, and the sheet's set stays two variants. The reference app
animates its scenes the same way, with a default `AnimationSpec` on the reducer instead of
per-node overrides (`reference/harry-app/src/services.rs`); the library keeps the overrides
so that open and close can differ (200 ms ease-out, 150 ms ease-in) and nothing else animates
by accident.

## Example: list items with an image (a news feed)

| Layer | What |
|-------|------|
| components.json | `list_item`: `kind` adds `image`, `lines` one / two (72) / three (128: headline and supporting text up to two lines each), `size` adds `column` (490); parts `#list-item/image`, `#list-item/trailing-text` (kind image) |
| Figma (components-patch.js) | the set has only the combinations that make sense (`componentSet(..., { only })`: image with lines two and three, the other kinds with one and two), 90 variants, on its own area of the Library |
| core | `ListItem::new(ListItemKind::Image, headline, supporting).lines(ListItemLines::Three).size(ListItemSize::Column).image(key).trailing_text(..)`; `ListView<R: Row>` works for any row (`Row`: key, tap, selected, state across a refresh) |
| harry (images.rs) | `everyware_ui::images`: `register_encoded(key, png_or_jpeg)` / `register_rgba`, `get(key)`; the app gives the renderer `images::storage()` (`RendererHelper::initialize_renderer`: `renderer.replace_raw_image_storage`); the glue sets the registered picture on `#list-item/image` per row (harry's image customization; an unknown key keeps the placeholder) |
| app | `news: Slot = "#news/list" interactive`, `model.news = list("#news/list", &view)`, as the Wi-Fi list |

## Example: a scrolling list with a scrollbar and icon slots (the Wi-Fi networks)

| Layer | What |
|-------|------|
| components.json | `list_item` with `size` (regular 440, column 490, wide 960), `lines`, and the two icon slots; `icon` (the glyphs); `scrollbar` (`length` per list height, transition `hide`) |
| Figma (components-patch.js) | `#list-item`: `#list-item/leading` and `/trailing` are **glyph stacks**: every glyph of `#icon` as an instance `#list-item/<slot>/<glyph>` at opacity 0; `#scrollbar`: a track and `#scrollbar/thumb`, a vertical progress marker 0..length-48 |
| Figma (the app's design) | the list slot is a frame that **clips** (`frameMaskDisabled: false`) with preview rows; the scrollbar slot next to it, as high as the list |
| core | `list_view.rs`: `ListView` with Android's touch handling from `scroller.rs` (touch slop 8, VelocityTracker LSQ2, OverScroller spline fling, a press catches a fling); `ListItem::leading/trailing(glyph)`; `Scroll { base, fling, moved_at }` and `scrollbar_shown()` |
| harry (slot.rs) | `list(slot, &view)`: one `ComponentItem` per row at `row * pitch`, the scroll subtracted **per frame in the customizer** (a fling steps there, the app gets no ticks), rows out of the slot not interactive (harry hits by bounds, no clipping); `scrollbar(slot, &view)`: the thumb meter and the fade from `Scroll`; glyphs by `.opacity(...).optional(...)` (optional: not every kind has the slot) |
| app | `wifi_networks: Slot = "#wifi/networks" interactive` (a drag may start between rows), `wifi_scrollbar: Slot`; `model.wifi_networks = list(...)`, `model.wifi_scrollbar = scrollbar(...)`; taps come as `#wifi/networks/<row>` or `#wifi/networks` → `view.on(row, gesture, now)` → `Chosen(row)` |
| input | `Gesture` carries `y` and, at the lift, the finger's speed: `everyware_ui::stamp` in the app's evdev callback (`main.rs`), `Input::with_velocity`; raw moves and lifts away from every node reach the node the touch started on |

Why glyph stacks and not a variant of an inner `#icon` instance: harry does not swap the
variant of an instance inside a list item (`overrides.rs` reads the global customizations
only); opacity is per item and display-only. Why the list refreshes safely: `set_rows` keeps
the pressed and selected state of a row with the same title, so a tap survives a topic update
between its down and its up.

## The .fig for a review

**Always give the user the built .fig for a review.** After every design build, in your reply:
the full absolute path of the built file (the build script prints it as its last line, `review in
Figma (File > Import): ...`), what changed in it, and where to look (the frame or the set's name).
Not a bare file name, not a path relative to the repository.
For the Library: `<tree>/vendor/sdv-everyware/ui-lib/design/build/library.fig`, the set on the
page Components; for the panel `<tree>/packages/services/display_safety/utils/smarthome-app/design/build/smarthome.fig`.

## 6. Done when

- [ ] components.json, Figma set, Rust `variant()` use the same names
- [ ] the new `.fig` is versioned, imported, fetched; key in `figma-key`
- [ ] `everyware_ui_core_test` and `everyware_ui_test` (contract with the fetched `.dcf`) pass;
      `libeveryware_ui` and the app build
- [ ] on the device: pressed shows, values move smoothly and the right way, the event reaches
      the app; two instances on one screen move independently
- [ ] docs: `docs/content/<component>.md` written (role, Figma, the app, traps), the site and
      the llms files regenerated (`node docs/build-docs.js`); the component in `smarthome/docs`
      if the dashboard uses it
- [ ] `scripts/check.sh` prints `all good` (tests, coverage, docs, author); commits in `ui-lib`
      and `smarthome` (author `Daniel Georg <passenger6@gmail.com>`, no AI trailer)

A change lands through its own branch, a review by another agent and the user's word:
`vendor/sdv-everyware/everyware/AGENTS.md`, "How a change lands" (`scripts/feature.sh` there).
