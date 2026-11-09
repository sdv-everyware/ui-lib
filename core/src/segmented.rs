//! Segmented button (#segmented, Material 3): one choice of 2 to 4. The variant is the number
//! of segments and the state (default, disabled); the choice is not a variant (a variant change
//! rebuilds the tree): every segment has its highlight (`#segmented/segment-<i>/selected`) and
//! two labels (`/label` in ink2, `/label-selected` in white), the chosen one is lit by opacity.
//! Each segment (`#segmented/segment-<i>`) takes taps by its own name.

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;

/// The fewest and the most segments of the set.
pub const SEGMENTS: std::ops::RangeInclusive<usize> = 2..=4;

/// A segmented button.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Segmented {
    /// The segments' labels, in order (2 to 4).
    pub labels: Vec<String>,
    /// Index of the chosen segment.
    pub selected: usize,
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
        Segmented { labels, selected: 0, press: Pressable::default(), segments }
    }

    /// The label of the chosen segment.
    pub fn value(&self) -> &str {
        &self.labels[self.selected]
    }

    /// Applies a gesture on segment `index`: a tap on another segment chooses it.
    pub fn on(&mut self, index: usize, gesture: Gesture) -> Option<SegmentedEvent> {
        if !self.press.enabled {
            return None;
        }
        let tapped = self.segments.get_mut(index)?.on(gesture);
        if !tapped || index == self.selected {
            return None;
        }
        self.select(index);
        Some(SegmentedEvent::Selected(index))
    }

    /// Chooses a segment from outside (the device reported it); out of range is ignored.
    pub fn select(&mut self, index: usize) {
        if index < self.labels.len() && index != self.selected {
            self.selected = index;
            self.press.last_change = Some(Change::Value);
        }
    }

    /// Opacity target of segment `index`'s highlight and white label: 100 chosen, 0 not (the
    /// ink2 label takes the rest).
    pub fn target(&self, index: usize) -> f32 {
        if index == self.selected {
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
