//! Skeleton (#skeleton): a placeholder while content loads, Material's skeleton loader. Grey
//! shapes stand where the content will be (a heading bar, four text lines or a 16:9 picture)
//! and a light band sweeps across them while loading, from the start again after each pass
//! (components.json, change `sweep`). The variant is the shape and the width; the sweep is a
//! meter the harry binding computes from the time (slot.rs `sweep`), so the app only says since
//! when it is loading. Display only.

use crate::transitions::{self, Change, ComponentKind};
use crate::Component;
use std::time::{Duration, Instant};

/// The kind property of #skeleton: which content it stands for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonKind {
    /// `heading`: one thick bar.
    #[default]
    Heading,
    /// `text`: four lines, the last one shorter.
    Text,
    /// `media`: a 16:9 picture.
    Media,
}

/// The size property of #skeleton: its width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonSize {
    /// `regular`: 440 wide.
    #[default]
    Regular,
    /// `wide`: 832 wide (a side panel's 880 content less 24 on each side).
    Wide,
}

/// A skeleton placeholder: its shape, and since when it is loading (None: still).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Skeleton {
    /// What it stands for.
    pub kind: SkeletonKind,
    /// How wide it is.
    pub size: SkeletonSize,
    /// Loading since then: the band sweeps; None: the shapes stand still.
    pub since: Option<Instant>,
}

impl Skeleton {
    /// A still skeleton of `kind` and `size`.
    pub fn new(kind: SkeletonKind, size: SkeletonSize) -> Self {
        Skeleton { kind, size, since: None }
    }

    /// Loading from `now`: the band starts sweeping (a skeleton already loading keeps its
    /// phase, so a second call does not make it jump).
    pub fn start(&mut self, now: Instant) {
        self.since.get_or_insert(now);
    }

    /// Loaded (or given up): the band stops.
    pub fn stop(&mut self) {
        self.since = None;
    }

    /// Whether the band sweeps.
    pub fn is_loading(&self) -> bool {
        self.since.is_some()
    }

    /// The length of one pass of the band (components.json, `sweep`).
    pub fn period() -> Duration {
        transitions::transition(ComponentKind::Skeleton, Change::Sweep).map_or(Duration::from_millis(1400), |t| t.duration)
    }

    /// Where the band is at `now`, 0..100 across the skeleton; None when it stands still.
    pub fn sweep_at(&self, now: Instant) -> Option<f32> {
        let since = self.since?;
        let period = Self::period().as_millis().max(1);
        let ms = now.saturating_duration_since(since).as_millis() % period;
        Some(ms as f32 / period as f32 * 100.0)
    }
}

impl Component for Skeleton {
    const KIND: ComponentKind = ComponentKind::Skeleton;

    fn variant(&self) -> String {
        let kind = match self.kind {
            SkeletonKind::Heading => "heading",
            SkeletonKind::Text => "text",
            SkeletonKind::Media => "media",
        };
        let size = match self.size {
            SkeletonSize::Regular => "regular",
            SkeletonSize::Wide => "wide",
        };
        format!("kind={kind}, size={size}")
    }

    fn last_change(&self) -> Option<Change> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_the_spec_names() {
        let spec: serde_json::Value = serde_json::from_str(include_str!("../../components.json")).unwrap();
        let props = &spec["components"]["skeleton"]["props"];
        assert_eq!(props["kind"], serde_json::json!(["heading", "text", "media"]));
        assert_eq!(props["size"], serde_json::json!(["regular", "wide"]));
        assert_eq!(Skeleton::new(SkeletonKind::Media, SkeletonSize::Wide).variant(), "kind=media, size=wide");
        assert_eq!(Skeleton::default().variant(), "kind=heading, size=regular");
    }

    #[test]
    fn the_band_sweeps_from_the_start_again_while_loading() {
        let t0 = Instant::now();
        let mut s = Skeleton::new(SkeletonKind::Text, SkeletonSize::Wide);
        assert_eq!(s.sweep_at(t0), None);
        s.start(t0);
        assert_eq!(Skeleton::period(), Duration::from_millis(1400));
        assert_eq!(s.sweep_at(t0), Some(0.0));
        assert_eq!(s.sweep_at(t0 + Duration::from_millis(700)), Some(50.0));
        assert_eq!(s.sweep_at(t0 + Duration::from_millis(1400 + 350)), Some(25.0));
        // a second start keeps the phase
        s.start(t0 + Duration::from_millis(500));
        assert_eq!(s.since, Some(t0));
        s.stop();
        assert!(!s.is_loading());
        assert_eq!(s.sweep_at(t0 + Duration::from_millis(100)), None);
    }
}
