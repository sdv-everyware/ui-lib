//! Pages in an app: a sheet's settings and the list one of its rows opens, in one place. A tap on
//! the row slides the list in from the right, Back slides it out, as Android's Settings goes one
//! page deeper. The docs show the parts between the doc marks (docs/build-docs.js); the tests at
//! the end compile and run them (everyware_ui_recipes_test). Edit here: this file is the example
//! of the guide docs/content/pages.md.

// doc:note The presenter places every page: one `ScreenView` per page from `state.my_pages.page(i, EFFECT, width)`. The pages lie side by side in the design; `width` is the distance between two of them.

use crate::prelude::*;
// doc:use
use everyware_ui::screens::{Effect, Pages};
use everyware_ui::ScreenView;

/// The pages in their order: the settings, then the list one of its rows opens (the panel's
/// settings sheet and its time zones).
pub const PAGES: [&str; 2] = ["#doc/pages/page-main", "#doc/pages/page-timezone"];
/// The component the pages are drawn in (a sheet's content): the pages' places are read from its
/// variant, where the frame that clips them stands at x 0.
pub const CONTENT: (&str, &str) = ("#doc/pages-content", "state=default");
/// One page deeper slides in from the right, Back slides it out (Material 3's shared axis). With
/// the pages side by side only Slide brings the next page into view.
pub const EFFECT: Effect = Effect::Slide;
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// Which page shows and which showed before.
    pub my_pages: Pages,
    /// The first page's x in the frame that clips the pages, and its width: the next page lies
    /// beside it.
    pub first_page: Frame,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_pages: Pages::default(),
            first_page: sizes.part(CONTENT.0, CONTENT.1, PAGES[0]).unwrap_or(Frame { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    // one field per page, the first one shown at the start
    main_page: ScreenView = "#doc/pages/page-main" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(100.0, None), x: None };
    list_page: ScreenView = "#doc/pages/page-timezone" ["ScreenCustomizer"] = ScreenView::default();
    // the row that opens the list, and Back (a list item and a button on a real page)
    _row: () = "#doc/pages/timezone" interactive;
    _back: () = "#doc/pages/back" interactive;
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    // every page from the first page's place: the one that shows is there, a slide moves the
    // others by the width of a page
    let page = |i: usize| ScreenView::new(state.my_pages.page(i, EFFECT, state.first_page.width), state.first_page.x);
    model.main_page = page(0);
    model.list_page = page(1);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // the row goes one page deeper, Back comes back; the finger has to lift on it
        "#doc/pages/timezone" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.my_pages.show(1),
        "#doc/pages/back" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.my_pages.show(0),
        // doc:end
        _ => {}
    }
}

// doc:change
/// The sheet opens again: it starts on its first page, and nothing slides.
pub fn reopen(state: &mut State) {
    state.my_pages.reset();
}
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn pages_go_one_deeper_and_back_as_a_slide() {
    let mut state = State { my_pages: Pages::default(), first_page: Frame { x: 24.0, y: 0.0, width: 856.0, height: 900.0 } };
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    // at the start the first page shows, the list waits beside it
    assert_eq!((model.main_page.opacity.target, model.list_page.opacity.target), (100.0, 0.0));
    // the row: the list comes from the right to the first page's place, the settings leave left
    tap(&mut state, "#doc/pages/timezone");
    present(&mut model, &state);
    assert_eq!((state.my_pages.selected, state.my_pages.previous), (1, Some(0)));
    assert_eq!((model.list_page.x.map(|x| x.target), model.main_page.x.map(|x| x.target)), (Some(24.0), Some(24.0 - 856.0)));
    assert_eq!((model.list_page.opacity.target, model.main_page.opacity.target), (100.0, 0.0));
    assert_eq!(model.list_page.x.and_then(|x| x.transition).map(|t| t.duration.as_millis()), Some(300));
    // Back: the list leaves to the right, the settings come back
    tap(&mut state, "#doc/pages/back");
    present(&mut model, &state);
    assert_eq!((model.main_page.x.map(|x| x.target), model.list_page.x.map(|x| x.target)), (Some(24.0), Some(24.0 + 856.0)));
}

#[test]
fn pages_start_on_the_first_page_when_the_sheet_opens_again() {
    let mut state = State::new(&Sizes::none());
    tap(&mut state, "#doc/pages/timezone");
    reopen(&mut state);
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!((state.my_pages.selected, model.main_page.opacity.target, model.list_page.opacity.target), (0, 100.0, 0.0));
    assert_eq!(model.main_page.x.and_then(|x| x.transition), None, "the pages take their places at once");
    // a finger that slides off the row before it lifts opens nothing
    on_gesture(&mut state, "#doc/pages/timezone", Gesture::DOWN);
    on_gesture(&mut state, "#doc/pages/timezone", Gesture::Up { x: 0.0, y: 0.0, inside: false, velocity: 0.0, velocity_x: 0.0 });
    assert_eq!(state.my_pages.selected, 0);
}
