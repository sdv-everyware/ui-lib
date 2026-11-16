//! Transitions between variants, from components.json (the same file the Figma generator
//! reads, so the code and the design cannot drift apart).

use serde::Deserialize;
use std::sync::OnceLock;
use std::time::Duration;

const SPEC: &str = include_str!("../../components.json");

/// The component sets of the library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    /// #button
    Button,
    /// #icon-button
    IconButton,
    /// #toggle
    Toggle,
    /// #radio
    Radio,
    /// #dropdown
    Dropdown,
    /// #list-item
    ListItem,
    /// #slider
    Slider,
    /// #input-field
    InputField,
    /// #bar-chart: a row of bars with a level and an accent
    BarChart,
    /// a component set of the app's own design (its pictures, its content): the app implements
    /// `Component` for its type with this kind; the library has no transition for it
    App,
    /// an overlay (a sheet set of the app's design, state closed/open)
    Overlay,
    /// #keyboard (one component; its keys fade a pressed layer)
    Keyboard,
    /// a form: an overlay with input fields (a sheet set of the app's design)
    Form,
    /// the screen's keyboard dock (#keyboard-dock): open while a field has the focus
    KeyboardDock,
    /// #scrollbar: the thumb of a scrolling list
    Scrollbar,
    /// #sheet: the library's overlay container, a dialog or a panel at an edge (layout, state)
    Sheet,
    /// #segmented: one choice of 2 to 4
    Segmented,
    /// #symbol: a room's or a device's symbol tile
    Symbol,
    /// #skeleton: a placeholder while content loads, a light band sweeping across it
    Skeleton,
    /// #tile: a device's card
    Tile,
    /// #tabs: the screen's navigation, one tab per screen
    Tabs,
    /// pages that take turns in one place without tabs (screens.rs, Pages)
    Pages,
}

/// A change of state that may be animated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    /// default -> pressed
    Press,
    /// pressed -> default
    Release,
    /// off <-> on
    Value,
    /// closed -> open
    Open,
    /// open -> closed
    Close,
    /// a new item is selected
    Select,
    /// not focused -> focused
    Focus,
    /// focused -> not focused
    Blur,
    /// shown -> hidden (a scrollbar after the content stopped)
    Hide,
    /// enabled <-> disabled: never animated
    Disable,
    /// one pass of a repeated sweep (a skeleton's shimmer), from the start again after it
    Sweep,
    /// one screen -> another (a tap on a tab): the screens' effect splits it (screens.rs)
    Switch,
}

/// The easing curves of components.json.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Easing {
    /// Constant speed.
    Linear,
    /// Starts slow.
    EaseIn,
    /// Ends slow.
    EaseOut,
    /// Starts and ends slow.
    EaseInOut,
    /// A critically damped spring, as iOS moves its sheets: fast at once, then it settles
    /// without overshoot (follow::ease).
    Spring,
}

/// How a change of variant is animated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    /// Length of the animation.
    pub duration: Duration,
    /// Curve of the animation.
    pub easing: Easing,
    /// How long the value waits before it moves (a screen that fades in after the old one
    /// faded out); zero for every transition of components.json.
    pub delay: Duration,
}

impl Transition {
    /// The same curve over a part of this transition: `start` and `length` are fractions of
    /// its duration (an effect that runs two moves one after the other, screens.rs).
    pub fn part(self, start: f32, length: f32, easing: Easing) -> Transition {
        Transition { duration: self.duration.mul_f32(length), easing, delay: self.delay + self.duration.mul_f32(start) }
    }
}

#[derive(Deserialize)]
struct Spec {
    transitions: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    components: Vec<ComponentKind>,
    change: Change,
    ms: u64,
    easing: Easing,
}

fn spec() -> &'static [Entry] {
    static SPEC_PARSED: OnceLock<Vec<Entry>> = OnceLock::new();
    SPEC_PARSED.get_or_init(|| {
        serde_json::from_str::<Spec>(SPEC).expect("components.json is part of the build").transitions
    })
}

/// The transition of a component for a change; None means the change is not animated.
pub fn transition(component: ComponentKind, change: Change) -> Option<Transition> {
    if change == Change::Disable {
        return None;
    }
    spec().iter().find(|e| e.change == change && e.components.contains(&component)).map(|e| {
        Transition { duration: Duration::from_millis(e.ms), easing: e.easing, delay: Duration::ZERO }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_the_spec() {
        let t = transition(ComponentKind::Toggle, Change::Value).unwrap();
        assert_eq!((t.duration.as_millis(), t.easing), (200, Easing::EaseInOut));
        let t = transition(ComponentKind::Button, Change::Press).unwrap();
        assert_eq!((t.duration.as_millis(), t.easing), (80, Easing::EaseOut));
        assert_eq!(transition(ComponentKind::Button, Change::Open), None);
        assert_eq!(transition(ComponentKind::Toggle, Change::Disable), None);
        let t = transition(ComponentKind::Tabs, Change::Switch).unwrap();
        assert_eq!((t.duration.as_millis(), t.easing, t.delay), (300, Easing::EaseOut, Duration::ZERO));
        assert_eq!(transition(ComponentKind::Pages, Change::Switch), Some(t), "pages switch as the tabs' screens");
    }

    #[test]
    fn a_part_of_a_transition_starts_later() {
        let t = Transition { duration: Duration::from_millis(300), easing: Easing::EaseOut, delay: Duration::ZERO };
        let p = t.part(0.3, 0.7, Easing::EaseIn);
        assert_eq!((p.duration.as_millis(), p.delay.as_millis(), p.easing), (210, 90, Easing::EaseIn));
    }
}
