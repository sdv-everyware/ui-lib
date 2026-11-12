//! #icon in an app: the glyphs of a list of Wi-Fi networks, the signal before the name and a lock
//! after it. An icon is not a slot of its own: it lives in another component (here a list item),
//! which lights one glyph of its stack. The docs show the parts between the doc marks
//! (docs/build-docs.js); the tests at the end compile and run them (everyware_ui_recipes_test).
//! Edit here: this file is the example of the guide docs/content/icon.md.

// doc:note An icon is picked by its name in the row (`leading`, `trailing`); the names are `list_item::GLYPHS`, the `icon` values of `#icon`.

use crate::prelude::*;
// doc:use
use everyware_ui::list_item::{ListItem, ListItemKind};
use everyware_ui::list_view::ListView;
// doc:end
use everyware_ui::list_item::GLYPHS;

/// Six rows of 72 with 8 between: the list's slot is 472 high.
const LIST_H: f32 = 6.0 * 72.0 + 5.0 * 8.0;
const ROW_PITCH: f32 = 72.0 + 8.0;

/// A network as the app has it. Yours comes from a service (smarthome-app: the Wi-Fi bundle).
#[derive(Clone, Debug)]
pub struct Network {
    pub ssid: String,
    /// 1 to 4 bars.
    pub bars: u8,
    pub secured: bool,
}

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    /// The rows of the list; each row lights its glyphs.
    pub my_icon: ListView,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_icon: ListView::new([], LIST_H, ROW_PITCH),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_icon: Slot = "#doc/icon" interactive ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    // the rows set their glyphs: the chosen one at opacity 1, every other one of the stack at 0
    model.my_icon = list("#doc/icon", &state.my_icon);
    // doc:end
}

// doc:change
/// The glyph of a signal: four steps of `wifi`, as Android shows them.
pub fn signal_glyph(bars: u8) -> &'static str {
    match bars {
        0 | 1 => "wifi-zero",
        2 => "wifi-low",
        3 => "wifi-high",
        _ => "wifi",
    }
}

/// A new scan: one row per network, the signal before the name, a lock after it when the
/// network has a password (None: no glyph at the end). A stronger signal is a new glyph on
/// the same row: the row keeps its place and its selection by its name.
pub fn show_networks(state: &mut State, networks: &[Network]) {
    state.my_icon.set_rows(networks.iter().map(|n| {
        let detail = if n.secured { "WPA2" } else { "Open" };
        ListItem::new(ListItemKind::Icon, n.ssid.clone(), detail)
            .leading(signal_glyph(n.bars))
            .trailing(n.secured.then_some("lock"))
            .wide()
    }));
}
// doc:end

#[cfg(test)]
fn network(ssid: &str, bars: u8, secured: bool) -> Network {
    Network { ssid: ssid.into(), bars, secured }
}

#[test]
fn icon_rows_light_the_glyph_of_the_signal_and_the_lock() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    show_networks(&mut state, &[network("Home", 4, true), network("Cafe", 2, false)]);
    present(&mut model, &state);
    let rows = state.my_icon.rows();
    assert_eq!(rows[0].variant(), "state=default, kind=icon, lines=two, size=wide");
    assert_eq!((rows[0].leading, rows[0].trailing), (Some("wifi"), Some("lock")));
    assert_eq!((rows[1].leading, rows[1].trailing), (Some("wifi-low"), None));
}

#[test]
fn icon_every_glyph_the_app_picks_is_in_the_stack() {
    for bars in 0..=5 {
        assert!(GLYPHS.contains(&signal_glyph(bars)), "{}", signal_glyph(bars));
    }
    assert!(GLYPHS.contains(&"lock"));
}

#[test]
fn icon_a_new_glyph_keeps_the_row_selected() {
    let mut state = State::new(&Sizes::none());
    show_networks(&mut state, &[network("Home", 2, true), network("Cafe", 2, false)]);
    state.my_icon.select(Some(0));
    // the next scan: the signal of "Home" is stronger
    show_networks(&mut state, &[network("Home", 3, true), network("Cafe", 2, false)]);
    let row = &state.my_icon.rows()[0];
    assert_eq!((row.leading, row.selected), (Some("wifi-high"), true));
}
