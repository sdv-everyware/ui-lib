#!/bin/bash
# A new EveryWare app (a harry app on the EveryWare UI library) from template-app, the template: the app's
# own git repository next to the others, with the design's patch in it. Run from anywhere:
#
#   scripts/new-app.sh <name>          # name: [a-z][a-z0-9]*, e.g. kitchen
#
# Makes
#   packages/services/display_safety/utils/<name>-app   template-app's committed files (no .git, no
#                                                       .dcf), every template name renamed, git init;
#                                                       design/<name>-patch.js is the screen
#                                                       (#<name>-root, #<name>/*)
#
# The tools the design is built with are <figma>: EVERYWARE_FIGMA, else the tree's
# vendor/sdv-everyware/everyware/design/figma (roundtrip.js, fig2json.js; the dark theme is ui-lib's
# design/dark-theme-patch.js); the
# script warns when that checkout is not on main. The new root gets the next free place on Page 1
# (below the lowest root of the apps' patches, utils/*-app/design/*-patch.js), so two apps' roots
# never lie on each other. The renamed app builds as it is (m <name>_app) and draws the template's screen
# under the new names; the skill skills/new-app/SKILL.md says what to change next.
set -euo pipefail
name=${1:?usage: new-app.sh <name>   (lower case letters and digits, e.g. kitchen)}
[[ $name =~ ^[a-z][a-z0-9]*$ ]] || { echo "name: lower case letters and digits, starting with a letter" >&2; exit 2; }
here=$(cd "$(dirname "$0")" && pwd)
top=$(cd "$here/../../../.." && pwd)
utils=$top/packages/services/display_safety/utils
src=$utils/template-app
dst=$utils/$name-app
figma=${EVERYWARE_FIGMA:-$top/vendor/sdv-everyware/everyware/design/figma}
Name=$(tr a-z A-Z <<< "${name:0:1}")${name:1}

[ -f "$here/../design/dark-theme-patch.js" ] || { echo "no $here/../design/dark-theme-patch.js" >&2; exit 1; }
[ -d "$src/.git" ] || { echo "no template: $src is not a git repository" >&2; exit 1; }
[ -e "$dst" ] && { echo "$dst exists" >&2; exit 1; }
for f in fig2json.js roundtrip.js; do
    [ -f "$figma/$f" ] || { echo "$figma has no $f: set EVERYWARE_FIGMA to the design/figma folder of an everyware main checkout" >&2; exit 1; }
done
# (everyware is on a feature branch whenever a change is open there: that is fine as long as the
# Figma tools are main's)
git -C "$figma" diff --quiet main -- . 2>/dev/null || echo "warning: the Figma tools in $figma are not the ones of everyware's main (a change of them is open on $(git -C "$figma" rev-parse --abbrev-ref HEAD 2>/dev/null || echo '?')): the design is built with those" >&2
[ -n "$(git -C "$src" status --porcelain)" ] && echo "note: template-app has uncommitted changes; only its committed files are copied" >&2

# the template's names -> the new ones; longest first (template-app before template)
rename() {
    sed -e "s/template_app/${name}_app/g; s/template-app/${name}-app/g; s/TemplateAction/${Name}Action/g" \
        -e "s/#template-root/#${name}-root/g; s|#template/|#${name}/|g; s|/data/local/tmp/template|/data/local/tmp/${name}|g" \
        -e "s/template-patch\.js/${name}-patch.js/g; s/template-light\.fig/${name}-light.fig/g; s/template\.fig/${name}.fig/g; s/template\.dcf/${name}.dcf/g" \
        -e "s/\"template: /\"${name}: /g; s/authoring(msg, 'template')/authoring(msg, '${name}')/g"
}

# ---- the app: the committed files of template-app, renamed
mkdir -p "$dst"
git -C "$src" ls-files -z | while IFS= read -r -d '' f; do
    # the built design: the new app builds its own (design/build.sh) under its name
    # and the template's own screenshot
    case "$f" in data/*.dcf|design/*.fig|design/screen-rb3.png) continue ;; esac
    to=${f/design\/template-patch.js/design\/$name-patch.js}
    mkdir -p "$dst/$(dirname "$to")"
    case "$f" in
        *.otf|*.ttf|*.png|*.jpg) cp "$src/$f" "$dst/$to" ;;
        *) rename < "$src/$f" > "$dst/$to" ;;
    esac
    if [ -x "$src/$f" ]; then chmod +x "$dst/$to"; fi    # design/build.sh and other scripts stay runnable
done
# the README and the guide describe the template: the new app's author describes the screen
python3 - "$dst/README.md" "$dst/AGENTS.md" "$name" <<'PY'
import re, sys
readme, agents, name = sys.argv[1:4]
s = open(readme).read()
head, rest = s.split('\n\n', 1)
# from the title to the design's section: the template's own words
rest = rest[rest.index('## The design\n'):]
open(readme, 'w').write(head + '\n\n' + f"The {name} app on harry and the EveryWare UI library (1920 x 1080), made from template-app by\n"
    "ui-lib's scripts/new-app.sh: describe the screen here. It is written against the EveryWare UI\n"
    "library (`vendor/sdv-everyware/ui-lib`) only.\n\n" + rest)
a = open(agents).read()
a, n1 = re.subn(r"The template of a new EveryWare app \(ui-lib's `scripts/new-app\.sh` copies it\):[^.]*\. ", "Describe the screen here. ", a, count=1)
a, n2 = re.subn(r"\n## This repository is the template\n.*", "\n", a, flags=re.S)
# the template's words changed: say so instead of leaving them in the new app
assert n1 == 1 and n2 == 1, "template-app's AGENTS.md no longer has the text new-app.sh replaces"
a = a.rstrip() + '\n'
a = a.replace("| `design/screen-rb3.png` | the screen on the RB3, for the README |\n", "")
open(agents, 'w').write(a)
PY
# the root at the next free place below the other apps' roots at x -2400 (a screen is 1080
# high; 320 between two)
patch=$dst/design/$name-patch.js
[ -f "$patch" ] || { echo "no $patch: template-app has no design/template-patch.js" >&2; exit 1; }
root_y=$(cat "$utils"/*-app/design/*-patch.js | grep -o "frame(page, '#[a-z0-9-]*-root', -2400, [0-9-]*" | awk -F', ' '{ if ($4 + 0 > m) m = $4 + 0 } END { print m + 1400 }')
sed -i "s/frame(page, '#${name}-root', -2400, [0-9-]*,/frame(page, '#${name}-root', -2400, ${root_y},/" "$patch"
grep -q "frame(page, '#${name}-root', -2400, ${root_y}," "$patch" || { echo "could not place #${name}-root in $patch" >&2; exit 1; }
# the template's description of its screen in the file headers: the new app's author rewrites it
left=$(grep -rnIE 'counts up|a title, a number' "$dst" --exclude-dir=.git || true)

(cd "$dst" && git init -q -b main && git config user.name 'Daniel Georg' && git config user.email passenger6@gmail.com &&
    mkdir -p .git/hooks && cp scripts/git-hooks/* .git/hooks/ && chmod +x .git/hooks/* &&   # a new repository: from its own files
    git add -A && git commit -q -m "$name-app: a new EveryWare app from template-app (ui-lib scripts/new-app.sh)

The committed files of template-app with every name of the template renamed: the crate and the
binary ${name}_app, ${Name}Action, the design's #${name}-root and #${name}/*, the data path
/data/local/tmp/${name} and ${name}.dcf. It still draws the template's screen under the new names." &&
    git rev-parse main > .git/everyware-main &&   # where main is, for feature.sh status
    # the template is main's first commit; the app's own work starts on a branch (everyware's
    # AGENTS.md, "How a change lands": a branch, a review, the user's word)
    git switch -q -c "feature/$name-first-screen")

cat <<EOF
made   $dst   (git: $(git -C "$dst" log --oneline -1); now on the branch feature/$name-first-screen)
made   $patch   (root at -2400, $root_y; in the app's first commit)
EOF
[ -n "$left" ] && { echo "the template's screen is still described in these headers (rewrite them with the screen):"; echo "$left" | sed "s|$dst/|   |" | head -20; }
cat <<EOF
next   skills/new-app/SKILL.md, step 2: the layout as ASCII, then $name-patch.js, ui_model.rs, state.rs, ...
EOF
