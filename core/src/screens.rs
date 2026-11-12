//! The screens behind tabs, and how they switch. The time of a switch is components.json's
//! (tabs/switch); what moves is the effect: per screen an opacity (0..100) and an offset from
//! where the design puts it, each with its own transition. A screen at opacity 0 is not drawn
//! and takes no taps (harry skips the subtree). An app draws its screens with
//! FollowOpacity/FollowPosition, so a switch is display only: no tree rebuild.
//!
//! A new animation is one more effect here, with its test: the tabs and the apps stay as they
//! are, the app names the effect in one place.

use crate::transitions::{Easing, Transition};
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
    /// The screens lie side by side in the order of the tabs and slide by one width per tab; the
    /// old one disappears when it is out.
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
                // the leaving screen stays drawn while it moves out and goes when the slide ends;
                // the others are already at 0 and stay there
                let leaving = previous == Some(screen) && !shown;
                ScreenTarget {
                    opacity: if leaving { 0.0 } else { opacity },
                    opacity_transition: if leaving {
                        switch.map(|t| Transition { duration: Duration::ZERO, easing: Easing::Linear, delay: t.delay + t.duration })
                    } else {
                        None
                    },
                    offset: Some((screen as f32 - selected as f32) * width),
                    offset_transition: switch,
                }
            }
            Effect::None => still(None),
        }
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
    fn slide_lays_the_screens_out_in_tab_order() {
        // from tab 0 to tab 2: 0 leaves to the left, 2 comes from the right, 1 passes hidden
        let t = |s| Effect::Slide.target(s, 2, Some(0), 1920.0, SWITCH);
        assert_eq!((t(0).offset, t(1).offset, t(2).offset), (Some(-3840.0), Some(-1920.0), Some(0.0)));
        assert_eq!((t(2).opacity, t(2).opacity_transition), (100.0, None));
        assert_eq!((t(0).opacity, ms(t(0).opacity_transition)), (0.0, Some((300, 0))));
        assert_eq!((t(1).opacity, t(1).opacity_transition), (0.0, None));
        assert_eq!(ms(t(2).offset_transition), Some((0, 300)));
        // back to the right: the direction follows the order
        assert_eq!(Effect::Slide.target(1, 0, Some(2), 1920.0, SWITCH).offset, Some(1920.0));
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
