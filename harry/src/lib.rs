//! everyware_ui: the UI components of the Smart HOme Figma file for harry.
//!
//! The components and their logic are in everyware_ui_core (re-exported here). This crate
//! connects them to harry: touch events become gestures, a component's variant becomes a
//! `RenderCustomization::Variant` on its instance node, and its transition an
//! `AnimationOverride` on the same node.
//!
//! In the app's UI model a component takes one field per customization of its instance node:
//!
//! ```ignore
//! #[Design(node = "#wifi/connect")]            connect: Variant,
//! #[Design(node = "#wifi/connect")]            connect_animation: AnimationOverride,
//! #[Design(node = "#wifi/connect/label")]      connect_label: String,
//! #[Design(node = "#wifi/connect", interactive)] _connect_tap: (),
//! ```
//! and the presenter writes `(connect, connect_animation) = view(&state.connect)`.
//!
//! A value that moves (toggle thumb, slider, dropdown list) is not a variant: it is a
//! `Meter` on a node, and its customizer moves the shown value towards the target frame by
//! frame (as harry's reference cluster moves its needles). harry treats meter value and
//! opacity as display-only, so these frames skip the tree rebuild and the layout:
//!
//! ```ignore
//! #[Design(node = "#toggle/thumb", customizer = "FollowMeterCustomizer")]   thumb: Meter,
//! #[Design(node = "#toggle/on", customizer = "FollowOpacityCustomizer")]    on: Meter,
//! ```
//! with `thumb = Meter::new(toggle.target(), toggle.value_transition())`.
//!
//! These node-name fields hold for a component that is once on the screen. A component that
//! can be there more than once goes into a slot (slot.rs): each slot customizes its own clone.

pub use everyware_ui_core::*;

pub mod design;
pub mod images;
pub mod model;
mod slot;
pub use slot::{clone_name, grid, grid_shown, list, row_name, row_of, rows, scrollbar, slot, slot_of, slot_shown, ComponentItem, Slot, SlotCustomizer, Slotted};

use customizations::render_customizations::{NodePositionOverride, RenderCustomization};
use customizations::{AnimationOverride, Customizer, CustomizerOutput};
use har_platform_api::user_input::{RawInputEventType, UserInputEvent};
use harry_animation::animations::Animations;
use harry_animation::api::RepeatType;
use harry_animation::smooth_animation::{BezierCurve, CustomEasing, CustomEasingFunction, Easing as Curve, SmoothAnimation};
use everyware_ui_core::scroller::VelocityTracker;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A Figma variant ("prop=value, prop=value") for a UI model field on an instance node.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Variant(pub String);

impl From<Variant> for CustomizerOutput {
    fn from(variant: Variant) -> Self {
        CustomizerOutput::new_finished(RenderCustomization::Variant(variant.0))
    }
}

/// harry's easing curves are one-dimensional cubic Béziers over the progress (p0 = 0,
/// p3 = 1): p1 sets the start, p2 the end.
fn curve(easing: Easing) -> Curve {
    let (p1, p2) = match easing {
        Easing::Spring => return Curve::Custom(CustomEasing::new("everyware-spring".into(), Spring)),
        Easing::Linear => return Curve::Linear,
        Easing::EaseIn => (0.0, 0.0),
        Easing::EaseOut => (1.0, 1.0),
        Easing::EaseInOut => (0.0, 1.0),
    };
    Curve::Bezier(BezierCurve::new(0.0, p1, p2, 1.0))
}

/// The spring of components.json for harry: the curve the core computes (follow::ease).
struct Spring;

impl CustomEasingFunction for Spring {
    fn calculate(&self, progress: f32) -> f32 {
        everyware_ui_core::follow::ease(Easing::Spring, progress)
    }
}

/// The animation override for a transition: none means the change happens at once.
pub fn animation(transition: Option<Transition>) -> AnimationOverride {
    match transition {
        Some(t) => AnimationOverride::new_custom(
            None,
            Animations::Smooth(SmoothAnimation::new(t.duration, RepeatType::NoRepeat, curve(t.easing))),
        ),
        None => AnimationOverride::DisableAnimation,
    }
}

/// The variant and the animation into it, for the two fields on a component's instance node.
pub fn view<C: Component>(component: &C) -> (Variant, AnimationOverride) {
    (Variant(component.variant()), animation(component.transition()))
}

/// A target value and how the shown value gets there: 0..100 for a meter or an opacity, the
/// y in pixels (in the parent's coordinates) for a position.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Meter {
    /// Where the value goes.
    pub target: f32,
    /// The curve and duration of the way there; none jumps.
    pub transition: Option<Transition>,
}

impl Meter {
    /// A target and its transition.
    pub fn new(target: f32, transition: Option<Transition>) -> Self {
        Meter { target, transition }
    }
}

/// The output for one step: refresh next frame while the value still moves.
fn follow_output(customization: RenderCustomization, moving: bool) -> CustomizerOutput {
    if moving {
        CustomizerOutput::new_update_next_frame(customization)
    } else {
        CustomizerOutput::new_finished(customization)
    }
}

/// Moves a node's meter value (DesignCompose dials & gauges: progress bar, progress marker)
/// towards the target. One per UI model field, so each node keeps its own shown value.
#[derive(Clone, Debug, Default)]
pub struct FollowMeterCustomizer {
    follow: Follow,
}

impl Customizer<Meter> for FollowMeterCustomizer {
    fn format(&mut self, meter: Meter) -> CustomizerOutput {
        let (value, moving) = self.follow.step(meter.target, meter.transition, Instant::now());
        follow_output(RenderCustomization::MeterValue(value), moving)
    }
}

/// Moves a node's opacity towards the target (0..100 becomes 0..1).
#[derive(Clone, Debug, Default)]
pub struct FollowOpacityCustomizer {
    follow: Follow,
}

impl Customizer<Meter> for FollowOpacityCustomizer {
    fn format(&mut self, meter: Meter) -> CustomizerOutput {
        let (value, moving) = self.follow.step(meter.target, meter.transition, Instant::now());
        follow_output(RenderCustomization::Opacity(value / 100.0), moving)
    }
}

/// Moves a node vertically towards the target y, as a position override (a sheet's card).
/// harry draws a moved node without a new tree or a layout (the position's value is not part of
/// the structural hash, pre_render_cache.rs), so the card slides as cheaply as a scrim fades.
#[derive(Clone, Debug, Default)]
pub struct FollowYCustomizer {
    follow: Follow,
}

impl Customizer<Meter> for FollowYCustomizer {
    fn format(&mut self, meter: Meter) -> CustomizerOutput {
        let (value, moving) = self.follow.step(meter.target, meter.transition, Instant::now());
        follow_output(RenderCustomization::NodePositionOverride(NodePositionOverride::new(None, Some(value))), moving)
    }
}

/// A target position and the way there, for a node that moves on both axes (a sheet's card
/// that comes in from the right or the left as well as from below). One customizer for both:
/// harry keeps one position override per node, so an x and a y customizer would overwrite
/// each other.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub transition: Option<Transition>,
}

impl Position {
    /// A target and its transition.
    pub fn new((x, y): (f32, f32), transition: Option<Transition>) -> Self {
        Position { x, y, transition }
    }
}

/// Moves a node towards the target position, as a position override on both axes (see
/// `FollowYCustomizer` for why that is cheap).
#[derive(Clone, Debug, Default)]
pub struct FollowPositionCustomizer {
    x: Follow,
    y: Follow,
}

impl Customizer<Position> for FollowPositionCustomizer {
    fn format(&mut self, position: Position) -> CustomizerOutput {
        let now = Instant::now();
        let (x, moving_x) = self.x.step(position.x, position.transition, now);
        let (y, moving_y) = self.y.step(position.y, position.transition, now);
        follow_output(RenderCustomization::NodePositionOverride(NodePositionOverride::new(Some(x), Some(y))), moving_x || moving_y)
    }
}

/// A node that takes taps only while `on` (a scrim while its overlay is open: closed, it lies
/// transparent over the screen and must not catch the taps of what is below). harry makes a
/// node interactive by its customization, so the field needs this customizer:
/// `#[Design(node = "#x/scrim", customizer = "TapTargetCustomizer")] scrim: TapTarget`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TapTarget(pub bool);

/// Sets a node interactive or not (see `TapTarget`).
#[derive(Clone, Debug, Default)]
pub struct TapTargetCustomizer;

impl Customizer<TapTarget> for TapTargetCustomizer {
    fn format(&mut self, target: TapTarget) -> CustomizerOutput {
        CustomizerOutput::new_finished(RenderCustomization::Interactive(target.0))
    }
}

/// The finger's speed, measured where the events come in. harry hands events to the
/// listener in per-frame batches, so their times there are useless for a fling: the app's
/// evdev callback calls `stamp` with every event, and `Input` reads the speed at a lift.
pub type Velocity = Arc<Mutex<VelocityTracker>>;

/// Feeds the velocity tracker with an event as the kernel hands it over (the evdev
/// callback of the app's main).
pub fn stamp(velocity: &Velocity, input: &UserInputEvent) {
    let UserInputEvent::RawUserInputEvent(raw) = input else { return };
    let Ok(mut v) = velocity.lock() else { return };
    let (now, x, y) = (Instant::now(), raw.x as f32, raw.y as f32);
    match raw.event_type {
        RawInputEventType::TouchStart | RawInputEventType::MouseDown => v.down(now, x, y),
        RawInputEventType::TouchMove | RawInputEventType::MouseMove => v.moved(now, x, y),
        RawInputEventType::TouchEnd | RawInputEventType::MouseUp => v.up(now),
    }
}

/// Turns harry's touch events into gestures on nodes (see GestureTracker): events on a node
/// (harry's hit test) and raw ones away from every node, which still reach the node the
/// touch started on (a drag that leaves the row, a lift outside the button).
#[derive(Debug, Default)]
pub struct Input {
    tracker: GestureTracker,
    velocity: Option<Velocity>,
}

impl Input {
    /// An input that knows the finger's speed (see `stamp`).
    pub fn with_velocity(velocity: Velocity) -> Self {
        Input { tracker: GestureTracker::default(), velocity: Some(velocity) }
    }

    /// Feeds one event from the UiEventListener; returns the gestures, as (node, gesture).
    pub fn feed(&mut self, event: &UserInputEvent) -> Vec<(String, Gesture)> {
        let (raw, node) = match event {
            // A component in a slot is reported by its clone's name: give the app the slot's.
            UserInputEvent::TouchEvent(touch) => (&touch.triggering_event, slot_of(&touch.node_id)),
            UserInputEvent::RawUserInputEvent(raw) => (raw, String::new()),
            #[allow(unreachable_patterns)]
            _ => return Vec::new(),
        };
        let phase = match raw.event_type {
            RawInputEventType::TouchStart | RawInputEventType::MouseDown => Phase::Start,
            RawInputEventType::TouchEnd | RawInputEventType::MouseUp => Phase::End,
            RawInputEventType::TouchMove | RawInputEventType::MouseMove => Phase::Move,
        };
        let velocity = match (phase, &self.velocity) {
            (Phase::End, Some(v)) => v.lock().map(|v| (v.velocity_x(), v.velocity())).unwrap_or((0.0, 0.0)),
            _ => (0.0, 0.0),
        };
        self.tracker.feed(raw.pointer_id.unwrap_or(0), phase, &node, raw.x as f32, raw.y as f32, velocity)
    }
}
