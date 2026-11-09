# #bar-chart

A row of 24 bars, each with a level and an accent; display only. The app decides what a bar stands for.

## Figma

- set: `#bar-chart` (node 2-2421 in the Library)
- property `bars`: `24`
- variant string: `bars=24` (the properties in this order; `Bar::variant()` produces it, never typed by hand)
- parts the code sets: display only: #bar-chart/bar/<1..24> vertical progress bar meters (the level, 0..100), #bar-chart/bar/<i>/accent the same bar in the accent colour, shown by opacity
- nodes set by code (`BarChart`, harry/src/slot.rs): `#bar-chart/bar/{}`, `#bar-chart/bar/<…>`
- on the screens now: `#weather/chart` in #overview (node 1-70)
- to place one: a frame named `#<screen>/<name>` with an instance of `#bar-chart` inside (the slot); texts in it are placeholders, nothing hidden

## Rust

- module: `core/src/bar_chart.rs`, type `Bar`; binding: `harry/src/slot.rs` (`impl Slotted for BarChart`)
- pub struct `Bar`: One bar.
  - field `pub level: f32`: The bar's height, 0..100.
  - field `pub accent: bool`: Drawn in the accent colour.
- pub struct `BarChart`: A row of bars, the first on the left.
  - field `pub bars: [Bar; BARS]`: The bars.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::bar_chart::BarChart;

// state.rs: in struct State, and its first value in State::new
pub weather_chart: BarChart,
// weather_chart: BarChart::new([(40.0, false); 24]),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
weather_chart: Slot = "#weather/chart" ["SlotCustomizer"];

// presenter.rs: after every action
model.weather_chart = slot("#weather/chart", &state.weather_chart);

// actions.rs: nothing, display only
```

Display only; `BarChart::new(bars)` takes up to 24 (level 0..100, accent) pairs. The app scales its data to levels and decides which bars are accented.

## Live: `#weather/chart` in smarthome-app (cut from the sources at build time)

```rust
// ui_model.rs:196
weather_chart: Slot = "#weather/chart" ["SlotCustomizer"];

// presenter.rs:139
model.weather_chart = slot("#weather/chart", &chart);
```

`#bar-chart` has one variant, `bars=24`: `#bar-chart/bar/<1..24>` are vertical progress bar meters (the level, 0..100), `#bar-chart/bar/<i>/accent` is the same bar in the accent colour, shown by opacity. `BarChart::new(bars)` takes (level, accent) pairs; it clamps a level to 0..100 and does nothing else. Scaling data to levels (a day's temperatures to the chart's height) and choosing the accented bars (the hours with rain) is the app's presenter.

## In Figma
A slot named by the app (`#weather/chart` on the smarthome panel) of the chart's size with the one instance of `#bar-chart`; 24 bars drawn, each a progress bar with its accent twin.

## Traps
- Fewer than 24 pairs leave the last bars at zero, never hidden: hidden is forever in harry.

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
