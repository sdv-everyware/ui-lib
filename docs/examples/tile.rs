//! #tile in an app: a dimmable light's card. A tap on the card opens the device, the slider
//! inside it dims the light, and the card shows what the light reports. The docs show the parts
//! between the doc marks (docs/build-docs.js); the tests at the end compile and run them
//! (everyware_ui_recipes_test). Edit here: this file is the example of the guide
//! docs/content/tile.md.

// doc:note The card answers `Opened`; the control inside (`state.my_tile.toggle`, `.slider`, `.segmented`, the stepper's minus and plus) answers through `on_part` with its part. The app's listener returns the status `Input::feed` gives.

use crate::prelude::*;
// doc:use
use everyware_ui::tile::{Tile, TileEvent, TileKind};
use everyware_ui::symbol::SymbolIcon;
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_tile: Tile,
    /// Where the design puts the card on the display: the slider reads the finger against it.
    pub tile_at: (f32, f32),
    /// The device's sheet is open (the app's own sheet, docs/content/sheet.md).
    pub device_open: bool,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_tile: {
                let mut tile = Tile::new(TileKind::Slider, SymbolIcon::BedDouble, "Bedroom", "Lights").value("65%");
                tile.slider.value = 65.0;
                tile
            },
            tile_at: sizes.frame("#doc/tile").map_or((0.0, 0.0), |f| (f.x, f.y)),
            device_open: false,
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_tile: Slot = "#doc/tile" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_tile = slot("#doc/tile", &state.my_tile);
    // doc:end
}

// actions.rs: the arm of the match on the gesture's node name
pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {
    match node {
        // doc:act
        // the card ("#doc/tile") or the control inside it ("#doc/tile/#tile/slider")
        _ if node == "#doc/tile" || part_of("#doc/tile", node).is_some() => {
            let (x, y) = state.tile_at;
            match state.my_tile.on_part(part_of("#doc/tile", node), gesture, x, y) {
                Some(TileEvent::Opened) => state.device_open = true,
                // the slider drags: the value text follows, the light waits
                Some(TileEvent::Moving(level)) => state.my_tile.value = format!("{level:.0}%"),
                // the finger is up: the light gets the level
                Some(TileEvent::Level(level)) => {
                    state.my_tile.value = format!("{level:.0}%");
                    if !send(Command::Level(level)) {
                        log::warn!("tile: the light is not there, nothing was sent");
                    }
                }
                // the other kinds: Toggled(on), Mode(i), Step(-1 or 1), the same way
                _ => {}
            }
        }
        // doc:end
        _ => {}
    }
}

// doc:change
/// The light reports (another panel or its own button changed it): the card shows the level, or
/// that the light is offline (the variant state=disabled, which takes no taps). Not while a
/// finger is on the slider, or the thumb would jump under it.
pub fn light_reported(state: &mut State, level: Option<f32>) {
    let tile = &mut state.my_tile;
    match level {
        Some(level) if !tile.slider.dragging() => {
            tile.slider.value = level.clamp(0.0, 100.0);
            tile.value = format!("{:.0}%", tile.slider.value);
            tile.subtitle = "Lights".into();
            tile.press.set_enabled(true);
        }
        Some(_) => {}
        None => {
            tile.subtitle = "Offline".into();
            tile.press.set_enabled(false);
        }
    }
}
// doc:end

// doc:send
/// What the card asks the light to do. An app has its own link to the device (smarthome-app:
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

#[cfg(test)]
fn tap(state: &mut State, node: &str) {
    for gesture in [Gesture::DOWN, Gesture::UP] {
        on_gesture(state, node, gesture);
    }
}

#[test]
fn tile_opens_on_the_card_and_dims_on_the_slider() {
    use everyware_ui::slider::TRACK_INSET;
    use everyware_ui::tile::{PART_SLIDER, TILE_PAD};
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_tile.variant(), "kind=slider, state=default");
    // a tap on the card opens the device; the level stays
    tap(&mut state, "#doc/tile");
    assert_eq!((state.device_open, state.my_tile.slider.value), (true, 65.0));
    // the slider inside: its track from 24 + 20 in the card, 203 long
    let node = format!("#doc/tile/{PART_SLIDER}");
    let track_x = state.tile_at.0 + TILE_PAD + TRACK_INSET;
    on_gesture(&mut state, &node, Gesture::down(track_x + 203.0 / 2.0, 0.0));
    assert_eq!((state.my_tile.slider.value, state.my_tile.value.as_str()), (50.0, "50%"));
    on_gesture(&mut state, &node, Gesture::up(track_x + 203.0, 0.0));
    assert_eq!((state.my_tile.slider.value, state.my_tile.value.as_str()), (100.0, "100%"));
    present(&mut model, &state);
}

#[test]
fn tile_shows_what_the_light_reports() {
    let mut state = State::new(&Sizes::none());
    light_reported(&mut state, Some(30.0));
    assert_eq!((state.my_tile.slider.value, state.my_tile.value.as_str()), (30.0, "30%"));
    // offline: disabled, and neither the card nor the slider takes a tap
    light_reported(&mut state, None);
    assert_eq!((state.my_tile.variant().as_str(), state.my_tile.subtitle.as_str()), ("kind=slider, state=disabled", "Offline"));
    tap(&mut state, "#doc/tile");
    tap(&mut state, "#doc/tile/#tile/slider");
    assert_eq!((state.device_open, state.my_tile.slider.value), (false, 30.0));
    // back: enabled again with the new level
    light_reported(&mut state, Some(80.0));
    assert_eq!((state.my_tile.variant().as_str(), state.my_tile.value.as_str()), ("kind=slider, state=default", "80%"));
    assert!(send(Command::Level(80.0)));
}
