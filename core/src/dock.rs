//! The keyboard dock: the screen's one keyboard, an overlay at the bottom that is open while
//! a field somewhere on the screen has the focus (as a system keyboard, not a part of each
//! sheet). The keys are the `#keyboard` component in the dock's slot. A form types through
//! it: `Form::on(.., &mut dock.keyboard)`.
//!
//! The dock moves as a sheet's card does (overlay.rs): the instance on the screen stays in
//! its `state=closed` drawing and the app moves the slot with the keys, a position override
//! that follows `keys_position()` with the dock's transition. A switch of the variant would
//! build the tree again and lay the screen out on every frame of the animation (the keys are
//! some forty components with texts): on the RB3 that was frames of 100 to 170 ms while the
//! dock opened, against 17 ms for a sheet. `variant()` still names the state, for tests and
//! the contract check.

use crate::keyboard::Keyboard;
use crate::overlay::Overlay;
use crate::sheet::CLOSED_GAP;
use crate::transitions::{Change, ComponentKind};
use crate::Component;

/// The dock's height on the screen (keyboard-patch.js KBD_H): the area above it is what a
/// dialog has while the dock is open (sheet.rs).
pub const KEYBOARD_DOCK_H: f32 = 420.0;

/// The screen's keyboard: an overlay with the keys.
#[derive(Clone, Debug, Default)]
pub struct KeyboardDock {
    /// Closed (below the screen) or open (docked at the bottom).
    pub overlay: Overlay,
    /// The keys: labels per mode, pressed keys, typing.
    pub keyboard: Keyboard,
}

impl KeyboardDock {
    /// A closed dock.
    pub fn new() -> Self {
        KeyboardDock::default()
    }

    /// Opens the dock while a field has the focus and closes it when none has: the screen
    /// calls it after every action with `form.focused().is_some()` (or several forms' or'ed).
    pub fn follow(&mut self, focus: bool) {
        if focus {
            self.overlay.open();
        } else {
            self.overlay.close();
        }
    }

    /// The dock is open: the keys are on the screen.
    pub fn open(&self) -> bool {
        self.overlay.open
    }

    /// Where the slot with the keys is now, on a screen of `(w, h)`: docked at the bottom while
    /// open, `CLOSED_GAP` below the screen's edge while closed (where the design draws it).
    /// The app gives both coordinates to the slot as a position override
    /// (FollowPositionCustomizer) with `transition()`.
    pub fn keys_position(&self, screen: (f32, f32)) -> (f32, f32) {
        (0.0, self.overlay.slide(screen.1 - KEYBOARD_DOCK_H, screen.1 + CLOSED_GAP))
    }
}

impl Component for KeyboardDock {
    const KIND: ComponentKind = ComponentKind::KeyboardDock;

    fn variant(&self) -> String {
        self.overlay.variant()
    }

    fn last_change(&self) -> Option<Change> {
        self.overlay.last_change()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_focus_with_the_open_and_close_transitions() {
        let mut d = KeyboardDock::new();
        assert_eq!((d.variant().as_str(), d.transition()), ("state=closed", None));
        d.follow(true);
        assert_eq!(d.variant(), "state=open");
        assert_eq!(d.transition().map(|t| t.duration.as_millis()), Some(550));
        d.follow(true);
        assert_eq!(d.variant(), "state=open");
        d.follow(false);
        assert_eq!((d.variant().as_str(), d.transition().map(|t| t.duration.as_millis())), ("state=closed", Some(400)));
    }

    #[test]
    fn the_keys_slide_between_the_screen_edge_and_the_bottom() {
        let mut d = KeyboardDock::new();
        assert_eq!(d.keys_position((1920.0, 1080.0)), (0.0, 1120.0), "closed: 40 below the edge, as drawn");
        d.follow(true);
        assert_eq!(d.keys_position((1920.0, 1080.0)), (0.0, 660.0), "open: 420 high at the bottom");
        d.follow(false);
        assert_eq!(d.keys_position((1920.0, 1080.0)), (0.0, 1120.0));
    }
}
