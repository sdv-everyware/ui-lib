//! Android's scrolling physics, so a list feels like a ScrollView / RecyclerView (ported from
//! the first app on the RB3, which proved it):
//!
//! - `VelocityTracker`: frameworks/native/libs/input/VelocityTracker.cpp, the LSQ2 strategy
//!   (2nd-degree least squares over the last 100 ms, at most 20 samples, velocity 0 when
//!   the pointer rested 40 ms before lifting). Feed it where the events come in (the evdev
//!   thread): past that point they reach the app in per-frame batches and their times are
//!   useless for speed.
//! - `Fling`: frameworks/base/core/java/android/widget/OverScroller.java,
//!   SplineOverScroller.fling / adjustDuration / update (case SPLINE).
//! - constants: android.view.ViewConfiguration: touch slop 8 dp, fling 50..8000 dp/s, scroll
//!   friction 0.015. The panels run at 160 dpi, so 1 dp = 1 px.

use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// ViewConfiguration.TOUCH_SLOP (dp).
pub const TOUCH_SLOP: f32 = 8.0;
/// ViewConfiguration.MINIMUM_FLING_VELOCITY (dp/s).
pub const MIN_FLING_VELOCITY: f32 = 50.0;
/// ViewConfiguration.MAXIMUM_FLING_VELOCITY (dp/s).
pub const MAX_FLING_VELOCITY: f32 = 8000.0;

const DENSITY_DPI: f32 = 160.0;
/// ViewConfiguration.SCROLL_FRICTION.
const SCROLL_FRICTION: f32 = 0.015;
/// OverScroller: GRAVITY_EARTH * inch/meter * ppi * a 0.84 "look and feel" tuning factor.
const PHYSICAL_COEFF: f32 = 9.80665 * 39.37 * DENSITY_DPI * 0.84;
const INFLEXION: f32 = 0.35;
const START_TENSION: f32 = 0.5;
const END_TENSION: f32 = 1.0;
const P1: f32 = START_TENSION * INFLEXION;
const P2: f32 = 1.0 - END_TENSION * (1.0 - INFLEXION);
const NB_SAMPLES: usize = 100;

fn deceleration_rate() -> f32 {
    (0.78f64.ln() / 0.9f64.ln()) as f32
}

/// SPLINE_POSITION and SPLINE_TIME, built exactly like OverScroller's static block.
fn spline() -> &'static ([f32; NB_SAMPLES + 1], [f32; NB_SAMPLES + 1]) {
    static TABLES: OnceLock<([f32; NB_SAMPLES + 1], [f32; NB_SAMPLES + 1])> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut position = [0f32; NB_SAMPLES + 1];
        let mut time = [0f32; NB_SAMPLES + 1];
        let (mut x_min, mut y_min) = (0f32, 0f32);
        for i in 0..NB_SAMPLES {
            let alpha = i as f32 / NB_SAMPLES as f32;
            let mut x_max = 1f32;
            let (mut x, mut coef);
            loop {
                x = x_min + (x_max - x_min) / 2.0;
                coef = 3.0 * x * (1.0 - x);
                let tx = coef * ((1.0 - x) * P1 + x * P2) + x * x * x;
                if (tx - alpha).abs() < 1e-5 {
                    break;
                }
                if tx > alpha { x_max = x } else { x_min = x }
            }
            position[i] = coef * ((1.0 - x) * START_TENSION + x) + x * x * x;
            let mut y_max = 1f32;
            let mut y;
            loop {
                y = y_min + (y_max - y_min) / 2.0;
                coef = 3.0 * y * (1.0 - y);
                let dy = coef * ((1.0 - y) * START_TENSION + y) + y * y * y;
                if (dy - alpha).abs() < 1e-5 {
                    break;
                }
                if dy > alpha { y_max = y } else { y_min = y }
            }
            time[i] = coef * ((1.0 - y) * P1 + y * P2) + y * y * y;
        }
        position[NB_SAMPLES] = 1.0;
        time[NB_SAMPLES] = 1.0;
        (position, time)
    })
}

/// A fling in progress: SplineOverScroller between fling() and the end of update().
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fling {
    start_time: Instant,
    start: f32,
    /// Signed distance of the full spline (before clamping to the content).
    distance: f32,
    spline_duration: Duration,
    /// Shortened by adjust_duration when the content ends before the spline does.
    duration: Duration,
    min: f32,
    max: f32,
}

impl Fling {
    /// SplineOverScroller.fling(start, velocity, min, max, over = 0). `velocity` in px/s of
    /// the scroll position (positive scrolls towards `max`). None below the minimum speed.
    pub fn start(now: Instant, start: f32, velocity: f32, min: f32, max: f32) -> Option<Self> {
        let velocity = velocity.clamp(-MAX_FLING_VELOCITY, MAX_FLING_VELOCITY);
        if velocity.abs() < MIN_FLING_VELOCITY {
            return None;
        }
        let l = (INFLEXION * velocity.abs() / (SCROLL_FRICTION * PHYSICAL_COEFF)).ln();
        let decel_minus_one = deceleration_rate() - 1.0;
        let total = SCROLL_FRICTION * PHYSICAL_COEFF * (deceleration_rate() / decel_minus_one * l).exp();
        let spline_duration = Duration::from_millis((1000.0 * (l / decel_minus_one).exp()) as u64);
        let distance = total * velocity.signum();
        let mut f = Fling { start_time: now, start, distance, spline_duration, duration: spline_duration, min, max };
        let end = start + distance;
        if end < min {
            f.adjust_duration(min - start);
        } else if end > max {
            f.adjust_duration(max - start);
        }
        Some(f)
    }

    /// SplineOverScroller.adjustDuration: the content ends at `new_distance`, so the fling
    /// stops when the spline gets there.
    fn adjust_duration(&mut self, new_distance: f32) {
        let (_, time) = spline();
        let x = (new_distance / self.distance).abs();
        let index = (NB_SAMPLES as f32 * x) as usize;
        if index < NB_SAMPLES {
            let x_inf = index as f32 / NB_SAMPLES as f32;
            let x_sup = (index + 1) as f32 / NB_SAMPLES as f32;
            let (t_inf, t_sup) = (time[index], time[index + 1]);
            let coef = t_inf + (x - x_inf) / (x_sup - x_inf) * (t_sup - t_inf);
            self.duration = self.duration.mul_f32(coef);
        }
    }

    /// When the fling is over.
    pub fn ends_at(&self) -> Instant {
        self.start_time + self.duration
    }

    /// SplineOverScroller.update, case SPLINE: (position, finished).
    pub fn position(&self, now: Instant) -> (f32, bool) {
        let elapsed = now.saturating_duration_since(self.start_time);
        if elapsed >= self.duration {
            let end = (self.start + self.distance).clamp(self.min, self.max);
            return (end, true);
        }
        let (position, _) = spline();
        let t = elapsed.as_secs_f32() / self.spline_duration.as_secs_f32();
        let index = (NB_SAMPLES as f32 * t) as usize;
        let mut distance_coef = 1.0;
        if index < NB_SAMPLES {
            let t_inf = index as f32 / NB_SAMPLES as f32;
            let t_sup = (index + 1) as f32 / NB_SAMPLES as f32;
            let (d_inf, d_sup) = (position[index], position[index + 1]);
            let velocity_coef = (d_sup - d_inf) / (t_sup - t_inf);
            distance_coef = d_inf + (t - t_inf) * velocity_coef;
        }
        ((self.start + distance_coef * self.distance).clamp(self.min, self.max), false)
    }
}

/// VelocityTracker with the LSQ2 strategy, one axis.
#[derive(Debug, Default)]
pub struct VelocityTracker {
    /// (time, x, y) of the pointer.
    samples: VecDeque<(Instant, f32, f32)>,
    lifted: Option<Instant>,
}

impl VelocityTracker {
    /// AccumulatingVelocityTrackerStrategy: HORIZON and HISTORY_SIZE.
    const HORIZON: Duration = Duration::from_millis(100);
    const HISTORY: usize = 20;
    /// VelocityTracker::ASSUME_POINTER_STOPPED_TIME.
    const STOPPED: Duration = Duration::from_millis(40);

    /// The pointer went down at (`x`, `y`).
    pub fn down(&mut self, t: Instant, x: f32, y: f32) {
        self.samples.clear();
        self.lifted = None;
        self.samples.push_back((t, x, y));
    }

    /// The pointer moved to (`x`, `y`).
    pub fn moved(&mut self, t: Instant, x: f32, y: f32) {
        self.samples.push_back((t, x, y));
        while self.samples.len() > Self::HISTORY {
            self.samples.pop_front();
        }
    }

    /// The UP position is not a sample (Android only adds DOWN/MOVE positions), only its time.
    pub fn up(&mut self, t: Instant) {
        self.lifted = Some(t);
    }

    /// Velocity in px/s of the pointer y at the last sample (a vertical list's fling).
    pub fn velocity(&self) -> f32 {
        self.velocity_of(|&(_, _, y)| y)
    }

    /// Velocity in px/s of the pointer x at the last sample (a horizontal list's fling).
    pub fn velocity_x(&self) -> f32 {
        self.velocity_of(|&(_, x, _)| x)
    }

    fn velocity_of(&self, axis: impl Fn(&(Instant, f32, f32)) -> f32) -> f32 {
        let Some(&(newest, _, _)) = self.samples.back() else { return 0.0 };
        if self.lifted.is_some_and(|up| up.saturating_duration_since(newest) > Self::STOPPED) {
            return 0.0;
        }
        // Samples within the horizon, time relative to the newest (s, <= 0).
        let pts: Vec<(f64, f64)> = self
            .samples
            .iter()
            .filter(|(t, _, _)| newest.duration_since(*t) <= Self::HORIZON)
            .map(|s| (-(newest.duration_since(s.0).as_secs_f64()), axis(s) as f64))
            .collect();
        let degree = match pts.len() {
            0 | 1 => return 0.0,
            2 => 1,
            _ => 2,
        };
        least_squares_slope(&pts, degree) as f32
    }
}

/// Slope at t = 0 of the least-squares polynomial fit y(t) = b0 + b1 t (+ b2 t^2): b1.
fn least_squares_slope(pts: &[(f64, f64)], degree: usize) -> f64 {
    let n = degree + 1;
    // normal equations A^T A b = A^T y
    let mut m = [[0f64; 4]; 3];
    for &(t, y) in pts {
        let pow = [1.0, t, t * t];
        for r in 0..n {
            for c in 0..n {
                m[r][c] += pow[r] * pow[c];
            }
            m[r][3] += pow[r] * y;
        }
    }
    // Gauss-Jordan with partial pivoting on the n x (n+1) system
    for col in 0..n {
        let pivot = (col..n).max_by(|&a, &b| m[a][col].abs().total_cmp(&m[b][col].abs())).unwrap_or(col);
        m.swap(col, pivot);
        if m[col][col].abs() < 1e-12 {
            return 0.0;
        }
        let pivot_row = m[col];
        for (r, row) in m.iter_mut().enumerate().take(n) {
            if r != col {
                let f = row[col] / pivot_row[col];
                for (x, p) in row.iter_mut().zip(pivot_row.iter()).skip(col) {
                    *x -= f * p;
                }
            }
        }
    }
    m[1][3] / m[1][1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_matches_android() {
        let (position, time) = spline();
        // the bisection stops within 1e-5, so the first sample is ~0, not exactly 0 (same
        // in OverScroller); the last one is set to 1
        assert!(position[0] < 1e-4 && time[0] < 1e-4);
        assert_eq!(position[NB_SAMPLES], 1.0);
        assert!(position.windows(2).all(|w| w[0] <= w[1]));
        assert!(time.windows(2).all(|w| w[0] <= w[1]));
        assert!((deceleration_rate() - 2.358_201_7).abs() < 1e-5);
    }

    #[test]
    fn fling_distance_and_duration_like_android() {
        // Android at 160 dpi, friction 0.015, 2000 px/s: l = ln(0.35 * 2000 / (0.015 * 51890))
        // = -0.106; distance = 778.4 * e^(2.358 / 1.358 * l) = 647 px; duration =
        // 1000 * e^(l / 1.358) = 925 ms (getSplineFlingDistance / getSplineFlingDuration).
        let t0 = Instant::now();
        let f = Fling::start(t0, 0.0, 2000.0, 0.0, 1e9).unwrap();
        assert!((f.distance - 647.0).abs() < 2.0, "{}", f.distance);
        assert!((f.spline_duration.as_millis() as i64 - 925).abs() < 3, "{:?}", f.spline_duration);
        let (end, done) = f.position(t0 + f.duration);
        assert!(done);
        assert!((end - f.distance).abs() < 0.01);
        // decelerating: the first 100 ms cover much more than the last 100 ms
        let early = f.position(t0 + Duration::from_millis(100)).0;
        let late = end - f.position(t0 + f.duration - Duration::from_millis(100)).0;
        assert!(early > 8.0 * late, "{early} vs {late}");
    }

    #[test]
    fn fling_into_the_edge_stops_there_early() {
        let t0 = Instant::now();
        let f = Fling::start(t0, 0.0, 2000.0, 0.0, 100.0).unwrap();
        assert!(f.duration < f.spline_duration);
        let (end, done) = f.position(t0 + f.duration);
        assert!(done);
        assert_eq!(end, 100.0);
        assert!(Fling::start(t0, 0.0, 30.0, 0.0, 100.0).is_none());
    }

    #[test]
    fn velocity_of_a_steady_swipe() {
        let t0 = Instant::now();
        let mut v = VelocityTracker::default();
        v.down(t0, 0.0, 700.0);
        for i in 1..=10u64 {
            v.moved(t0 + Duration::from_millis(i * 8), 0.0, 700.0 - i as f32 * 16.0); // -2000 px/s
        }
        v.up(t0 + Duration::from_millis(85));
        assert!((v.velocity() + 2000.0).abs() < 1.0, "{}", v.velocity());
    }

    #[test]
    fn resting_before_lift_gives_no_velocity() {
        let t0 = Instant::now();
        let mut v = VelocityTracker::default();
        v.down(t0, 0.0, 700.0);
        v.moved(t0 + Duration::from_millis(10), 0.0, 650.0);
        v.moved(t0 + Duration::from_millis(20), 0.0, 600.0);
        v.up(t0 + Duration::from_millis(80));
        assert_eq!(v.velocity(), 0.0);
    }

    #[test]
    fn x_and_y_have_their_own_velocity() {
        let t0 = Instant::now();
        let mut v = VelocityTracker::default();
        v.down(t0, 900.0, 300.0);
        for i in 1..=6u64 {
            v.moved(t0 + Duration::from_millis(i * 8), 900.0 - i as f32 * 24.0, 300.0); // -3000 px/s in x
        }
        assert!((v.velocity_x() + 3000.0).abs() < 50.0, "{}", v.velocity_x());
        assert!(v.velocity().abs() < 1.0, "{}", v.velocity());
    }
}
