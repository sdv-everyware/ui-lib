---
name: new-app
description: Make a new EveryWare app (a new screen as its own harry app on the EveryWare UI library) from one sentence, end to end - the layout agreed as an ASCII sketch, the app's repository from the template-app template (scripts/new-app.sh), the design as a kiwi patch of Library slots, the .fig and the .dcf without Figma (roundtrip.js, fig-to-dcf.sh), the app's state, actions, presenter and UI model, the build for the RB3 or Cuttlefish, and the run on the target with a screenshot. Use when the user asks for a new EveryWare app, board, dashboard, screen or device UI ("a kitchen panel", "an app with only the time", "a launcher like webOS"), or wants to try the library from a new app. For one more control on an existing screen use use-component; for a new component ui-component.
---

# A new EveryWare app from one sentence

An EveryWare app is a harry app on the EveryWare UI library: `packages/services/display_safety/utils/<name>-app`, its own git
repository (like smarthome-app, hub-app, tv-app; display_safety does not track it), with its
design `design/<name>-patch.js` and `data/<name>.dcf`. The design is generated: the patch puts Library
variants into slots under `#<name>-root` of the EveryWare Library .fig (ui-lib `design/build-library.sh`). The app drives the
`#<name>/*` names; the names are the contract between the patch and `src/ui_model.rs`.

```
sentence ──► 1 layout (ASCII, the user picks) ──► 2 new-app.sh <name>
                                                   └─ utils/<name>-app   (template-app renamed, git)
                                                       └─ design/<name>-patch.js (the template's, renamed)
         ──► 3 the patch ──► 4 .fig ──► 5 .dcf ──► 6 the app's code ──► 7 m <name>_app ──► 8 target
```

## Which tree for which step

| Step | Where | Why |
|---|---|---|
| 2 app and patch, 3, 6 code, 7 build | the app's repository in the tree, `out_rb3` or `cuttlefish_out` | the patch and `ui_model.rs` are the contract and change in one commit; Soong builds the app from its folder |
| 4, 5 (the tools) | the tree's everyware: `vendor/sdv-everyware/everyware` (new-app.sh warns when its Figma tools differ from main's, which is so only while a change of them is open) | roundtrip.js, fig2json.js and fig-to-dcf.sh (the dark theme is `$UI/design/dark-theme-patch.js`); fig-to-dcf.sh imports with everyware's `tools/dcf` (dcf-fetch `--local`, on our fork of DesignCompose) |
| 8 | the RB3 or Cuttlefish over adb | |

Below, from the tree's top: `FIGMA=vendor/sdv-everyware/everyware/design/figma`,
`UI=vendor/sdv-everyware/ui-lib`, `APP=packages/services/display_safety/utils/<name>-app`.

## 1. The layout first

The layout is the user's decision. Before any file: two or three options as ASCII sketches of the
1920 x 1080 screen, which Library component sits where (button, icon-button, toggle, slider,
segmented, list-item, tile, symbol, bar-chart; `$UI/docs/llms.txt` lists them,
`docs/content/<component>.md` and `components.json` have each one's variants), a line of trade-offs per option. Wait for the
choice. Only names of generic roles: a new kind of control is a new Library component
(ui-component skill), not a drawing in the patch.

## 2. The app and its patch

```bash
# from the tree's top
$UI/scripts/new-app.sh <name>       # [a-z][a-z0-9]*, e.g. kitchen; EVERYWARE_FIGMA=<dir> for another checkout
```

It copies template-app's committed files (no .git, no .dcf, no screenshot), renames every
template name (`<name>_app`, `<Name>Action`, `#<name>-root`, `#<name>/*`, `/data/local/tmp/<name>`),
makes the git repository with its first commit; `APP/design/<name>-patch.js` is the template's
patch renamed, its root at the next free place on Page 1 (it prints where). The copy builds as it
is and draws the template's screen (the time, a title, a number and a button that counts up)
under the new names; the README and AGENTS.md say "describe the screen here".

## 3. The patch: the screen from Library slots

Edit `APP/design/<name>-patch.js` to the chosen layout (tv-app's, hub-app's and pairs-app's
`design/*-patch.js` are complete examples; `FIGMA/fig-helpers.js` is the authoring API, which
roundtrip.js hands to the patch as its second argument: `function (msg, { key, C, FONT, authoring })`,
no `require` of it):

- `slot(parent, '#<name>/<part>', x, y, variant('#set', 'prop=value, ...'))`: a frame named for the
  app with a preview instance of the variant; harry clones the variant the app picks into it.
- `text(parent, '#<name>/<part>', x, y, w, h, 'preview', size, FONT.x, C.x)`: a text the app sets.
- A list (vertical or a ribbon of `#tile`): a clipping frame `#<name>/<list>` with preview
  instances `#<name>/<list>/preview-<n>`; a ribbon is `RIBBON_LIFT` (20) higher than a card.
- A grid of the same component (cards, keys, a wall of tiles): one frame `#<name>/<grid>` with
  preview instances at the grid's pitch, not one slot per cell; the app writes it with `grid`.
  Two grids of the same pitch lie on each other for a card with a back and a face
  (`pairs-patch.js`).
- Nothing hidden (a hidden layer is gone for harry too); the app hides a whole component with
  `slot_shown` / `grid_shown` (opacity 0, no taps), a part with opacity.
- The light Library palette (`C.*`); `$UI/design/dark-theme-patch.js` maps it to the dark theme in step 4.
- The root stays where new-app.sh put it (`frame(page, '#<name>-root', -2400, y, ...)`): its own
  place on Page 1, not over another app's root.

## 4 and 5. The .fig and the .dcf

```bash
CHECK='#<name>/time #<name>/<part> ...' $APP/design/build.sh
```

`design/build.sh` (from the template) builds the Library (`$UI/design/build-library.sh`, only
the Library, no app's screen in it), applies `design/<name>-patch.js` and the dark theme into
`$APP/design/<name>.fig`, and makes `$APP/data/<name>.dcf` with fig-to-dcf.sh.
`design/<name>.fig` is in git: the file a designer opens in Figma (the app's README, "The design
in Figma, without scripts"); `design/build/` holds the intermediates. `CHECK` lists names that must
be in the .dcf (0 = misnamed or hidden). It needs `node`, `unzip`, `zip` and the tree's
importer (cargo); the kiwi format comes with roundtrip.js. Optional
check on the host: `dcf-render <dcf> '#<name>-root' out.png` (har-rendering-parity, headless)
draws it with harry.

## 6. The app's code

The flow is harry's: touch, ticker, house link `--<Name>Action-->` reducer (`actions.rs`: the
action changes `state.rs`) `->` presenter (`presenter.rs`: state to UI model) `->` harry draws.

- `src/ui_model.rs`: one field per `#<name>/*` name the app writes or taps (`ui_model!`; `Slot`
  with `["SlotCustomizer"]` for a component, `interactive` for a list). A grid is one field.
- `src/state.rs`: the components (`everyware_ui::*`), `src/presenter.rs`: `slot(name, &state.x)`,
  `slot_shown(name, &state.x, shown)`, `list(name, &state.l)`, `grid(name, &state.cells, columns,
  (pitch_x, pitch_y))`, texts; `src/actions.rs`: `on_gesture` per node (`row_of` for a row of a
  list or a cell of a grid: `"#<name>/<grid>/<index>"`).
- What the screen shows that is the app's own is the app's: a component set in its design and a
  type in its code (`Component` with `ComponentKind::App`, `Slotted` with the set). tv-app's
  weather pictures are an example (`design/weather-pictures.js`, `#tv-weather`, `src/weather.rs`).
  The library has components only; never add an app's content to ui-lib.
- Each component's code with real names: `$UI/docs/examples/<component>.rs`.
- The time comes from `clock.rs`. Data from a bundle comes on a thread of the app that sends
  actions (tv-app's `house_link.rs`: the dashboard bundle's gRPC on port 7100). Replace the
  template's description in the file headers (`new-app.sh` lists them).
- Fit the README's first paragraph to the app.

## 7. Build

```bash
ps -C soong_ui -o pid,etime,args       # another build in the same out dir: wait for it
# env.sh is bash (BASH_SOURCE): from zsh, the Bash tool's shell, run it under bash
bash -c 'source device/rb3/sdv_media_db845c/tools/env.sh >/dev/null 2>&1; m <name>_app'
# or Cuttlefish (no board needed):
bash -c 'source device/cf-everyware/sdv_media_everyware_cf/tools/env.sh >/dev/null 2>&1; m <name>_app'
```

Warnings are errors (`-D warnings`): an unused import or function of the template fails the build.

## 8. On the target (RB3 or Cuttlefish)

Say before every command on the RB3 whether it changes its state. The RB3 is shared with
other sessions: do not stop or kill what you did not start; ask.

```bash
# read only: who draws now (one app at a time holds the display and port 7100)
adb shell 'pidof smarthome_app hub_app tv_app; for p in /proc/[0-9]*; do ls -l $p/fd 2>/dev/null | grep -q card0 && cat $p/cmdline && echo; done'
# changes state: stop the one on the screen, by pid (smarthome_app is init's service: stop smarthome)
adb shell 'kill $(pidof <name>_app)'
# changes state: the app and its design to /data/local/tmp/<name>, then start it
adb shell mkdir -p /data/local/tmp/<name>
adb push out_rb3/target/product/sdv_media_db845c/system/bin/<name>_app /data/local/tmp/<name>/
adb push packages/services/display_safety/utils/<name>-app/data/<name>.dcf /data/local/tmp/<name>/
adb shell "cd /data/local/tmp/<name>; LD_LIBRARY_PATH=/system/lib64:/vendor/lib64:/apex/com.everyware.smarthome/lib64 \
    nohup ./<name>_app --data-path /data/local/tmp/<name> --touch-size 1920x1080 > <name>.log 2>&1 &"
# read only: the screenshot (rb3_fbgrab writes fb.<n>.raw: RGBA 1920 x 1080, modifier must be 0x0)
adb shell 'rm -f /data/local/tmp/fb*.raw; /data/local/tmp/rb3_fbgrab /data/local/tmp/fb'
adb pull "/data/local/tmp/$(adb shell 'cd /data/local/tmp; ls fb.*.raw' | tr -d '\r')" fb.raw
python3 -c "from PIL import Image; d=open('fb.raw','rb').read()[:1920*1080*4]; Image.frombytes('RGBA',(1920,1080),d).convert('RGB').save('screen.png')"
```

A tap for a test (the USB touch reports 0..32767, `x*32767/1920`, `y*32767/1080`): `sendevent
/dev/input/event3` with ABS_MT_SLOT 0, TRACKING_ID, POSITION_X/Y, BTN_TOUCH 1, SYN, then
TRACKING_ID -1, BTN_TOUCH 0, SYN. The app logs `<name>: down on <node>` for every touch.

On Cuttlefish (`adb -s 127.0.0.1:6526`, or `0.0.0.0:6526` on older `cvd`; see `cvd fleet`), push
from `cuttlefish_out/target/product/sdv_media_everyware_cf/system/bin/<name>_app`, interact in
the WebRTC page at `https://localhost:1443`, and capture a screenshot with
`device/cf-everyware/sdv_media_everyware_cf/tools/run.sh screenshot screen.png`.

## 9. The .fig for a review

**Always give the user the built .fig for a review.** After every design build, in your reply:
the full absolute path of the built file (the build script prints it as its last line, `review in
Figma (File > Import): ...`), what changed in it, and where to look (the frame or the set's name).
Not a bare file name, not a path relative to the repository.
For an app: `<tree>/packages/services/display_safety/utils/<name>-app/design/<name>.fig`,
the frame `#<name>-root` on Page 1.

## 10. Review, then the user

The new repository starts on the branch `feature/<name>-first-screen` (new-app.sh makes it): commit
there, with `data/<name>.dcf` and `design/<name>-patch.js`. Then the project's way for every
change (`vendor/sdv-everyware/everyware/AGENTS.md`, "How a change lands"): another agent reviews
the branch (the `review` skill), you give the user the package (what was done, the findings and
what became of them, the built `.fig`, the screenshots from the board), and
`feature.sh land <name>-first-screen "<summary>"` only after the user's word.

## Traps

| Symptom | Cause | Fix |
|---|---|---|
| "Failed to make Impeller GL context current ... Permission denied" | another app holds `/dev/dri/card0` | step 8's read-only check; stop it only if you started it |
| an app that uses the dashboard bundle starts, its data never comes | another app holds port 7100 | one app at a time |
| a name binds nothing, the slot shows the preview | the patch and `ui_model.rs` disagree | `CHECK` with the name in step 4 |
| fig-to-dcf.sh: `--local` unknown, or `from_local` missing | run from a tree whose display_safety or external is not on the branches above | run the tree's everyware script |
| build fails on the template's unused code | `-D warnings` | delete it with the feature (step 6) |
| `pkill -f <name>` killed the shell running it | `-f` matches its own command line | kill by `pidof` |
| `source device/rb3/sdv_media_db845c/tools/env.sh` fails or sets nothing | the shell is zsh; the script uses `BASH_SOURCE` | `bash -c '...'` as in step 7 |
| an edit of Android.bp (a new rustlib, a new test) is ignored: "unresolved import", an unknown target | another session's build read the Android.bp files while you edited, so its ninja is newer than your edit and Soong says "No need to regenerate" | `touch <app>/Android.bp`, build again |
| a tap during a timed step (a card turns back after 1 s) comes late | the template's ticker ticks once a second | tick as often as the shortest wait needs, and only while it runs: every tick redraws |
