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
}

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
            Some(_) => self.value_at(now),
        };
        if target != self.target {
            self.from = shown;
            self.target = target;
            self.started = Some(now);
            self.duration = transition.map(|t| t.duration).unwrap_or_default();
            self.easing = transition.map(|t| t.easing);
        }
        let value = self.value_at(now);
        self.shown = Some(value);
        (value, value != self.target)
    }

    fn value_at(&self, now: Instant) -> f32 {
        let (Some(started), Some(easing)) = (self.started, self.easing) else { return self.target };
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
        Some(Transition { duration: Duration::from_millis(ms), easing: Easing::Linear })
    }

    #[test]
    fn first_value_at_once_then_moves() {
        let t0 = Instant::now();
        let mut f = Follow::default();
        assert_eq!(f.step(0.0, tr(200), t0), (0.0, false));
        let (v, moving) = f.step(100.0, tr(200), t0);
        assert_eq!((v, moving), (0.0, true));
        let (v, moving) = f.step(100.0, tr(200), t0 + Duration::from_millis(100));
        assert!((v - 50.0).abs() < 0.5 && moving);
        assert_eq!(f.step(100.0, tr(200), t0 + Duration::from_millis(250)), (100.0, false));
    }

    #[test]
    fn retarget_starts_from_shown() {
        let t0 = Instant::now();
        let mut f = Follow::default();
        f.step(0.0, tr(200), t0);
        f.step(100.0, tr(200), t0);
        f.step(100.0, tr(200), t0 + Duration::from_millis(100));
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
}
