//! #keyboard in an app: the keys that type into a text the app owns, with Shift, the symbols
//! and Done. The docs show the parts between the doc marks (docs/build-docs.js); the tests at
//! the end compile and run them (everyware_ui_recipes_test). Edit here: this file is the
//! example of the guide docs/content/keyboard.md.

// doc:note On a screen the keyboard sits in the keyboard dock, which slides it in while a field has the focus (docs/content/keyboard_dock.md); a `Form` sends its keys to the field in focus.

use crate::prelude::*;
// doc:use
use everyware_ui::keyboard::{Keyboard, Mode, Outcome};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The keys: the mode (letters, Shift, symbols) and which key is down.
    pub my_keyboard: Keyboard,
    /// What the keys type into: the keyboard keeps no text of its own.
    pub typed: String,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_keyboard: Default::default(), // the letters, no key down
            typed: String::new(),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_keyboard: Slot = "#doc/keyboard" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    // the labels of the mode, the pressed keys, a tap target per key
    model.my_keyboard = slot("#doc/keyboard", &state.my_keyboard);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // the keys report by their own names: #kbd/r1/3, #kbd/shift, #kbd/done
        _ if node.starts_with("#kbd/") => match state.my_keyboard.on(node, gesture, &mut state.typed) {
            Some(Outcome::Edited) => {} // a character, a space or a delete: the text changed
            Some(Outcome::Done) => {
                let text = std::mem::take(&mut state.typed);
                log::info!("{} characters typed", text.len()); // yours: hand it on
            }
            None => {} // the key is still down, or Shift or 123 changed the labels
        },
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app picks what the keys show for the next text: digits first for a PIN, the letters for
/// a name. The labels change at the next frame; Shift and 123 still switch.
pub fn start_text(state: &mut State, digits: bool) {
    state.my_keyboard.mode = if digits { Mode::Symbols } else { Mode::Lower };
    state.typed.clear();
}
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn keyboard_types_with_shift_and_delete() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    // a key shows pressed while the finger is on it, and types when it lifts
    on_gesture(&mut state, "#kbd/r1/1", Gesture::DOWN);
    assert_eq!((state.my_keyboard.pressed_target("#kbd/r1/1"), state.typed.as_str()), (100.0, ""));
    on_gesture(&mut state, "#kbd/r1/1", Gesture::UP);
    assert_eq!((state.my_keyboard.pressed_target("#kbd/r1/1"), state.typed.as_str()), (0.0, "q"));
    // Shift holds for one letter
    tap(&mut state, "#kbd/shift");
    assert_eq!(state.my_keyboard.label_of("#kbd/r1/2").as_deref(), Some("W"));
    tap(&mut state, "#kbd/r1/2");
    tap(&mut state, "#kbd/r1/3");
    tap(&mut state, "#kbd/space");
    assert_eq!(state.typed, "qWe ");
    tap(&mut state, "#kbd/delete");
    assert_eq!(state.typed, "qWe");
    present(&mut model, &state);
    // Done hands the text on and starts over
    tap(&mut state, "#kbd/done");
    assert_eq!(state.typed, "");
}

#[test]
fn keyboard_starts_with_digits_from_the_app() {
    let mut state = State::new(&Sizes::none());
    start_text(&mut state, true);
    assert_eq!((state.my_keyboard.label_of("#kbd/r1/1").as_deref(), state.my_keyboard.label_of("#kbd/mode").as_deref()), (Some("1"), Some("abc")));
    tap(&mut state, "#kbd/r1/4");
    tap(&mut state, "#kbd/r1/2");
    assert_eq!(state.typed, "42");
    // abc: back to the letters
    tap(&mut state, "#kbd/mode");
    assert_eq!(state.my_keyboard.label_of("#kbd/r1/1").as_deref(), Some("q"));
}
