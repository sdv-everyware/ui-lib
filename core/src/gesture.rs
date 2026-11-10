//! Gestures per node: down, move, up, cancel, with the finger's position.
//!
//! harry's TapHandler reports only completed and cancelled taps; a pressed state needs the
//! moment the finger goes down, a slider the finger's position. The tracker follows each
//! pointer from start to end and keeps sending its events to the node it started on.

use std::collections::HashMap;

/// A phase of a touch, as the platform reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The finger touched the screen.
    Start,
    /// The finger moved.
    Move,
    /// The finger left the screen.
    End,
}

/// What happened on a node. `x`, `y` are the finger's position on the display; `inside`
/// says whether the finger is still over the node the touch started on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
    /// A finger went down on the node.
    Down {
        /// Finger position on the display.
        x: f32,
        /// Finger position on the display.
        y: f32,
    },
    /// The finger moved.
    Move {
        /// Finger position on the display.
        x: f32,
        /// Finger position on the display.
        y: f32,
        /// Still over the node.
        inside: bool,
    },
    /// The finger went up.
    Up {
        /// Finger position on the display.
        x: f32,
        /// Finger position on the display.
        y: f32,
        /// Over the node: a tap.
        inside: bool,
        /// Speed of the finger's y at the lift, px/s, positive downwards (a list flings
        /// with it); 0 when unknown or when the finger rested before lifting.
        velocity: f32,
        /// The same for x, positive to the right (a horizontal list flings with it).
        velocity_x: f32,
    },
    /// The touch was lost (a new touch of the same pointer started).
    Cancel,
}

impl Gesture {
    /// A down at (0, 0), for tests and components that ignore the position.
    pub const DOWN: Gesture = Gesture::Down { x: 0.0, y: 0.0 };
    /// An up over the node at (0, 0) (a tap), likewise.
    pub const UP: Gesture = Gesture::Up { x: 0.0, y: 0.0, inside: true, velocity: 0.0, velocity_x: 0.0 };

    /// A down at a position.
    pub fn down(x: f32, y: f32) -> Gesture {
        Gesture::Down { x, y }
    }

    /// An up at a position, over the node, without speed.
    pub fn up(x: f32, y: f32) -> Gesture {
        Gesture::Up { x, y, inside: true, velocity: 0.0, velocity_x: 0.0 }
    }
    /// The same gesture with the finger's position moved by (dx, dy): a control inside a
    /// row gets the gesture in the row's coordinates.
    pub fn shifted(self, dx: f32, dy: f32) -> Gesture {
        match self {
            Gesture::Down { x, y } => Gesture::Down { x: x + dx, y: y + dy },
            Gesture::Move { x, y, inside } => Gesture::Move { x: x + dx, y: y + dy, inside },
            Gesture::Up { x, y, inside, velocity, velocity_x } => Gesture::Up { x: x + dx, y: y + dy, inside, velocity, velocity_x },
            Gesture::Cancel => Gesture::Cancel,
        }
    }
}

/// Turns touch phases (pointer, node under it, x) into gestures on nodes.
#[derive(Debug, Default)]
pub struct GestureTracker {
    /// Pointer id -> the node the touch started on.
    active: HashMap<u64, String>,
}

impl GestureTracker {
    /// Feeds one touch phase; returns the gestures it causes, as (node, gesture). `node` is
    /// the node under the finger (empty when there is none: a move or a lift outside every
    /// node still reaches the node the touch started on), `velocity` the finger's speed in
    /// (x, y) at a lift.
    pub fn feed(&mut self, pointer: u64, phase: Phase, node: &str, x: f32, y: f32, velocity: (f32, f32)) -> Vec<(String, Gesture)> {
        match phase {
            Phase::Start => {
                if node.is_empty() {
                    return Vec::new();
                }
                // A pointer that never ended (lost event) is cancelled first.
                let mut out: Vec<_> =
                    self.active.remove(&pointer).map(|n| (n, Gesture::Cancel)).into_iter().collect();
                self.active.insert(pointer, node.to_string());
                out.push((node.to_string(), Gesture::Down { x, y }));
                out
            }
            Phase::Move => match self.active.get(&pointer) {
                Some(start) => vec![(start.clone(), Gesture::Move { x, y, inside: start == node })],
                None => Vec::new(),
            },
            Phase::End => match self.active.remove(&pointer) {
                Some(start) => {
                    let inside = start == node;
                    vec![(start, Gesture::Up { x, y, inside, velocity: velocity.1, velocity_x: velocity.0 })]
                }
                None => Vec::new(),
            },
        }
    }

    /// The node a pointer is on since its down, if any.
    pub fn active(&self, pointer: u64) -> Option<&str> {
        self.active.get(&pointer).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifted_moves_the_finger_and_keeps_the_rest() {
        assert_eq!(Gesture::down(100.0, 50.0).shifted(-96.0, -10.0), Gesture::Down { x: 4.0, y: 40.0 });
        let up = Gesture::Up { x: 10.0, y: 0.0, inside: false, velocity: 1.0, velocity_x: 2.0 };
        assert_eq!(up.shifted(5.0, 5.0), Gesture::Up { x: 15.0, y: 5.0, inside: false, velocity: 1.0, velocity_x: 2.0 });
        assert_eq!(Gesture::Move { x: 1.0, y: 1.0, inside: true }.shifted(1.0, 0.0), Gesture::Move { x: 2.0, y: 1.0, inside: true });
        assert_eq!(Gesture::Cancel.shifted(9.0, 9.0), Gesture::Cancel);
    }

    #[test]
    fn tap_is_down_then_up_inside() {
        let mut t = GestureTracker::default();
        assert_eq!(t.feed(1, Phase::Start, "a", 10.0, 0.0, (0.0, 0.0)), vec![("a".into(), Gesture::Down { x: 10.0, y: 0.0 })]);
        assert_eq!(t.feed(1, Phase::Move, "a", 12.0, 0.0, (0.0, 0.0)), vec![("a".into(), Gesture::Move { x: 12.0, y: 0.0, inside: true })]);
        assert_eq!(t.feed(1, Phase::End, "a", 12.0, 0.0, (0.0, 0.0)), vec![("a".into(), Gesture::Up { x: 12.0, y: 0.0, inside: true, velocity: 0.0, velocity_x: 0.0 })]);
    }

    #[test]
    fn events_stay_with_the_start_node() {
        let mut t = GestureTracker::default();
        t.feed(1, Phase::Start, "slider", 100.0, 0.0, (0.0, 0.0));
        assert_eq!(t.feed(1, Phase::Move, "card", 300.0, 0.0, (0.0, 0.0)), vec![("slider".into(), Gesture::Move { x: 300.0, y: 0.0, inside: false })]);
        assert_eq!(t.feed(1, Phase::End, "card", 310.0, 0.0, (0.0, 0.0)), vec![("slider".into(), Gesture::Up { x: 310.0, y: 0.0, inside: false, velocity: 0.0, velocity_x: 0.0 })]);
    }

    #[test]
    fn two_fingers_are_separate() {
        let mut t = GestureTracker::default();
        t.feed(1, Phase::Start, "a", 0.0, 0.0, (0.0, 0.0));
        t.feed(2, Phase::Start, "b", 0.0, 0.0, (0.0, 0.0));
        assert_eq!(t.feed(2, Phase::End, "b", 0.0, 0.0, (0.0, 0.0)), vec![("b".into(), Gesture::UP)]);
        assert_eq!(t.feed(1, Phase::Start, "c", 0.0, 0.0, (0.0, 0.0))[0], ("a".into(), Gesture::Cancel));
    }
}
