//! Input field (#input-field). The variant is only the state (default, disabled, error); focus
//! fades in a ring (#input-field/focus opacity), the caret (#input-field/caret) blinks by
//! opacity while focused, the placeholder shows while the text is empty. Texts are set by code.

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;
use std::time::Instant;

/// How long the caret is on, and then off.
pub const BLINK_MS: u64 = 530;

/// A one-line text field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputField {
    /// The text the user entered.
    pub text: String,
    /// Shown while the text is empty.
    pub placeholder: String,
    /// Shown under the field while there is no error.
    pub helper: String,
    /// An error: the field turns red and shows it instead of the helper.
    pub error: Option<String>,
    /// Shows dots instead of the characters (passwords).
    pub secure: bool,
    /// Takes the keyboard's input.
    pub focused: bool,
    /// Disabled; taps do not focus it.
    pub press: Pressable,
    /// When the caret starts its blink: on focus and on every edit, so it stays on while typing.
    caret_since: Option<Instant>,
    last_focus_change: Option<Change>,
}

/// What an input field reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    /// A tap focused it.
    Focused,
}

impl InputField {
    /// An empty, enabled field.
    pub fn new(placeholder: impl Into<String>, helper: impl Into<String>) -> Self {
        InputField { placeholder: placeholder.into(), helper: helper.into(), ..Default::default() }
    }

    /// Applies a gesture: a tap focuses the field.
    pub fn on(&mut self, gesture: Gesture, now: Instant) -> Option<InputEvent> {
        if !self.press.on(gesture) || self.focused {
            return None;
        }
        self.focus(now);
        Some(InputEvent::Focused)
    }

    /// Takes the focus (the keyboard writes here).
    pub fn focus(&mut self, now: Instant) {
        if self.press.enabled && !self.focused {
            self.focused = true;
            self.caret_since = Some(now);
            self.last_focus_change = Some(Change::Focus);
        }
    }

    /// Gives the focus away.
    pub fn blur(&mut self) {
        if self.focused {
            self.focused = false;
            self.caret_since = None;
            self.last_focus_change = Some(Change::Blur);
        }
    }

    /// The text was changed (by the keyboard): the caret restarts on, an error goes away.
    pub fn edited(&mut self, now: Instant) {
        self.error = None;
        if self.focused {
            self.caret_since = Some(now);
        }
    }

    /// The text to show: dots for a secure field.
    pub fn shown(&self) -> String {
        if self.secure {
            "•".repeat(self.text.chars().count())
        } else {
            self.text.clone()
        }
    }

    /// The line under the field: the error, else the helper.
    pub fn helper_line(&self) -> &str {
        self.error.as_deref().unwrap_or(&self.helper)
    }

    /// Opacity target of the focus ring: 100 focused, 0 not.
    pub fn focus_target(&self) -> f32 {
        if self.focused {
            100.0
        } else {
            0.0
        }
    }

    /// How the ring fades: focus and blur have their own curves.
    pub fn focus_transition(&self) -> Option<Transition> {
        transitions::transition(Self::KIND, self.last_focus_change.unwrap_or(Change::Focus))
    }

    /// Opacity target of the placeholder: 100 while the text is empty.
    pub fn placeholder_target(&self) -> f32 {
        if self.text.is_empty() {
            100.0
        } else {
            0.0
        }
    }

    /// Since when the caret blinks; none while not focused (no caret).
    pub fn caret_since(&self) -> Option<Instant> {
        self.caret_since
    }

    /// Whether the caret is on at `now`: on for BLINK_MS, off for BLINK_MS, from `caret_since`.
    pub fn caret_on(&self, now: Instant) -> bool {
        self.caret_since.is_some_and(|since| caret_phase_on(since, now))
    }
}

/// On for BLINK_MS, then off for BLINK_MS, counted from `since`.
pub fn caret_phase_on(since: Instant, now: Instant) -> bool {
    let ms = now.saturating_duration_since(since).as_millis() as u64;
    (ms / BLINK_MS) % 2 == 0
}

impl Component for InputField {
    const KIND: ComponentKind = ComponentKind::InputField;

    fn variant(&self) -> String {
        // No pressed state: a tap only focuses.
        let state = if !self.press.enabled {
            "disabled"
        } else if self.error.is_some() {
            "error"
        } else {
            "default"
        };
        format!("state={state}")
    }

    fn last_change(&self) -> Option<Change> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn tap_focuses_typing_keeps_caret_on() {
        let t0 = Instant::now();
        let mut f = InputField::new("Enter password", "8 characters or more");
        f.secure = true;
        f.on(Gesture::DOWN, t0);
        assert_eq!(f.on(Gesture::UP, t0), Some(InputEvent::Focused));
        assert_eq!((f.focus_target(), f.placeholder_target()), (100.0, 100.0));
        assert_eq!(f.focus_transition().unwrap().duration.as_millis(), 150);
        assert!(f.caret_on(t0) && !f.caret_on(t0 + Duration::from_millis(600)));
        f.text.push_str("abc");
        f.edited(t0 + Duration::from_millis(600));
        assert!(f.caret_on(t0 + Duration::from_millis(700)), "on again after an edit");
        assert_eq!((f.shown().as_str(), f.placeholder_target()), ("•••", 0.0));
        f.blur();
        assert!(!f.caret_on(t0) && f.focus_target() == 0.0);
    }

    #[test]
    fn error_is_the_variant_until_the_next_edit() {
        let mut f = InputField::new("", "hint");
        f.error = Some("At least 8 characters".into());
        assert_eq!((f.variant().as_str(), f.helper_line()), ("state=error", "At least 8 characters"));
        f.edited(Instant::now());
        assert_eq!((f.variant().as_str(), f.helper_line()), ("state=default", "hint"));
        f.press.set_enabled(false);
        assert_eq!(f.variant(), "state=disabled");
    }
}
