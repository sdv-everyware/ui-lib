//! #toggle in an app: a light switch that a tap flips and sends on to the light, and that follows
//! what the light reports. The docs show the parts between the doc marks (docs/build-docs.js); the
//! tests at the end compile and run them (everyware_ui_recipes_test). Edit here: this file is the
//! example of the guide docs/content/toggle.md.

// doc:note Read the value any time as `state.my_toggle.value` (a pub field); the device's value comes in with `set`, never with a gesture.

use crate::prelude::*;
// doc:use
use everyware_ui::toggle::{Toggle, ToggleEvent};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_toggle: Toggle,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_toggle: Toggle::new(false),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_toggle: Slot = "#doc/toggle" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_toggle = slot("#doc/toggle", &state.my_toggle);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        "#doc/toggle" => {
            if let Some(ToggleEvent::Toggled(on)) = state.my_toggle.on(gesture) {
                // the switch moved already; the light follows when the command arrives
                if !send(Command::Light(on)) {
                    log::warn!("light: the service is not there, nothing was sent");
                }
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app shows what the light reported: its value moves the thumb (no tap, no event), and no
/// light (gone from the network) turns the switch off: the variant state=disabled, no taps.
pub fn light_reported(state: &mut State, on: Option<bool>) {
    match on {
        Some(on) => {
            state.my_toggle.set(on);
            state.my_toggle.press.set_enabled(true);
        }
        None => state.my_toggle.press.set_enabled(false),
    }
}
// doc:end

// doc:send
/// What the screen asks the light to do. An app has its own link to the service (smarthome-app:
/// screen_binder::send_matter_command, a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Light(bool),
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn toggle_builds_a_model_and_takes_a_tap() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!((state.my_toggle.variant().as_str(), state.my_toggle.target()), ("state=default", 0.0));
    // down: the thumb stretches (the variant state=pressed), the value stays
    on_gesture(&mut state, "#doc/toggle", Gesture::DOWN);
    assert_eq!((state.my_toggle.variant().as_str(), state.my_toggle.value), ("state=pressed", false));
    // up: on, the thumb goes to the right and the track fades in over 200 ms
    on_gesture(&mut state, "#doc/toggle", Gesture::UP);
    assert_eq!((state.my_toggle.variant().as_str(), state.my_toggle.value, state.my_toggle.target()), ("state=default", true, 100.0));
    assert_eq!(state.my_toggle.value_transition().unwrap().duration.as_millis(), 200);
    present(&mut model, &state);
}

#[test]
fn toggle_follows_the_device() {
    let mut state = State::new(&Sizes::none());
    tap(&mut state, "#doc/toggle");
    // the light reports off (someone used the wall switch): the thumb goes back, no event
    light_reported(&mut state, Some(false));
    assert_eq!((state.my_toggle.value, state.my_toggle.target()), (false, 0.0));
    // the light is gone: off, a tap changes nothing
    light_reported(&mut state, None);
    assert_eq!(state.my_toggle.variant(), "state=disabled");
    tap(&mut state, "#doc/toggle");
    assert!(!state.my_toggle.value);
    // it is back, and on
    light_reported(&mut state, Some(true));
    assert_eq!((state.my_toggle.variant().as_str(), state.my_toggle.value), ("state=default", true));
}
