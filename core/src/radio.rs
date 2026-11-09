//! Radio buttons (#radio): a group where one option is on.

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{Change, ComponentKind};
use crate::Component;

/// One radio button; part of a RadioGroup.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Radio {
    /// The text next to it.
    pub label: String,
    /// This option is the selected one.
    pub on: bool,
    /// Pressed and disabled.
    pub press: Pressable,
}

impl Component for Radio {
    const KIND: ComponentKind = ComponentKind::Radio;

    fn variant(&self) -> String {
        format!("value={}, state={}", if self.on { "on" } else { "off" }, self.press.state())
    }

    fn last_change(&self) -> Option<Change> {
        self.press.last_change
    }
}

/// A group of radio buttons with one selected.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RadioGroup {
    /// The options, in order.
    pub options: Vec<Radio>,
}

/// What a radio group reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioEvent {
    /// The user picked the option with this index.
    Selected(usize),
}

impl RadioGroup {
    /// A group with these labels; the first is selected.
    pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self {
        let options: Vec<Radio> = labels
            .into_iter()
            .enumerate()
            .map(|(i, l)| Radio { label: l.into(), on: i == 0, press: Pressable::default() })
            .collect();
        RadioGroup { options }
    }

    /// The index of the selected option.
    pub fn selected(&self) -> Option<usize> {
        self.options.iter().position(|o| o.on)
    }

    /// Applies a gesture on option `index`.
    pub fn on(&mut self, index: usize, gesture: Gesture) -> Option<RadioEvent> {
        let tapped = self.options.get_mut(index)?.press.on(gesture);
        if !tapped || self.options[index].on {
            return None;
        }
        for (i, o) in self.options.iter_mut().enumerate() {
            if o.on != (i == index) {
                o.on = i == index;
                o.press.last_change = Some(Change::Value);
            }
        }
        Some(RadioEvent::Selected(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_on_at_a_time() {
        let mut g = RadioGroup::new(["Auto", "Cool", "Heat"]);
        g.on(2, Gesture::DOWN);
        assert_eq!(g.on(2, Gesture::UP), Some(RadioEvent::Selected(2)));
        assert_eq!(g.selected(), Some(2));
        assert_eq!(g.options[0].variant(), "value=off, state=default");
        assert_eq!(g.options[2].variant(), "value=on, state=default");
        g.on(2, Gesture::DOWN);
        assert_eq!(g.on(2, Gesture::UP), None);
    }
}
