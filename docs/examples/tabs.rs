//! #tabs in an app: the header's tabs and the three screens behind them. A tap on a tab shows
//! its screen with the screens' effect; the app can open a screen itself. The docs show the
//! parts between the doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/tabs.md.

// doc:note The presenter shows the chosen screen: one `ScreenView` per screen from `state.my_tabs.screen(i, EFFECT, width)`. Another `Effect` in `EFFECT` changes how every switch looks; the time is `tabs`/`switch` in components.json.

use crate::prelude::*;
// doc:use
use everyware_ui::tabs::{Tabs, TabsEvent};
use everyware_ui::screens::Effect;
use everyware_ui::part_of;
use everyware_ui::ScreenView;

/// The screens behind the tabs, in the tabs' order: the label and the screen's node.
pub const SCREENS: [(&str, &str); 3] = [("Overview", "#overview"), ("Rooms", "#rooms"), ("Automations", "#automations")];
/// How the screens change: Effect::Fade (the default), FadeThrough, Slide or None.
pub const EFFECT: Effect = Effect::Fade;
/// The distance between two screens of a slide: the screen's width.
pub const SCREEN_WIDTH: f32 = 1920.0;
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_tabs: Tabs,
    /// Where the design puts each screen in its parent (its x): a slide starts from there.
    pub screen_x: [f32; 3],
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_tabs: Tabs::new(SCREENS.map(|(label, _)| label)),
            screen_x: SCREENS.map(|(_, node)| sizes.frame(node).map_or(0.0, |f| f.x)),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_tabs: Slot = "#doc/tabs" ["SlotCustomizer"];
    // one field per screen behind the tabs, the first one shown at the start
    overview: ScreenView = "#overview" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(100.0, None), x: None };
    rooms: ScreenView = "#rooms" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(0.0, None), x: None };
    automations: ScreenView = "#automations" ["ScreenCustomizer"] = ScreenView { opacity: Meter::new(0.0, None), x: None };
    // a screen drawn as a set with state=hidden/shown: shown once, the effect does the rest
    rooms_variant: Variant = "#rooms" = Variant("state=shown".to_string());
    automations_variant: Variant = "#automations" = Variant("state=shown".to_string());
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_tabs = slot("#doc/tabs", &state.my_tabs);
    // each screen: its opacity, and its x for Effect::Slide, from the effect
    let screen = |i: usize| ScreenView::new(state.my_tabs.screen(i, EFFECT, SCREEN_WIDTH), state.screen_x[i]);
    model.overview = screen(0);
    model.rooms = screen(1);
    model.automations = screen(2);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // a tab reports with its tab bar: "#doc/tabs/#tabs/tab-2"
        _ if part_of("#doc/tabs", node).is_some() => {
            let tab = part_of("#doc/tabs", node).and_then(|p| p.strip_prefix("#tabs/tab-")).and_then(|n| n.parse::<usize>().ok());
            if let Some(TabsEvent::Selected(i)) = tab.and_then(|n| n.checked_sub(1)).and_then(|i| state.my_tabs.on(i, gesture)) {
                // the presenter already shows screen i; tell anyone else who needs it here
                log::info!("screen {}", SCREENS[i].0);
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app opens a screen itself (a notification about a routine opens Automations): the tab is
/// chosen and the screens switch with the same effect as after a tap.
pub fn open_screen(state: &mut State, node: &str) {
    if let Some(i) = SCREENS.iter().position(|(_, n)| *n == node) {
        state.my_tabs.select(i);
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
fn tabs_switch_the_screen_on_a_tap() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!((state.my_tabs.variant().as_str(), model.overview.opacity.target, model.rooms.opacity.target), ("tabs=3, state=default", 100.0, 0.0));
    // a tap on the second tab: its pill lights, Rooms fades in, Overview fades out
    tap(&mut state, "#doc/tabs/#tabs/tab-2");
    present(&mut model, &state);
    assert_eq!((state.my_tabs.selected, state.my_tabs.previous, state.my_tabs.target(1)), (1, Some(0), 100.0));
    assert_eq!((model.overview.opacity.target, model.rooms.opacity.target, model.automations.opacity.target), (0.0, 100.0, 0.0));
    // Fade moves nothing: no x, the design's layout stays
    assert_eq!(model.rooms.x, None);
    // the chosen tab again, or a tab of another tab bar: nothing
    tap(&mut state, "#doc/tabs/#tabs/tab-2");
    tap(&mut state, "#rooms/tabs/#tabs/tab-3");
    assert_eq!(state.my_tabs.selected, 1);
}

#[test]
fn tabs_open_a_screen_from_the_app_and_another_effect() {
    let mut state = State::new(&Sizes::none());
    open_screen(&mut state, "#automations");
    assert_eq!((state.my_tabs.selected, state.my_tabs.previous), (2, Some(0)));
    // another effect: Slide lays the screens side by side, Overview leaves to the left
    let old = state.my_tabs.screen(0, Effect::Slide, SCREEN_WIDTH);
    let new = state.my_tabs.screen(2, Effect::Slide, SCREEN_WIDTH);
    assert_eq!((old.offset, new.offset, new.opacity), (Some(-3840.0), Some(0.0), 100.0));
    assert_eq!(ScreenView::new(new, 96.0).x.map(|x| x.target), Some(96.0));
    // FadeThrough: the old screen goes in the first 90 ms of 300, then the new one comes
    let out = state.my_tabs.screen(0, Effect::FadeThrough, SCREEN_WIDTH).opacity_transition.unwrap();
    assert_eq!(out.duration.as_millis(), 90);
    // disabled tabs take no taps
    state.my_tabs.press.set_enabled(false);
    tap(&mut state, "#doc/tabs/#tabs/tab-1");
    assert_eq!((state.my_tabs.selected, state.my_tabs.variant().as_str()), (2, "tabs=3, state=disabled"));
}
