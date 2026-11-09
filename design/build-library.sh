#!/bin/bash
# The EveryWare Library as a .fig of its own: the component sets (components-patch.js, from
# ../components.json) on library-base.fig, a document with nothing on it but the icon sources.
# Every app's design starts from this file; an app's design/build.sh runs this first. Usage,
# from anywhere:
#
#   build-library.sh            # -> design/build/library.fig (a build output, not in git)
#   build-library.sh --dcf      # and data/library.dcf from it: the Library as the device reads it,
#                               # for the tests of the harry binding (fig-to-dcf.sh, the tree's importer)
#   build-library.sh --base     # library-base.fig again, from the designer's export
#                               # (smarthome-app's design/source/Smart HOme.fig, or the path given)
#
# --base runs empty-patch.js on the export and drops its images and thumbnail (the drawing they
# belonged to is gone). The tools are everyware's design/figma (EVERYWARE_FIGMA, found in the
# tree by default). Needs node, unzip and zip.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
TOP=$(cd "$HERE/../../../.." && pwd)
FIGMA=${EVERYWARE_FIGMA:-$TOP/vendor/sdv-everyware/everyware/design/figma}
if [ "${1:-}" = --base ]; then
    IN=${2:-$TOP/packages/services/display_safety/utils/smarthome-app/design/source/Smart HOme.fig}
    node "$FIGMA/roundtrip.js" "$IN" "$HERE/library-base.fig.new" "$HERE/empty-patch.js"
    zip -q -d "$HERE/library-base.fig.new" 'images/*' >/dev/null || true
    # the export's thumbnail shows the drawing: a blank one instead (Figma makes its own)
    t=$(mktemp -d)
    python3 -c "import zlib,struct,sys
w,h=400,225
row=b'\\x00'+bytes([0xf3,0xf6,0xf9])*w
c=lambda k,d: struct.pack('>I',len(d))+k+d+struct.pack('>I',zlib.crc32(k+d))
open(sys.argv[1],'wb').write(b'\\x89PNG\\r\\n\\x1a\\n'+c(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+c(b'IDAT',zlib.compress(row*h))+c(b'IEND',b''))" "$t/thumbnail.png"
    (cd "$t" && zip -q "$HERE/library-base.fig.new" thumbnail.png)
    rm -rf "$t"
    mv "$HERE/library-base.fig.new" "$HERE/library-base.fig"
    echo "written $HERE/library-base.fig"
    exit 0
fi
mkdir -p "$HERE/build"
# (library.code.json: the code-drawn sets as fingerprints, for fetch-library.js)
LIBRARY_CODE_OUT="$HERE/build/library.code.json" node "$FIGMA/roundtrip.js" "$HERE/library-base.fig" "$HERE/build/library.fig" "$HERE/components-patch.js"
if [ "${1:-}" = --dcf ]; then
    # the root is the Library frame itself (no app screen); two sets checked by name
    DCF_ROOT=Library DCF_NODES= DCF_CHECK='#button #toggle' \
        "$TOP/vendor/sdv-everyware/everyware/scripts/fig-to-dcf.sh" "$HERE/build/library.fig" "$HERE/../data/library.dcf"
fi
# the last line: the file to give the user for a review
echo "review in Figma (File > Import): $HERE/build/library.fig"
