---
title: Bar chart
role: A day at a glance as a row of 24 bars: a forecast by the hour, the energy a house used.
rust: core/src/bar_chart.rs
types: Bar, BarChart
---
The user reads a shape: where the bars are high, where they are low, which ones stand out in the accent colour. Nothing to tap. The app decides what a bar stands for, scales its data to the bars' heights and picks the accented ones.

## When to use it
- A series of up to 24 values whose shape matters more than each number: the temperature per hour with the rainy hours accented, the power per hour with the peak hours.
- One level (a battery, a volume): a `#slider` the user cannot move is the wrong tool; show the number as text. Exact values per item: a list of `#list-item` rows.

## Quick start
**What.** The 24 bars of the weather forecast: the app gives a temperature and the chance of rain per hour, and the chart shows them as bars, rainy hours in the accent colour.

**Why.** The chart only draws heights from 0 to 100. The app knows its data, so it turns degrees into heights.

<!-- example: docs/examples/bar_chart.rs use slot=#weather/chart field=forecast -->
```rust
use everyware_ui::bar_chart::{BarChart, BARS};
```

<!-- example: docs/examples/bar_chart.rs state slot=#weather/chart field=forecast -->
```rust
pub forecast: BarChart,
```

<!-- example: docs/examples/bar_chart.rs init slot=#weather/chart field=forecast -->
```rust
// all bars at zero until the first forecast
forecast: BarChart::new([(0.0, false); BARS]),
```

<!-- example: docs/examples/bar_chart.rs model slot=#weather/chart field=forecast -->
```rust
forecast: Slot = "#weather/chart" ["SlotCustomizer"];
```

<!-- example: docs/examples/bar_chart.rs present slot=#weather/chart field=forecast -->
```rust
model.forecast = slot("#weather/chart", &state.forecast);
```

## How it behaves
- **Bars.** 24 bars from the left, each as high as its level; a level outside 0..100 is clamped.
- **Accent.** An accented bar is drawn in the accent colour, the others in the chart's ink.
- **Less data.** Bars without a value stay at zero: they are there, only flat. Values beyond 24 are ignored.

## From the app
**What.** New data comes (the forecast for the next 24 hours): the app scales it to levels and builds the chart again.

**Why.** `BarChart::new(bars)` takes (level, accent) pairs and does nothing else, so the presenter draws exactly what the app computed. Here the coldest hour sits near the bottom and the warmest near the top, and an hour with rain is accented.

<!-- example: docs/examples/bar_chart.rs change slot=#weather/chart field=forecast -->
```rust
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
    state.forecast = BarChart::new(hours.iter().map(|&(temperature, rain)| ((temperature - lo) / (hi - lo) * 100.0, rain >= 50.0)));
}
```

## In Figma
A slot named by the app (`#weather/chart` on the smarthome panel) of the chart's size with the one instance of `#bar-chart`: 24 bars drawn, each a progress bar with its accent twin.

## How it works
`#bar-chart` has one variant, `bars=24`. `#bar-chart/bar/<1..24>` are vertical progress bar meters (the level, 0..100); `#bar-chart/bar/<i>/accent` is the same bar in the accent colour, at the same level, shown by opacity. Both are display only: new data moves meters, no tree is rebuilt.

## Traps
- Fewer than 24 values leave the last bars at zero, never hidden: a hidden node stays hidden in harry.
- Levels from raw data (19 °C as level 19) fill a fifth of the chart: scale them to the range of the data, as the example does.

## Example
[docs/examples/bar_chart.rs](../examples/bar_chart.rs): the whole path, from a forecast to its 24 bars with the rainy hours accented, compiled and run by `everyware_ui_recipes_test`.
