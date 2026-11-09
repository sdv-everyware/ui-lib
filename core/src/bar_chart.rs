//! A bar chart, display only (#bar-chart): a row of bars, each with a level and an accent. The
//! component does not know what a bar stands for: the app scales its data to 0..100 and says
//! which bars are accented.

use crate::transitions::{Change, ComponentKind};
use crate::Component;

/// The bars of #bar-chart.
pub const BARS: usize = 24;

/// One bar.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bar {
    /// The bar's height, 0..100.
    pub level: f32,
    /// Drawn in the accent colour.
    pub accent: bool,
}

/// A row of bars, the first on the left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BarChart {
    /// The bars.
    pub bars: [Bar; BARS],
}

impl BarChart {
    /// Bars from (level, accent) pairs: a level outside 0..100 is clamped, pairs beyond the
    /// chart's bars are ignored, bars without a pair stay at zero.
    pub fn new(bars: impl IntoIterator<Item = (f32, bool)>) -> Self {
        let mut chart = BarChart::default();
        for (bar, (level, accent)) in chart.bars.iter_mut().zip(bars) {
            *bar = Bar { level: level.clamp(0.0, 100.0), accent };
        }
        chart
    }
}

impl Component for BarChart {
    const KIND: ComponentKind = ComponentKind::BarChart;

    fn variant(&self) -> String {
        format!("bars={BARS}")
    }

    fn last_change(&self) -> Option<Change> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_are_clamped_and_the_rest_stays_at_zero() {
        let chart = BarChart::new([(50.0, false), (130.0, true), (-4.0, false)]);
        assert_eq!(chart.bars[0], Bar { level: 50.0, accent: false });
        assert_eq!(chart.bars[1], Bar { level: 100.0, accent: true });
        assert_eq!(chart.bars[2].level, 0.0);
        assert_eq!(chart.bars[BARS - 1], Bar::default());
        assert_eq!(chart.variant(), "bars=24");
    }

    #[test]
    fn pairs_beyond_the_bars_are_ignored() {
        let chart = BarChart::new((0..40).map(|i| (i as f32, i % 2 == 0)));
        assert_eq!(chart.bars[BARS - 1], Bar { level: 23.0, accent: false });
    }
}
