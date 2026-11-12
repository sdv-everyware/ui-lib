#!/usr/bin/env python3
"""The components a staged change touches, and whether their docs moved with them.

A component changed when its state machine changed (the Rust file its guide names in the front
matter, `rust:`; one file can serve two components, button.rs for button and icon_button), its entry in
components.json, or its binding (the `impl Slotted for <Type>` block in harry/src/slot.rs whose
SET is `#<component>`). For each such component without a change of docs/content/<component>.md
or docs/examples/<component>.rs, one line: scripts/check.sh shows them as notes (the code may
change without the behaviour, so the author decides).

    scripts/changed-components.py          # the staged change (git add first)
"""
import json
import re
import subprocess
import sys


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, check=False).stdout


staged = git("diff", "--cached", "--name-only").split()
spec = json.loads(git("show", ":components.json") or "{}").get("components", {})
keys = set(spec)
changed = {}  # component -> why

# the state machine: the Rust file each guide names (`rust:` in its front matter)
owners = {}  # file -> components
for k in keys:
    guide = git("show", f":docs/content/{k}.md")
    front = guide.split("\n---", 1)[0] if guide.startswith("---") else ""
    m = re.search(r"^rust:\s*(.+)$", front, re.M)
    for f in re.findall(r"[\w./-]+\.rs", m.group(1)) if m else [f"core/src/{k}.rs"]:
        owners.setdefault(f, []).append(k)
for f in staged:
    for k in owners.get(f, []):
        changed.setdefault(k, f)

# its entry in the spec
if "components.json" in staged:
    before = json.loads(git("show", "HEAD:components.json") or "{}").get("components", {})
    for k in keys:
        if before.get(k) != spec.get(k):
            changed.setdefault(k, "components.json")

# its binding: the impl Slotted block a changed line is in
if "harry/src/slot.rs" in staged:
    text = git("show", ":harry/src/slot.rs").split("\n")
    # the impl blocks: start line -> component, from the SET inside each
    blocks = []
    for i, line in enumerate(text):
        if re.match(r"impl(<[^>]*>)? Slotted for ", line):
            for j in range(i, min(i + 8, len(text))):
                s = re.search(r'const SET: &\'static str = "#([\w-]+)"', text[j])
                if s:
                    blocks.append((i, s.group(1).replace("-", "_")))
                    break
    for hunk in re.finditer(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", git("diff", "--cached", "-U0", "harry/src/slot.rs"), re.M):
        start, count = int(hunk.group(1)), int(hunk.group(2) or 1)
        for line in range(start, start + max(count, 1)):
            owner = [c for i, c in blocks if i < line]
            if owner and owner[-1] in keys:
                changed.setdefault(owner[-1], "harry/src/slot.rs")

for k in sorted(changed):
    if f"docs/content/{k}.md" not in staged and f"docs/examples/{k}.rs" not in staged:
        print(f"{k}: {changed[k]} changed and its guide and example did not: are docs/content/{k}.md and docs/examples/{k}.rs still right?")
sys.exit(0)
