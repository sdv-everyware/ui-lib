//! #bar-chart in an app: the next 24 hours of a weather forecast, a bar per hour for the
//! temperature and the accent for the hours with rain. The chart only draws; the app scales its
//! data to levels. The docs show the parts between the doc marks (docs/build-docs.js); the
//! tests at the end compile and run them (everyware_ui_recipes_test). Edit here: this file is
//! the example of the guide docs/content/bar_chart.md.

// doc:note Display only: `BarChart::new(bars)` takes up to 24 (level 0..100, accent) pairs. What a bar stands for, its scale and its accent are the app's.

use crate::prelude::*;
// doc:use
use everyware_ui::bar_chart::{BarChart, BARS};
// doc:end

// state.rs: in struct State, and its first value in State::new
pub struct State {
    // doc:state
    pub my_bar_chart: BarChart,
    // doc:end
}

impl State {
    pub fn new(sizes: &Sizes) -> Self {
        State {
            // doc:init
            // all bars at zero until the first forecast
            my_bar_chart: BarChart::new([(0.0, false); BARS]),
            // doc:end
        }
    }
}

// ui_model.rs: one field per node the app writes (the contract with the design)
everyware_ui::ui_model! {
    // doc:model
    my_bar_chart: Slot = "#doc/bar_chart" ["SlotCustomizer"];
    // doc:end
}

// presenter.rs: after every action
pub fn present(model: &mut UiModel, state: &State) {
    // doc:present
    model.my_bar_chart = slot("#doc/bar_chart", &state.my_bar_chart);
    // doc:end
}

// doc:change
/// A new forecast: per hour the temperature (°C) and the chance of rain (%). The day's coldest
/// hour is near the bottom and its warmest near the top; a flat day spans four degrees, so a
/// change of one degree does not fill the chart. An hour with rain of 50 % or more is accented.
pub fn forecast_reported(state: &mut State, hours: &[(f32, f32)]) {
    let hours = &hours[..hours.len().min(BARS)];
    let (lo, hi) = hours.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), h| (lo.min(h.0), hi.max(h.0)));
    let (mut lo, mut hi) = (lo - 1.0, hi + 1.0);
    if hi - lo < 4.0 {
        let mid = (lo + hi) / 2.0;
        (lo, hi) = (mid - 2.0, mid + 2.0);
    }
    state.my_bar_chart = BarChart::new(hours.iter().map(|&(temperature, rain)| ((temperature - lo) / (hi - lo) * 100.0, rain >= 50.0)));
}
// doc:end

#[test]
fn bar_chart_shows_a_forecast() {
    let mut state = State::new(&Sizes::none());
    let mut model = UiModel::new(HashMap::new());
    present(&mut model, &state);
    assert_eq!(state.my_bar_chart.variant(), "bars=24");
    assert!(state.my_bar_chart.bars.iter().all(|b| b.level == 0.0 && !b.accent));
    // 24 hours from 8 to 19 °C and back, rain in the afternoon
    let hours: Vec<(f32, f32)> = (0..24).map(|h| (8.0 + 11.0 * (1.0 - ((h as f32 - 12.0) / 12.0).abs()), if (14..18).contains(&h) { 80.0 } else { 10.0 })).collect();
    forecast_reported(&mut state, &hours);
    present(&mut model, &state);
    let bars = &state.my_bar_chart.bars;
    // the coldest hour near the bottom, the warmest near the top: one degree of margin each
    assert_eq!((bars[0].level.round(), bars[12].level.round()), (8.0, 92.0));
    assert_eq!(bars.iter().filter(|b| b.accent).count(), 4);
    assert!(bars[15].accent && !bars[13].accent);
}

#[test]
fn bar_chart_with_a_flat_or_short_day() {
    let mut state = State::new(&Sizes::none());
    // the same temperature all day: the bars in the middle, not at the top
    forecast_reported(&mut state, &[(12.0, 0.0); 24]);
    assert_eq!(state.my_bar_chart.bars[0].level, 50.0);
    // six hours only: the other bars stay at zero, they are never hidden
    forecast_reported(&mut state, &[(10.0, 0.0), (11.0, 0.0), (12.0, 60.0), (13.0, 0.0), (14.0, 0.0), (15.0, 0.0)]);
    assert!(state.my_bar_chart.bars[2].accent && state.my_bar_chart.bars[5].level > 0.0);
    assert!(state.my_bar_chart.bars[6..].iter().all(|b| b.level == 0.0));
    // no data: all at zero
    forecast_reported(&mut state, &[]);
    assert!(state.my_bar_chart.bars.iter().all(|b| b.level == 0.0));
}
