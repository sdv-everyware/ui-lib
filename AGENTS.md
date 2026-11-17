# ui-lib: UI components for harry apps

The project's guide (the repositories, every skill, the rules for every change) is
`vendor/sdv-everyware/everyware/AGENTS.md`: read it first.

Rust library `everyware_ui` (harry binding) on top of `everyware_ui_core` (state machines,
gestures, transitions, keyboard; no harry, tested on the host). `components.json` is the spec
shared with the Figma generator in `vendor/sdv-everyware/everyware/design/figma/`.

## The library knows no use case

ui-lib is UI components and nothing else. It does not know what an app is about: no weather, no
Wi-Fi, no rooms, no news. An app has the use case; the library gives it generic parts to fill.

| The library has | The library never has |
|---|---|
| a component named by its role: button, toggle, list item, tile, pictogram, bar chart | a component named by its use: `WeatherIcon`, `WifiRow`, `NewsCard`, `RoomTile` |
| variants, parts, values the code sets (a text, 0..100, on or off) | data of a domain and its meaning: a weather code, a signal strength, a temperature |
| a state machine of gestures: pressed, selected, dragged | a mapping from data to a look: `from_wmo(code)`, "rain from 50 %" |
| icons and pictograms as pictures with plain names (`house`, `wifi`, `cloud`) | the texts of a domain ("Partly cloudy"), the name of a data source |

Where the rest goes: the mapping from data to a component's values is the app's (its presenter)
or a data crate's; the library takes the result.

A new need: first a property or a part of an existing component, then a new component named by
its role. When the role is not clear, stop and ask the user for the name.

`scripts/check-roles.py` (part of `scripts/check.sh`) fails a commit that brings a use-case word
into a module's name, a public item or components.json. Its `KNOWN` list (what breaks the
rule) is empty and stays empty.
A new domain: add its words to the script.

What an app shows that is its own (its pictures, its content) is a component set of the app's
design and a type in the app's code: the type implements `Component` with `ComponentKind::App`
and `Slotted` with the set's name, and `slot`, `slot_shown`, `grid` take it like a library
component (tv-app: `design/weather-pictures.js`, `src/weather.rs`).

People and programs read the same files: the guide per component in `docs/content/`, its code in
`docs/examples/`, the facts in the code itself. The HTML site is the same, built for people and
not in git. Pick the file for the task, do the task, run the check; do not read the site.

| Task | Read | Then |
|---|---|---|
| a new EveryWare app: a new screen as an app of its own, from one sentence | `skills/new-app/SKILL.md` | the layout as ASCII first, `scripts/new-app.sh <name>`, the patch, `fig-to-dcf.sh`, `m <name>_app`, the RB3 |
| put a library component on a screen (a button, a toggle, a list ...) | `skills/use-component/SKILL.md`, then `docs/content/<component>.md` and `docs/examples/<component>.rs` | the Figma slot, the four places in the app, `everyware_ui_test` |
| a sheet, dialog or side panel over the screen | `skills/sheet/SKILL.md`, `docs/content/sheet.md` | the content component in Figma, `Sheet` in the app |
| a new component, a new variant, a new transition | `skills/ui-component/SKILL.md`, `skills/ui-component/traps.md` | `components.json`, Figma generator, Rust, binding, docs |
| a screen, a tab, the designer's prototype in Figma | `../everyware/skills/figma-design/SKILL.md` | the kiwi generators, a versioned .fig |
| the app itself (state, actions, presenter, UI model, build, run, debug) | `skills/harry-app/SKILL.md`, then `reference.md` and `traps.md` next to it | smarthome-app's `docs/harry-app-anatomy.md` for every part with file:line links |
| what the library is, its principles, where to read next | `docs/llms.txt`, then `docs/content/index.md` and `rules.md` | |
| one component: its role, Figma, traps; the app's lines | `docs/content/<component>.md`, `docs/examples/<component>.rs`; variants and transitions in `components.json`, the API in the Rust file its front matter names | |
| whether a change is right | `scripts/check.sh` | |

Nothing under `docs/` in git is generated. `node docs/build-docs.js` builds the site (and
`docs/ai/`, `docs/llms-full.txt`) locally from the sources; none of it is committed.

Build and test: `m libeveryware_ui everyware_ui_core_test everyware_ui_test` (RB3: `source
device/rb3/sdv_media_db845c/tools/env.sh`, in bash), then run both
tests from `out_rb3/host/linux-x86/nativetest64/<test>/<test>`. `everyware_ui_test` checks the
Figma contract against the Library's own `data/library.dcf` (`design/build-library.sh --dcf`
writes it and `design/library.fig` again after a change of the Library; commit both with the
change).

## After a design build

**Always give the user the built .fig for a review.** After every design build, in your reply:
the full absolute path of the built file (the build script prints it as its last line, `review in
Figma (File > Import): ...`), what changed in it, and where to look (the frame or the set's name).
Not a bare file name, not a path relative to the repository.

## Before a commit

A change lands through its own branch, a review by another agent and the user's word
(everyware's `AGENTS.md`, "How a change lands"). `scripts/check.sh` (the staged changes; `--all`
for the whole tree) runs the checks below and says what to fix; a commit waits until it prints
`all good`. The hook `scripts/git-hooks/pre-commit`
runs it at every `git commit` and refuses the commit otherwise (everyware's
`scripts/setup-agents.sh` installs the hooks, once per checkout):

1. **Tests.** Every changed Rust module has tests of its own, or the other modules' tests cover
   at least 80 % of its lines; `everyware_ui_core_test` and `everyware_ui_test` are built from the
   current sources and pass.
2. **Coverage.** `scripts/coverage.sh` (line coverage per file of `everyware_ui_core_test`,
   from the test's own rustc command with `-C instrument-coverage`; nothing in out/ changes):
   the changed modules and the total are in the report, a module under 80 % is named. New code
   comes with the tests that cover it; a drop of the total is a reason to add tests, not a
   number to accept.
3. **Documentation.** A changed component has its guide in `docs/content/<component>.md` (role,
   Figma, the app, traps) and its example in `docs/examples/<component>.rs` (state, UI model,
   presenter, the gesture's arm, sending the event on; everyware_ui_recipes_test compiles and runs
   it), and a line in `docs/llms.txt`. The check names a missing one and fails when the site does
   not build. Generated files (`docs/site`, `docs/ai`, `docs/llms-full.txt`) are never committed.
   The skills under `skills/` describe the procedure that changed, if it did.
4. **The commit itself.** Author `Daniel Georg <passenger6@gmail.com>`, no AI trailer; no user
   name, work e-mail or Figma file key in the diff.

`skills/` follows the open Agent Skills format (`SKILL.md` with a name and a description, plain
Markdown otherwise): any coding agent or person reads it from here; nothing here depends on a
particular tool. Commits: author `Daniel Georg <passenger6@gmail.com>`, no AI trailer.
