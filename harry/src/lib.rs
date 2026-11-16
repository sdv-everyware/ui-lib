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
pub use slot::{clone_name, grid, grid_shown, list, part_of, row_name, row_of, rows, scrollbar, slot, slot_of, slot_shown, ComponentItem, Slot, SlotCustomizer, Slotted};

use customizations::render_customizations::{NodePositionOverride, RenderCustomization};
use customizations::{AnimationOverride, Customizer, CustomizerOutput};
use har_platform_api::looper::HandleEventStatus;
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

/// A screen behind tabs, as the screens' effect places it (core screens.rs): its opacity and,
/// for an effect that moves it, its x in the parent's coordinates. One field per screen:
/// `#[Design(node = "#rooms", customizer = "ScreenCustomizer")] rooms: ScreenView`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScreenView {
    pub opacity: Meter,
    pub x: Option<Meter>,
}

impl ScreenView {
    /// The view of a screen from its target; `design_x` is where the design puts the screen in
    /// its parent, the offset of a slide is added to it: Sizes::frame for a screen right under
    /// the root, Sizes::part for a page in a component (docs/content/pages.md).
    pub fn new(target: everyware_ui_core::screens::ScreenTarget, design_x: f32) -> Self {
        ScreenView {
            opacity: Meter::new(target.opacity, target.opacity_transition),
            x: target.offset.map(|dx| Meter::new(design_x + dx, target.offset_transition)),
        }
    }
}

/// Fades and moves a screen (see `ScreenView`). One customizer for both, as a screen is one
/// node; both are display only, so a switch draws from the cache without a new tree.
#[derive(Clone, Debug, Default)]
pub struct ScreenCustomizer {
    opacity: Follow,
    x: Follow,
}

impl Customizer<ScreenView> for ScreenCustomizer {
    fn format(&mut self, view: ScreenView) -> CustomizerOutput {
        let now = Instant::now();
        let (opacity, fading) = self.opacity.step(view.opacity.target, view.opacity.transition, now);
        let mut parts = vec![RenderCustomization::Opacity(opacity / 100.0)];
        let mut moving = false;
        if let Some(x) = view.x {
            let (x, m) = self.x.step(x.target, x.transition, now);
            moving = m;
            parts.push(RenderCustomization::NodePositionOverride(NodePositionOverride::new(Some(x), None)));
        }
        follow_output(RenderCustomization::Multiple(parts), fading || moving)
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
///
/// A control inside a row (a tile's toggle) is a part: its node carries the same name in
/// every row. harry delivers a touch to every interactive node under the finger, top-most
/// first, and goes on to the next while the listener answers NotConsumed (as the reference
/// app does: `reference/harry-app/src/events.rs`). So a part is held back with NotConsumed,
/// the clone root under it comes next (a row of a list, or the component's plain slot), and
/// the two make one gesture on "<slot>/<row>/<part>" or "<slot>/<part>" (`row_of`, `part_of`).
/// A part with nothing under it keeps its own name. The listener returns the status `feed`
/// gives it.
#[derive(Debug, Default)]
pub struct Input {
    tracker: GestureTracker,
    velocity: Option<Velocity>,
    /// A part's touch start waiting for the node below it: (pointer, part, raw events seen).
    /// harry sends one raw event after a NotConsumed and one more when no node consumed the
    /// touch (`dispatch_raw_user_input`): the second raw means the part stands alone.
    pending: Option<(u64, String, u8)>,
}

/// The gestures of one event and the answer for harry (see `Input`).
pub type Fed = (Vec<(String, Gesture)>, HandleEventStatus);

/// Is this node a part inside a row, named the same in every row (tile.rs)?
fn is_part(node: &str) -> bool {
    node.starts_with("#tile/") || node.starts_with("#segmented/segment-") || node.starts_with("#tabs/tab-")
}

impl Input {
    /// An input that knows the finger's speed (see `stamp`).
    pub fn with_velocity(velocity: Velocity) -> Self {
        Input { tracker: GestureTracker::default(), velocity: Some(velocity), pending: None }
    }

    /// Feeds one event from the UiEventListener; returns the gestures, as (node, gesture),
    /// and the status the listener answers harry with.
    pub fn feed(&mut self, event: &UserInputEvent) -> Fed {
        let (raw, hit) = match event {
            UserInputEvent::TouchEvent(touch) => (&touch.triggering_event, Some(touch.node_id.as_str())),
            UserInputEvent::RawUserInputEvent(raw) => (raw, None),
            #[allow(unreachable_patterns)]
            _ => return (Vec::new(), HandleEventStatus::Consumed),
        };
        let phase = match raw.event_type {
            RawInputEventType::TouchStart | RawInputEventType::MouseDown => Phase::Start,
            RawInputEventType::TouchEnd | RawInputEventType::MouseUp => Phase::End,
            RawInputEventType::TouchMove | RawInputEventType::MouseMove => Phase::Move,
        };
        let pointer = raw.pointer_id.unwrap_or(0);
        let node = match (phase, hit) {
            // a part's start: wait for the row below it
            (Phase::Start, Some(hit)) if is_part(hit) && self.pending.is_none() => {
                self.pending = Some((pointer, hit.to_string(), 0));
                return (Vec::new(), HandleEventStatus::NotConsumed);
            }
            // the row below the part: one gesture on "<slot>/<row>/<part>"; a component in a
            // slot is reported by its clone's name: the app gets the slot's
            (Phase::Start, Some(hit)) => match self.pending.take() {
                Some((p, part, _)) if p == pointer => format!("{}/{part}", slot_of(hit)),
                _ => slot_of(hit),
            },
            // the raw events after a NotConsumed: the second says no row took the part
            (Phase::Start, None) => match self.pending.take() {
                Some((p, part, 0)) if p == pointer => {
                    self.pending = Some((p, part, 1));
                    return (Vec::new(), HandleEventStatus::Consumed);
                }
                Some((p, part, _)) if p == pointer => part,
                _ => String::new(),
            },
            // a move or a lift over the part the touch started on counts as inside it
            (_, Some(hit)) => match self.tracker.active(pointer) {
                Some(start) if is_part(hit) && start.ends_with(&format!("/{hit}")) => start.to_string(),
                _ => slot_of(hit),
            },
            (_, None) => String::new(),
        };
        let velocity = match (phase, &self.velocity) {
            (Phase::End, Some(v)) => v.lock().map(|v| (v.velocity_x(), v.velocity())).unwrap_or((0.0, 0.0)),
            _ => (0.0, 0.0),
        };
        (self.tracker.feed(pointer, phase, &node, raw.x as f32, raw.y as f32, velocity), HandleEventStatus::Consumed)
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use har_platform_api::looper::DisplayId;
    use har_platform_api::user_input::{RawUserInputEvent, TouchEvent};

    fn raw(event_type: RawInputEventType, x: f64) -> RawUserInputEvent {
        RawUserInputEvent { display_id: DisplayId::new(0), x, y: 0.0, event_type, pointer_id: Some(1) }
    }
    fn touch(node: &str, event_type: RawInputEventType, x: f64) -> UserInputEvent {
        UserInputEvent::TouchEvent(TouchEvent { triggering_event: raw(event_type, x), node_id: node.to_string() })
    }
    fn loose(event_type: RawInputEventType, x: f64) -> UserInputEvent {
        UserInputEvent::RawUserInputEvent(raw(event_type, x))
    }
    /// The gestures, and whether harry may go on to the next node (HandleEventStatus has no
    /// PartialEq).
    fn fed(input: &mut Input, event: &UserInputEvent) -> (Vec<(String, Gesture)>, bool) {
        let (g, s) = input.feed(event);
        (g, matches!(s, HandleEventStatus::Consumed))
    }
    const START: RawInputEventType = RawInputEventType::TouchStart;
    const MOVE: RawInputEventType = RawInputEventType::TouchMove;
    const END: RawInputEventType = RawInputEventType::TouchEnd;

    #[test]
    fn a_part_in_a_row_is_held_back_and_paired_with_the_row() {
        let mut input = Input::default();
        // harry: the toggle first (not consumed), its raw event, then the row's clone root
        assert_eq!(fed(&mut input, &touch("#tile/toggle", START, 400.0)), (vec![], false));
        assert_eq!(fed(&mut input, &loose(START, 400.0)), (vec![], true));
        assert_eq!(
            fed(&mut input, &touch("#tv/cards@$List-r3", START, 400.0)),
            (vec![("#tv/cards/3/#tile/toggle".to_string(), Gesture::down(400.0, 0.0))], true)
        );
        // a move over the toggle stays inside; a lift over the card's root is outside
        let (g, _) = fed(&mut input, &touch("#tile/toggle", MOVE, 402.0));
        assert_eq!(g, vec![("#tv/cards/3/#tile/toggle".to_string(), Gesture::Move { x: 402.0, y: 0.0, inside: true })]);
        let (g, _) = fed(&mut input, &touch("#tv/cards@$List-r3", END, 300.0));
        assert_eq!(g, vec![("#tv/cards/3/#tile/toggle".to_string(), Gesture::Up { x: 300.0, y: 0.0, inside: false, velocity: 0.0, velocity_x: 0.0 })]);
    }

    #[test]
    fn a_part_in_a_plain_slot_is_paired_with_the_slot() {
        let mut input = Input::default();
        assert!(!fed(&mut input, &touch("#segmented/segment-2", START, 10.0)).1);
        assert_eq!(fed(&mut input, &loose(START, 10.0)), (vec![], true));
        // the slot's clone root (not a row): "<slot>/<part>"
        assert_eq!(
            fed(&mut input, &touch("#hub/fan@$List-i", START, 10.0)),
            (vec![("#hub/fan/#segmented/segment-2".to_string(), Gesture::down(10.0, 0.0))], true)
        );
        let (g, _) = fed(&mut input, &touch("#segmented/segment-2", END, 10.0));
        assert_eq!(g, vec![("#hub/fan/#segmented/segment-2".to_string(), Gesture::up(10.0, 0.0))]);
    }

    #[test]
    fn a_tab_is_paired_with_its_tab_bar() {
        // two tab bars on one screen (the header's and a screen's) stay apart by their slots
        for slot in ["#home/tabs", "#rooms/tabs"] {
            let mut input = Input::default();
            assert!(!fed(&mut input, &touch("#tabs/tab-2", START, 10.0)).1);
            assert_eq!(fed(&mut input, &loose(START, 10.0)), (vec![], true));
            let want = format!("{slot}/#tabs/tab-2");
            assert_eq!(fed(&mut input, &touch(&format!("{slot}@$List-i"), START, 10.0)), (vec![(want.clone(), Gesture::down(10.0, 0.0))], true));
            let (g, _) = fed(&mut input, &touch("#tabs/tab-2", END, 10.0));
            assert_eq!(g, vec![(want, Gesture::up(10.0, 0.0))]);
        }
    }

    #[test]
    fn a_screen_fades_and_moves_only_for_an_effect_that_moves_it() {
        use everyware_ui_core::screens::Effect;
        use everyware_ui_core::tabs::Tabs;
        let mut tabs = Tabs::new(["Overview", "Rooms", "Automations"]);
        tabs.select(1);
        let fade = ScreenView::new(tabs.screen(1, Effect::Fade, 1920.0), 48.0);
        assert_eq!((fade.opacity.target, fade.x), (100.0, None));
        let slide = ScreenView::new(tabs.screen(0, Effect::Slide, 1920.0), 48.0);
        assert_eq!(slide.x.map(|x| x.target), Some(48.0 - 1920.0));
        // the first frame shows the target at once, without a position for a fade
        let parts = |v| match ScreenCustomizer::default().format(v).result {
            RenderCustomization::Multiple(parts) => parts,
            _ => panic!("a screen is one Multiple"),
        };
        assert_eq!(parts(fade), vec![RenderCustomization::Opacity(1.0)]);
        assert_eq!(parts(slide), vec![RenderCustomization::Opacity(0.0), RenderCustomization::NodePositionOverride(NodePositionOverride::new(Some(48.0 - 1920.0), None))]);
    }

    #[test]
    fn a_part_with_no_row_under_it_keeps_its_name() {
        let mut input = Input::default();
        assert!(!fed(&mut input, &touch("#segmented/segment-2", START, 10.0)).1);
        // the raw after the NotConsumed, then the raw harry sends when nothing consumed
        assert_eq!(fed(&mut input, &loose(START, 10.0)), (vec![], true));
        assert_eq!(fed(&mut input, &loose(START, 10.0)), (vec![("#segmented/segment-2".to_string(), Gesture::down(10.0, 0.0))], true));
    }

    #[test]
    fn a_plain_node_is_consumed_at_once_by_its_slot_name() {
        let mut input = Input::default();
        assert_eq!(fed(&mut input, &touch("#tv/nav/home", START, 1.0)), (vec![("#tv/nav/home".to_string(), Gesture::down(1.0, 0.0))], true));
        let (g, _) = fed(&mut input, &touch("#tv/cards@$List-r0", START, 1.0));
        assert_eq!(g, vec![("#tv/nav/home".to_string(), Gesture::Cancel), ("#tv/cards/0".to_string(), Gesture::down(1.0, 0.0))]);
    }
}
