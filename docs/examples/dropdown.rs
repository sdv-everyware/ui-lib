//! #dropdown in an app: the house's mode (At home, Away, Asleep) in a pill that a tap opens, a
//! list the user picks from, and the mode the house reports shown from the app. The docs show
//! the parts between the doc marks (docs/build-docs.js); the tests at the end compile and run
//! them (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/dropdown.md.

// doc:note The pill opens and closes the list (`on_field`); an item reports by its own name (`#dropdown/item/2`), so one list is open on the screen at a time.

use crate::prelude::*;
// doc:use
use everyware_ui::dropdown::{Dropdown, DropdownEvent};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_dropdown: Dropdown,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_dropdown: Dropdown::new(["At home", "Away", "Asleep"]),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_dropdown: Slot = "#doc/dropdown" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_dropdown = slot("#doc/dropdown", &state.my_dropdown);
    // doc:end
}

// actions.rs: the arms of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // a tap on the pill opens the list, the next one closes it
        "#doc/dropdown" => state.my_dropdown.on_field(gesture),
        // an item of the open list, by its name in the component: #dropdown/item/1, /2, /3
        _ if node.starts_with("#dropdown/item/") => {
            let n = node["#dropdown/item/".len()..].parse::<usize>().ok().and_then(|n| n.checked_sub(1));
            if let Some(DropdownEvent::Selected(i)) = n.and_then(|n| state.my_dropdown.on_item(n, gesture)) {
                // the list is closed and the pill shows the item
                log::info!("mode {}", state.my_dropdown.items[i]); // yours: tell the house
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The house reports its mode (another panel, a schedule): the pill shows it. Not while the
/// list is open, the user is choosing; a name that is not an item changes nothing.
pub fn mode_reported(state: &mut State, mode: &str) {
    if let Some(i) = state.my_dropdown.items.iter().position(|item| item == mode) {
        if !state.my_dropdown.open {
            state.my_dropdown.selected = i;
        }
    }
}
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn dropdown_opens_and_takes_a_choice() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!((state.my_dropdown.value(), state.my_dropdown.list_target()), ("At home", 0.0));
    // a tap on the closed list does nothing: it is transparent
    tap(&mut state, "#dropdown/item/2");
    assert_eq!(state.my_dropdown.value(), "At home");
    // a tap on the pill opens the list; the variant stays, the list fades in
    tap(&mut state, "#doc/dropdown");
    assert_eq!((state.my_dropdown.variant().as_str(), state.my_dropdown.list_target()), ("state=default", 100.0));
    // the second item: chosen, marked, and the list closes
    tap(&mut state, "#dropdown/item/2");
    assert_eq!((state.my_dropdown.value(), state.my_dropdown.mark_target(1), state.my_dropdown.list_target()), ("Away", 100.0, 0.0));
    present(&mut model, &state);
}

#[test]
fn dropdown_shows_the_mode_the_house_reports() {
    let mut state = State::new(&Sizes::none());
    mode_reported(&mut state, "Asleep");
    assert_eq!(state.my_dropdown.value(), "Asleep");
    mode_reported(&mut state, "On holiday");
    assert_eq!(state.my_dropdown.value(), "Asleep");
    // while the user has the list open, the report waits
    tap(&mut state, "#doc/dropdown");
    mode_reported(&mut state, "At home");
    assert_eq!(state.my_dropdown.value(), "Asleep");
    // disabled: the pill does not open
    state.my_dropdown.close();
    state.my_dropdown.press.set_enabled(false);
    tap(&mut state, "#doc/dropdown");
    assert_eq!((state.my_dropdown.variant().as_str(), state.my_dropdown.open), ("state=disabled", false));
}
