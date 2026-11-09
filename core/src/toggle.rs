//! Toggle (#toggle). The variant is only the state (pressed, disabled); the value moves the
//! thumb (progress marker meter, #toggle/thumb) and fades the blue track (#toggle/on), both
//! display-only in harry.

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;

/// An on/off switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Toggle {
    /// On or off.
    pub value: bool,
    /// Pressed and disabled.
    pub press: Pressable,
}

/// What a toggle reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToggleEvent {
    /// The user switched it; the new value.
    Toggled(bool),
}

impl Toggle {
    /// A new, enabled toggle.
    pub fn new(value: bool) -> Self {
        Toggle { value, press: Pressable::default() }
    }

    /// Applies a gesture: a tap flips the value.
    pub fn on(&mut self, gesture: Gesture) -> Option<ToggleEvent> {
        if !self.press.on(gesture) {
            return None;
        }
        self.value = !self.value;
        Some(ToggleEvent::Toggled(self.value))
    }

    /// Sets the value from outside (the device reported it).
    pub fn set(&mut self, value: bool) {
        self.value = value;
    }

    /// Where the thumb and the blue track go: 0 (off) or 100 (on), for meter and opacity.
    pub fn target(&self) -> f32 {
        if self.value {
            100.0
        } else {
            0.0
        }
    }

    /// How the thumb and the track move to a new target.
    pub fn value_transition(&self) -> Option<Transition> {
        transitions::transition(Self::KIND, Change::Value)
    }
}

impl Component for Toggle {
    const KIND: ComponentKind = ComponentKind::Toggle;

    fn variant(&self) -> String {
        format!("state={}", self.press.state())
    }

    fn last_change(&self) -> Option<Change> {
        self.press.last_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_flips_value_not_variant() {
        let mut t = Toggle::new(false);
        t.on(Gesture::DOWN);
        assert_eq!(t.variant(), "state=pressed");
        assert_eq!(t.on(Gesture::UP), Some(ToggleEvent::Toggled(true)));
        assert_eq!((t.variant().as_str(), t.target()), ("state=default", 100.0));
        assert_eq!(t.value_transition().unwrap().duration.as_millis(), 200);
    }
}
