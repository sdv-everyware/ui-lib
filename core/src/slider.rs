//! Slider (#slider). The variant is only the state; the value (0..100) moves #slider/fill
//! (progress bar meter) and #slider/thumb (progress marker meter).

use crate::gesture::Gesture;
use crate::pressable::Pressable;
use crate::transitions::{self, Change, ComponentKind, Transition};
use crate::Component;

/// How far the track starts inside the #slider component, on both sides (components-patch.js).
pub const TRACK_INSET: f32 = 20.0;

/// A horizontal slider over a track at a known place on the display.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Slider {
    /// 0..100.
    pub value: f32,
    /// Steps of the value; 0 for none.
    pub step: f32,
    /// Display x of the track's start (where the thumb's centre is at 0).
    pub track_x: f32,
    /// Track length on the display (thumb centre travel from 0 to 100).
    pub track_width: f32,
    /// Pressed and disabled.
    pub press: Pressable,
    dragging: bool,
}

/// What a slider reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderEvent {
    /// The value while dragging.
    Moving(f32),
    /// The value when the finger went up.
    Changed(f32),
}

impl Slider {
    /// A slider at `value` in a slot whose left edge is `slot_x` on the display and which is
    /// `slot_width` wide: the track is inset TRACK_INSET on both sides (the #slider design). The
    /// slot's place comes from the design (everyware_ui::design::Sizes::frame), so a slot moved
    /// in Figma moves the touch area with it.
    pub fn in_slot(value: f32, slot_x: f32, slot_width: f32) -> Self {
        Slider::new(value, slot_x + TRACK_INSET, slot_width - 2.0 * TRACK_INSET)
    }

    /// A slider at `value`, its track at `track_x`..`track_x + track_width` on the display.
    pub fn new(value: f32, track_x: f32, track_width: f32) -> Self {
        Slider { value, step: 0.0, track_x, track_width, press: Pressable::default(), dragging: false }
    }

    fn value_at(&self, x: f32) -> f32 {
        let v = ((x - self.track_x) / self.track_width * 100.0).clamp(0.0, 100.0);
        if self.step > 0.0 {
            (v / self.step).round() * self.step
        } else {
            v
        }
    }

    /// Applies a gesture: down jumps to the finger, move drags (also outside the slider),
    /// up reports the value.
    pub fn on(&mut self, gesture: Gesture) -> Option<SliderEvent> {
        if !self.press.enabled {
            return None;
        }
        match gesture {
            Gesture::Down { x, .. } => {
                self.press.on(gesture);
                self.dragging = true;
                self.value = self.value_at(x);
                Some(SliderEvent::Moving(self.value))
            }
            Gesture::Move { x, .. } if self.dragging => {
                self.value = self.value_at(x);
                Some(SliderEvent::Moving(self.value))
            }
            Gesture::Up { x, .. } if self.dragging => {
                self.press.on(Gesture::UP);
                self.dragging = false;
                self.value = self.value_at(x);
                Some(SliderEvent::Changed(self.value))
            }
            Gesture::Cancel => {
                self.press.on(Gesture::Cancel);
                self.dragging = false;
                None
            }
            _ => None,
        }
    }

    /// A finger is on the slider: a value from outside (the device reported one) waits until
    /// it is up, or the thumb would jump under the finger.
    pub fn dragging(&self) -> bool {
        self.dragging
    }

    /// How fill and thumb follow the value: short, so the thumb stays under the finger.
    pub fn value_transition(&self) -> Option<Transition> {
        transitions::transition(Self::KIND, Change::Value)
    }
}

impl Component for Slider {
    const KIND: ComponentKind = ComponentKind::Slider;

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
    fn drag_follows_the_finger_also_outside() {
        let mut s = Slider::new(60.0, 100.0, 300.0);
        assert_eq!(s.on(Gesture::down(250.0, 0.0)), Some(SliderEvent::Moving(50.0)));
        assert_eq!(s.variant(), "state=pressed");
        assert_eq!(s.on(Gesture::Move { x: 1000.0, y: 0.0, inside: false }), Some(SliderEvent::Moving(100.0)));
        assert_eq!(s.on(Gesture::Up { x: 40.0, y: 0.0, inside: false, velocity: 0.0, velocity_x: 0.0 }), Some(SliderEvent::Changed(0.0)));
        assert_eq!(s.variant(), "state=default");
    }

    #[test]
    fn steps() {
        let mut s = Slider::new(0.0, 0.0, 100.0);
        s.step = 10.0;
        assert_eq!(s.on(Gesture::down(64.0, 0.0)), Some(SliderEvent::Moving(60.0)));
        assert_eq!(s.value_transition().unwrap().duration.as_millis(), 80);
    }

    #[test]
    fn a_slider_in_a_slot_has_its_track_inside_the_slot() {
        // the #slider design: 243 wide, the track 20 in on both sides
        let s = Slider::in_slot(65.0, 589.0, 243.0);
        assert_eq!((s.track_x, s.track_width), (609.0, 203.0));
    }
}
