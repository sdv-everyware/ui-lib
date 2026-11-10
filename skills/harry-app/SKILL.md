---
name: harry-app
description: Write, build, run and debug a harry app on the EveryWare UI library - the Figma design with #-named layers and its .dcf, the Rust app with State / actions / reducer / presenter / UI model (everyware_ui::ui_model!), fonts, touch, side effects, the build for the RB3 (sdv_media_db845c, out_rb3), the run on the board's HDMI, the log and frame-rate checks. Use when the user asks to change or debug an existing harry app (smarthome-app, tv-app, hub-app, pairs-app or a new one), to add a plain text, a tap or a side effect, to bind a Figma layer to code, to run an app on the RB3, or when an app shows broken text, a black or noisy screen, dead touch or a low frame rate. For a new app from one sentence use new-app; for a library component on a screen use-component; for a sheet sheet.
---

# harry apps

A harry app is a Rust binary on harry, the UI framework of `packages/services/display_safety`
(DS below): harry draws a Figma design document (`.dcf`) with Impeller on the GPU and drives the
display with KMS itself, without SurfaceFlinger. An EveryWare app adds the UI library on top.

The reference app is `DS/utils/tv-app` (the template of `scripts/new-app.sh`). The smart home
panel `DS/utils/smarthome-app` is the full example; its `docs/harry-app-anatomy.md` explains every
part with file:line links: read it before writing code. Others: `DS/utils/hub-app` (a grid of
tiles and a list), `DS/utils/pairs-app` (a game: state over time).

Supporting files: [reference.md](reference.md) (the loop, threads, UI model field types, side
effects, fonts, build, run, verify), [traps.md](traps.md) (symptom, cause, fix: all seen on the
RB3).

## 0. Rules

The project's rules are in `vendor/sdv-everyware/everyware/AGENTS.md` (the tree's top links it as
`AGENTS.md`): the plan before code, the user decides layouts, a change through a branch, a review
and the user's word. For apps in particular:

- Every command on the board: one line before it, saying what it does and whether it changes the
  board's state. Changes on the board are runtime only (`stop`, `setprop`, `/data/local/tmp`);
  the image is not changed unless the user asks.
- harry itself (`DS/framework/...`) changes only with the user's OK, as a commit of its own on
  `super-harry`.
- Layout, resolution and where things go on the screen are the user's decision: offer options.

## 1. Where an app lives

Under `DS/utils/<name>-app/`, as a git repository of its own. Not elsewhere: libharry,
libharry_app_utils, libhar_user_input_evdev and libhar_log are visible only to
`//packages/services/display_safety/...`, and Soong refuses other paths.

## 2. Workflow

1. **Design.** A Figma frame of the display's size (the RB3's HDMI: 1920 x 1080) named
   `#<app>-root`. Every layer the code touches gets a `#name` (`#home/title`,
   `#room/bedroom/light`); a hidden layer is not in the `.dcf`. An app on the library builds its
   design from a patch on the Library: `design/build.sh` writes `design/<app>.fig` and
   `data/<app>.dcf` (the figma-design skill of everyware).
2. **Fetch, when the design is edited in Figma.** From the tree's top:
   `vendor/sdv-everyware/everyware/scripts/fetch-dcf.sh packages/services/display_safety/utils/<app>-app/data/<app>.dcf <key>`.
   The user exports `FIGMA_TOKEN` and runs it with the file key from
   `figma.com/design/<KEY>/...`; the token is never put in a command or a file.
3. **Fonts.** List the families and weights the `.dcf` uses (reference.md). Load every face into
   the renderer and into the layout helper: one `FONTS` table (`src/resource_loader.rs` of
   tv-app). A missing face: ask before downloading one.
4. **The element in code:** a state field (the fact); an action and its `apply` (fast, no I/O);
   a field in `everyware_ui::ui_model!` (`interactive` for a node that takes taps between its
   children); a presenter line; an arm in the listener or in `actions.rs` from the node's name to
   the action. I/O, SDV and the network go through side effects or a thread of their own that
   sends actions. A library component is one `Slot` (the use-component skill). The checklist and
   the field types: reference.md.
5. **Build** for the RB3: `m <name>_app` after `source device/rb3/sdv_media_db845c/tools/env.sh`,
   in bash. Show the last lines of the build.
6. **Run.** `vendor/sdv-everyware/everyware/scripts/run-smarthome-rb3.sh` is the model: adb root,
   stop what holds the display (the installed panel: `stop smarthome`), push the binary and
   `data/`, `setprop log.tag INFO`, run with the libraries' path and `--touch-size` from
   `/sys/class/drm/card0-HDMI-A-1/modes`. Run it in the background. The new-app skill, step 8,
   has the commands for any app.
7. **Verify.** The process is alive (`pidof`), `logcat` with the app's tag, the frame rate from
   the DRM trace (reference.md). Look at the screen before saying it works.
8. **Commit** in the app's repository; update its docs when a new pattern appeared.

## 3. When something is wrong

Read [traps.md](traps.md) first: broken text, `Permission denied`, noise blocks, a black screen,
dead touch, a low frame rate, a missing layer and the build errors (`missing-docs`, visibility,
chrono) are there with the fix that worked.
