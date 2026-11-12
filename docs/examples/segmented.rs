//! #segmented in an app: a fan's speed, Low, Med or High; a tap on a segment picks it and sends it
//! on, the app shows the speed the fan reported and turns the control off while the fan is gone.
//! The docs show the parts between the doc marks (docs/build-docs.js); the tests at the end
//! compile and run them (everyware_ui_recipes_test). Edit here: this file is the example of the
//! guide docs/content/segmented.md.

// doc:note The segments report with their slot, `#doc/segmented/#segmented/segment-2`; `state.my_segmented.value()` is the chosen label.

use crate::prelude::*;
// doc:use
use everyware_ui::segmented::{Segmented, SegmentedEvent};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_segmented: Segmented,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            // 2 to 4 labels, the first chosen; the number picks the variant (segments=3)
            my_segmented: Segmented::new(["Low", "Med", "High"]),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_segmented: Slot = "#doc/segmented" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_segmented = slot("#doc/segmented", &state.my_segmented);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // a segment reports with its slot: "#doc/segmented/#segmented/segment-2"
        _ if part_of("#doc/segmented", node).is_some() => {
            let segment = part_of("#doc/segmented", node).and_then(|p| p.strip_prefix("#segmented/segment-")).and_then(|n| n.parse::<usize>().ok());
            if let Some(SegmentedEvent::Selected(_)) = segment.and_then(|n| n.checked_sub(1)).and_then(|i| state.my_segmented.on(i, gesture)) {
                if !send(Command::Fan(state.my_segmented.value().to_string())) {
                    log::warn!("fan: the service is not there, nothing was sent");
                }
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app shows what the fan reported: its speed lights that segment (no tap, no event), and no
/// fan turns the control off (the variant state=disabled, the segments take no taps).
pub fn fan_reported(state: &mut State, speed: Option<&str>) {
    state.my_segmented.press.set_enabled(speed.is_some());
    if let Some(i) = speed.and_then(|s| state.my_segmented.labels.iter().position(|l| l == s)) {
        state.my_segmented.select(i);
    }
}
// doc:end

// doc:send
/// What the screen asks the fan to do. An app has its own link to the service (smarthome-app:
/// screen_binder::send_matter_command, a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Fan(String),
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
fn segmented_builds_a_model_and_takes_a_tap() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_segmented.variant(), "segments=3, state=default");
    // the second segment, as harry reports it with the slot
    tap(&mut state, "#doc/segmented/#segmented/segment-2");
    assert_eq!(state.my_segmented.value(), "Med");
    // the choice is light, not a variant: Med's highlight at 100, Low's at 0, over 160 ms
    assert_eq!((state.my_segmented.target(0), state.my_segmented.target(1)), (0.0, 100.0));
    assert_eq!(state.my_segmented.variant(), "segments=3, state=default");
    assert_eq!(state.my_segmented.value_transition().unwrap().duration.as_millis(), 160);
    present(&mut model, &state);
}

#[test]
fn segmented_follows_the_device() {
    let mut state = State::new(&Sizes::none());
    fan_reported(&mut state, Some("High"));
    assert_eq!(state.my_segmented.value(), "High");
    // the fan is gone: off, a tap picks nothing
    fan_reported(&mut state, None);
    assert_eq!(state.my_segmented.variant(), "segments=3, state=disabled");
    tap(&mut state, "#doc/segmented/#segmented/segment-1");
    assert_eq!(state.my_segmented.value(), "High");
    // back at Low
    fan_reported(&mut state, Some("Low"));
    assert_eq!((state.my_segmented.value(), state.my_segmented.variant().as_str()), ("Low", "segments=3, state=default"));
}
