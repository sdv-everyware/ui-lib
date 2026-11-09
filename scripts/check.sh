#!/bin/bash
# The checks before a commit in ui-lib (AGENTS.md, "Before a commit"). Reports, and exits 1
# when something has to be fixed first. Run from anywhere:
#
#   scripts/check.sh            # the staged changes (git add first)
#   scripts/check.sh --all      # the whole working tree
#
# 1. tests: every changed Rust module has tests; the two host tests pass
# 2. coverage: line coverage of the changed core modules (scripts/coverage.sh), the total
# 3. docs: the site, docs/ai, llms.txt and llms-full.txt are what docs/build-docs.js produces from the
#    sources now (nothing stale is committed); a changed component has its written guide
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
    if [ -n "$(find core harry components.json -newer "$bin" -name '*.rs' -o -newer "$bin" -name components.json 2>/dev/null)" ]; then
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
# stale = the generator changes a file: what is on disk (and goes into the commit) is not what the sources produce
docs_hash() { find docs/site docs/ai docs/llms.txt docs/llms-full.txt -type f -print0 | sort -z | xargs -0 sha1sum | sha1sum; }
before=$(docs_hash)
if ! node docs/build-docs.js > /dev/null 2>&1; then fail "docs/build-docs.js failed"; else
    if [ "$before" != "$(docs_hash)" ]; then fail "docs were stale, regenerated now: review and git add docs"; git status --porcelain docs | sed 's/^/       /'; else echo "  ok   docs/site, llms.txt, llms-full.txt match the sources"; fi
fi
# the guides name only things that exist (a renamed fn or layer fails here)
if g=$(node docs/check-guides.js 2>&1); then echo "  ok   $g"; else fail "the guides name things that do not exist:"; echo "$g" | sed 's/^/       /'; fi
# the app's slots against its design: the app's own tree, so a note here, the FAIL is the app's commit
if a=$(node docs/check-guides.js --app 2>&1); then echo "  ok   $a"; else note "the app's ui_model.rs binds names its .dcf does not have (fetch, or the app's WIP):"; echo "$a" | sed 's/^/       /' | head -8; fi
for f in $rs; do
    name=$(basename "$f" .rs)
    python3 -c "import json, sys; sys.exit(sys.argv[1] not in json.load(open('components.json'))['components'])" "$name" || continue
    [ -f "docs/content/$name.md" ] && echo "  ok   docs/content/$name.md" || note "docs/content/$name.md: no written guide for #${name//_/-} yet"
done

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
