//! #symbol in an app: the symbol of the device a sheet shows (a door, a bedroom), which changes
//! when the sheet opens for another device. The docs show the parts between the doc marks
//! (docs/build-docs.js); the tests at the end compile and run them (everyware_ui_recipes_test).
//! Edit here: this file is the example of the guide docs/content/symbol.md.

// doc:note Display only: the symbol is the variant (`icon=door-open`), so the app gives a new `Symbol` and the slot shows that variant.

use crate::prelude::*;
// doc:use
use everyware_ui::symbol::{Symbol, SymbolIcon};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_symbol: Symbol,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            my_symbol: Symbol::new(SymbolIcon::House),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_symbol: Slot = "#doc/symbol" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_symbol = slot("#doc/symbol", &state.my_symbol);
    // doc:end
}

// doc:change
/// The sheet opens for another device: its symbol comes with it. The next frame shows the
/// variant of that symbol.
pub fn open_device(state: &mut State, icon: SymbolIcon) {
    state.my_symbol = Symbol::new(icon);
}
// doc:end

#[test]
fn symbol_shows_the_variant_of_its_icon() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_symbol.variant(), "icon=house");
    open_device(&mut state, SymbolIcon::DoorOpen);
    present(&mut model, &state);
    assert_eq!(state.my_symbol.variant(), "icon=door-open");
}

#[test]
fn symbol_every_icon_is_a_variant_of_its_own() {
    let variants: Vec<String> = SymbolIcon::ALL.iter().map(|&icon| Symbol::new(icon).variant()).collect();
    assert_eq!(variants, ["icon=house", "icon=door-open", "icon=bed-double", "icon=armchair", "icon=droplets"]);
}
