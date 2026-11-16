//! The examples of the library's guides, one module per component (docs/content/<component>.md
//! links to its example). Each is a small app: State, the UI model, the presenter and the
//! gesture's arm, as an app writes them. everyware_ui_recipes_test compiles and runs them all, so
//! an example that does not build fails the build; the docs show the parts between the doc marks.
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

/// What every example has from the start: the UI model's prelude and the binding's helpers.
mod prelude {
    pub use everyware_ui::design::{Frame, Sizes};
    pub use everyware_ui::model::prelude::*;
    pub use everyware_ui::{list, part_of, row_of, scrollbar, slot, Component, Gesture, ScreenView};
    pub use std::time::Instant;
}

pub mod bar_chart;
pub mod button;
pub mod dropdown;
pub mod form;
pub mod icon;
pub mod icon_button;
pub mod input_field;
pub mod keyboard;
pub mod keyboard_dock;
pub mod list_item;
pub mod overlay;
pub mod pages;
pub mod radio;
pub mod scrollbar;
pub mod segmented;
pub mod sheet;
pub mod skeleton;
pub mod slider;
pub mod symbol;
pub mod tabs;
pub mod tile;
pub mod toggle;
