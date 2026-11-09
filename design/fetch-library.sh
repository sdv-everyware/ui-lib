#!/bin/bash
# The Library as the designers have it in Figma, into git: design/source/library.json (the
# component sets drawn in Figma, fingerprints of the ones the code draws). It reports what is new
# (a set a designer added), what changed, and what has two owners. Then design/build-library.sh
# builds the Library with the sets from Figma in it. Usage, from anywhere:
#
#   FIGMA_TOKEN=... fetch-library.sh [DOC_ID]     # export the token, never write it down
#   fetch-library.sh --local <file.fig>           # the same from a .fig, without Figma (a test,
#                                                 # or a local copy saved from Figma)
#   fetch-library.sh --accept '#button'           # a code-drawn set a designer changed: keep the code
#
# DOC_ID = the file key from the Figma URL figma.com/design/<DOC_ID>/<name>; without it the key
# in everyware's scripts/figma-key (a local file, in .gitignore: no key in git). The file must
# have the frame "Library" on the page "Components": the Library imported from
# design/build/library.fig, or a file that started from it.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
TOP=$(cd "$HERE/../../../.." && pwd)
EW=$TOP/vendor/sdv-everyware/everyware
if [ "${1:-}" = --accept ]; then shift; exec node "$HERE/fetch-library.js" --accept "$@"; fi
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
if [ "${1:-}" = --local ]; then
    FIG=${2:?usage: fetch-library.sh --local <file.fig>}
    node "$EW/design/figma/fig2json.js" "$FIG" "$T/json" >/dev/null
    exec node "$HERE/fetch-library.js" --local "$T/json/file.json"
fi
DOC_ID="${1:-${DOC_ID:-$(cat "$EW/scripts/figma-key" 2>/dev/null || true)}}"
[ -n "$DOC_ID" ] || { echo "usage: FIGMA_TOKEN=... $0 [DOC_ID]   (or the key in $EW/scripts/figma-key)"; exit 1; }
[ -n "${FIGMA_TOKEN:-}" ] || { echo "!! FIGMA_TOKEN is not set (export it, do not paste it into scripts)"; exit 1; }
# the token goes to curl through its config on stdin, not on the command line (ps shows that)
api() { curl -sS -m 300 --fail-with-body --config - -o "$2" "https://api.figma.com/v1/files/$DOC_ID$1" <<< "header = \"X-Figma-Token: $FIGMA_TOKEN\""; }
echo ">> Figma: the pages and their frames"
api '?depth=2' "$T/top.json" || { echo "!! Figma refused the key or the token:"; head -c 300 "$T/top.json"; echo; exit 1; }
ID=$(python3 - "$T/top.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
for page in d["document"]["children"]:
    if page["name"] == "Components":
        for n in page.get("children", []):
            if n["name"] == "Library":
                print(n["id"]); sys.exit(0)
sys.exit('!! no frame "Library" on a page "Components" in this file')
PY
)
echo ">> Figma: the Library frame $ID"
api "/nodes?ids=$ID&geometry=paths&plugin_data=shared" "$T/nodes.json" || { echo "!! the fetch failed:"; head -c 300 "$T/nodes.json"; echo; exit 1; }
node "$HERE/fetch-library.js" --rest "$T/nodes.json"
