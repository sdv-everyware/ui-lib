# harry-app reference

From the tree's top; `DS=packages/services/display_safety`.

## The loop

```
touch / timer / data thread --Actions::CustomAction(a)--> reducer thread (harry's DefaultReducer)
   a batch: a.apply(&mut state, &mut side_effects) for each  ->  Presenter::present(&mut model, &state) once
   -> pre-render (resolve, layout, animate, display list) -> looper (main thread) -> GPU -> KMS
```

harry's source: `DS/framework/harry/src/reducer.rs` (the `Actions` enum, `apply` and `present`
in the reducer's loop), `state.rs` (`StateAction`), `side_effects.rs` (`SideEffects::Custom`),
`presentation.rs` (`Presenter`), `pre_renderer/` (the passes and their cache).

## Threads

| Thread | Made by | Job |
|---|---|---|
| main, the looper | `looper.start(rx)` | EGL and KMS, draws the display lists, hit-tests the touch, calls the app's `UiEventListener` |
| reducer | `reducer.execute::<Presenter, _>(looper_tx, LocalDiskFigmaLoader)` | apply, present, pre-render |
| evdev | `EvdevService::new` or `new_scaled` | `/dev/input` to `Actions::PerformUserInputEvent` |
| the app's own | the app (tv-app: the clock, `house_link.rs`) | timers, SDV subscriptions and RPC answers, sent as actions |
| side effects | the app (`side_effects_rx`) | I/O asked for by `apply`, answered as actions |

Only the reducer thread touches the state; every other thread sends actions. The listener is
written once: it hands each gesture to the app as an action with the node's name
(`TvAction::Gesture(node, gesture)` in tv-app's `ui_event_listener.rs`).

## UI model field types

The app declares its model with `everyware_ui::ui_model!` (`ui-lib/harry/src/model.rs`): one line
per node, `name: Type = "#node"`, then `interactive` for a node that takes taps between its
children, then the customizer, then `= expr` for a first value. `Meter` and `Position` have no
first value of their own and always need the `= expr`. smarthome-app keeps a copy of the macro
with the same syntax in its own `ui_model.rs`; the rows below quote both.

| Type | What it does to the node | In an app |
|---|---|---|
| `String` | its text | hub-app `time: String = "#hub/time";` |
| `Slot` | a library component in its slot (the use-component skill) | hub-app `weather_icon: Slot = "#hub/weather/icon" ["SlotCustomizer"];` |
| `()` with `interactive` | a node that only takes taps | smarthome-app `_climate_header: () = "#climate/header" interactive;` |
| `bool` | on or off for a customizer (here smarthome-app's own `ZoneCustomizer`) | smarthome-app `camera_zone: bool = "#camera/feed" ["ZoneCustomizer"]` |
| `Variant` | which variant of a component set the node shows | smarthome-app `wifi_step: Variant = "#wifi/content"` |
| `AnimationOverride` | how the next variant change moves | smarthome-app `wifi_step_animation: AnimationOverride = "#wifi/content";` |
| `Meter` | a value that a customizer turns into an opacity (0 to 100), a meter, or a y in pixels (`FollowYCustomizer`) | smarthome-app `wifi_scrim: Meter = "#wifi/scrim" ["FollowOpacityCustomizer"]` |
| `Position` | a translation, no layout (cheap: the tree is kept) | smarthome-app `wifi_card: Position = "#wifi/card" ["FollowPositionCustomizer"]` |
| `Scrolled` | a body that scrolls in its place with a `ListView` of one row: its y is the scroll's, a fling stepped frame by frame | smarthome-app `event_body: Scrolled = "#event/body" ["ScrolledCustomizer"]` |
| `TapTarget` | where the taps of a node go | smarthome-app `wifi_scrim_taps: TapTarget = "#wifi/scrim" ["TapTargetCustomizer"];` |

Structural (the tree is built again, with its layout): a text, a variant, the items of a list, an
image, a size. Display only (the cache is kept): visibility, opacity, position.

## Element checklist

1. The Figma layer `#name`, visible; the `.dcf` built or fetched again.
2. State: the fact.
3. An action and its `apply` (fast; I/O as a side effect or on a thread of the app).
4. The UI model field (`interactive` for taps).
5. The presenter: state to field.
6. The node's name in the gesture's match, to the action.
7. A new font face in `FONTS`.

## Side effects

```rust
// apply(): never block here
side_effects.push(SideEffects::Custom(AppEvent::SetLight { room, on }));
// the side effects thread: make the call, then send the result back
actions.send(Actions::CustomAction(AppAction::LightConfirmed { room, on }))
```

Example: `DS/utils/gui-playground-app/src/actions.rs`. The four EveryWare apps run threads of
their own instead (tv-app's `house_link.rs`: the dashboard bundle's binder link, answers as actions)
and keep the side effects channel empty (`channel::<()>()` with a thread that drains it).

## Fonts in a .dcf

```bash
python3 - DS/utils/<app>-app/data/<doc>.dcf <<'PY'
import re,sys,struct,collections
b=open(sys.argv[1],'rb').read(); c=collections.Counter()
for m in re.finditer(rb'\x1a([\x01-\x40])([ -~]+?)\x22\x07\x0a\x05\x0d(....)', b, re.S):
    if len(m.group(2))==m.group(1)[0]: c[(m.group(2).decode(), int(struct.unpack('<f',m.group(3))[0]))]+=1
for k,v in sorted(c.items()): print(k,v)
PY
```

In the tree: Inter 400, 700 and 900 and IBM Plex Mono (`DS/utils/smarthome-app/data/`), Inter
in tv-app's `data/`, Noto Sans and others
(`DS/service/harry-prebuilt/data/assets/common/localization/fonts/`).

## Build

The RB3 (arm64, `out_rb3`; in bash, env.sh uses `BASH_SOURCE`):

```bash
bash -c 'source device/rb3/sdv_media_db845c/tools/env.sh >/dev/null && m <name>_app'
# an app for /system: out_rb3/target/product/sdv_media_db845c/system/bin/<name>_app
# an app in an APEX (smarthome_app): the APEX variant under
#   out_rb3/soong/.intermediates/packages/services/display_safety/utils/<name>-app/<name>_app/android_vendor_arm64_*_apex*/
```

## Run

| | The RB3 |
|---|---|
| adb | over USB; the run scripts take the board's serial from `SERIAL` |
| display | HDMI 1920 x 1080; the touch reports 0..32767, so `--touch-size 1920x1080` |
| holds the display | the installed panel, init's service `smarthome` (`stop smarthome`, `start smarthome` brings it back; the board is shared: stop only what you started, or ask) |
| a script to copy | `vendor/sdv-everyware/everyware/scripts/run-smarthome-rb3.sh` |

Common: `adb root`, then poll until `id -u` is 0; `setprop log.tag INFO`; the libraries' path as
in the new-app skill, step 8.

## Verify

```bash
# who holds the display
adb shell 'for p in /proc/[0-9]*; do ls -l $p/fd 2>/dev/null | grep -q dri/card0 && echo "${p#/proc/} $(cat $p/comm)"; done'
# frames in 5 s (each page flip delivers one event)
adb shell 'T=/sys/kernel/tracing; echo > $T/trace; echo 1 > $T/events/drm/drm_vblank_event_delivered/enable; echo 1 > $T/tracing_on; sleep 5; echo 0 > $T/tracing_on; grep -c drm_vblank_event_delivered $T/trace'
# where the pre-render's time goes: run the app with HAR_PASS_TIMING=1, then
adb logcat -d | grep "pre-render ms"
```

A screen at rest makes no page flips: animate something (a tap) before measuring. The logcat
ring buffer is 256 KiB: capture while reproducing (`logcat -T 1 | grep ...`). A screenshot reads
the framebuffer, as `screencap` needs SurfaceFlinger: the commands are in the new-app skill,
step 8 ([#20](https://github.com/sdv-everyware/everyware/issues/20) publishes the reader).
