//! A value that follows its target over a transition, frame by frame.
//!
//! As Google's FollowCustomizer for the cluster gauges (reference/harry-app-core
//! src/ui/approx.rs): the customizer of a meter or an opacity keeps one, sets the target on
//! every frame, and asks harry for the next frame while the shown value still moves. Meter
//! values and opacities are display-only in harry, so these frames come from the cache.

use crate::transitions::{Easing, Transition};
use std::time::{Duration, Instant};

/// The shown value of a meter or an opacity, moving towards a target.
#[derive(Clone, Debug, Default)]
pub struct Follow {
    shown: Option<f32>,
    from: f32,
    target: f32,
    started: Option<Instant>,
    duration: Duration,
    easing: Option<Easing>,
    /// The frame that set off the running animation, until the next frame starts its clock
    /// (`LATE`).
    set_off: Option<Instant>,
}

/// The first frame of an animation that comes later than this after the frame that set it off
/// is late (two periods of a 60 Hz display). The time over it is taken out of the animation:
/// a heavy frame that starts it (a new tree, a first drawing) delays it instead of cutting its
/// start off. Android starts an animation's clock at its first frame for the same reason
/// (ValueAnimator.doAnimationFrame: "First frame", mStartTime = frameTime). Only the first
/// frame: a late frame later in the animation is not held, so on a slow display the value keeps
/// the time of its transition, as harry's own transitions do.
pub const LATE: Duration = Duration::from_millis(34);

/// How far the spring has come at the end of its transition, in units of its natural period
/// (omega * duration): at 8 the unnormalized curve is within 0.3 % of its target.
const SPRING_OMEGA_T: f32 = 8.0;

/// The easing curves as harry draws them: one-dimensional cubic Béziers over the progress, and
/// the spring, the step response of a critically damped oscillator, 1 - (1 + wt) e^(-wt),
/// scaled to reach 1 at the end of the transition.
pub fn ease(easing: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let (p1, p2) = match easing {
        Easing::Spring => {
            let response = |x: f32| 1.0 - (1.0 + x) * (-x).exp();
            return response(SPRING_OMEGA_T * t) / response(SPRING_OMEGA_T);
        }
        Easing::Linear => return t,
        Easing::EaseIn => (0.0, 0.0),
        Easing::EaseOut => (1.0, 1.0),
        Easing::EaseInOut => (0.0, 1.0),
    };
    let u = 1.0 - t;
    3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t
}

impl Follow {
    /// Sets the target and returns the value to show at `now`, and whether it still moves.
    /// The first value is shown at once; a new target starts from the value shown now.
    pub fn step(&mut self, target: f32, transition: Option<Transition>, now: Instant) -> (f32, bool) {
        let shown = match self.shown {
            None => {
                self.shown = Some(target);
                self.target = target;
                return (target, false);
            }
            Some(_) => {
                self.start_clock(now);
                self.value_at(now)
            }
        };
        if target != self.target {
            self.from = shown;
            self.target = target;
            // a delayed transition starts later: the value stays where it is until then
            self.started = Some(now + transition.map(|t| t.delay).unwrap_or_default());
            self.duration = transition.map(|t| t.duration).unwrap_or_default();
            self.easing = transition.map(|t| t.easing);
            self.set_off = Some(now);
        }
        let value = self.value_at(now);
        self.shown = Some(value);
        (value, value != self.target)
    }

    /// The first frame after the one that set off an animation: moves the animation's start by
    /// the time this frame is late (`LATE`).
    fn start_clock(&mut self, now: Instant) {
        let (Some(set_off), Some(started)) = (self.set_off.take(), self.started) else { return };
        let late = now.saturating_duration_since(set_off).saturating_sub(LATE);
        if !late.is_zero() {
            self.started = Some(started + late);
        }
    }

    fn value_at(&self, now: Instant) -> f32 {
        let (Some(started), Some(easing)) = (self.started, self.easing) else { return self.target };
        if now < started {
            return self.from;
        }
        if self.duration.is_zero() {
            return self.target;
        }
        let t = now.duration_since(started).as_secs_f32() / self.duration.as_secs_f32();
        if t >= 1.0 {
            return self.target;
        }
        self.from + (self.target - self.from) * ease(easing, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tr(ms: u64) -> Option<Transition> {
        Some(Transition { duration: Duration::from_millis(ms), easing: Easing::Linear, delay: Duration::ZERO })
    }

    /// Steps `f` towards `target` frame by frame (every 10 ms) from `from` to `to` ms after
    /// `t0`, as a screen does, and returns the last step.
    fn frames(f: &mut Follow, target: f32, transition: Option<Transition>, t0: Instant, from: u64, to: u64) -> (f32, bool) {
        let mut last = (0.0, false);
        for ms in (from..=to).step_by(10) {
            last = f.step(target, transition, t0 + Duration::from_millis(ms));
        }
        last
    }

    #[test]
    fn first_value_at_once_then_moves() {
        let t0 = Instant::now();
        let mut f = Follow::default();
        assert_eq!(f.step(0.0, tr(200), t0), (0.0, false));
        let (v, moving) = f.step(100.0, tr(200), t0);
        assert_eq!((v, moving), (0.0, true));
        let (v, moving) = frames(&mut f, 100.0, tr(200), t0, 10, 100);
        assert!((v - 50.0).abs() < 0.5 && moving);
        assert_eq!(frames(&mut f, 100.0, tr(200), t0, 110, 250), (100.0, false));
    }

    #[test]
    fn only_the_first_frame_of_an_animation_is_held() {
        let t0 = Instant::now();
        let mut f = Follow::default();
        f.step(0.0, tr(200), t0);
        f.step(100.0, tr(200), t0);
        // frames of 50 ms (20 fps) after a first one in time: the animation keeps its 200 ms
        let (v, _) = frames(&mut f, 100.0, tr(200), t0, 10, 10);
        assert!((v - 5.0).abs() < 0.5, "{v}");
        let mut last = (v, true);
        for ms in [60, 110, 160] {
            last = f.step(100.0, tr(200), t0 + Duration::from_millis(ms));
        }
        assert!((last.0 - 80.0).abs() < 0.5 && last.1, "{last:?}");
        assert_eq!(f.step(100.0, tr(200), t0 + Duration::from_millis(210)), (100.0, false));
    }

    #[test]
    fn a_late_frame_delays_the_animation_instead_of_cutting_it() {
        let t0 = Instant::now();
        let mut f = Follow::default();
        f.step(0.0, tr(200), t0);
        assert_eq!(f.step(100.0, tr(200), t0), (0.0, true));
        // the frame that started the animation took 160 ms (a new tree): the next one shows
        // the animation one late-frame limit in, not 80 % in
        let (v, moving) = f.step(100.0, tr(200), t0 + Duration::from_millis(160));
        assert!((v - 100.0 * LATE.as_secs_f32() / 0.2).abs() < 0.5 && moving, "{v}");
        // from there it runs its whole time: done 200 - 34 ms later, not before
        let (v, moving) = frames(&mut f, 100.0, tr(200), t0, 170, 310);
        assert!(v < 100.0 && moving, "{v}");
        assert_eq!(frames(&mut f, 100.0, tr(200), t0, 320, 330), (100.0, false));
    }

    #[test]
    fn retarget_starts_from_shown() {
        let t0 = Instant::now();
        let mut f = Follow::default();
        f.step(0.0, tr(200), t0);
        f.step(100.0, tr(200), t0);
        frames(&mut f, 100.0, tr(200), t0, 10, 100);
        let (v, _) = f.step(0.0, tr(200), t0 + Duration::from_millis(100));
        assert!((v - 50.0).abs() < 0.5);
    }

    #[test]
    fn eases_end_at_the_ends() {
        for e in [Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut, Easing::Spring] {
            assert_eq!(ease(e, 0.0), 0.0);
            assert!((ease(e, 1.0) - 1.0).abs() < 1e-6);
        }
        assert!(ease(Easing::EaseOut, 0.3) > 0.3 && ease(Easing::EaseIn, 0.3) < 0.3);
    }

    #[test]
    fn the_spring_starts_fast_and_settles_without_overshoot() {
        // half the way in a fifth of the time, 90 % at half of it
        assert!(ease(Easing::Spring, 0.2) > 0.45 && ease(Easing::Spring, 0.5) > 0.9);
        let mut last = 0.0;
        for i in 0..=100 {
            let v = ease(Easing::Spring, i as f32 / 100.0);
            assert!(v >= last && v <= 1.0, "monotonic, at most 1: {v} at {i}");
            last = v;
        }
    }

    #[test]
    fn a_delayed_transition_waits_then_moves() {
        let t0 = Instant::now();
        let wait = Some(Transition { duration: Duration::from_millis(100), easing: Easing::Linear, delay: Duration::from_millis(50) });
        let mut f = Follow::default();
        f.step(0.0, wait, t0);
        assert_eq!(f.step(100.0, wait, t0), (0.0, true));
        assert_eq!(frames(&mut f, 100.0, wait, t0, 10, 40), (0.0, true));
        let (v, moving) = frames(&mut f, 100.0, wait, t0, 50, 100);
        assert!((v - 50.0).abs() < 0.5 && moving, "{v}");
        assert_eq!(frames(&mut f, 100.0, wait, t0, 110, 150), (100.0, false));
    }

    #[test]
    fn a_delayed_jump_happens_after_the_delay() {
        let t0 = Instant::now();
        let jump = Some(Transition { duration: Duration::ZERO, easing: Easing::Linear, delay: Duration::from_millis(300) });
        let mut f = Follow::default();
        f.step(100.0, jump, t0);
        assert_eq!(f.step(0.0, jump, t0), (100.0, true));
        assert_eq!(frames(&mut f, 0.0, jump, t0, 10, 200), (100.0, true));
        assert_eq!(frames(&mut f, 0.0, jump, t0, 210, 310), (0.0, false));
    }
}
