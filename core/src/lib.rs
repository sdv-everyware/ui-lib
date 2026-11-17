//! everyware_ui_core: the UI components of the Smart HOme Figma file without harry.
//!
//! Each component is a small state machine: it takes gestures (down, up, cancel on its node),
//! says which Figma variant it shows and which change of state led there, and reports what the
//! user did (clicked, toggled, selected). The transitions of `components.json` turn a change
//! into a duration and an easing; `everyware_ui` hands both to harry.
//!
//! Nothing here knows harry, so all of it is tested on the host.

pub mod bar_chart;
pub mod button;
pub mod dock;
pub mod dropdown;
pub mod follow;
pub mod form;
pub mod gesture;
pub mod input_field;
pub mod keyboard;
pub mod list_item;
pub mod list_view;
pub mod overlay;
pub mod pressable;
pub mod radio;
pub mod screens;
pub mod scroller;
pub mod segmented;
pub mod sheet;
pub mod skeleton;
pub mod slider;
pub mod symbol;
pub mod tabs;
pub mod tile;
pub mod toggle;
pub mod tone;
pub mod transitions;

pub use follow::Follow;
pub use gesture::{Gesture, GestureTracker, Phase};
pub use transitions::{Change, ComponentKind, Easing, Transition};

/// What every component tells the renderer.
pub trait Component {
    /// The component set in Figma.
    const KIND: ComponentKind;

    /// The Figma variant to show, as harry expects it: "prop=value, prop=value" in the order
    /// of the set's properties.
    fn variant(&self) -> String;

    /// The last change of state, for the transition into the current variant.
    fn last_change(&self) -> Option<Change>;

    /// The transition into the current variant, from components.json.
    fn transition(&self) -> Option<Transition> {
        self.last_change().and_then(|change| transitions::transition(Self::KIND, change))
    }
}
