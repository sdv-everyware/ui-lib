// The lines an app writes to put a component on a screen, per component, read from its example:
// docs/examples/<component>.rs, real Rust that everyware_ui_recipes_test compiles and runs. The
// docs (the site's right side, the guide's code) show the parts between `// doc:<part>` and
// `// doc:end`, with the example's names replaced by the page's:
//
//   #doc/<component>   the slot's #name          my_<component>   the field's name
//   my_list            the list a scrollbar follows
//
//   use      the imports            state, init   the field of State, its first value
//   model    the ui_model! field    present       the presenter's line(s)
//   act      the arm(s) of the match on the gesture's node name, if it takes gestures
//   change   how the app changes it from its own state, outside a gesture, if shown
//   send     how the app sends the event on, if the example shows it
//   note     `// doc:note <sentence>`: one sentence under the code
const fs = require('fs');
const path = require('path');

const DIR = path.join(__dirname, 'examples');
const KEYS = fs.readdirSync(DIR).filter(f => f.endsWith('.rs') && f !== 'lib.rs').map(f => f.slice(0, -3)).sort();

// the marked parts of one example, each dedented to its least indent
function parts(key) {
  const text = fs.readFileSync(path.join(DIR, `${key}.rs`), 'utf8').split('\n');
  const out = {};
  const notes = [];
  let open = null;
  for (const line of text) {
    const m = line.match(/^\s*\/\/ doc:(\w+)(?: (.*))?$/);
    if (m && m[1] === 'note') { notes.push(m[2] || ''); continue; }
    if (m && m[1] === 'end') { open = null; continue; }
    if (m) { open = m[1]; out[open] = out[open] || []; continue; }
    if (open) out[open].push(line);
  }
  for (const k of Object.keys(out)) {
    const ind = Math.min(...out[k].filter(l => l.trim()).map(l => l.match(/^ */)[0].length));
    out[k] = out[k].map(l => l.slice(Number.isFinite(ind) ? ind : 0));
  }
  out.note = notes.length ? notes.join(' ') : null;
  return out;
}

// a recipe with its slot name and field name filled in; null for a component without an example
function recipe(key, slot, field, list) {
  if (!KEYS.includes(key)) return null;
  const p = parts(key);
  const sub = s => s.split(`#doc/${key}`).join(slot).replace(new RegExp(`\\bmy_${key}(?=_\\d|\\b)`, 'g'), field).replace(/\bmy_list\b/g, list || 'list');
  const get = k => (p[k] || []).map(sub);
  return {
    use: get('use'), state: get('state'), init: get('init'), model: get('model'), present: get('present'),
    act: p.act ? get('act') : null, change: get('change'), send: get('send'), note: p.note ? sub(p.note) : null,
  };
}

// the components that have an example (the keys of the old table)
const R = Object.fromEntries(KEYS.map(k => [k, true]));

module.exports = { R, recipe, KEYS };
