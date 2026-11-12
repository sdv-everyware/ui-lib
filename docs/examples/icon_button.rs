//! #icon-button in an app: a minus button that lowers the heating's setpoint by one degree, sends
//! it on to a service, and goes off at the lowest setpoint. The docs show the parts between the
//! doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/icon_button.md.

// doc:note The glyph is part of the variant: pick it once in `IconButton::new`, the button answers only `Clicked`.

use crate::prelude::*;
// doc:use
use everyware_ui::button::{ButtonEvent, Icon, IconButton, IconButtonKind};
// doc:end

/// The setpoints the heating takes, in degrees.
const SETPOINT: std::ops::RangeInclusive<i32> = 16..=28;

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_icon_button: IconButton,
    /// What the button changes: the heating's setpoint in degrees.
    pub setpoint: i32,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_icon_button: IconButton::new(IconButtonKind::Tonal, Icon::Minus),
            setpoint: 21,
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_icon_button: Slot = "#doc/icon_button" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_icon_button = slot("#doc/icon_button", &state.my_icon_button);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        "#doc/icon_button" => {
            if let Some(ButtonEvent::Clicked) = state.my_icon_button.on(gesture) {
                // one degree down, sent on; at the lowest setpoint the button goes off
                setpoint_reported(state, state.setpoint - 1);
                if !send(Command::Setpoint(state.setpoint)) {
                    log::warn!("setpoint: the service is not there, nothing was sent");
                }
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app sets the setpoint from its own state (a tap, or the device reported one) and the
/// button follows: off at the lowest setpoint (the variant state=disabled, which takes no taps).
pub fn setpoint_reported(state: &mut State, setpoint: i32) {
    state.setpoint = setpoint.clamp(*SETPOINT.start(), *SETPOINT.end());
    state.my_icon_button.press.set_enabled(state.setpoint > *SETPOINT.start());
}
// doc:end

// doc:send
/// What the screen asks the heating to do. An app has its own link to the service
/// (smarthome-app: screen_binder, a binder call); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Setpoint(i32),
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
fn icon_button_builds_a_model_and_takes_a_tap() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_icon_button.variant(), "kind=tonal, state=default, icon=minus, size=regular");
    // down: the pressed layer fades in, the variant stays
    state.my_icon_button.on(Gesture::DOWN);
    assert_eq!(state.my_icon_button.press.pressed_target(), 100.0);
    state.my_icon_button.on(Gesture::Cancel);
    // a tap lowers the setpoint by one
    tap(&mut state, "#doc/icon_button");
    assert_eq!(state.setpoint, 20);
    present(&mut model, &state);
}

#[test]
fn icon_button_goes_off_at_the_lowest_setpoint() {
    let mut state = State::new(&Sizes::none());
    setpoint_reported(&mut state, 17);
    tap(&mut state, "#doc/icon_button");
    assert_eq!(state.setpoint, 16);
    assert_eq!(state.my_icon_button.variant(), "kind=tonal, state=disabled, icon=minus, size=regular");
    // off: a tap does nothing
    tap(&mut state, "#doc/icon_button");
    assert_eq!(state.setpoint, 16);
    // the device reports a higher setpoint: the button is on again
    setpoint_reported(&mut state, 22);
    assert_eq!(state.my_icon_button.variant(), "kind=tonal, state=default, icon=minus, size=regular");
}
