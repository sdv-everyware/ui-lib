Every component lives in three places that agree, and one file is the source for two of them:

```
components.json  ──► Figma: the library's component sets, their variants and transitions
                 ──► Rust:  transitions (core/src/transitions.rs)
core/src/<component>.rs   the state machine: gestures in, variant and values out, events to the app
harry/src/slot.rs         the binding: a slot on the screen gets a clone of the variant, with the parts the code sets
```

On a screen a component sits in a **slot**: a frame named for the app (`#wifi/connect`) with a preview instance of the component inside. harry clones the chosen variant into the slot and gives the clone its own values (texts, opacities, meters), so one component can be on the screen many times. The app keeps one `Slot` field per place on the screen and writes it in its presenter: `model.connect = slot("#wifi/connect", &state.connect)`.

Components are named by their **role** (Material 3: button, list item, segmented button, tabs, tile), never by what the app puts in them. A new need extends a component by a property or a part before a new component is made.

## What lives where: Figma, the code, the spec

Three sources make every frame, and each decides one thing: the code decides **what** is shown,
the spec **how it moves** there, Figma **what it looks like**.

![Each frame: the presenter picks the variant from the state (1), the component's transition moves the change with the spec's duration and curve (2), the renderer draws the variant's picture from the .dcf (3).](diagrams/what-lives-where.svg)

| What | Where it is decided | Who reads it |
|---|---|---|
| layout and geometry (where a tile is, a sheet's 880 x 1000) | Figma, fetched into the `.dcf` | harry draws it as it is |
| colours, fonts, radii, shadows | Figma | harry |
| the variants: every state as a picture (button default / pressed / disabled, toggle on / off, sheet closed / open) | Figma, the component sets of the Library | harry shows the variant the code picks |
| the `#names`: which layer the code touches | Figma | the app's `ui_model.rs` (the contract) |
| preview texts and pictures | Figma | harry, until the code writes its own |
| which variant is shown now | the code: State → presenter | |
| what a tap does | the code: `HomeAction::Gesture` → the reducer | |
| how long a change takes, on which curve | `components.json` → `core/src/transitions.rs` | the component's `transition()` |
| the data: news, weather, devices, Wi-Fi | the code: the bundles → the reducer | |
| the prototype: taps and transitions in Play | Figma | **designers only**: harry never reads it |

![One tap, two worlds. In Play the prototype opens a frame that exists only in Figma; on the device the same tap goes through the reducer. DesignCompose, for comparison, runs the prototype's reactions itself.](diagrams/a-tap-two-worlds.svg)

**The prototype is for designers.** harry has no interpreter for Figma's prototype: the `.dcf`
format has a field for it (`View.reactions`, from DesignCompose), but the fetcher fills it only
from the DesignCompose plugin's synced data (`vsw-reactions`), the design's file carries none, and
the harry framework does not read it. So behaviour is written twice: in the app for the device,
in the Figma prototype for Play. When the two differ, the device is
right; the prototype is a picture of the code. Known gap: Play opens a sheet in 300 ms
ease-out, the device in 550 ms spring (`components.json`, `open`).

**Compared with DesignCompose.** DesignCompose's runtime executes the reactions: a tap that
opens an overlay in Play opens it in the app, and the designer owns the navigation. harry keeps
the state in one place, the code (one reducer for taps and data alike, deterministic, testable),
and Figma supplies the pictures and the names.
