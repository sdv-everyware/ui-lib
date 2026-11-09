//! The part every tappable component shares: enabled, pressed, the last change.

use crate::gesture::Gesture;
use crate::transitions::{self, Change, ComponentKind, Transition};

/// Pressed / disabled handling of a tappable component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pressable {
    /// Taps are ignored when false; the component shows its disabled variant.
    pub enabled: bool,
    /// A finger is down on the component (and still over it).
    pub pressed: bool,
    /// The last change of state, for the transition.
    pub last_change: Option<Change>,
}

impl Default for Pressable {
    fn default() -> Self {
        Pressable { enabled: true, pressed: false, last_change: None }
    }
}

impl Pressable {
    /// Applies a gesture; true when it completed a tap (up over the node after a down).
    pub fn on(&mut self, gesture: Gesture) -> bool {
        if !self.enabled {
            return false;
        }
        match gesture {
            Gesture::Down { .. } => {
                self.set_pressed(true);
                false
            }
            // Leaving the node undoes the pressed state; coming back does not restore it.
            Gesture::Move { inside, .. } => {
                if !inside {
                    self.set_pressed(false);
                }
                false
            }
            Gesture::Up { inside, .. } => {
                let tapped = inside && self.pressed;
                self.set_pressed(false);
                tapped
            }
            Gesture::Cancel => {
                self.set_pressed(false);
                false
            }
        }
    }

    fn set_pressed(&mut self, pressed: bool) {
        if self.pressed != pressed {
            self.pressed = pressed;
            self.last_change = Some(if pressed { Change::Press } else { Change::Release });
        }
    }

    /// Enables or disables; a disabled component is never pressed.
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled {
            self.enabled = enabled;
            self.pressed = false;
            self.last_change = Some(Change::Disable);
        }
    }

    /// Opacity target of a pressed layer (`#button/pressed`): 100 while pressed, 0 not. The
    /// layer fades (display-only in harry) instead of a variant change (a tree rebuild).
    pub fn pressed_target(&self) -> f32 {
        if self.pressed {
            100.0
        } else {
            0.0
        }
    }

    /// How the pressed layer of a `kind` fades: press and release have their own curves.
    pub fn press_transition(&self, kind: ComponentKind) -> Option<Transition> {
        transitions::transition(kind, if self.pressed { Change::Press } else { Change::Release })
    }

    /// "default" or "disabled": the state of sets whose pressed look is a layer.
    pub fn enabled_state(&self) -> &'static str {
        if self.enabled {
            "default"
        } else {
            "disabled"
        }
    }

    /// "default", "pressed" or "disabled", the state property of the Figma sets.
    pub fn state(&self) -> &'static str {
        match (self.enabled, self.pressed) {
            (false, _) => "disabled",
            (true, true) => "pressed",
            (true, false) => "default",
        }
    }
}
