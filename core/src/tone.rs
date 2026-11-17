// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The tone of a component: the colour role it takes beside its look, as Material 3 gives a
//! component the error role, here with success and warning too. A button takes a tone as a
//! whole (`Button::tone`); a segmented button per segment, for the segment that is chosen
//! (`Segmented::tones`). The colours are the design's (fig-helpers.js `C`: `success`, `warning`,
//! `error`, their soft and pressed shades); every label passes WCAG AA on its fill.

/// A colour role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tone {
    /// The component's own colours (the accent).
    #[default]
    Default,
    /// Green: something good or done.
    Success,
    /// Yellow: take care, not sure.
    Warning,
    /// Red: something that cannot be undone, or failed (Material 3's error role).
    Error,
}

impl Tone {
    /// Every tone, in the order of the design.
    pub const ALL: [Tone; 4] = [Tone::Default, Tone::Success, Tone::Warning, Tone::Error];

    /// Its name in a variant or a layer ("success"); empty for the default.
    pub fn name(self) -> &'static str {
        match self {
            Tone::Default => "",
            Tone::Success => "success",
            Tone::Warning => "warning",
            Tone::Error => "error",
        }
    }

    /// The label on the tone's strong fill is dark (ink), not white: the warning's yellow is too
    /// light for a white label.
    pub fn dark_label(self) -> bool {
        self == Tone::Warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_labels() {
        assert_eq!(Tone::ALL.map(Tone::name), ["", "success", "warning", "error"]);
        assert_eq!(Tone::ALL.map(Tone::dark_label), [false, false, true, false]);
        assert_eq!(Tone::default(), Tone::Default);
    }
}
