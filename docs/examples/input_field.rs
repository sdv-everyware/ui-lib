//! #input-field in an app: a password field that a tap focuses, the screen's keyboard types into,
//! and Enter hands to the app. The docs show the parts between the doc marks (docs/build-docs.js);
//! the tests at the end compile and run them (everyware_ui_recipes_test). Edit here: this file is
//! the example of the guide docs/content/input_field.md.

// doc:note The field alone answers only `Focused`; the screen's keyboard (`KeyboardDock`) types into it. In a sheet with several fields a `Form` does this routing for you (docs/content/form.md).

use crate::prelude::*;
// doc:use
use everyware_ui::dock::KeyboardDock;
use everyware_ui::input_field::{InputEvent, InputField};
use everyware_ui::keyboard::Outcome;
// doc:end

/// The screen's size, for the keyboard's place at its bottom edge.
const SCREEN: (f32, f32) = (1920.0, 1080.0);

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_input_field: InputField,
    /// The screen's one keyboard: open while a field has the focus.
    pub osk: KeyboardDock,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_input_field: {
                let mut field = InputField::new("Password", "At least 8 characters");
                field.secure = true; // dots instead of the characters
                field
            },
            osk: KeyboardDock::new(),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_input_field: Slot = "#doc/input_field" ["SlotCustomizer"];
    // the keyboard: its keys, and its place (it slides in from the bottom edge)
    osk_keys: Slot = "#osk/keys" ["SlotCustomizer"];
    osk_keys_position: Position = "#osk/keys" ["FollowPositionCustomizer"] = Position::new(KeyboardDock::new().keys_position(SCREEN), None);
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_input_field = slot("#doc/input_field", &state.my_input_field);
    model.osk_keys = slot("#osk/keys", &state.osk.keyboard);
    model.osk_keys_position = Position::new(state.osk.keys_position(SCREEN), state.osk.transition());
    // doc:end
}

// actions.rs: the arms of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // a tap on the field: it takes the focus, the keyboard opens
        "#doc/input_field" => {
            if let Some(InputEvent::Focused) = state.my_input_field.on(gesture, Instant::now()) {
                state.osk.follow(true);
            }
        }
        // a key of the keyboard: it types into the field in focus
        _ if node.starts_with("#kbd/") && state.my_input_field.focused => {
            match state.osk.keyboard.on(node, gesture, &mut state.my_input_field.text) {
                Some(Outcome::Edited) => state.my_input_field.edited(Instant::now()),
                Some(Outcome::Done) => {
                    // Enter: the text is complete, the keyboard closes
                    state.my_input_field.blur();
                    state.osk.follow(false);
                    let password = state.my_input_field.text.clone();
                    log::info!("password of {} characters entered", password.len()); // yours: connect with it
                }
                None => {}
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app shows what went wrong under the field (the variant state=error, in red) and clears
/// it for the next try; the error goes again when the user types.
pub fn wrong_password(state: &mut State) {
    state.my_input_field.error = Some("Wrong password".into());
    state.my_input_field.text.clear();
}
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn input_field_builds_a_model_and_takes_a_tap() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    // a tap focuses the field and opens the keyboard
    tap(&mut state, "#doc/input_field");
    assert!(state.my_input_field.focused && state.osk.open());
    // q w e on the first row, then Done
    for key in ["#kbd/r1/1", "#kbd/r1/2", "#kbd/r1/3"] {
        tap(&mut state, key);
    }
    assert_eq!((state.my_input_field.text.as_str(), state.my_input_field.shown().as_str()), ("qwe", "•••"));
    tap(&mut state, "#kbd/done");
    assert!(!state.my_input_field.focused && !state.osk.open());
    present(&mut model, &state);
}

#[test]
fn input_field_shows_an_error_from_the_app() {
    let mut state = State::new(&Sizes::none());
    wrong_password(&mut state);
    assert_eq!(state.my_input_field.variant(), "state=error");
    assert_eq!(state.my_input_field.helper_line(), "Wrong password");
    // a key without the focus types nothing
    tap(&mut state, "#kbd/r1/1");
    assert_eq!(state.my_input_field.text, "");
    // the next try: the first key takes the error away
    tap(&mut state, "#doc/input_field");
    tap(&mut state, "#kbd/r1/1");
    assert_eq!((state.my_input_field.error.as_deref(), state.my_input_field.variant().as_str()), (None, "state=default"));
}
