//! Overlay in an app: the closed or open state under every sheet and the keyboard dock, used
//! here by itself to slide the right panel in and fade its scrim. The docs show the parts
//! between the doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/overlay.md.

// doc:note For a card over the screen use a `Sheet` (docs/content/sheet.md): it is this overlay with the card's places worked out for every layout.

use crate::prelude::*;
// doc:use
use everyware_ui::overlay::Overlay;
// doc:end

/// The screen's size.
const SCREEN: (f32, f32) = (1920.0, 1080.0);
/// The panel's x while open (half the screen) and while closed (40 px past the right edge).
const OPEN_X: f32 = 960.0;
const CLOSED_X: f32 = 1960.0;

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The panel over the screen: closed or open, and the transition of the last change.
    pub my_overlay: Overlay,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_overlay: Overlay::new(),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    // the card moves, the scrim fades and takes taps while open
    card: Position = "#doc/overlay/card" ["FollowPositionCustomizer"] = Position::new((CLOSED_X, 0.0), None);
    scrim: Meter = "#doc/overlay/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
    scrim_taps: TapTarget = "#doc/overlay/scrim" ["TapTargetCustomizer"];
    // a tap on the card stays on the card
    _card_tap: () = "#doc/overlay/card" interactive;
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    // both parts move with the overlay's transition: 550 ms in, 400 ms out, a spring
    let slide = state.my_overlay.transition();
    model.card = Position::new((state.my_overlay.slide(OPEN_X, CLOSED_X), 0.0), slide);
    model.scrim = Meter::new(state.my_overlay.slide(35.0, 0.0), slide);
    model.scrim_taps = TapTarget(state.my_overlay.open);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // a tap on the scrim closes it; the finger has to lift on the scrim
        "#doc/overlay/scrim" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.my_overlay.close(),
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app opens and closes it from what happened: a story to read, the page is gone. Open
/// twice is open once: the transition stays the one of the first change.
pub fn show_page(state: &mut State, page: Option<&str>) {
    match page {
        Some(_) => state.my_overlay.open(),
        None => state.my_overlay.close(),
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
fn overlay_slides_the_card_in_and_out() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!((state.my_overlay.variant().as_str(), model.card.x, model.scrim.target), ("state=closed", CLOSED_X, 0.0));
    // the app opens it: the card comes in from the right, the scrim darkens
    show_page(&mut state, Some("https://example.org"));
    present(&mut model, &state);
    assert_eq!((state.my_overlay.variant().as_str(), model.card.x, model.scrim.target), ("state=open", OPEN_X, 35.0));
    assert_eq!(model.card.transition.map(|t| t.duration.as_millis()), Some(550));
    assert!(model.scrim_taps.0, "the open scrim takes taps");
    // a tap on the scrim closes it with the close transition
    tap(&mut state, "#doc/overlay/scrim");
    present(&mut model, &state);
    assert_eq!((state.my_overlay.variant().as_str(), model.card.x, model.scrim.target), ("state=closed", CLOSED_X, 0.0));
    assert_eq!(model.scrim.transition.map(|t| t.duration.as_millis()), Some(400));
    assert!(!model.scrim_taps.0, "the closed scrim lets taps through to the screen");
}

#[test]
fn overlay_ignores_a_finger_that_leaves_the_scrim() {
    let mut state = State::new(&Sizes::none());
    show_page(&mut state, Some("https://example.org"));
    on_gesture(&mut state, "#doc/overlay/scrim", Gesture::DOWN);
    on_gesture(&mut state, "#doc/overlay/scrim", Gesture::Up { x: 0.0, y: 0.0, inside: false, velocity: 0.0, velocity_x: 0.0 });
    assert!(state.my_overlay.open);
    // the page is gone: the app closes it
    show_page(&mut state, None);
    assert_eq!(state.my_overlay.variant(), "state=closed");
}
