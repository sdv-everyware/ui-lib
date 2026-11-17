//! Segmented button (#segmented, Material 3): one choice of 2 to 4. The variant is the number
//! of segments and the state (default, disabled); the choice is not a variant (a variant change
//! rebuilds the tree): every segment has its highlight (`#segmented/segment-<i>/selected`) and
//! two labels (`/label` in ink2, `/label-selected` in white), the chosen one is lit by opacity.
//! Each segment (`#segmented/segment-<i>`) takes taps by its own name. A choice not made yet is a
//! state of its own (`clear`): no segment lit, every segment takes a tap. A segment may have a
//! tone (`tones`): chosen, it is lit in the tone's colour (`/selected-<tone>`) instead of the
//! accent, its label white or, on the warning's yellow, ink (`/label-selected-ink`).

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::tone::Tone;
use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;

/// The fewest and the most segments of the set.
pub const SEGMENTS: std::ops::RangeInclusive<usize> = 2..=4;

/// A segmented button.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Segmented {
    /// The segments' labels, in order (2 to 4).
    pub labels: Vec<String>,
    /// Index of the chosen segment; no meaning while `cleared`.
    pub selected: usize,
    /// No segment is chosen (a choice not made yet): none is lit, `chosen()` is None, `value()`
    /// is empty and every segment takes a tap. Set with `clear`, ended by `select` or a tap.
    pub cleared: bool,
    /// Per segment its tone when it is chosen (`Tone::Default`: the accent).
    pub tones: Vec<Tone>,
    /// Enabled, and the last change; a segment's press is in `segments`.
    pub press: Pressable,
    /// One press per segment: a tap is a down and an up on the same segment.
    segments: Vec<Pressable>,
}

/// What a segmented button reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentedEvent {
    /// The user chose the segment with this index.
    Selected(usize),
}

impl Segmented {
    /// A segmented button with these labels (2 to 4), the first chosen.
    pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        assert!(SEGMENTS.contains(&labels.len()), "#segmented has 2 to 4 segments, not {}", labels.len());
        let segments = vec![Pressable::default(); labels.len()];
        let tones = vec![Tone::Default; labels.len()];
        Segmented { labels, selected: 0, cleared: false, tones, press: Pressable::default(), segments }
    }

    /// The same segmented button with a tone per segment, in the labels' order.
    pub fn tones(mut self, tones: impl IntoIterator<Item = Tone>) -> Self {
        let tones: Vec<Tone> = tones.into_iter().collect();
        assert_eq!(tones.len(), self.labels.len(), "one tone per segment");
        self.tones = tones;
        self
    }

    /// The tone of segment `index` (the default out of range).
    pub fn tone_of(&self, index: usize) -> Tone {
        self.tones.get(index).copied().unwrap_or_default()
    }

    /// The label of the chosen segment; empty while none is chosen.
    pub fn value(&self) -> &str {
        self.chosen().and_then(|i| self.labels.get(i)).map_or("", String::as_str)
    }

    /// The index of the chosen segment, or None while none is chosen.
    pub fn chosen(&self) -> Option<usize> {
        (!self.cleared && self.selected < self.labels.len()).then_some(self.selected)
    }

    /// No segment chosen from outside (the value is not known, or not given yet): none lit.
    pub fn clear(&mut self) {
        if !self.cleared {
            self.cleared = true;
            self.press.last_change = Some(Change::Value);
        }
    }

    /// Applies a gesture on segment `index`: a tap on another segment chooses it.
    pub fn on(&mut self, index: usize, gesture: Gesture) -> Option<SegmentedEvent> {
        if !self.press.enabled {
            return None;
        }
        let tapped = self.segments.get_mut(index)?.on(gesture);
        if !tapped || self.chosen() == Some(index) {
            return None;
        }
        self.select(index);
        Some(SegmentedEvent::Selected(index))
    }

    /// Chooses a segment from outside (the device reported it); out of range is ignored.
    pub fn select(&mut self, index: usize) {
        if index < self.labels.len() && self.chosen() != Some(index) {
            self.selected = index;
            self.cleared = false;
            self.press.last_change = Some(Change::Value);
        }
    }

    /// Opacity target of segment `index`'s highlight and white label: 100 chosen, 0 not (the
    /// ink2 label takes the rest); 0 for every segment while none is chosen.
    pub fn target(&self, index: usize) -> f32 {
        if self.chosen() == Some(index) {
            100.0
        } else {
            0.0
        }
    }

    /// How the highlight and the labels move to a new choice.
    pub fn value_transition(&self) -> Option<Transition> {
        transitions::transition(Self::KIND, Change::Value)
    }
}

impl Component for Segmented {
    const KIND: ComponentKind = ComponentKind::Segmented;

    fn variant(&self) -> String {
        format!("segments={}, state={}", self.labels.len(), self.press.enabled_state())
    }

    fn last_change(&self) -> Option<Change> {
        self.press.last_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tap_chooses_without_a_variant_change() {
        let mut s = Segmented::new(["Low", "Med", "High"]);
        assert_eq!((s.variant().as_str(), s.value(), s.target(0)), ("segments=3, state=default", "Low", 100.0));
        s.on(2, Gesture::DOWN);
        assert_eq!(s.on(2, Gesture::UP), Some(SegmentedEvent::Selected(2)));
        assert_eq!((s.value(), s.target(0), s.target(2), s.variant().as_str()), ("High", 0.0, 100.0, "segments=3, state=default"));
        assert_eq!(s.value_transition().unwrap().duration.as_millis(), 160);
        // the chosen one again, an up without a down, another segment's up: nothing
        s.on(2, Gesture::DOWN);
        assert_eq!(s.on(2, Gesture::UP), None);
        assert_eq!(s.on(1, Gesture::UP), None);
    }

    #[test]
    fn nothing_chosen_lights_none_and_every_segment_takes_a_tap() {
        let mut s = Segmented::new(["Yes", "Maybe", "No"]);
        s.clear();
        assert_eq!(s.press.last_change, Some(Change::Value));
        assert_eq!((s.chosen(), s.value(), s.variant().as_str()), (None, "", "segments=3, state=default"));
        assert_eq!((0..3).map(|i| s.target(i)).collect::<Vec<_>>(), [0.0, 0.0, 0.0]);
        // the segment that was chosen before takes a tap too
        s.on(0, Gesture::DOWN);
        assert_eq!(s.on(0, Gesture::UP), Some(SegmentedEvent::Selected(0)));
        assert_eq!((s.chosen(), s.value(), s.target(0)), (Some(0), "Yes", 100.0));
        // chosen from outside after a clear, the same index too
        s.clear();
        s.select(0);
        assert_eq!(s.chosen(), Some(0));
        // an index out of range is no choice, never a panic
        s.selected = 7;
        assert_eq!((s.chosen(), s.value(), s.target(7)), (None, "", 0.0));
    }

    #[test]
    fn a_segment_has_its_tone() {
        let s = Segmented::new(["Yes", "Maybe", "No"]).tones([Tone::Success, Tone::Warning, Tone::Error]);
        assert_eq!((0..4).map(|i| s.tone_of(i)).collect::<Vec<_>>(), [Tone::Success, Tone::Warning, Tone::Error, Tone::Default]);
        // the tone does not change the variant: the colours are layers lit by opacity
        assert_eq!(s.variant(), "segments=3, state=default");
        assert_eq!(Segmented::new(["A", "B"]).tone_of(1), Tone::Default);
    }

    #[test]
    #[should_panic(expected = "one tone per segment")]
    fn one_tone_per_segment() {
        let _ = Segmented::new(["A", "B"]).tones([Tone::Error]);
    }

    #[test]
    fn disabled_takes_no_taps() {
        let mut s = Segmented::new(["A", "B"]);
        s.press.set_enabled(false);
        s.on(1, Gesture::DOWN);
        assert_eq!((s.on(1, Gesture::UP), s.variant().as_str()), (None, "segments=2, state=disabled"));
    }

    #[test]
    #[should_panic]
    fn one_segment_is_not_a_choice() {
        Segmented::new(["Only"]);
    }
}
