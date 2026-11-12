//! #sheet in an app: a dialog with a device's details that a tap opens, a tap on the scrim or
//! on its close button closes, and the app fills with a title. The docs show the parts between
//! the doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/sheet.md.

// doc:note The app never sets the sheet's variant: it moves the card and fades the scrim, and the instance keeps its own names and content.

use crate::prelude::*;
// doc:use
use everyware_ui::button::{ButtonEvent, Icon, IconButton, IconButtonKind};
use everyware_ui::dock::KEYBOARD_DOCK_H;
use everyware_ui::sheet::{Layout, Sheet};
// doc:end

/// The screen's size: the card's places are worked out on it.
const SCREEN: (f32, f32) = (1920.0, 1080.0);
/// The layout of the instance in the design: a dialog in the middle (or `Layout::Panel(Edge::Right)`).
const DEVICE_LAYOUT: Layout = Layout::Dialog;

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The sheet: closed or open, its layout, the height of its content.
    pub my_sheet: Sheet,
    /// The content's own controls and texts.
    pub close: IconButton,
    pub title: String,
    // doc:end
    /// The screen's keyboard, open or not: a dialog moves above it.
    pub keyboard_open: bool,
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            // the card hugs the content: its height comes from the design (the .dcf)
            my_sheet: Sheet::new(DEVICE_LAYOUT).with_content_height(sizes.height("#device-content", "state=default")),
            close: IconButton::new(IconButtonKind::Tonal, Icon::Close),
            title: String::new(),
            // doc:end
            keyboard_open: false,
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    // the card moves on both axes, the scrim fades and takes taps while open
    device_card: Position = "#doc/sheet/card" ["FollowPositionCustomizer"] = Position::new(DEVICE_LAYOUT.closed(SCREEN), None);
    device_scrim: Meter = "#doc/sheet/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
    device_scrim_taps: TapTarget = "#doc/sheet/scrim" ["TapTargetCustomizer"];
    // a tap on the card between its parts stays on the card
    _device_card_tap: () = "#doc/sheet/card" interactive;
    // the content: the app's own slots and texts in the card
    device_close: Slot = "#doc/sheet/close" ["SlotCustomizer"];
    device_title: String = "#doc/sheet/title";
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    // a dialog is centred in the screen above the keyboard while the keyboard is open
    let free_height = if state.keyboard_open { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 };
    let slide = state.my_sheet.transition();
    model.device_card = Position::new(state.my_sheet.card_position(SCREEN, free_height), slide);
    model.device_scrim = Meter::new(state.my_sheet.scrim(), slide);
    model.device_scrim_taps = TapTarget(state.my_sheet.is_open());
    model.device_close = slot("#doc/sheet/close", &state.close);
    model.device_title = state.title.clone();
    // doc:end
}

// actions.rs: the arms of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // the close button and a tap on the scrim close the sheet
        "#doc/sheet/close" => {
            if let Some(ButtonEvent::Clicked) = state.close.on(gesture) {
                state.my_sheet.close();
            }
        }
        "#doc/sheet/scrim" if matches!(gesture, Gesture::Up { inside: true, .. }) => state.my_sheet.close(),
        // doc:end
        _ => {}
    }
}

// doc:change
/// The app opens the sheet with what it shows (a tap on a tile, a device that needs the user)
/// and closes it when there is nothing to show any more (the device is gone).
pub fn show_device(state: &mut State, name: Option<&str>) {
    match name {
        Some(name) => {
            state.title = name.to_string();
            state.my_sheet.open();
        }
        None => state.my_sheet.close(),
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
fn sheet_opens_in_the_middle_and_closes_from_the_scrim() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    // closed: the card waits 40 px below the screen, the scrim is clear and lets taps through
    assert_eq!(state.my_sheet.variant(), "layout=dialog, state=closed");
    assert_eq!(((model.device_card.x, model.device_card.y), model.device_scrim.target, model.device_scrim_taps.0), ((400.0, 1120.0), 0.0, false));
    show_device(&mut state, Some("Kitchen light"));
    present(&mut model, &state);
    // open: no content height known (no design), so the default 840 high card, centred
    assert_eq!(((model.device_card.x, model.device_card.y), model.device_scrim.target, model.device_scrim_taps.0), ((400.0, 120.0), 35.0, true));
    assert_eq!((model.device_title.as_str(), model.device_card.transition.map(|t| t.duration.as_millis())), ("Kitchen light", Some(550)));
    // a tap on the card itself does nothing
    tap(&mut state, "#doc/sheet/card");
    assert!(state.my_sheet.is_open());
    tap(&mut state, "#doc/sheet/scrim");
    present(&mut model, &state);
    assert_eq!(((model.device_card.x, model.device_card.y), model.device_scrim.target), ((400.0, 1120.0), 0.0));
    assert_eq!(model.device_card.transition.map(|t| t.duration.as_millis()), Some(400));
}

#[test]
fn sheet_closes_from_its_button_and_moves_above_the_keyboard() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    state.my_sheet = state.my_sheet.with_content_height(Some(312.0));
    show_device(&mut state, Some("Hall sensor"));
    present(&mut model, &state);
    assert_eq!((model.device_card.x, model.device_card.y), (400.0, 344.0), "312 + 80 high, centred in 1080");
    state.keyboard_open = true;
    present(&mut model, &state);
    assert_eq!((model.device_card.x, model.device_card.y), (400.0, 134.0), "centred in the 660 above the keyboard");
    tap(&mut state, "#doc/sheet/close");
    assert_eq!(state.my_sheet.variant(), "layout=dialog, state=closed");
    // a side panel keeps its edge
    let mut panel = Sheet::new(Layout::Panel(everyware_ui::sheet::Edge::Right));
    assert_eq!(panel.card_position(SCREEN, SCREEN.1), (1960.0, 0.0));
    panel.open();
    assert_eq!(panel.card_position(SCREEN, 660.0), (960.0, 0.0));
}
