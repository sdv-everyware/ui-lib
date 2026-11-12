//! #slider in an app: a light's dimmer that follows the finger, sends the level to the light
//! when the finger is up, and shows the level the light reports. The docs show the parts
//! between the doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/slider.md.

// doc:note `frame` is where the design puts the slot: `let frame = sizes.frame("#doc/slider")` with `sizes = Sizes::load(<the .dcf>)` at the start, so the touch area follows the design.

use crate::prelude::*;
// doc:use
use everyware_ui::slider::{Slider, SliderEvent};
use everyware_ui::design::Sizes;
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_slider: Slider,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        let frame = sizes.frame("#doc/slider").unwrap_or(Frame { x: 0.0, y: 0.0, width: 243.0, height: 56.0 });
        State {
            // doc:init
            // at 65 %, the track where the design puts the slot (the finger's x is the display's)
            my_slider: Slider::in_slot(65.0, frame.x, frame.width),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_slider: Slot = "#doc/slider" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_slider = slot("#doc/slider", &state.my_slider);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        "#doc/slider" => match state.my_slider.on(gesture) {
            // the finger drags: the thumb follows it, the light waits
            Some(SliderEvent::Moving(_)) => {}
            // the finger is up: the light gets the level
            Some(SliderEvent::Changed(level)) => {
                if !send(Command::Level(level)) {
                    log::warn!("dimmer: the light is not there, nothing was sent");
                }
            }
            None => {}
        },
        // doc:end
        _ => {}
    }
}

// doc:change
/// The light reports its level (another panel or its own button changed it): the thumb moves
/// there. Not while a finger is on the slider, or the thumb would jump under it.
pub fn level_reported(state: &mut State, level: f32) {
    if !state.my_slider.dragging() {
        state.my_slider.value = level.clamp(0.0, 100.0);
    }
}
// doc:end

// doc:send
/// What the screen asks the light to do. An app has its own link to the device (smarthome-app:
/// a binder call to the Matter bundle); this one stands in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// The light's level, 0..100.
    Level(f32),
}

/// Hands the command to the device; false when it is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    log::info!("send {command:?}");
    true
}
// doc:end

#[test]
fn slider_follows_the_finger_and_sends_on_up() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!((state.my_slider.value, state.my_slider.variant().as_str()), (65.0, "state=default"));
    // the slot at x 0, 243 wide: the track from 20 to 223; a finger in its middle is 50 %
    let track = (state.my_slider.track_x, state.my_slider.track_width);
    assert_eq!(track, (20.0, 203.0));
    on_gesture(&mut state, "#doc/slider", Gesture::down(20.0 + 203.0 / 2.0, 0.0));
    assert_eq!((state.my_slider.value, state.my_slider.variant().as_str()), (50.0, "state=pressed"));
    // the finger leaves the slider to the right: the value stops at the end
    on_gesture(&mut state, "#doc/slider", Gesture::Move { x: 400.0, y: 0.0, inside: false });
    assert_eq!(state.my_slider.value, 100.0);
    // up at the quarter of the track: the level that goes to the light
    on_gesture(&mut state, "#doc/slider", Gesture::up(20.0 + 203.0 / 4.0, 0.0));
    assert_eq!((state.my_slider.value, state.my_slider.variant().as_str()), (25.0, "state=default"));
    present(&mut model, &state);
}

#[test]
fn slider_shows_the_level_the_light_reports() {
    let mut state = State::new(&Sizes::none());
    level_reported(&mut state, 30.0);
    assert_eq!(state.my_slider.value, 30.0);
    // a finger on the slider: the report does not move the thumb under it
    on_gesture(&mut state, "#doc/slider", Gesture::down(223.0, 0.0));
    level_reported(&mut state, 10.0);
    assert_eq!(state.my_slider.value, 100.0);
    // disabled: no gesture moves it
    on_gesture(&mut state, "#doc/slider", Gesture::Cancel);
    state.my_slider.press.set_enabled(false);
    on_gesture(&mut state, "#doc/slider", Gesture::down(20.0, 0.0));
    assert_eq!((state.my_slider.value, state.my_slider.variant().as_str()), (100.0, "state=disabled"));
    assert!(send(Command::Level(40.0)));
}
