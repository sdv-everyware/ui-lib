//! The UI model of a harry app: one field per node of the design the app writes to or taps,
//! the root frame per display, the first value of every field. Every app wrote the same ~60
//! lines for this (a macro, a trait, an impl); here once:
//!
//! ```ignore
//! use everyware_ui::model::prelude::*;
//!
//! everyware_ui::ui_model! {
//!     time: String = "#hub/time";
//!     lights: Slot = "#hub/lights" ["SlotCustomizer"];
//!     news: Slot = "#hub/news" interactive ["SlotCustomizer"];
//!     card: Position = "#hub/card" ["FollowPositionCustomizer"] = Position::new((0.0, 1080.0), None);
//! }
//! // UiModel::new(display_map), and UiModel is the harry DesignDocumentRoot: the root frame
//! // of a display is the one the display map names.
//! ```
//!
//! A field is `name: Type = "#node"`, then `interactive` for a node that takes gestures between
//! its children (a list), then the customizer that turns the value into harry's customizations,
//! then `= expr` for a first value other than the type's [`Initial`]. The prelude brings the
//! names the derives expect at the call site (harry's derive macros resolve them there).

use crate::{Slot, TapTarget, Variant};
use customizations::render_customizations::{DisplayState, List};
use customizations::AnimationOverride;

/// The value of a field before the presenter writes it.
pub trait Initial {
    fn initial() -> Self;
}

impl Initial for String {
    fn initial() -> Self {
        String::new()
    }
}

impl Initial for () {
    fn initial() -> Self {}
}

impl Initial for bool {
    fn initial() -> Self {
        false
    }
}

impl Initial for DisplayState {
    fn initial() -> Self {
        DisplayState::Visible
    }
}

/// A variant field starts closed: the state of an overlay before the app opens it.
impl Initial for Variant {
    fn initial() -> Self {
        Variant("state=closed".to_string())
    }
}

impl Initial for AnimationOverride {
    fn initial() -> Self {
        AnimationOverride::DisableAnimation
    }
}

impl Initial for TapTarget {
    fn initial() -> Self {
        TapTarget(false)
    }
}

/// An empty slot shows the design's preview until the presenter fills it.
impl Initial for Slot {
    fn initial() -> Self {
        List::new(Vec::new())
    }
}

/// What a `ui_model!` call site needs in scope: the derives, the traits and types their code
/// names, the library's field types and customizers.
pub mod prelude {
    pub use crate::{FollowOpacityCustomizer, FollowPositionCustomizer, Meter, Position, ScreenCustomizer, ScreenView, Slot, SlotCustomizer, TapTarget, TapTargetCustomizer, Variant};
    pub use customizations::render_customizations::{DisplayState, List};
    pub use customizations::{AnimationOverride, Customizer, CustomizerOutput, DesignDocument, DesignDocumentRoot, UpdatePolicy};
    pub use derive_customizations::DesignDocument;
    pub use har_platform_api::looper::DisplayId;
    pub use har_ui_model_derive::{HarUiModel, NoDesignTokensOrVariables};
    pub use std::collections::HashMap;
}

/// The app's UI model: see the module's documentation.
#[macro_export]
macro_rules! ui_model {
    (@init $ty:tt) => { <$ty as $crate::model::Initial>::initial() };
    (@init $ty:tt, $init:expr) => { $init };
    ($($field:ident: $ty:tt = $node:literal $($interactive:ident)? $([$customizer:literal])? $(= $init:expr)?;)*) => {
        #[derive(DesignDocument, Debug, HarUiModel, NoDesignTokensOrVariables)]
        #[DesignDocument(root)]
        pub struct UiModel {
            customizers: UiModelCustomizers,
            display_map: HashMap<DisplayId, String>,
            $(#[Design(node = $node $(, $interactive)? $(, customizer = $customizer)?)] pub $field: $ty,)*
        }

        impl UiModel {
            /// The model with every field at its first value; `display_map` names the root
            /// frame of each display.
            pub fn new(display_map: HashMap<DisplayId, String>) -> Self {
                UiModel { customizers: Default::default(), display_map, $($field: $crate::ui_model!(@init $ty $(, $init)?),)* }
            }
        }

        impl DesignDocumentRoot for UiModel {
            fn get_root_node(&self, display_id: u32) -> Option<String> {
                self.display_map.get(&DisplayId::new(display_id)).cloned()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_starts_closed_empty_and_without_taps() {
        // what a node shows before the presenter writes it: a closed overlay, no clone in a
        // slot, a scrim that takes no taps, no text
        assert_eq!(Variant::initial(), Variant("state=closed".to_string()));
        assert_eq!(TapTarget::initial(), TapTarget(false));
        assert_eq!(<String as Initial>::initial(), "");
        assert!(<Slot as Initial>::initial().items().is_empty());
    }
}
