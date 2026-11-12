//! Tabs (#tabs, Material 3): the screen's navigation, one tab per screen. The variant is the
//! number of tabs and the state (default, disabled); the chosen tab is not a variant (a variant
//! change rebuilds the tree): every tab has its pill (`#tabs/tab-<i>/selected`) and two labels
//! (`/label` in ink2, `/label-selected` in the accent), the chosen one is lit by opacity. Each
//! tab (`#tabs/tab-<i>`) takes taps by its own name.
//!
//! Tabs only know which tab is chosen and which one was before it. What the screens do on a
//! switch is the screens' effect (screens.rs).

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::screens::{Effect, ScreenTarget};
use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;

/// The fewest and the most tabs of the set.
pub const TABS: std::ops::RangeInclusive<usize> = 2..=4;

/// A tab bar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tabs {
    /// The tabs' labels, in order (2 to 4).
    pub labels: Vec<String>,
    /// Index of the chosen tab.
    pub selected: usize,
    /// The tab chosen before the last switch: the screen that leaves (a slide's direction).
    pub previous: Option<usize>,
    /// Enabled, and the last change; a tab's press is in `tabs`.
    pub press: Pressable,
    /// One press per tab: a tap is a down and an up on the same tab.
    tabs: Vec<Pressable>,
}

/// What tabs report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabsEvent {
    /// The user chose the tab with this index.
    Selected(usize),
}

impl Tabs {
    /// Tabs with these labels (2 to 4), the first chosen.
    pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        assert!(TABS.contains(&labels.len()), "#tabs has 2 to 4 tabs, not {}", labels.len());
        let tabs = vec![Pressable::default(); labels.len()];
        Tabs { labels, selected: 0, previous: None, press: Pressable::default(), tabs }
    }

    /// Applies a gesture on tab `index`: a tap on another tab chooses it.
    pub fn on(&mut self, index: usize, gesture: Gesture) -> Option<TabsEvent> {
        if !self.press.enabled {
            return None;
        }
        let tapped = self.tabs.get_mut(index)?.on(gesture);
        if !tapped || index == self.selected {
            return None;
        }
        self.select(index);
        Some(TabsEvent::Selected(index))
    }

    /// Chooses a tab from outside (the app opens a screen itself); out of range or the chosen
    /// one is ignored.
    pub fn select(&mut self, index: usize) {
        if index < self.labels.len() && index != self.selected {
            self.previous = Some(self.selected);
            self.selected = index;
            self.press.last_change = Some(Change::Value);
        }
    }

    /// Opacity target of tab `index`'s pill and accent label: 100 chosen, 0 not (the ink2
    /// label takes the rest).
    pub fn target(&self, index: usize) -> f32 {
        if index == self.selected {
            100.0
        } else {
            0.0
        }
    }

    /// How the pill and the labels move to a new tab.
    pub fn value_transition(&self) -> Option<Transition> {
        transitions::transition(Self::KIND, Change::Value)
    }

    /// The time of a screen switch, which the effect splits (components.json, tabs/switch).
    pub fn switch_transition(&self) -> Option<Transition> {
        transitions::transition(Self::KIND, Change::Switch)
    }

    /// Where the screen of tab `screen` goes with this effect: its opacity and its offset from
    /// where the design puts it. `width` is the distance between two screens of a slide.
    pub fn screen(&self, screen: usize, effect: Effect, width: f32) -> ScreenTarget {
        effect.target(screen, self.selected, self.previous, width, self.switch_transition())
    }
}

impl Component for Tabs {
    const KIND: ComponentKind = ComponentKind::Tabs;

    fn variant(&self) -> String {
        format!("tabs={}, state={}", self.labels.len(), self.press.enabled_state())
    }

    fn last_change(&self) -> Option<Change> {
        self.press.last_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tap(t: &mut Tabs, i: usize) -> Option<TabsEvent> {
        t.on(i, Gesture::DOWN);
        t.on(i, Gesture::UP)
    }

    #[test]
    fn a_tap_chooses_and_remembers_the_tab_before() {
        let mut t = Tabs::new(["Overview", "Rooms", "Automations"]);
        assert_eq!((t.variant().as_str(), t.selected, t.previous, t.target(0)), ("tabs=3, state=default", 0, None, 100.0));
        assert_eq!(tap(&mut t, 2), Some(TabsEvent::Selected(2)));
        assert_eq!((t.selected, t.previous, t.target(0), t.target(2)), (2, Some(0), 0.0, 100.0));
        assert_eq!(t.value_transition().unwrap().duration.as_millis(), 160);
        assert_eq!(t.switch_transition().unwrap().duration.as_millis(), 300);
        assert_eq!(t.variant().as_str(), "tabs=3, state=default");
    }

    #[test]
    fn the_chosen_tab_an_up_alone_and_out_of_range_do_nothing() {
        let mut t = Tabs::new(["A", "B"]);
        assert_eq!(tap(&mut t, 0), None);
        assert_eq!(t.on(1, Gesture::UP), None);
        assert_eq!(tap(&mut t, 5), None);
        t.select(7);
        assert_eq!((t.selected, t.previous), (0, None));
    }

    #[test]
    fn disabled_takes_no_taps() {
        let mut t = Tabs::new(["A", "B"]);
        t.press.set_enabled(false);
        assert_eq!((tap(&mut t, 1), t.variant().as_str()), (None, "tabs=2, state=disabled"));
    }

    #[test]
    #[should_panic]
    fn one_tab_is_not_a_navigation() {
        Tabs::new(["Only"]);
    }
}
