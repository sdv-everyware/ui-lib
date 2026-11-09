//! An overlay over the dashboard (a sheet): closed or open, with the open and close transitions
//! of components.json. Two ways to draw the change:
//! - the variant is the state (`variant()`): harry animates between the two drawings, but a
//!   variant change is structural: it builds the tree again and lays the screen out (~70 ms on
//!   the RB3), the first frame of the animation stalls;
//! - the overlay stays in its open variant and its parts move (`slide()`): the card's y as a
//!   position override and the scrim's opacity, both display-only in harry (no new tree, no
//!   layout), each following its target with the transition (everyware_ui Follow customizers).
//! The device sheet slides; one component set per overlay, the same two states in each.

use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;

/// An overlay: closed or open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Overlay {
    /// Shown over the dashboard.
    pub open: bool,
    last_change: Option<Change>,
}

impl Overlay {
    /// A closed overlay.
    pub fn new() -> Self {
        Overlay::default()
    }

    /// Opens it; the change animates with the open transition.
    pub fn open(&mut self) {
        if !self.open {
            self.open = true;
            self.last_change = Some(Change::Open);
        }
    }

    /// Closes it; the change animates with the close transition.
    pub fn close(&mut self) {
        if self.open {
            self.open = false;
            self.last_change = Some(Change::Close);
        }
    }

    /// Where a part that moves with the overlay goes: `open` while it is open, `closed` while
    /// it is closed (a card's y in the open drawing and below the screen, a scrim's opacity).
    pub fn slide(&self, open: f32, closed: f32) -> f32 {
        if self.open { open } else { closed }
    }

    /// The transition of the last change (open or close), for harry's animation of the variant.
    pub fn transition(&self) -> Option<Transition> {
        self.last_change.and_then(|change| transitions::transition(Self::KIND, change))
    }
}

impl Component for Overlay {
    const KIND: ComponentKind = ComponentKind::Overlay;

    fn variant(&self) -> String {
        format!("state={}", if self.open { "open" } else { "closed" })
    }

    fn last_change(&self) -> Option<Change> {
        self.last_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_and_close_with_their_transitions() {
        let mut o = Overlay::new();
        assert_eq!((o.variant().as_str(), o.transition()), ("state=closed", None));
        o.open();
        assert_eq!(o.variant(), "state=open");
        let open = o.transition().expect("open transition");
        o.close();
        let close = o.transition().expect("close transition");
        assert!(open.duration.as_millis() == 550 && close.duration.as_millis() == 400, "{open:?} {close:?}");
    }

    #[test]
    fn a_part_slides_to_its_place() {
        let mut o = Overlay::new();
        assert_eq!(o.slide(120.0, 1120.0), 1120.0);
        o.open();
        assert_eq!(o.slide(120.0, 1120.0), 120.0);
    }
}
