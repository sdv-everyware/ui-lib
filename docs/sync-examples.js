// sync-examples.js: copies the code a guide shows from an example again. A guide marks such a
// block with `<!-- example: docs/examples/<component>.rs <part> slot=#x field=y -->` right before
// it; after an example changed, this writes the part (with those names) into every marked block.
// check-guides.js fails while a block differs from its example.
//
//   node docs/sync-examples.js     (from ui-lib; no dependencies)
const fs = require('fs');
const path = require('path');
const { recipe } = require('./recipes.js');

const content = path.join(__dirname, 'content');
let changed = 0;
for (const f of fs.readdirSync(content).filter(f => f.endsWith('.md'))) {
  const file = path.join(content, f);
  const text = fs.readFileSync(file, 'utf8');
  const out = text.replace(/(<!-- example: docs\/examples\/(\w+)\.rs (\w+) slot=(\S+) field=(\w+) -->\n```[a-z]*\n)([\s\S]*?)(\n```)/g,
    (all, head, key, part, slot, field, shown, tail) => {
      const r = recipe(key, slot, field, 'list');
      if (!r || !Array.isArray(r[part]) || !r[part].length) throw new Error(`${f}: docs/examples/${key}.rs has no part ${part}`);
      return head + r[part].join('\n') + tail;
    });
  if (out !== text) { fs.writeFileSync(file, out); changed++; console.log(`synced docs/content/${f}`); }
}
console.log(`${changed} guide(s) synced`);
