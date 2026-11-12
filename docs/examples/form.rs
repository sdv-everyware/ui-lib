//! Form in an app: the Wi-Fi dialog with its password field. A tap on the field opens the
//! screen's keyboard, Enter sends the password on, the app shows the answer. The docs show the
//! parts between the doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/form.md.

// doc:note The form owns the fields, the focus and the sheet; what a submitted text means (join, search, rename) is the app's.

use crate::prelude::*;
// doc:use
use everyware_ui::dock::{KeyboardDock, KEYBOARD_DOCK_H};
use everyware_ui::form::{Form, FormEvent};
use everyware_ui::input_field::InputField;
use everyware_ui::sheet::Layout;
// doc:end

/// The screen's size: the card's places and the keyboard's are worked out on it.
const SCREEN: (f32, f32) = (1920.0, 1080.0);
/// The layout of the instance in the design.
const WIFI_LAYOUT: Layout = Layout::Dialog;
/// The shortest WPA password.
const MIN_LEN: usize = 8;

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The sheet with its fields, in the order given: 0 is the password.
    pub my_form: Form,
    /// The screen's one keyboard: open while a field of any form has the focus.
    pub osk: KeyboardDock,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_form: {
                let mut password = InputField::new("Password", "At least 8 characters");
                password.secure = true;
                let mut form = Form::new(WIFI_LAYOUT, [("#doc/form/password", password)]);
                // the card hugs the content: its height comes from the design (the .dcf)
                form.sheet = form.sheet.with_content_height(sizes.height("#wifi-content", "step=password"));
                form
            },
            osk: KeyboardDock::new(),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    // the sheet: the card moves, the scrim fades and takes taps while open
    wifi_card: Position = "#doc/form/card" ["FollowPositionCustomizer"] = Position::new(WIFI_LAYOUT.closed(SCREEN), None);
    wifi_scrim: Meter = "#doc/form/scrim" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
    wifi_scrim_taps: TapTarget = "#doc/form/scrim" ["TapTargetCustomizer"];
    _wifi_card_tap: () = "#doc/form/card" interactive;
    // the content shows its password step; the field in it
    wifi_step: Variant = "#doc/form/content" = Variant("step=password".to_string());
    wifi_password: Slot = "#doc/form/password" ["SlotCustomizer"];
    // the screen's keyboard
    osk_keys: Slot = "#osk/keys" ["SlotCustomizer"];
    osk_keys_position: Position = "#osk/keys" ["FollowPositionCustomizer"] = Position::new(KeyboardDock::new().keys_position(SCREEN), None);
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    // a dialog is centred in the screen above the keyboard while the keyboard is open
    let free_height = if state.osk.open() { SCREEN.1 - KEYBOARD_DOCK_H } else { SCREEN.1 };
    let slide = state.my_form.transition();
    model.wifi_card = Position::new(state.my_form.sheet.card_position(SCREEN, free_height), slide);
    model.wifi_scrim = Meter::new(state.my_form.sheet.scrim(), slide);
    model.wifi_scrim_taps = TapTarget(state.my_form.is_open());
    model.wifi_password = slot("#doc/form/password", state.my_form.field(0));
    model.osk_keys = slot("#osk/keys", &state.osk.keyboard);
    model.osk_keys_position = Position::new(state.osk.keys_position(SCREEN), state.osk.transition());
    // doc:end
}

// actions.rs: the gestures while the sheet is open, and the rule after them
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    // doc:act
    if state.my_form.is_open() {
        match node {
            // a tap on the scrim closes the sheet and takes the focus away
            "#doc/form/scrim" => {
                if matches!(gesture, Gesture::Up { inside: true, .. }) {
                    state.my_form.close();
                }
            }
            // a tap on a field, a key of the keyboard: the form routes them
            _ => match state.my_form.on(node, gesture, Instant::now(), &mut state.osk.keyboard) {
                Some(FormEvent::Submitted(_)) => submit(state),
                Some(FormEvent::Focused(_)) | Some(FormEvent::Edited(_)) | None => {}
            },
        }
    }
    // after every action: the keyboard is open while a field has the focus
    state.osk.follow(state.my_form.focused().is_some());
    // doc:end
}

// doc:change
/// Enter: a short password turns the field red at once; a long enough one goes to the
/// service, and the keyboard closes while the app waits for the answer.
fn submit(state: &mut State) {
    let len = state.my_form.field(0).text.chars().count();
    if len < MIN_LEN {
        state.my_form.field_mut(0).error = Some(format!("At least {MIN_LEN} characters ({len} so far)"));
        return;
    }
    let password = state.my_form.field(0).text.clone();
    state.my_form.blur();
    if send(Command::Join { password }) {
        state.my_form.field_mut(0).helper = "Joining...".into();
    }
}

/// The app opens the form empty (a tap on the Wi-Fi indicator) and shows the service's answer:
/// joined closes it, a failure shows under the field.
pub fn open_wifi(state: &mut State) {
    state.my_form.clear();
    state.my_form.open();
}

pub fn joined(state: &mut State, result: Result<(), String>) {
    match result {
        Ok(()) => state.my_form.close(),
        Err(error) => {
            state.my_form.field_mut(0).error = Some(format!("{error}: try again"));
            state.my_form.field_mut(0).text.clear();
        }
    }
}
// doc:end

// doc:send
/// What the screen asks the Wi-Fi service to do. An app has its own link to the service
/// (smarthome-app: screen_binder::send_wifi_command, a binder call to the Wi-Fi bundle); this
/// one stands in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Join { password: String },
}

/// Hands the command to the service; false when the service is not there (nothing is queued).
pub fn send(command: Command) -> bool {
    match &command {
        Command::Join { password } => log::info!("join with a password of {} characters", password.len()),
    }
    true
}
// doc:end

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[cfg(test)]
fn type_letters(state: &mut State, count: usize) {
    for i in 1..=count {
        tap(state, &format!("#kbd/r1/{i}"));
    }
}

#[test]
fn form_opens_types_and_sends_on_enter() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    // closed: gestures do nothing, the card waits below the screen
    tap(&mut state, "#doc/form/password");
    assert_eq!((state.my_form.focused(), model.wifi_card.y), (None, 1120.0));
    open_wifi(&mut state);
    present(&mut model, &state);
    assert_eq!(state.my_form.variant(), "layout=dialog, state=open");
    assert_eq!(((model.wifi_card.x, model.wifi_card.y), model.wifi_scrim.target, model.wifi_scrim_taps.0), ((400.0, 120.0), 35.0, true));
    // a tap on the field: the focus, the keyboard slides in, the dialog moves above it
    tap(&mut state, "#doc/form/password");
    present(&mut model, &state);
    assert_eq!((state.my_form.focused(), state.osk.variant().as_str(), model.osk_keys_position.y), (Some(0), "state=open", 660.0));
    assert_eq!(model.wifi_card.y, 0.0, "840 high does not fit in the 660 above the keys: at the top");
    // eight letters, shown as dots, then Enter
    type_letters(&mut state, 8);
    assert_eq!((state.my_form.field(0).text.as_str(), state.my_form.field(0).shown().chars().count()), ("qwertyui", 8));
    tap(&mut state, "#kbd/done");
    present(&mut model, &state);
    assert_eq!((state.my_form.focused(), state.osk.open(), state.my_form.field(0).helper_line()), (None, false, "Joining..."));
    assert_eq!(model.wifi_card.y, 120.0, "the keyboard is gone: back in the middle");
    // the service joined: the sheet closes with the close spring
    joined(&mut state, Ok(()));
    present(&mut model, &state);
    assert_eq!((state.my_form.variant().as_str(), model.wifi_card.y, model.wifi_scrim.target), ("layout=dialog, state=closed", 1120.0, 0.0));
    assert_eq!(model.wifi_card.transition.map(|t| t.duration.as_millis()), Some(400));
}

#[test]
fn form_shows_errors_and_closes_from_the_scrim() {
    let mut state = State::new(&Sizes::none());
    open_wifi(&mut state);
    tap(&mut state, "#doc/form/password");
    // too short: red at once, the keyboard stays for the next try
    type_letters(&mut state, 3);
    tap(&mut state, "#kbd/done");
    assert_eq!((state.my_form.field(0).variant().as_str(), state.osk.open()), ("state=error", true));
    assert_eq!(state.my_form.field(0).helper_line(), "At least 8 characters (3 so far)");
    // the next key takes the error away
    tap(&mut state, "#kbd/r2/1");
    assert_eq!(state.my_form.field(0).variant(), "state=default");
    // the service says no: the error under the field, the text cleared
    joined(&mut state, Err("wrong password".into()));
    assert_eq!((state.my_form.field(0).helper_line(), state.my_form.field(0).text.as_str()), ("wrong password: try again", ""));
    // a tap on the scrim closes the sheet and the keyboard with it
    tap(&mut state, "#doc/form/scrim");
    assert_eq!((state.my_form.is_open(), state.my_form.focused(), state.osk.open()), (false, None, false));
    // opened again, it starts empty
    open_wifi(&mut state);
    assert_eq!((state.my_form.field(0).text.as_str(), state.my_form.field(0).error.as_deref()), ("", None));
}
