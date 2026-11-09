#!/bin/bash
# Line coverage of a host test of this library, per source file, without touching the build's
# configuration: the rustc command Soong generated for the test is run again with
# `-C instrument-coverage` into a scratch directory, the test runs there, llvm-cov reports.
#
#   scripts/coverage.sh                      # everyware_ui_core_test (the state machines)
#   scripts/coverage.sh everyware_ui_test    # the harry binding
#   OUT_DIR=out_media scripts/coverage.sh    # another out directory (default out_rb3)
#
# The test must have been built once (`m <test>`): its rlibs and the ninja command come from
# that build. Nothing in out/ is written.
set -e

test=${1:-everyware_ui_core_test}
here=$(cd "$(dirname "$0")" && pwd)
top=$(cd "$here/../../../.." && pwd)
out=${OUT_DIR:-out_rb3}
work=${COVERAGE_DIR:-/tmp/everyware-ui-coverage}/$test
clang=$(ls -d "$top"/prebuilts/clang/host/linux-x86/clang-r* | sort -V | tail -1)
ninja="$top/prebuilts/build-tools/linux-x86/bin/ninja"
target="$out/soong/.intermediates/vendor/sdv-everyware/ui-lib/$test/linux_glibc_x86_64/$test"
# one combined ninja per product in out/: the one that knows the test (TARGET_PRODUCT first)
cd "$top"
graph=
for g in $(ls -t "$top/$out"/combined-${TARGET_PRODUCT:-}*.ninja "$top/$out"/combined-*.ninja 2>/dev/null | awk '!seen[$0]++'); do
    if [ -n "$("$ninja" -f "$g" -t commands "$target" 2>/dev/null | tail -1)" ]; then graph=$g; break; fi
done
[ -n "$graph" ] || { echo "no build of $test in $top/$out: build it first (m $test)" >&2; exit 1; }

mkdir -p "$work"
rm -f "$work"/*.profraw
cmd=$("$ninja" -f "$graph" -t commands "$target" 2>/dev/null | tail -1)
# the same compilation, instrumented, into the scratch directory
eval "${cmd/-o $target/-o $work/$test -C instrument-coverage -g}"
LLVM_PROFILE_FILE="$work/%p.profraw" "$work/$test" > "$work/test.log" 2>&1 || { cat "$work/test.log"; exit 1; }
grep "^test result" "$work/test.log"
"$clang/bin/llvm-profdata" merge -sparse "$work"/*.profraw -o "$work/merged.profdata"
"$clang/bin/llvm-cov" report "$work/$test" -instr-profile="$work/merged.profdata" \
    -ignore-filename-regex='(prebuilts|external|out_[a-z0-9]*|packages)/' \
    | awk -v lib="vendor/sdv-everyware/ui-lib/" '
        /^Filename/ { printf "%-32s %8s %8s\n", "file", "lines", "fns"; next }
        /^-+$/ { next }
        NF { sub(lib, "", $1); printf "%-32s %8s %8s\n", $1, $10, $7 }'
