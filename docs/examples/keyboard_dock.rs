//! #keyboard-dock in an app: the screen's one keyboard, which slides in while a field has the
//! focus and out when none has. The docs show the parts between the doc marks
//! (docs/build-docs.js); the tests at the end compile and run them (everyware_ui_recipes_test).
//! Edit here: this file is the example of the guide docs/content/keyboard_dock.md.

// doc:note One dock per screen for every field on it; with several forms, follow the focus of all of them: `follow(a.focused().is_some() || b.focused().is_some())`.

use crate::prelude::*;
// doc:use
use everyware_ui::dock::{KeyboardDock, KEYBOARD_DOCK_H};
use everyware_ui::input_field::InputField;
use everyware_ui::keyboard::Outcome;
// doc:end

/// The screen's size: the keys dock at its bottom edge.
const SCREEN: (f32, f32) = (1920.0, 1080.0);

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The screen's one keyboard: closed below the screen, or docked at the bottom.
    pub my_keyboard_dock: KeyboardDock,
    /// A field it types into (yours: every field of the screen).
    pub password: InputField,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_keyboard_dock: KeyboardDock::new(),
            password: InputField::new("Password", "At least 8 characters"),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    // the keys in the dock's slot, and the slot's place: it moves, the dock's variant never changes
    keys: Slot = "#doc/keyboard_dock" ["SlotCustomizer"];
    keys_position: Position = "#doc/keyboard_dock" ["FollowPositionCustomizer"] = Position::new(KeyboardDock::new().keys_position(SCREEN), None);
    password: Slot = "#wifi/password" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.keys = slot("#doc/keyboard_dock", &state.my_keyboard_dock.keyboard);
    model.keys_position = Position::new(state.my_keyboard_dock.keys_position(SCREEN), state.my_keyboard_dock.transition());
    model.password = slot("#wifi/password", &state.password);
    // doc:end
}

// actions.rs: the match on the gesture's node name, and the rule after it
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    // doc:act
    match node {
        // a tap on the field gives it the focus
        "#wifi/password" => {
            state.password.on(gesture, Instant::now());
        }
        // a key types into the field in focus; Enter takes the focus away
        _ if node.starts_with("#kbd/") && state.password.focused => {
            match state.my_keyboard_dock.keyboard.on(node, gesture, &mut state.password.text) {
                Some(Outcome::Edited) => state.password.edited(Instant::now()),
                Some(Outcome::Done) => state.password.blur(),
                None => {}
            }
        }
        _ => {}
    }
    // after every action: the dock is open while a field has the focus
    state.my_keyboard_dock.follow(state.password.focused);
    // doc:end
}

// doc:change
/// The app closes the keyboard without a tap (the screen goes to sleep, the sheet with the
/// field closes): it takes the focus away and the dock follows.
pub fn screen_off(state: &mut State) {
    state.password.blur();
    state.my_keyboard_dock.follow(false);
}

/// The height a dialog has while the dock is open: the screen above the keys.
pub fn free_height(state: &State) -> f32 {
    if state.my_keyboard_dock.open() { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 }
}
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn keyboard_dock_follows_the_focus() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    // closed: the keys wait 40 px below the screen, as the design draws them
    assert_eq!((state.my_keyboard_dock.variant().as_str(), model.keys_position.x, model.keys_position.y), ("state=closed", 0.0, 1120.0));
    // a key without a focused field types nothing and opens nothing
    tap(&mut state, "#kbd/r1/1");
    assert_eq!((state.password.text.as_str(), state.my_keyboard_dock.open()), ("", false));
    // the focus opens it: 420 high at the bottom, with the open spring
    tap(&mut state, "#wifi/password");
    present(&mut model, &state);
    assert_eq!((state.my_keyboard_dock.variant().as_str(), model.keys_position.y, free_height(&state)), ("state=open", 660.0, 660.0));
    assert_eq!(model.keys_position.transition.map(|t| t.duration.as_millis()), Some(550));
    tap(&mut state, "#kbd/r2/1");
    assert_eq!(state.password.text, "a");
    // Enter: the field loses the focus, the dock slides out with the close spring
    tap(&mut state, "#kbd/done");
    present(&mut model, &state);
    assert_eq!((state.my_keyboard_dock.variant().as_str(), model.keys_position.y, free_height(&state)), ("state=closed", 1120.0, 1080.0));
    assert_eq!(model.keys_position.transition.map(|t| t.duration.as_millis()), Some(400));
}

#[test]
fn keyboard_dock_closes_from_the_app() {
    let mut state = State::new(&Sizes::none());
    tap(&mut state, "#wifi/password");
    assert!(state.my_keyboard_dock.open());
    screen_off(&mut state);
    assert_eq!((state.password.focused, state.my_keyboard_dock.variant().as_str()), (false, "state=closed"));
}
