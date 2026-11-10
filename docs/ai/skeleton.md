# #skeleton

a placeholder while content loads (Material's skeleton loader): grey shapes where the content will be, a heading bar, four text lines or a 16:9 picture, 440 or 832 wide (wide: a side panel's 880 content less 24 on each side)

## Figma

- set: `#skeleton` (node 2-3007 in the Library)
- property `kind`: `heading` | `text` | `media`
- property `size`: `regular` | `wide`
- variant string: `kind=heading, size=regular` (the properties in this order; `Skeleton::variant()` produces it, never typed by hand)
- parts the code sets: a placeholder while content loads (Material's skeleton loader): grey shapes where the content will be, a heading bar, four text lines or a 16:9 picture, 440 or 832 wide (wide: a side panel's 880 content less 24 on each side); a light band (#skeleton/shimmer/1..3, horizontal progress markers with the same travel) sweeps across it while loading (change sweep, repeated from the start), still when not; the app fades it out with change hide when the content shows. Display only: the clone takes no taps
- nodes set by code (`Skeleton`, harry/src/slot.rs): `#skeleton/shimmer/{i}`, `#skeleton/shimmer/<…>`
- on the screens now: `#panel/skeleton-1` in #panel-content (state=default) (node 7-10); `#panel/skeleton-2` in #panel-content (state=default) (node 7-12); `#panel/skeleton-3` in #panel-content (state=default) (node 7-14); `#panel/skeleton-4` in #panel-content (state=default) (node 7-16)
- to place one: a frame named `#<screen>/<name>` with an instance of `#skeleton` inside (the slot); texts in it are placeholders, nothing hidden

## Transitions (components.json, through `transition()`)

- hide: 250 ms ease-in
- sweep: 1400 ms linear

## Rust

- module: `core/src/skeleton.rs`, type `Skeleton`; binding: `harry/src/slot.rs` (`impl Slotted for Skeleton`)
- `pub fn new(kind: SkeletonKind, size: SkeletonSize) -> Self`
- pub struct `Skeleton`: A skeleton placeholder: its shape, and since when it is loading (None: still).
  - field `pub kind: SkeletonKind`: What it stands for.
  - field `pub size: SkeletonSize`: How wide it is.
  - field `pub since: Option<Instant>`: Loading since then: the band sweeps; None: the shapes stand still.
  - `pub fn start(&mut self, now: Instant)`: Loading from `now`: the band starts sweeping (a skeleton already loading keeps its phase, so a second call does not make it jump).
  - `pub fn stop(&mut self)`: Loaded (or given up): the band stops.
  - `pub fn is_loading(&self) -> bool`: Whether the band sweeps.
  - `pub fn period() -> Duration`: The length of one pass of the band (components.json, `sweep`).
  - `pub fn sweep_at(&self, now: Instant) -> Option<f32>`: Where the band is at `now`, 0..100 across the skeleton; None when it stands still.

## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)

```rust
// imports
use everyware_ui::skeleton::{Skeleton, SkeletonKind, SkeletonSize};

// state.rs: in struct State, and its first value in State::new
pub panel_skeleton_1: Skeleton,
// panel_skeleton_1: Skeleton::new(SkeletonKind::Text, SkeletonSize::Regular),

// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)
panel_skeleton_1: Slot = "#panel/skeleton-1" ["SlotCustomizer"];

// presenter.rs: after every action
model.panel_skeleton_1 = slot("#panel/skeleton-1", &state.panel_skeleton_1);

// actions.rs: nothing, display only
```

Display only; `state.panel_skeleton_1.start(Instant::now())` while the content loads.

## Live: `#panel/skeleton-1` in smarthome-app (cut from the sources at build time)

```rust
// ui_model.rs:257
panel_skeleton_1: Slot = "#panel/skeleton-1" ["SlotCustomizer"];

// presenter.rs:106
model.panel_skeleton_1 = slot("#panel/skeleton-1", &browser.skeletons[0]);
```

## Check

- `everyware_ui_test` (host): the variants and parts the code uses exist in the app's .dcf
- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)
