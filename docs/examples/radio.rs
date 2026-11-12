//! #radio in an app: three heating modes, one on, a tap on another picks it and sends it on; the
//! app shows the mode the heating reported and turns off a mode it cannot take. The docs show the
//! parts between the doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/radio.md.

// doc:note One slot per option, `#doc/radio/1`, `#doc/radio/2` ...; the group decides which is on.

use crate::prelude::*;
// doc:use
use everyware_ui::radio::{RadioEvent, RadioGroup};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_radio: RadioGroup,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            // the labels are drawn in each option (#radio/label); the first is on
            my_radio: RadioGroup::new(["Off", "Eco", "Comfort"]),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_radio_1: Slot = "#doc/radio/1" ["SlotCustomizer"];
    my_radio_2: Slot = "#doc/radio/2" ["SlotCustomizer"];
    my_radio_3: Slot = "#doc/radio/3" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_radio_1 = slot("#doc/radio/1", &state.my_radio.options[0]);
    model.my_radio_2 = slot("#doc/radio/2", &state.my_radio.options[1]);
    model.my_radio_3 = slot("#doc/radio/3", &state.my_radio.options[2]);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        _ if node.starts_with("#doc/radio/") => {
            // the option from its slot's name: #doc/radio/1 is index 0
            if let Some(i) = row_of("#doc/radio", node).and_then(|n| n.checked_sub(1)) {
                if let Some(RadioEvent::Selected(i)) = state.my_radio.on(i, gesture) {
                    let mode = state.my_radio.options[i].label.clone();
                    if !send(Command::Mode(mode)) {
                        log::warn!("mode: the service is not there, nothing was sent");
                    }
                }
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app shows what the heating reported: the mode it is in is on, the others off (no tap, no
/// event), and a mode it cannot take now is off (the variant state=disabled, no taps).
pub fn heating_reported(state: &mut State, mode: usize, can_comfort: bool) {
    for (i, option) in state.my_radio.options.iter_mut().enumerate() {
        option.on = i == mode;
    }
    state.my_radio.options[2].press.set_enabled(can_comfort);
}
// doc:end

// doc:send
/// What the screen asks the heating to do. An app has its own link to the service
/// (smarthome-app: screen_binder, a binder call); this one stands in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Mode(String),
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

#[cfg(test)]
fn variants(state: &State) -> Vec<String> {
    state.my_radio.options.iter().map(|o| o.variant()).collect()
}

#[test]
fn radio_builds_a_model_and_takes_a_tap() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_radio.selected(), Some(0));
    // down on Eco: that option shows its halo (state=pressed)
    on_gesture(&mut state, "#doc/radio/2", Gesture::DOWN);
    assert_eq!(state.my_radio.options[1].variant(), "value=off, state=pressed");
    // up: Eco is on, Off is off
    on_gesture(&mut state, "#doc/radio/2", Gesture::UP);
    assert_eq!(state.my_radio.selected(), Some(1));
    assert_eq!(variants(&state), ["value=off, state=default", "value=on, state=default", "value=off, state=default"]);
    // a tap on the option that is on changes nothing
    assert_eq!(state.my_radio.on(1, Gesture::DOWN), None);
    assert_eq!(state.my_radio.on(1, Gesture::UP), None);
    present(&mut model, &state);
}

#[test]
fn radio_follows_the_device() {
    let mut state = State::new(&Sizes::none());
    heating_reported(&mut state, 1, false);
    assert_eq!(variants(&state), ["value=off, state=default", "value=on, state=default", "value=off, state=disabled"]);
    // Comfort is off: a tap does not pick it
    tap(&mut state, "#doc/radio/3");
    assert_eq!(state.my_radio.selected(), Some(1));
    // the heating can take it again
    heating_reported(&mut state, 1, true);
    tap(&mut state, "#doc/radio/3");
    assert_eq!(state.my_radio.selected(), Some(2));
}
