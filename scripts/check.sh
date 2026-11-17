#!/bin/bash
# The checks before a commit in ui-lib (AGENTS.md, "Before a commit"). Reports, and exits 1
# when something has to be fixed first. Run from anywhere:
#
#   scripts/check.sh            # the staged changes (git add first)
#   scripts/check.sh --all      # the whole working tree
#
# 1. tests: every changed Rust module has tests; the two host tests pass
# 2. coverage: line coverage of the changed core modules (scripts/coverage.sh), the total
# 3. docs: the site builds from the sources and nothing generated is in git; every component has
#    its guide, its line in docs/llms.txt, its example, its row in the README's table and its
#    place in the README's picture, which is built again when the Library changes; a
#    component whose code changed without its guide or example is named (a note)
# 4. the commit itself: the author, nothing private in the diff
# 5. roles: no name of the API says what an app is about (scripts/check-roles.py)
here=$(cd "$(dirname "$0")" && pwd)
lib=$(cd "$here/.." && pwd)
top=$(cd "$lib/../../.." && pwd)
out=${OUT_DIR:-out_rb3}
cd "$lib"
status=0
fail() { echo "  FAIL $*"; status=1; }
note() { echo "  note $*"; }

if [ "$1" = --all ]; then
    changed=$(git ls-files)
    diff() { git diff HEAD "$@"; }
else
    changed=$(git diff --cached --name-only)
    [ -n "$changed" ] || { echo "nothing staged (git add first, or --all)"; exit 1; }
    diff() { git diff --cached "$@"; }
fi
rs=$(echo "$changed" | grep -E '^(core|harry)/src/.*\.rs$' | grep -v '/lib\.rs$')

echo "== 1. tests"
untested=
for f in $rs; do
    n=$(grep -c '#\[test\]' "$f")
    # a module without tests of its own passes only if the other modules' tests cover it (step 2)
    if [ "$n" = 0 ]; then note "$f: no tests of its own"; untested="$untested $f"; else echo "  ok   $f: $n tests"; fi
done
for t in everyware_ui_core_test everyware_ui_test everyware_ui_recipes_test; do
    bin="$top/$out/host/linux-x86/nativetest64/$t/$t"
    if [ ! -x "$bin" ]; then fail "$t not built (m $t)"; continue; fi
    case $t in   # each test against the sources it is built from (Android.bp)
        everyware_ui_core_test) src=core ;;
        everyware_ui_test) src="core harry" ;;
        *) src="core harry docs/examples" ;;
    esac
    if [ -n "$(find $src components.json -newer "$bin" \( -name '*.rs' -o -name components.json \) 2>/dev/null)" ]; then
        fail "$t is older than the sources: m $t, then run again"
        continue
    fi
    r=$(cd "$(dirname "$bin")" && "$bin" 2>&1 | grep '^test result') || true
    case "$r" in *" 0 failed"*) echo "  ok   $t: $r" ;; *) fail "$t: ${r:-did not run}" ;; esac
done

echo "== 2. coverage (everyware_ui_core_test)"
if cov=$("$here/coverage.sh" everyware_ui_core_test 2>&1); then
    for f in $rs; do
        # llvm-cov prints the files without their common directory
        case "$f" in core/src/*) line=$(echo "$cov" | awk -v f="$(basename "$f")" '$1 == f'); [ -n "$line" ] && echo "  $line" ;; esac
    done
    echo "  $(echo "$cov" | awk '$1 == "TOTAL"')"
    low=$(echo "$cov" | awk '$1 ~ /\.rs$/ && $2+0 < 80 { printf "%s %s  ", $1, $2 }')
    [ -n "$low" ] && note "under 80 % lines: $low"
    for f in $untested; do
        pct=$(echo "$cov" | awk -v f="$(basename "$f")" '$1 == f { print $2+0 }')
        [ -n "$pct" ] && [ "${pct%.*}" -ge 80 ] && echo "  ok   $f: ${pct}% lines through the other tests" || fail "$f: no tests and ${pct:-no} coverage"
    done
else
    fail "coverage: $(echo "$cov" | tail -1)"
fi

echo "== 3. docs"
# the sources are whole when the site builds from them; what it builds is not committed
if ! node docs/build-docs.js > /dev/null 2>&1; then fail "docs/build-docs.js failed: run it to see why"; else echo "  ok   the site builds from the sources"; fi
tracked=$(git ls-files docs/site docs/ai docs/llms-full.txt docs/recipes docs/assets)
[ -z "$tracked" ] && echo "  ok   nothing generated in git" || { fail "generated files in git (they are built, not committed): git rm --cached them"; echo "$tracked" | head -5 | sed 's/^/       /'; }
# every component: its guide, its line in llms.txt (the entry), its example
for name in $(python3 -c "import json; print(' '.join(json.load(open('components.json'))['components']))"); do
    [ -f "docs/content/$name.md" ] || fail "docs/content/$name.md: #${name//_/-} has no guide"
    grep -q "docs/content/$name.md" docs/llms.txt || fail "docs/llms.txt: #${name//_/-} is not in the list of components"
    [ -f "docs/examples/$name.rs" ] || fail "docs/examples/$name.rs: #${name//_/-} has no example"
    grep -q "docs/content/$name.md" README.md || fail "README.md: #${name//_/-} has no row in the table of components"
done
# the README's picture shows every component that has a Figma set, and is built again with them
for set in $(python3 -c "import json; print(' '.join(c['set'][1:] for c in json.load(open('components.json'))['components'].values() if c.get('set', '').startswith('#')))"); do
    grep -q "'$set/" docs/readme/gallery.py || fail "docs/readme/gallery.py: #$set is not in the README's picture (add it to GROUPS, build the picture again)"
done
if [ "$1" != --all ] && echo "$changed" | grep -qE '^(components\.json|design/components-patch\.js|design/library\.fig|data/library\.dcf|docs/readme/gallery\.py)$' \
    && ! echo "$changed" | grep -qx 'docs/readme/components.png'; then
    fail "the Library or the gallery changed without docs/readme/components.png: build it again (the commands are at the top of docs/readme/gallery.py)"
fi
# a changed component and its docs: the author decides whether they are still right
if [ "$1" != --all ]; then "$here/changed-components.py" | while read -r line; do note "$line"; done; fi
# the guides name only things that exist (a renamed fn or layer fails here)
if g=$(node docs/check-guides.js 2>&1); then echo "  ok   $g"; else fail "the guides name things that do not exist:"; echo "$g" | sed 's/^/       /'; fi
# the app's slots against its design: the app's own tree, so a note here, the FAIL is the app's commit
if a=$(node docs/check-guides.js --app 2>&1); then echo "  ok   $a"; else note "the app's ui_model.rs binds names its .dcf does not have (fetch, or the app's WIP):"; echo "$a" | sed 's/^/       /' | head -8; fi

echo "== 4. the commit"
email=$(git config user.email)
[ "$email" = passenger6@gmail.com ] && echo "  ok   author $(git config user.name) <$email>" || fail "author is $email: git config user.email passenger6@gmail.com"
# the brackets keep this line from matching itself
private=$(diff -U0 | grep '^+' | grep -nE 'fixcz[z]g|cari[a]d|Co-Authored-B[y]|figma\.com/(design|file)/[A-Za-z0-9]{15,}' || true)
[ -z "$private" ] && echo "  ok   nothing private in the diff" || { fail "private names or keys in the diff:"; echo "$private" | head -5 | sed 's/^/       /'; }

echo "== 5. the library knows no use case"
"$here/check-roles.py" || status=1

[ $status = 0 ] && echo "all good" || echo "fix the FAIL lines first"
exit $status
