---
title: Bar chart
role: A row of 24 bars, each with a level and an accent; display only. The app decides what a bar stands for.
rust: core/src/bar_chart.rs
types: Bar, BarChart
---
`#bar-chart` has one variant, `bars=24`: `#bar-chart/bar/<1..24>` are vertical progress bar meters (the level, 0..100), `#bar-chart/bar/<i>/accent` is the same bar in the accent colour, shown by opacity. `BarChart::new(bars)` takes (level, accent) pairs; it clamps a level to 0..100 and does nothing else. Scaling data to levels (a day's temperatures to the chart's height) and choosing the accented bars (the hours with rain) is the app's presenter.

## In Figma
A slot named by the app (`#weather/chart` on the smarthome panel) of the chart's size with the one instance of `#bar-chart`; 24 bars drawn, each a progress bar with its accent twin.

## Traps
- Fewer than 24 pairs leave the last bars at zero, never hidden: hidden is forever in harry.
