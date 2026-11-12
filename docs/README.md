# Docs of the UI library

The guides and examples here are the sources. A script builds the HTML site and the files for
coding agents from them. The built files are not in git.

## What is here

| Folder or file | What it is |
|---|---|
| `content/<component>.md` | The guide of one component |
| `content/getting-started.md` | The first app, step by step |
| `content/index.md`, `rules.md` | The overview and the rules every app follows |
| `examples/<component>.rs` | The example code of one component. The tests compile and run it |
| `llms.txt` | The entry point for coding agents |
| `diagrams/` | Diagrams of how the library works |
| `*.js` | The scripts that build the site, check the guides and copy the examples into them |

## Build and read the site

```bash
node docs/build-docs.js
python3 -m http.server -d docs 8765    # open http://localhost:8765/site/
```

`build-docs.js` writes `site/`, `ai/` and `llms-full.txt`. It also draws a picture of every
variant into `assets/`, from the Library's own `.fig` (`design/build-library.sh`).

## Before a commit

Run `scripts/check.sh`. It fails when a guide names code that does not exist, when a guide and
its example differ, or when a component has no guide or no example.
