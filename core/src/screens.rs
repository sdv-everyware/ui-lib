//! The screens behind tabs, the pages that take turns without them, and how they switch. The
//! time of a switch is components.json's (tabs/switch, pages/switch); what moves is the effect:
//! per screen an opacity (0..100) and an offset from where the design puts it, each with its own
//! transition. A screen at opacity 0 is not drawn and takes no taps (harry skips the subtree). An
//! app draws its screens with FollowOpacity/FollowPosition, so a switch is display only: no tree
//! rebuild.
//!
//! A new animation is one more effect here, with its test: the tabs and the apps stay as they
//! are, the app names the effect in one place.

use crate::transitions::{self, Change, ComponentKind, Easing, Transition};
use std::time::Duration;

/// How the screens change on a switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Effect {
    /// The old screen fades out while the new one fades in.
    #[default]
    Fade,
    /// Material 3's fade through: the old screen fades out in the first 30 %, then the new one
    /// fades in.
    FadeThrough,
    /// The screens lie side by side in their order and slide (Material 3's shared axis X): the
    /// new one comes in one width from the side the order gives (a higher one from the right),
    /// the old one leaves one width to the other side and disappears when it is out, whatever
    /// screens lie between them. The others wait hidden one width beside the one that shows.
    Slide,
    /// The new screen is there at once.
    None,
}

/// Where one screen goes: its opacity and its x offset from the design's place, each with the
/// way there. `offset` is None for an effect that never moves a screen: the app then sets no
/// position and the design's layout stays as it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenTarget {
    pub opacity: f32,
    pub opacity_transition: Option<Transition>,
    pub offset: Option<f32>,
    pub offset_transition: Option<Transition>,
}

/// The share of a fade through that the old screen takes to go (Material 3: 90 of 300 ms).
const FADE_THROUGH_OUT: f32 = 0.3;

impl Effect {
    /// Where screen `screen` goes when `selected` is the chosen screen and `previous` the one
    /// before it; `switch` is the time of the whole switch, `width` the distance between two
    /// screens of a slide.
    pub fn target(self, screen: usize, selected: usize, previous: Option<usize>, width: f32, switch: Option<Transition>) -> ScreenTarget {
        let shown = screen == selected;
        let opacity = if shown { 100.0 } else { 0.0 };
        let still = |opacity_transition| ScreenTarget { opacity, opacity_transition, offset: None, offset_transition: None };
        match self {
            Effect::Fade => still(switch),
            Effect::FadeThrough => still(switch.map(|t| {
                if shown {
                    t.part(FADE_THROUGH_OUT, 1.0 - FADE_THROUGH_OUT, Easing::EaseOut)
                } else {
                    t.part(0.0, FADE_THROUGH_OUT, Easing::EaseIn)
                }
            })),
            Effect::Slide => {
                // only the old and the new screen move, one width each: the leaving one stays
                // drawn while it moves out and goes when the slide ends. The others are already
                // at 0 and wait one width away on their side of the one that shows, so the next
                // one comes from there.
                let leaving = previous == Some(screen) && !shown;
                let side = match screen.cmp(&selected) {
                    std::cmp::Ordering::Less => -1.0,
                    std::cmp::Ordering::Equal => 0.0,
                    std::cmp::Ordering::Greater => 1.0,
                };
                ScreenTarget {
                    opacity: if leaving { 0.0 } else { opacity },
                    opacity_transition: if leaving {
                        switch.map(|t| Transition { duration: Duration::ZERO, easing: Easing::Linear, delay: t.delay + t.duration })
                    } else {
                        None
                    },
                    offset: Some(side * width),
                    offset_transition: if shown || leaving { switch } else { None },
                }
            }
            Effect::None => still(None),
        }
    }
}

/// Pages that take turns in one place without tabs: a sheet's settings and the list one of its
/// rows opens, as Android's Settings goes one page deeper and Back comes back (Material 3's shared
/// axis: Effect::Slide). Which page shows and which showed before; the effect places every page.
/// The switch takes its time from components.json (pages/switch, as the tabs' screens).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pages {
    /// The page that shows.
    pub selected: usize,
    /// The page before it, while there was one.
    pub previous: Option<usize>,
}

impl Pages {
    /// Shows page `page`; the one that showed leaves with the effect.
    pub fn show(&mut self, page: usize) {
        if page != self.selected {
            self.previous = Some(self.selected);
            self.selected = page;
        }
    }

    /// Back to the first page with nothing before it (a sheet that opens again starts there):
    /// the pages take their places at once, nothing slides.
    pub fn reset(&mut self) {
        *self = Pages::default();
    }

    /// Pages that open on page `page` with nothing before it (a sheet that opens on a deeper
    /// page): the pages take their places at once, nothing slides.
    pub fn at(page: usize) -> Self {
        Pages { selected: page, previous: None }
    }

    /// The time of a switch (components.json, pages/switch).
    pub fn switch_transition() -> Option<Transition> {
        transitions::transition(ComponentKind::Pages, Change::Switch)
    }

    /// Where page `page` goes with this effect; `width` is the distance between two pages of a
    /// slide. Before the first switch (or after a reset) nothing moves.
    pub fn page(&self, page: usize, effect: Effect, width: f32) -> ScreenTarget {
        let switch = self.previous.and(Self::switch_transition());
        effect.target(page, self.selected, self.previous, width, switch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SWITCH: Option<Transition> = Some(Transition { duration: Duration::from_millis(300), easing: Easing::EaseOut, delay: Duration::ZERO });

    fn ms(t: Option<Transition>) -> Option<(u128, u128)> {
        t.map(|t| (t.delay.as_millis(), t.duration.as_millis()))
    }

    #[test]
    fn fade_crossfades_over_the_whole_switch_and_moves_nothing() {
        let new = Effect::Fade.target(1, 1, Some(0), 1920.0, SWITCH);
        let old = Effect::Fade.target(0, 1, Some(0), 1920.0, SWITCH);
        assert_eq!((new.opacity, ms(new.opacity_transition), new.offset), (100.0, Some((0, 300)), None));
        assert_eq!((old.opacity, ms(old.opacity_transition), old.offset), (0.0, Some((0, 300)), None));
    }

    #[test]
    fn fade_through_goes_out_first_then_in() {
        let old = Effect::FadeThrough.target(0, 1, Some(0), 1920.0, SWITCH);
        let new = Effect::FadeThrough.target(1, 1, Some(0), 1920.0, SWITCH);
        assert_eq!((old.opacity, ms(old.opacity_transition), old.opacity_transition.unwrap().easing), (0.0, Some((0, 90)), Easing::EaseIn));
        assert_eq!((new.opacity, ms(new.opacity_transition), new.opacity_transition.unwrap().easing), (100.0, Some((90, 210)), Easing::EaseOut));
    }

    #[test]
    fn slide_moves_the_old_and_the_new_screen_one_width_whatever_lies_between() {
        // from tab 0 to tab 2: 2 comes one width from the right, 0 leaves one width to the left,
        // 1 waits hidden on the left of 2 and does not move there
        let t = |s| Effect::Slide.target(s, 2, Some(0), 1920.0, SWITCH);
        assert_eq!((t(0).offset, t(1).offset, t(2).offset), (Some(-1920.0), Some(-1920.0), Some(0.0)));
        assert_eq!((t(2).opacity, t(2).opacity_transition), (100.0, None));
        assert_eq!((t(0).opacity, ms(t(0).opacity_transition)), (0.0, Some((300, 0))));
        assert_eq!((t(1).opacity, t(1).opacity_transition, t(1).offset_transition), (0.0, None, None));
        assert_eq!((ms(t(2).offset_transition), ms(t(0).offset_transition)), (Some((0, 300)), Some((0, 300))));
        // before the switch 2 waited one width to the right of 0, so it comes from there
        assert_eq!(Effect::Slide.target(2, 0, None, 1920.0, SWITCH).offset, Some(1920.0));
        // back from 2 to 0: 0 comes from the left, where it left to; 2 leaves to the right
        let back = |s| Effect::Slide.target(s, 0, Some(2), 1920.0, SWITCH);
        assert_eq!((back(0).offset, back(1).offset, back(2).offset), (Some(0.0), Some(1920.0), Some(1920.0)));
        // the direction follows the order of the screens, not where the switch comes from
        assert_eq!(Effect::Slide.target(2, 3, Some(2), 880.0, SWITCH).offset, Some(-880.0));
    }

    /// The slide that moves every screen by the screens between it and the one that shows, each
    /// with the switch's transition: the reference for a switch between neighbours.
    fn by_the_screens_between(screen: usize, selected: usize, previous: Option<usize>, width: f32, switch: Option<Transition>) -> ScreenTarget {
        let leaving = previous == Some(screen) && screen != selected;
        ScreenTarget {
            opacity: if screen == selected { 100.0 } else { 0.0 },
            opacity_transition: if leaving { switch.map(|t| Transition { duration: Duration::ZERO, easing: Easing::Linear, delay: t.delay + t.duration }) } else { None },
            offset: Some((screen as f32 - selected as f32) * width),
            offset_transition: switch,
        }
    }

    #[test]
    fn a_slide_between_neighbours_moves_what_shows_as_by_the_screens_between() {
        for n in 2..=4usize {
            for from in 0..n {
                for to in [from.wrapping_sub(1), from + 1].into_iter().filter(|&to| to < n) {
                    for s in 0..n {
                        let new = Effect::Slide.target(s, to, Some(from), 880.0, SWITCH);
                        let old = by_the_screens_between(s, to, Some(from), 880.0, SWITCH);
                        if s == from || s == to || n == 2 {
                            // the old and the new screen: the same place, opacity and transitions
                            assert_eq!(new, old, "{n} screens, {from} -> {to}, screen {s}");
                        } else {
                            // a screen further away is hidden in both and is not drawn
                            assert_eq!((new.opacity, new.opacity_transition, old.opacity, old.opacity_transition), (0.0, None, 0.0, None));
                        }
                    }
                }
            }
        }
        // two pages before the first switch (Pages gives no transition then): the same too
        for s in 0..2 {
            assert_eq!(Effect::Slide.target(s, 0, None, 880.0, None), by_the_screens_between(s, 0, None, 880.0, None));
        }
    }

    #[test]
    fn pages_go_one_deeper_and_back_as_a_slide() {
        let mut pages = Pages::default();
        pages.show(1);
        // the deeper page comes from the right, the first leaves to the left
        assert_eq!((pages.page(1, Effect::Slide, 880.0).offset, pages.page(0, Effect::Slide, 880.0).offset), (Some(0.0), Some(-880.0)));
        assert_eq!(ms(pages.page(1, Effect::Slide, 880.0).offset_transition), Some((0, 300)));
        pages.show(0);
        assert_eq!((pages.selected, pages.previous, pages.page(1, Effect::Slide, 880.0).offset), (0, Some(1), Some(880.0)));
        // showing the page that shows changes nothing
        pages.show(0);
        assert_eq!(pages.previous, Some(1));
        pages.reset();
        assert_eq!(pages, Pages::default());
        // after a reset the pages take their places at once
        assert_eq!(pages.page(0, Effect::Slide, 880.0).offset_transition, None);
    }

    #[test]
    fn pages_open_on_another_page_without_a_slide() {
        let mut pages = Pages::at(2);
        assert_eq!((pages.selected, pages.previous), (2, None));
        let p = |pages: &Pages, i| pages.page(i, Effect::Slide, 880.0);
        // the page shows at its place at once, the others wait on their sides
        assert_eq!((p(&pages, 2).offset, p(&pages, 2).opacity, p(&pages, 2).offset_transition), (Some(0.0), 100.0, None));
        assert_eq!((p(&pages, 0).offset, p(&pages, 3).offset, p(&pages, 3).opacity), (Some(-880.0), Some(880.0), 0.0));
        // a step from there slides as from any page
        pages.show(3);
        assert_eq!((p(&pages, 3).offset, ms(p(&pages, 3).offset_transition), p(&pages, 2).offset), (Some(0.0), Some((0, 300)), Some(-880.0)));
    }

    #[test]
    fn none_switches_at_once() {
        let new = Effect::None.target(1, 1, Some(0), 1920.0, SWITCH);
        let old = Effect::None.target(0, 1, Some(0), 1920.0, SWITCH);
        assert_eq!((new.opacity, new.opacity_transition, old.opacity, old.opacity_transition), (100.0, None, 0.0, None));
    }

    #[test]
    fn at_start_the_first_screen_shows_with_every_effect() {
        for e in [Effect::Fade, Effect::FadeThrough, Effect::Slide, Effect::None] {
            assert_eq!(e.target(0, 0, None, 1920.0, SWITCH).opacity, 100.0, "{e:?}");
            assert_eq!(e.target(1, 0, None, 1920.0, SWITCH).opacity, 0.0, "{e:?}");
        }
    }
}
