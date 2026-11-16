//! #scrollbar in an app: the thumb beside a scrolling list, which shows while the list moves and
//! fades out after it stops. The docs show the parts between the doc marks (docs/build-docs.js);
//! the tests at the end compile and run them (everyware_ui_recipes_test). Edit here: this file is
//! the example of the guide docs/content/scrollbar.md.

// doc:note Display only: it follows the `ListView` of its list (`my_list`) and has no state of its own.

use crate::prelude::*;
// doc:use
use everyware_ui::list_view::ListView;
// doc:end
use everyware_ui::list_item::{ListItem, ListItemKind, ListItemLines, ListItemSize};

/// Rows 128 high, 8 between; the list is as high as its slot in the design (#agenda/list).
const ROW_PITCH: f32 = 128.0 + 8.0;

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The list the scrollbar follows (the list item's guide shows its slot and its gestures).
    pub my_list: ListView,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            // as high as the design draws the list's slot: no height in the code
            my_list: ListView::new([], sizes.frame("#agenda/list").map_or(0.0, |f| f.height), ROW_PITCH),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    // not interactive: it takes no taps
    my_scrollbar: Slot = "#doc/scrollbar" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    // the same ListView as the list's line: the thumb moves with its scroll, frame by frame
    model.my_scrollbar = scrollbar("#doc/scrollbar", &state.my_list);
    // doc:end
}

// doc:change
/// The app changes the list, never the scrollbar: new rows give the content a new length and
/// the thumb follows at the next frame. Rows that fit in the slot scroll nowhere: the thumb
/// stays at the top and the scrollbar does not show.
pub fn show_titles(state: &mut State, titles: &[&str]) {
    state.my_list.set_rows(titles.iter().map(|title| {
        ListItem::new(ListItemKind::Image, *title, "").lines(ListItemLines::Three).size(ListItemSize::Column)
    }));
}
// doc:end

#[cfg(test)]
const TITLES: [&str; 12] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"];

/// The state with a list 744 high, as a design would give it.
#[cfg(test)]
fn state_744() -> State {
    State { my_list: ListView::new([], 744.0, ROW_PITCH) }
}

/// A drag of the list: down, up by `dy`, lift with `velocity` (pixels a second, up is negative).
#[cfg(test)]
fn drag(state: &mut State, dy: f32, velocity: f32, now: Instant) {
    state.my_list.on(Some(0), Gesture::down(0.0, 400.0), now);
    state.my_list.on(None, Gesture::Move { x: 0.0, y: 400.0 - dy, inside: false }, now);
    state.my_list.on(None, Gesture::Up { x: 0.0, y: 400.0 - dy, inside: false, velocity, velocity_x: 0.0 }, now);
}

#[test]
fn scrollbar_is_one_component_and_hidden_at_rest() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    show_titles(&mut state, &TITLES);
    present(&mut model, &state);
    assert_eq!(scrollbar("#doc/scrollbar", &state.my_list).items()[0].variant(), "state=default");
    let now = Instant::now();
    assert_eq!((state.my_list.thumb(now), state.my_list.scroll().scrollbar_shown(now).0), (0.0, false));
}

#[test]
fn scrollbar_shows_while_the_list_moves_and_fades_after() {
    let mut state = state_744();
    show_titles(&mut state, &TITLES);
    let now = Instant::now();
    drag(&mut state, 200.0, 0.0, now);
    let thumb = state.my_list.thumb(now);
    assert!(thumb > 0.0 && thumb < 100.0, "{thumb}");
    assert!(state.my_list.scroll().scrollbar_shown(now).0);
    // 300 ms after the content stopped it fades
    let later = now + std::time::Duration::from_millis(400);
    assert!(!state.my_list.scroll().scrollbar_shown(later).0);
}

#[test]
fn scrollbar_a_fling_takes_the_thumb_to_the_end() {
    let mut state = state_744();
    show_titles(&mut state, &TITLES);
    let now = Instant::now();
    drag(&mut state, 300.0, -8000.0, now);
    assert_eq!(state.my_list.thumb(now + std::time::Duration::from_secs(5)), 100.0);
}

#[test]
fn scrollbar_follows_new_rows_that_fit() {
    let mut state = state_744();
    show_titles(&mut state, &TITLES);
    drag(&mut state, 200.0, 0.0, Instant::now());
    // three rows fit in the slot: nothing scrolls, the thumb is back at the top
    show_titles(&mut state, &TITLES[..3]);
    assert_eq!((state.my_list.max_scroll(), state.my_list.thumb(Instant::now())), (0.0, 0.0));
}
