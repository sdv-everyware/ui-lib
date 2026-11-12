#!/bin/bash
# update-apps.sh -- every app on the library, brought to the library as it is now. The apps build
# ui-lib from the tree, so their code follows a change of it at the next build; their design does
# not: each app's .fig and .dcf hold the Library of the day they were built. Per app:
#
#   1. design/build.sh        the app's .fig and .dcf again, with the Library as it is now
#   2. check-guides --app     every slot of its ui_model.rs is a node of the new .dcf
#   3. everyware_ui_test      every variant and part the library's code sets is in that .dcf
#   4. m <app>_app            the app builds against the library as it is now (BUILD=0: skip)
#
# What changes (data/<app>.dcf, and design/<app>.fig where the app keeps it in git) stays uncommitted in the app's own repository:
# its own commit, on a branch of its own. An app with uncommitted changes is skipped, so a
# rebuild is never mixed with someone's work.
#
# Usage, from anywhere:
#   scripts/update-apps.sh                 # every <name>-app with design/build.sh
#   scripts/update-apps.sh hub tv          # these apps
set -u
here=$(cd "$(dirname "$0")" && pwd)
lib=$(cd "$here/.." && pwd)
top=$(cd "$lib/../../.." && pwd)
utils=$top/packages/services/display_safety/utils
out=${OUT_DIR:-out_rb3}
apps=("$@")
[ ${#apps[@]} -gt 0 ] || for d in "$utils"/*-app; do [ -x "$d/design/build.sh" ] && apps+=("$(basename "$d" -app)"); done
logs=$(mktemp -d)

# the library's test, built once: it checks a .dcf given by LIBRARY_DCF
( cd "$top" && bash -c "source device/rb3/sdv_media_db845c/tools/env.sh >/dev/null 2>&1 && m everyware_ui_test" ) > "$logs/test-build.log" 2>&1 \
    || { echo "!! everyware_ui_test does not build: $logs/test-build.log"; exit 1; }
test_bin=$(find "$top/$out/host/linux-x86/testcases/everyware_ui_test" -type f -name everyware_ui_test | head -1)

status=0
for app in "${apps[@]}"; do
    dir=$utils/$app-app
    dcf=$dir/data/$app.dcf
    echo "== $app-app"
    [ -x "$dir/design/build.sh" ] || { echo "  skip  no design/build.sh"; continue; }
    if [ -n "$(git -C "$dir" status --porcelain)" ]; then echo "  skip  uncommitted changes in $app-app: commit or stash them first"; status=1; continue; fi
    if "$dir/design/build.sh" --dcf > "$logs/$app-design.log" 2>&1; then echo "  ok    design rebuilt"; else echo "  FAIL  design/build.sh: $logs/$app-design.log"; status=1; continue; fi
    if r=$(EVERYWARE_APP=$dir EVERYWARE_APP_DCF=$dcf node "$lib/docs/check-guides.js" --app 2>&1); then echo "  ok    $r"; else echo "  FAIL  the app binds names its design does not have:"; echo "$r" | sed 's/^/        /' | head -8; status=1; fi
    if LIBRARY_DCF=$dcf "$test_bin" every_part > "$logs/$app-test.log" 2>&1; then echo "  ok    every variant and part of the library is in $app.dcf"; else echo "  FAIL  the library's code and $app.dcf disagree: $logs/$app-test.log"; status=1; fi
    if [ "${BUILD:-1}" = 1 ]; then
        if ( cd "$top" && bash -c "source device/rb3/sdv_media_db845c/tools/env.sh >/dev/null 2>&1 && m ${app}_app" ) > "$logs/$app-build.log" 2>&1; then echo "  ok    m ${app}_app"; else echo "  FAIL  m ${app}_app: $logs/$app-build.log"; status=1; fi
    fi
    changed=$(git -C "$dir" status --porcelain)
    [ -n "$changed" ] && { echo "  todo  commit in $app-app (a branch of its own):"; echo "$changed" | sed 's/^/        /'; } || echo "  ok    nothing changed: the app was up to date"
done
echo "logs: $logs"
[ $status = 0 ] && echo "all apps follow the library" || echo "fix the FAIL lines"
exit $status
