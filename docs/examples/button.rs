//! #button in an app: a Talk button that a tap sends on to a service, and that the app relabels
//! and turns off from its own state. The docs show the parts between the doc marks
//! (docs/build-docs.js); the tests at the end compile and run them (everyware_ui_recipes_test).
//! Edit here: this file is the example of the guide docs/content/button.md and of
//! docs/content/getting-started.md.

// doc:note The button answers only `Clicked`; what the tap means (a command to a service) is the app's.

use crate::prelude::*;
// doc:use
use everyware_ui::button::{Button, ButtonEvent, ButtonKind};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_button: Button,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_button: Button::new(ButtonKind::Primary, "Talk").compact(),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_button: Slot = "#doc/button" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_button = slot("#doc/button", &state.my_button);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        "#doc/button" => {
            if let Some(ButtonEvent::Clicked) = state.my_button.on(gesture) {
                // the tap is done: send it on to the service that acts on it
                if !send(Command::Talk) {
                    log::warn!("talk: the service is not there, nothing was sent");
                }
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app changes the button from its own state, outside a tap: a new label, or off while it
/// cannot act (no service). The presenter shows it at the next frame: the label as text, off as
/// the variant state=disabled, which takes no taps.
pub fn on_call(state: &mut State, in_call: bool, service_up: bool) {
    state.my_button.label = if in_call { "End".into() } else { "Talk".into() };
    state.my_button.press.set_enabled(service_up);
}
// doc:end

// doc:send
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
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn button_builds_a_model_and_takes_a_tap() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_button.variant(), "kind=primary, state=default, size=compact");
    // the finger is down: the pressed layer fades in, the variant stays
    assert_eq!(state.my_button.on(Gesture::DOWN), None);
    assert_eq!((state.my_button.press.pressed_target(), state.my_button.variant().as_str()), (100.0, "kind=primary, state=default, size=compact"));
    // up over it: the tap is done, the layer fades out
    assert_eq!(state.my_button.on(Gesture::UP), Some(ButtonEvent::Clicked));
    assert_eq!(state.my_button.press.pressed_target(), 0.0);
    // the same through the app's arm, which sends Talk on
    tap(&mut state, "#doc/button");
    assert!(send(Command::Talk));
    present(&mut model, &state);
}

#[test]
fn button_takes_no_tap_when_the_finger_leaves() {
    let mut state = State::new(&Sizes::none());
    assert_eq!(state.my_button.on(Gesture::DOWN), None);
    // the finger slides off: the press is undone, coming back does not restore it
    assert_eq!(state.my_button.on(Gesture::Move { x: 300.0, y: 0.0, inside: false }), None);
    assert_eq!(state.my_button.press.pressed_target(), 0.0);
    assert_eq!(state.my_button.on(Gesture::UP), None);
}

#[test]
fn button_changes_from_the_app_and_takes_no_tap_while_off() {
    let mut state = State::new(&Sizes::none());
    on_call(&mut state, true, false);
    assert_eq!(state.my_button.label, "End");
    assert_eq!(state.my_button.variant(), "kind=primary, state=disabled, size=compact");
    assert_eq!(state.my_button.on(Gesture::DOWN), None);
    assert_eq!(state.my_button.on(Gesture::UP), None);
    // the service is back, the call is over
    on_call(&mut state, false, true);
    assert_eq!((state.my_button.label.as_str(), state.my_button.variant().as_str()), ("Talk", "kind=primary, state=default, size=compact"));
    tap(&mut state, "#doc/button");
}
