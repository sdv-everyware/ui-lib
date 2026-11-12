---
title: Skeleton
role: Tells the user that content is on its way: grey shapes where it will be, with a light band sweeping across them.
rust: core/src/skeleton.rs
types: Skeleton, SkeletonKind, SkeletonSize
---
While a page or a card loads, the user sees grey shapes in its place (a heading bar, four lines of text, a 16:9 picture) and a light band that sweeps across them from left to right, again and again. When the content is there the shapes fade out and the content shows. The app says when loading starts and when it ends.

## When to use it
- Content that takes a moment and whose shape is known: a page in a side panel, the first stories of a feed. The shapes tell the user what will come and where.
- Not for a value that is only missing (show the last one, or nothing), and not for an action in progress the user waits on: that is a state of the control the user tapped.

## Quick start
**What.** Grey shapes where a page will appear, with a light band sweeping over them while it loads. When the page is there, the app fades them out.

**Why.** The user sees at once that something is coming and where. The app only says when loading starts and when it ends; the sweep runs by itself.

<!-- example: docs/examples/skeleton.rs use slot=#panel/skeleton-1 field=skeleton -->
```rust
use everyware_ui::skeleton::{Skeleton, SkeletonKind, SkeletonSize};
use everyware_ui::transitions::{self, Change, ComponentKind};
```

<!-- example: docs/examples/skeleton.rs state slot=#panel/skeleton-1 field=skeleton -->
```rust
pub skeleton: Skeleton,
```

<!-- example: docs/examples/skeleton.rs init slot=#panel/skeleton-1 field=skeleton -->
```rust
// four text lines, 832 wide: the width of a side panel's content
skeleton: Skeleton::new(SkeletonKind::Text, SkeletonSize::Wide),
```

<!-- example: docs/examples/skeleton.rs model slot=#panel/skeleton-1 field=skeleton -->
```rust
skeleton: Slot = "#panel/skeleton-1" ["SlotCustomizer"];
// the frame that holds the skeletons over the content: shown while loading, then it fades
loading: Meter = "#panel/loading" ["FollowOpacityCustomizer"] = Meter::new(0.0, None);
```

<!-- example: docs/examples/skeleton.rs present slot=#panel/skeleton-1 field=skeleton -->
```rust
model.skeleton = slot("#panel/skeleton-1", &state.skeleton);
model.loading = if state.skeleton.is_loading() {
    Meter::new(100.0, None)
} else {
    Meter::new(0.0, transitions::transition(ComponentKind::Skeleton, Change::Hide))
};
```

## How it behaves
- **Loading.** The band (`#skeleton/shimmer/<i>`, three markers: a bright core between two soft edges) sweeps across the shapes in 1400 ms, then starts again at the left edge. harry draws it at 30 frames a second.
- **Still.** Before `start` and after `stop` the shapes stand still.
- **Done.** The loader's frame fades out in 250 ms (the transition `hide`), and the content under it shows.
- **No taps.** The skeleton takes no taps; a tap reaches what lies under it.

## From the app
**What.** The app starts the skeleton when it asks for the content and stops it when the content is there, or when it gives up waiting.

**Why.** Only the app knows when loading starts and ends (a page asked for, a page shown). `start` keeps the phase if the skeleton loads already, so the example stops it first: a new page starts the band again from its edge.

<!-- example: docs/examples/skeleton.rs change slot=#panel/skeleton-1 field=skeleton -->
```rust
/// The app asked for the page: the loader shows at once and the band starts to sweep. A new
/// page while one still loads starts the band again from its edge.
pub fn page_asked(state: &mut State, now: Instant) {
    state.skeleton.stop();
    state.skeleton.start(now);
}

/// The page is there (or the app gave up): the band stops and the loader fades out in 250 ms.
pub fn page_shown(state: &mut State) {
    state.skeleton.stop();
}
```

## In Figma
A slot named `#<screen>/<name>` (`#panel/skeleton-1`) with an instance of `#skeleton` of the shape and width the content will have, where the content will be. Several skeletons for one page (a heading, text, a picture) go into one frame over the content (`#panel/loading`), which the app fades.

## How it works
The variant is `kind` (`heading`, `text`, `media`) and `size` (`regular` 440, `wide` 832, a side panel's content). The band is three horizontal progress markers, `#skeleton/shimmer/<i>` (1 to 3), with the same travel: the binding sets them to `sweep_at(now)`, 0 to 100 over one period (`Skeleton::period`, components.json change `sweep`). The clone is passive: it takes no taps.

## Traps
- The skeleton does not hide itself: fade its frame out with the transition `hide` when the content shows, as the example does, or the grey shapes stay over the content.
- A second `start` while it loads keeps the phase (no jump); for a new load call `stop` first.
- Several skeletons of one loader start at the same time, so their bands sweep together.

## Example
[docs/examples/skeleton.rs](../examples/skeleton.rs): the whole path, from the page asked for to the sweeping band and the fade when it shows, compiled and run by `everyware_ui_recipes_test`.
