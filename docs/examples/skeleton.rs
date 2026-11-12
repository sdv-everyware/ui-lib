//! #skeleton in an app: the placeholder of a page while it loads. The shapes stand where the page
//! will be, a light band sweeps across them, and the loader fades out when the page shows. The
//! docs show the parts between the doc marks (docs/build-docs.js); the tests at the end compile
//! and run them (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/skeleton.md.

// doc:note Display only: the app says since when it loads (`start`) and when it is done (`stop`); the binding moves the band from the time, no ticks reach the app.

use crate::prelude::*;
// doc:use
use everyware_ui::skeleton::{Skeleton, SkeletonKind, SkeletonSize};
use everyware_ui::transitions::{self, Change, ComponentKind};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_skeleton: Skeleton,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            // four text lines, 832 wide: the width of a side panel's content
            my_skeleton: Skeleton::new(SkeletonKind::Text, SkeletonSize::Wide),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_skeleton: Slot = "#doc/skeleton" ["SlotCustomizer"];
    // the frame that holds the skeletons over the content: shown while loading, then it fades
    loading: Meter = "#panel/loading" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_skeleton = slot("#doc/skeleton", &state.my_skeleton);
    model.loading = if state.my_skeleton.is_loading() {
        Meter::new(100.0, None)
    } else {
        Meter::new(0.0, transitions::transition(ComponentKind::Skeleton, Change::Hide))
    };
    // doc:end
}

// doc:change
/// The app asked for the page: the loader shows at once and the band starts to sweep. A new
/// page while one still loads starts the band again from its edge.
pub fn page_asked(state: &mut State, now: Instant) {
    state.my_skeleton.stop();
    state.my_skeleton.start(now);
}

/// The page is there (or the app gave up): the band stops and the loader fades out in 250 ms.
pub fn page_shown(state: &mut State) {
    state.my_skeleton.stop();
}
// doc:end

#[test]
fn skeleton_sweeps_while_the_page_loads() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_skeleton.variant(), "kind=text, size=wide");
    assert_eq!((state.my_skeleton.is_loading(), model.loading.target), (false, 0.0));
    let t0 = Instant::now();
    page_asked(&mut state, t0);
    present(&mut model, &state);
    assert_eq!(model.loading.target, 100.0);
    // one pass is 1400 ms: halfway after 700, from the start again after 1400
    let ms = std::time::Duration::from_millis;
    assert_eq!(state.my_skeleton.sweep_at(t0 + ms(700)), Some(50.0));
    assert_eq!(state.my_skeleton.sweep_at(t0 + ms(1400 + 350)), Some(25.0));
}

#[test]
fn skeleton_fades_out_when_the_page_shows() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    page_asked(&mut state, Instant::now());
    page_shown(&mut state);
    present(&mut model, &state);
    assert_eq!(state.my_skeleton.sweep_at(Instant::now()), None);
    assert_eq!(model.loading.target, 0.0);
    assert_eq!(model.loading.transition.as_ref().map(|t| t.duration.as_millis()), Some(250));
}

#[test]
fn skeleton_a_new_page_starts_the_band_again() {
    let mut state = State::new(&Sizes::none());
    let t0 = Instant::now();
    page_asked(&mut state, t0);
    let t1 = t0 + std::time::Duration::from_millis(500);
    page_asked(&mut state, t1);
    assert_eq!(state.my_skeleton.since, Some(t1));
    assert_eq!(state.my_skeleton.sweep_at(t1), Some(0.0));
}
