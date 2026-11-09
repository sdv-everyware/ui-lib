//! A scrolling list of rows (`#list-item`, or any `Row`) in a slot that clips: Android's touch handling
//! (scroller.rs: touch slop, drag, fling on release, a press catches a running fling) over
//! harry's lists. The rows sit one under the other, `pitch` apart, and the whole content
//! moves by the scroll; the harry glue (everyware_ui::list) puts every row at its y minus
//! the scroll of the moment, and steps a fling frame by frame on its own: the state here
//! only holds where the fling started, so no ticks reach the app.
//!
//! Gestures come per row (`#<slot>/<row>` from harry's hit test) or without a row (a move or
//! a lift outside every row, routed to the row the touch started on by `GestureTracker`).

use crate::gesture::Gesture;
use crate::list_item::{ListItem, ListItemEvent};
use std::fmt::Debug;
use crate::scroller::{Fling, TOUCH_SLOP};
use std::time::{Duration, Instant};

/// What the list reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListEvent {
    /// A row was tapped (not dragged).
    Chosen(usize),
}

/// A scrollbar stays for this long after the content stopped (Android's
/// ViewConfiguration.getScrollDefaultDelay), then fades.
pub const SCROLLBAR_DELAY: Duration = Duration::from_millis(300);

/// Where the content is: a settled scroll, or a fling from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scroll {
    /// The settled position (the start of the fling while one runs).
    pub base: f32,
    /// A fling in progress.
    pub fling: Option<Fling>,
    /// When a drag last moved the content.
    pub moved_at: Option<Instant>,
}

impl Scroll {
    /// The scroll at `now`, and whether it still moves.
    pub fn at(&self, now: Instant) -> (f32, bool) {
        match &self.fling {
            Some(fling) => {
                let (y, done) = fling.position(now);
                (y, !done)
            }
            None => (self.base, false),
        }
    }

    /// The content moved at `now` or within the scrollbar delay before: the scrollbar is
    /// shown. Also when it hides (for the next frame request).
    pub fn scrollbar_shown(&self, now: Instant) -> (bool, Option<Instant>) {
        let last = match (self.moved_at, self.fling.map(|f| f.ends_at())) {
            (Some(m), Some(f)) => Some(m.max(f)),
            (m, f) => m.or(f),
        };
        match last {
            Some(last) => {
                let until = last + SCROLLBAR_DELAY;
                (now < until, (now < until).then_some(until))
            }
            None => (false, None),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Drag {
    y0: f32,
    scroll0: f32,
    moved: bool,
    /// The row under the finger at the down, tapped if the finger lifts without dragging.
    row: Option<usize>,
}

/// A row of a `ListView`: a component that takes the list's gestures, can be the selected one
/// and is known by a key across a refresh of the rows (`set_rows`).
pub trait Row: Clone + Debug {
    /// What identifies the row across a refresh (a network's name, a story's id).
    fn key(&self) -> &str;
    /// Applies a gesture; true when it was a tap.
    fn tap(&mut self, gesture: Gesture) -> bool;
    /// Marks or unmarks the row as the selected one.
    fn set_selected(&mut self, selected: bool);
    /// Whether it is the selected one.
    fn is_selected(&self) -> bool;
    /// Takes over the pressed and selected state of the row it replaces.
    fn keep_state_of(&mut self, before: &Self);
}

impl Row for ListItem {
    fn key(&self) -> &str {
        &self.title
    }
    fn tap(&mut self, gesture: Gesture) -> bool {
        self.on(gesture) == Some(ListItemEvent::Clicked)
    }
    fn set_selected(&mut self, selected: bool) {
        ListItem::set_selected(self, selected)
    }
    fn is_selected(&self) -> bool {
        self.selected
    }
    fn keep_state_of(&mut self, before: &Self) {
        self.press = before.press.clone();
        self.selected = before.selected;
    }
}

/// A scrolling list of rows (`#list-item` by default, any `Row`).
#[derive(Clone, Debug)]
pub struct ListView<R: Row = ListItem> {
    rows: Vec<R>,
    /// Height of the slot (the clipping viewport).
    view_h: f32,
    /// Row height plus gap.
    pitch: f32,
    scroll: Scroll,
    drag: Option<Drag>,
    /// Rows side by side, scrolling in x (a ribbon of cards); `view_h` and `pitch` are then
    /// the slot's width and a row's width plus gap.
    horizontal: bool,
}

impl<R: Row> ListView<R> {
    /// The same list, scrolling sideways: rows left to right, the finger's x moves it, `view_h`
    /// is the slot's width and `pitch` a row's width plus gap.
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Whether the rows lie side by side.
    pub fn is_horizontal(&self) -> bool {
        self.horizontal
    }

    /// The finger's coordinate along the list.
    fn along(&self, x: f32, y: f32) -> f32 {
        if self.horizontal { x } else { y }
    }

    /// A list of these rows in a slot `view_h` high, rows `pitch` apart (row height plus gap).
    pub fn new(rows: impl IntoIterator<Item = R>, view_h: f32, pitch: f32) -> Self {
        ListView {
            rows: rows.into_iter().collect(),
            view_h,
            pitch,
            scroll: Scroll { base: 0.0, fling: None, moved_at: None },
            drag: None,
            horizontal: false,
        }
    }

    /// The rows, in order.
    pub fn rows(&self) -> &[R] {
        &self.rows
    }

    /// A row to change.
    pub fn row_mut(&mut self, row: usize) -> Option<&mut R> {
        self.rows.get_mut(row)
    }

    /// Replaces the rows (a new scan). A row with the same key keeps its pressed and
    /// selected state, so a tap in progress survives a refresh between its down and its up
    /// (a list fed by a topic refreshes at any moment); the scroll stays within the content.
    pub fn set_rows(&mut self, rows: impl IntoIterator<Item = R>) {
        let old = std::mem::take(&mut self.rows);
        self.rows = rows
            .into_iter()
            .map(|mut row| {
                if let Some(before) = old.iter().find(|r| r.key() == row.key()) {
                    row.keep_state_of(before);
                }
                row
            })
            .collect();
        if let Some(d) = self.drag.as_mut() {
            d.row = d.row.and_then(|i| old.get(i)).and_then(|r| self.rows.iter().position(|n| n.key() == r.key()));
        }
        self.settle(Instant::now());
        self.scroll.base = self.scroll.base.min(self.max_scroll());
    }

    /// The row pitch (height plus gap).
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// The viewport's height.
    pub fn view_h(&self) -> f32 {
        self.view_h
    }

    /// Where the content is (for the harry glue, which steps a fling itself).
    pub fn scroll(&self) -> Scroll {
        self.scroll
    }

    /// The scroll at `now`.
    pub fn scroll_at(&self, now: Instant) -> f32 {
        self.scroll.at(now).0
    }

    /// The content's height (rows times pitch; the gap after the last row stays).
    pub fn content_h(&self) -> f32 {
        self.rows.len() as f32 * self.pitch
    }

    /// How far the content scrolls at most.
    pub fn max_scroll(&self) -> f32 {
        (self.content_h() - self.view_h).max(0.0)
    }

    /// The scrollbar's thumb: where (0..100) along the track; 0 when nothing scrolls.
    pub fn thumb(&self, now: Instant) -> f32 {
        let max = self.max_scroll();
        if max <= 0.0 { 0.0 } else { self.scroll_at(now) / max * 100.0 }
    }

    /// The selected row.
    pub fn selected(&self) -> Option<usize> {
        self.rows.iter().position(|r| r.is_selected())
    }

    /// Marks a row as the selected one (none: clears the mark).
    pub fn select(&mut self, row: Option<usize>) {
        for (i, r) in self.rows.iter_mut().enumerate() {
            r.set_selected(Some(i) == row);
        }
    }

    /// Ends a finished fling: the settled position is where it stopped.
    fn settle(&mut self, now: Instant) {
        let (y, moving) = self.scroll.at(now);
        if !moving {
            let moved_at = self.scroll.moved_at.max(self.scroll.fling.map(|f| f.ends_at()));
            self.scroll = Scroll { base: y, fling: None, moved_at };
        }
    }

    /// Applies a gesture: on a row (harry's hit test), or on none (a move or a lift away from
    /// every row). A tap on a row chooses it; a finger that moves past the touch slop drags
    /// the content; a fast lift flings it; a press on a running fling stops it.
    pub fn on(&mut self, row: Option<usize>, gesture: Gesture, now: Instant) -> Option<ListEvent> {
        self.settle(now);
        match gesture {
            Gesture::Down { x, y } => {
                let y = self.along(x, y);
                // Like ScrollView: a press on a running fling stops it where it is and
                // starts a drag, it is never a click.
                let (y_now, _) = self.scroll.at(now);
                let caught = self.scroll.fling.take().is_some();
                let moved_at = if caught { Some(now) } else { self.scroll.moved_at };
                self.scroll = Scroll { base: y_now, fling: None, moved_at };
                let row = if caught { None } else { row.filter(|&i| i < self.rows.len()) };
                self.drag = Some(Drag { y0: y, scroll0: y_now, moved: caught, row });
                if let Some(i) = row {
                    self.rows[i].tap(gesture);
                }
                None
            }
            Gesture::Move { x, y, .. } => {
                let y = self.along(x, y);
                let max = self.max_scroll();
                let Some(d) = self.drag.as_mut() else { return None };
                if !d.moved && (y - d.y0).abs() > TOUCH_SLOP {
                    // The drag starts at the slop edge, not where the finger went down, so
                    // the content does not jump by the slop (ScrollView.onTouchEvent).
                    d.moved = true;
                    d.y0 += TOUCH_SLOP * (y - d.y0).signum();
                    if let Some(i) = d.row.take() {
                        self.rows[i].tap(Gesture::Cancel);
                    }
                }
                if d.moved {
                    self.scroll.base = (d.scroll0 + d.y0 - y).clamp(0.0, max);
                    self.scroll.moved_at = Some(now);
                }
                None
            }
            Gesture::Up { velocity, velocity_x, .. } => {
                let velocity = if self.horizontal { velocity_x } else { velocity };
                let d = self.drag.take()?;
                if !d.moved {
                    let i = d.row?;
                    return self.rows[i].tap(gesture).then_some(ListEvent::Chosen(i));
                }
                // The finger moving up scrolls towards the end of the list.
                let max = self.max_scroll();
                self.scroll.fling = Fling::start(now, self.scroll.base, -velocity, 0.0, max);
                None
            }
            Gesture::Cancel => {
                if let Some(d) = self.drag.take() {
                    if let Some(i) = d.row {
                        self.rows[i].tap(Gesture::Cancel);
                    }
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list_item::ListItemKind;
    use std::time::Duration;

    fn list(n: usize) -> ListView {
        ListView::new((0..n).map(|i| ListItem::new(ListItemKind::Plain, format!("row {i}"), "")), 424.0, 72.0)
    }

    #[test]
    fn content_and_max_scroll() {
        let l = list(12);
        assert_eq!(l.content_h(), 12.0 * 72.0);
        assert_eq!(l.max_scroll(), 12.0 * 72.0 - 424.0);
        assert_eq!(list(3).max_scroll(), 0.0);
    }

    #[test]
    fn a_tap_chooses_the_row_a_drag_does_not() {
        let now = Instant::now();
        let mut l = list(12);
        assert_eq!(l.on(Some(2), Gesture::down(0.0, 300.0), now), None);
        assert_eq!(l.on(Some(2), Gesture::up(0.0, 302.0), now), Some(ListEvent::Chosen(2)));
        // a drag past the slop scrolls and is no tap
        l.on(Some(2), Gesture::down(0.0, 300.0), now);
        l.on(None, Gesture::Move { x: 0.0, y: 200.0, inside: false }, now);
        assert_eq!(l.scroll_at(now), 100.0 - TOUCH_SLOP);
        assert_eq!(l.on(None, Gesture::Up { x: 0.0, y: 200.0, inside: false, velocity: 0.0, velocity_x: 0.0 }, now), None);
        assert_eq!(l.rows()[2].press.pressed, false);
    }

    #[test]
    fn a_fast_lift_flings_and_a_press_catches_it() {
        let now = Instant::now();
        let mut l = list(12);
        l.on(Some(0), Gesture::down(0.0, 400.0), now);
        l.on(None, Gesture::Move { x: 0.0, y: 300.0, inside: false }, now);
        l.on(None, Gesture::Up { x: 0.0, y: 300.0, inside: false, velocity: -2000.0, velocity_x: 0.0 }, now);
        assert!(l.scroll().fling.is_some());
        let later = now + Duration::from_millis(200);
        let y = l.scroll_at(later);
        assert!(y > 100.0 - TOUCH_SLOP + 50.0, "{y}");
        // the press stops it where it is; no tap comes from that press
        l.on(Some(3), Gesture::down(0.0, 200.0), later);
        assert!(l.scroll().fling.is_none());
        assert_eq!(l.scroll_at(later), y);
        assert_eq!(l.on(Some(3), Gesture::up(0.0, 200.0), later), None);
    }

    #[test]
    fn the_scrollbar_shows_while_moving_and_a_little_after() {
        let now = Instant::now();
        let mut l = list(12);
        assert_eq!(l.scroll().scrollbar_shown(now), (false, None));
        l.on(Some(0), Gesture::down(0.0, 400.0), now);
        l.on(None, Gesture::Move { x: 0.0, y: 300.0, inside: false }, now);
        assert!(l.scroll().scrollbar_shown(now).0);
        assert!(l.scroll().scrollbar_shown(now + Duration::from_millis(200)).0);
        assert!(!l.scroll().scrollbar_shown(now + Duration::from_millis(400)).0);
        assert!((l.thumb(now) - (100.0 - TOUCH_SLOP) / l.max_scroll() * 100.0).abs() < 0.01);
    }

    #[test]
    fn a_refresh_between_down_and_up_keeps_the_tap() {
        let now = Instant::now();
        let mut l = list(4);
        l.on(Some(2), Gesture::down(0.0, 300.0), now);
        // the rows come again, in another order
        l.set_rows((0..5).rev().map(|i| ListItem::new(ListItemKind::Plain, format!("row {i}"), "")));
        assert_eq!(l.on(None, Gesture::up(0.0, 302.0), now), Some(ListEvent::Chosen(2)));
        assert_eq!(l.rows()[2].title, "row 2");
    }

    #[test]
    fn a_fling_ends_at_the_content_end() {
        let now = Instant::now();
        let mut l = list(8);
        l.on(Some(0), Gesture::down(0.0, 400.0), now);
        l.on(None, Gesture::Move { x: 0.0, y: 100.0, inside: false }, now);
        l.on(None, Gesture::Up { x: 0.0, y: 100.0, inside: false, velocity: -8000.0, velocity_x: 0.0 }, now);
        let end = l.scroll_at(now + Duration::from_secs(5));
        assert_eq!(end, l.max_scroll());
    }

    #[test]
    fn a_horizontal_list_scrolls_with_the_finger_s_x() {
        let mut l = list(12).horizontal();
        let now = Instant::now();
        l.on(None, Gesture::Down { x: 600.0, y: 10.0 }, now);
        l.on(None, Gesture::Move { x: 400.0, y: 300.0, inside: false }, now);
        // 200 px to the left minus the slop; the y moved too and does not count
        assert_eq!(l.scroll_at(now), 200.0 - TOUCH_SLOP);
        l.on(None, Gesture::Up { x: 400.0, y: 300.0, inside: false, velocity: 0.0, velocity_x: -3000.0 }, now);
        assert!(l.scroll_at(now + Duration::from_millis(300)) > 200.0, "flings with x");
    }
}
