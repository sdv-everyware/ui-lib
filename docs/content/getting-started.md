---
title: Getting started
role: From nothing to a button on the screen of your own app, step by step: what each piece is and why it is there.
---
An EveryWare app is one harry process: a design (a `.dcf` made from a Figma file) and a little Rust that decides what the screen shows and what a tap does. This page puts one button on the screen of a new app and sends its tap on to a service. The code is the button's example, `docs/examples/button.rs`, which a test compiles and runs.

```
 finger ──► harry finds the node (#intercom/talk) ──► the listener: a Gesture ──► actions.rs: the button decides
                                                                                   │ ButtonEvent::Clicked
 pixels ◄── harry draws the variant ◄── ui_model.rs ◄── presenter.rs ◄── state.rs ◄┘──► send(Command::Talk)
```

## 1. A new app
**What.** `scripts/new-app.sh kitchen` copies template-app into `packages/services/display_safety/utils/kitchen-app` and renames it: a screen with the time, a title and a button that counts.

**Why.** Every app needs the same frame around its own code: the display, the fonts, the touch, the listener that turns touches into actions. The template has it, so you only write what your screen does.

| Keep as it is | Change |
|---|---|
| `src/main.rs`, `config.rs`, `clock.rs`, `resource_loader.rs`, `src/ui_event_listener.rs` | `design/<name>-patch.js` (the screen), `src/state.rs`, `src/ui_model.rs`, `src/presenter.rs`, `src/actions.rs` |

## 2. The button in the design
**What.** A frame named `#intercom/talk`, as large as the button's variant, with an instance of the library's `#button` inside (Figma, or a slot in the app's design patch).

**Why.** The name is the contract between the design and the code: harry finds the node by it, the app writes to it by it. The instance is only a preview: the app picks the variant, the texts are the app's.

## 3. state.rs: the button lives in the state
**What.** One `Button` value in the app's `State`, made once in `State::new`.

**Why.** The button keeps what happens to it (pressed, enabled, its label). The state is the one place that knows: the screen is drawn from it, a test can build it and tap it without a screen.

<!-- example: docs/examples/button.rs use slot=#intercom/talk field=talk -->
```rust
use everyware_ui::button::{Button, ButtonEvent, ButtonKind};
```

<!-- example: docs/examples/button.rs state slot=#intercom/talk field=talk -->
```rust
pub talk: Button,
```

<!-- example: docs/examples/button.rs init slot=#intercom/talk field=talk -->
```rust
talk: Button::new(ButtonKind::Primary, "Talk").compact(),
```

## 4. ui_model.rs: the node the code writes
**What.** One field per `#name` the code touches, here the slot of the button.

**Why.** The UI model is what harry reads to draw. A `Slot` tells it which variant of `#button` to clone into the frame and which texts and opacities to set.

<!-- example: docs/examples/button.rs model slot=#intercom/talk field=talk -->
```rust
talk: Slot = "#intercom/talk" ["SlotCustomizer"];
```

## 5. presenter.rs: from the state to the screen
**What.** After every action, one line writes the button into its slot.

**Why.** The screen never changes by itself: the presenter turns the state into the UI model, so what you see is always the state. The button chooses its variant (`kind=primary, state=default, size=compact`) and its transition.

<!-- example: docs/examples/button.rs present slot=#intercom/talk field=talk -->
```rust
model.talk = slot("#intercom/talk", &state.talk);
```

## 6. actions.rs: the tap
**What.** An arm of the match on the node's name hands the gesture to the button. The button answers `Some(ButtonEvent::Clicked)` when a down and an up land on it.

**Why.** The library decides what a tap is (pressed while down, nothing when the finger leaves, nothing while disabled), the app decides what it means. The listener that turns harry's touches into gestures is the template's `src/ui_event_listener.rs`, written once per app.

<!-- example: docs/examples/button.rs act slot=#intercom/talk field=talk -->
```rust
"#intercom/talk" => {
    if let Some(ButtonEvent::Clicked) = state.talk.on(gesture) {
        // the tap is done: send it on to the service that acts on it
        if !send(Command::Talk) {
            log::warn!("talk: the service is not there, nothing was sent");
        }
    }
}
```

## 7. Sending it on
**What.** On the click, a command goes to the service that acts on it.

**Why.** The screen does not do the work: a service does (a bundle of the device). Each app has its own link to its services; smarthome-app's is `screen_binder`, a binder call. Check the answer: a service can be gone.

<!-- example: docs/examples/button.rs send slot=#intercom/talk field=talk -->
```rust
/// What the screen asks a service to do. An app has its own link to the service (smarthome-app:
/// screen_binder::send_matter_command, a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Talk,
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
```

## 8. Changing it from the app
**What.** The app changes the button from its own state, without a tap: another label, or off.

**Why.** The button follows what the app knows (a call started, the service went away). Off is the variant `state=disabled`, and a disabled button takes no taps.

<!-- example: docs/examples/button.rs change slot=#intercom/talk field=talk -->
```rust
/// The app changes the button from its own state, outside a tap: a new label, or off while it
/// cannot act (no service). The presenter shows it at the next frame: the label as text, off as
/// the variant state=disabled, which takes no taps.
pub fn on_call(state: &mut State, in_call: bool, service_up: bool) {
    state.talk.label = if in_call { "End".into() } else { "Talk".into() };
    state.talk.press.set_enabled(service_up);
}
```

## 9. Build and run
**What.** Build the app, put it and its design on the board, start it in place of the panel. From the root of the tree, the board on adb (details: `skills/new-app/SKILL.md`, step 8):

```bash
# build: the app for the board (env.sh is bash; from zsh run it under bash)
bash -c 'source device/rb3/sdv_media_db845c/tools/env.sh >/dev/null 2>&1; m kitchen_app'

# root: stopping a service and taking the display need it
adb root

# who draws now: one app at a time holds the display
adb shell 'for p in /proc/[0-9]*; do ls -l $p/fd 2>/dev/null | grep -q card0 && cat $p/cmdline && echo; done'
# stop the panel (an init service: it comes back with start smarthome or a reboot)
adb shell stop smarthome

# the app and its design to the board
adb shell mkdir -p /data/local/tmp/kitchen
adb push out_rb3/target/product/sdv_media_db845c/system/bin/kitchen_app /data/local/tmp/kitchen/
adb push packages/services/display_safety/utils/kitchen-app/data/kitchen.dcf /data/local/tmp/kitchen/

# start it (its own output in kitchen.log, its log in logcat)
adb shell "cd /data/local/tmp/kitchen; LD_LIBRARY_PATH=/system/lib64:/vendor/lib64:/apex/com.everyware.smarthome/lib64     nohup ./kitchen_app --data-path /data/local/tmp/kitchen --touch-size 1920x1080 > kitchen.log 2>&1 &"
adb logcat -s kitchen-app

# back to the panel
adb shell 'kill $(pidof kitchen_app); start smarthome'
```

**Why.** The app runs where the panel runs and takes the display while it runs, so the panel stops first and starts again after. You should see the button; a tap presses it, and logcat shows `send Talk`.

## Next
- the guide of a component: `docs/content/<component>.md`, its code: `docs/examples/<component>.rs`
- the rules every app follows: `docs/content/rules.md`
- a whole new app with an agent: `skills/new-app/SKILL.md`
