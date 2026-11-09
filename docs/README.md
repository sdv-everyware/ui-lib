# Documentation of the UI library

Generated from one set of sources, for people and for programs:

```bash
# the previews: every variant of every set as SVG, drawn from the .fig itself (roundtrip.js of
# everyware/design/figma, which carries the kiwi format)
# (the panel's build has every set, the keyboard dock too: ../../../packages/services/display_safety/utils/smarthome-app/design/build/smarthome.fig)
node ../everyware/design/figma/roundtrip.js ../../../packages/services/display_safety/utils/smarthome-app/design/build/smarthome.fig /tmp/x.fig docs/render-previews.js
node docs/build-docs.js     # -> docs/site/*.html (+ play.js), docs/llms.txt, docs/llms-full.txt
python3 -m http.server -d docs 8765   # to read it: http://localhost:8765/site/
```

Why the previews are drawn here and not exported by Figma: Figma's image API renders a
generated file without its texts (it needs the derived text layout the editor computes), so
`render-previews.js` draws the kiwi nodes itself (frames, rectangles, texts, the icons' path
blobs, instances with their overrides), with the layer names as element ids: the playground
on a component's page switches variants and moves the parts the code sets by those ids.

| Source | Gives |
|---|---|
| `components.json` | the sets, their properties and values, the transitions |
| `core/src/*.rs` | the Rust API: `pub` types and functions with their `///` docs |
| `harry/src/slot.rs` | the parts of each component the code sets (`impl Slotted`) |
| `docs/content/<component>.md` | the written guide: role, Figma, the app, traps (front matter: `role`, `rust`, `types`) |
| `docs/content/index.md`, `rules.md` | the overview and the rules every app follows (the head of `llms.txt`) |
| `docs/assets/<set>/<variant>.svg` | the previews (render-previews.js), inlined into the pages |

`docs/site/` is committed, so the pages can be read without building; `scripts/check.sh`
(run before every commit, AGENTS.md) fails when the committed site or llms files are not what
the sources produce, so regenerate and stage them with the change. `llms.txt` is the entry
for a program (what the library is, the rules, the list), `llms-full.txt` holds every component
with its exact variant strings, parts, API, transitions and guide.
