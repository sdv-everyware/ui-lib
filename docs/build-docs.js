#!/usr/bin/env node
// build-docs.js: the UI library's documentation, for people (docs/site/*.html) and for AI
// (docs/llms.txt, docs/llms-full.txt), from one set of sources, so that nothing is written
// twice and nothing drifts:
//
//   components.json          the sets, their properties and values, the transitions (also read
//                            by the Rust library and the Figma generator)
//   core/src/*.rs            the public API: pub enum / struct / fn with their /// docs
//   harry/src/slot.rs        the parts of each component the code sets (impl Slotted)
//   docs/content/<name>.md   what a person writes: the role, how to use it in Figma and in the
//                            app, the traps; a front matter names the Rust types to document
//   docs/assets/<set>/<variant>.png   previews (export-previews.sh); a placeholder without
//
// Usage: node docs/build-docs.js        (from ui-lib; no dependencies)
'use strict';
const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const { R: RECIPES, recipe } = require('./recipes.js');
const SPEC = JSON.parse(fs.readFileSync(path.join(ROOT, 'components.json'), 'utf8'));
const CONTENT = path.join(__dirname, 'content');
const SITE = path.join(__dirname, 'site');
const ASSETS = path.join(__dirname, 'assets');

// ---------------------------------------------------------------- sources

// The content file of a component: a front matter (key: value lines between --- lines) and
// markdown sections.
function readContent(name) {
  const file = path.join(CONTENT, `${name}.md`);
  if (!fs.existsSync(file)) return null;
  const text = fs.readFileSync(file, 'utf8');
  const m = text.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
  const meta = {};
  if (m) for (const line of m[1].split('\n')) { const i = line.indexOf(':'); if (i > 0) meta[line.slice(0, i).trim()] = line.slice(i + 1).trim(); }
  return { meta, body: m ? m[2] : text };
}

// The public API of the named types in a Rust file: the type's doc and declaration, and the
// pub fns of its impl blocks (not the trait impls), each with its /// doc.
function rustApi(file, types) {
  const src = fs.readFileSync(path.join(ROOT, file), 'utf8').split('\n');
  // the /// lines above a declaration, skipping #[attributes] between them and the declaration
  const docOf = (i) => { const d = []; let j = i - 1; while (j >= 0 && /^\s*#\[/.test(src[j])) j--; for (; j >= 0 && /^\s*\/\/\//.test(src[j]); j--) d.unshift(src[j].replace(/^\s*\/\/\/ ?/, '')); return d.join(' '); };
  const out = [];
  for (const type of types) {
    const decl = src.findIndex(l => new RegExp(`^pub (enum|struct|trait) ${type}\\b`).test(l));
    if (decl < 0) { console.warn(`  ${file}: no pub type ${type}`); continue; }
    const item = { type, kind: src[decl].match(/^pub (\w+)/)[1], doc: docOf(decl), variants: [], fns: [] };
    item.fields = [];
    if (item.kind === 'struct') {
      for (let i = decl + 1; i < src.length && !/^}/.test(src[i]); i++) {
        const f = src[i].match(/^\s{4}pub (\w+): (.+),\s*$/);
        if (f) item.fields.push({ name: f[1], type: f[2], doc: docOf(i) });
      }
    }
    if (item.kind === 'enum') {
      for (let i = decl + 1; i < src.length && !/^}/.test(src[i]); i++) {
        const v = src[i].match(/^\s{4}(\w+)[,(]/);
        if (v && !/^\s*#\[/.test(src[i])) item.variants.push({ name: v[1], doc: docOf(i) });
      }
    }
    // impl blocks of the type (inherent), with their pub fns
    for (let i = 0; i < src.length; i++) {
      if (!new RegExp(`^impl(<[^>]*>)? ${type}\\b(<[^>]*>)? \\{`).test(src[i])) continue;
      let depth = 0;
      for (let j = i; j < src.length; j++) {
        depth += (src[j].match(/{/g) || []).length - (src[j].match(/}/g) || []).length;
        const f = src[j].match(/^\s{4}pub fn (\w+)(.*)$/);
        if (f) {
          let sig = src[j].trim().replace(/\s*\{$/, '');
          for (let k = j + 1; !/\)( -> .*)?\s*\{?$/.test(sig) && k < j + 6; k++) sig += ' ' + src[k].trim();
          item.fns.push({ name: f[1], sig: sig.replace(/\s*\{$/, '').replace(/\s+/g, ' '), doc: docOf(j) });
        }
        if (depth === 0 && j > i) break;
      }
    }
    out.push(item);
  }
  return out;
}

// The parts of each set the code sets (harry/src/slot.rs: impl Slotted for X { SET; fn parts }).
function slotParts() {
  const src = fs.readFileSync(path.join(ROOT, 'harry/src/slot.rs'), 'utf8');
  const parts = {};
  for (const m of src.matchAll(/impl Slotted for (\w+) \{([\s\S]*?)\n\}/g)) {
    const set = (m[2].match(/const SET: &'static str = "([^"]+)"/) || [])[1];
    if (!set) continue;
    const names = new Set();
    for (const s of m[2].matchAll(/"(#[a-z0-9-]+\/[^"]+)"/g)) names.add(s[1]);
    for (const s of m[2].matchAll(/format!\("(#[a-z0-9-]+\/[^"]*)\{[^}]*\}([^"]*)"/g)) names.add(s[1] + '<…>' + s[2]);
    parts[set] = { type: m[1], names: [...names], optional: [...m[2].matchAll(/\.optional\("([^"]+)"\)/g)].map(x => x[1]) };
  }
  return parts;
}

const TREE = path.resolve(ROOT, '..', '..', '..');       // the Android tree
const NODES = fs.existsSync(path.join(ASSETS, 'nodes.json')) ? JSON.parse(fs.readFileSync(path.join(ASSETS, 'nodes.json'), 'utf8')) : {};

// file:line of a pattern in a file under the tree, or null
function lineOf(rel, pattern) {
  const file = path.join(TREE, rel);
  if (!fs.existsSync(file)) return null;
  const i = fs.readFileSync(file, 'utf8').split('\n').findIndex(l => pattern.test(l));
  return i >= 0 ? { rel, line: i + 1 } : null;
}

// How a component takes part in the simulation and the recipe: a list row, a sheet-like overlay,
// display only, or a tappable slot.
const kindOf = (key, content) => (content && content.meta.kind) || (/list_item/.test(key) ? 'list' : /^(sheet|overlay|form|keyboard_dock)$/.test(key) ? 'sheet' : /^(icon|symbol|bar_chart|keyboard|scrollbar)$/.test(key) ? 'display' : 'slot');

// Where a component lives: the Figma set (node id), the generator that draws it, its line in
// components.json, the Rust module (the type slot.rs binds), the binding.
function locate(key, c, parts, content) {
  const where = { figma: NODES[c.set] || null, spec: lineOf('vendor/sdv-everyware/ui-lib/components.json', new RegExp(`^\\s*"${key}":`)), rust: null, slot: null, type: null, file: null };
  const sp = parts[c.set];
  const bound = sp ? sp.type : null;
  if (bound) where.slot = lineOf('vendor/sdv-everyware/ui-lib/harry/src/slot.rs', new RegExp(`^impl Slotted for ${bound}\\b`));
  // the state machine in core: the bound type, or the core type it wraps (ListRow -> ListItem),
  // or the first type the guide names
  const candidates = [...(content && content.meta.types ? content.meta.types.split(',').map(x => x.trim()) : []), bound, bound && bound.replace(/Row$/, 'Item')].filter(Boolean);
  for (const type of candidates) {
    for (const f of fs.readdirSync(path.join(ROOT, 'core/src')).filter(f => f.endsWith('.rs'))) {
      const src = fs.readFileSync(path.join(ROOT, 'core/src', f), 'utf8');
      const i = src.split('\n').findIndex(l => new RegExp(`^pub struct ${type}\\b`).test(l));
      if (i >= 0) { where.type = type; where.rust = { rel: `vendor/sdv-everyware/ui-lib/core/src/${f}`, line: i + 1 }; where.file = `core/src/${f}`; break; }
    }
    if (where.file) break;
  }
  return where;
}

// The event enum of a type (pub enum <Type>Event) and the `new` signature, from its module.
function recipeApi(where) {
  if (!where.file) return {};
  const src = fs.readFileSync(path.join(ROOT, where.file), 'utf8');
  // the module's event enum: named after the type (ToggleEvent), or the one event enum it has (InputEvent for InputField)
  const ev = src.match(new RegExp(`pub enum (${where.type}Event|${where.type.replace(/Item$/, '')}Event)\\s*\\{([\\s\\S]*?)\\n\\}`)) || src.match(/pub enum (\w+Event)\s*\{([\s\S]*?)\n\}/);
  // every variant with its payload type (Toggled(bool), Selected(usize), Clicked)
  const events = ev ? [...ev[2].matchAll(/^\s{4}(\w+)(?:\(([^)]*)\))?/gm)].map(m => ({ name: m[1], payload: m[2] || null })) : [];
  const newSig = (rustApi(where.file, [where.type]).flatMap(t => t.fns).find(f => f.name === 'new') || {}).sig || null;
  const on = (rustApi(where.file, [where.type]).flatMap(t => t.fns).find(f => f.name === 'on') || {}).sig || null;
  return { eventEnum: ev ? ev[1] : null, events, newSig, on };
}

const transitionsOf = (key) => SPEC.transitions.filter(t => t.components.includes(key));
const variantsOf = (props) => Object.entries(props).reduce((acc, [p, vals]) => acc.flatMap(a => vals.map(v => [...a, [p, v]])), [[]]).map(c => c.map(([p, v]) => `${p}=${v}`).join(', '));

// ---------------------------------------------------------------- markdown (the subset the content uses)

const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const inline = (s) => esc(s).replace(/`([^`]+)`/g, '<code>$1</code>').replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>').replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');
function markdown(md) {
  const lines = md.split('\n'); const out = []; let i = 0;
  while (i < lines.length) {
    const l = lines[i];
    if (/^```/.test(l)) { const lang = l.slice(3).trim(); const code = []; i++; while (i < lines.length && !/^```/.test(lines[i])) code.push(lines[i++]); i++; out.push(`<pre><code class="language-${lang || 'plaintext'}">${esc(code.join('\n'))}</code></pre>`); continue; }
    // a diagram, alone on its line: ![caption](diagrams/<name>.svg), inlined so it takes the page's colours (dark mode)
    const fig = l.match(/^!\[([^\]]*)\]\((diagrams\/[a-z0-9-]+\.svg)\)\s*$/);
    if (fig) { const svg = fs.readFileSync(path.join(__dirname, fig[2]), 'utf8').replace(/<\?xml[^>]*>\s*/, ''); out.push(`<figure class="diagram">${svg}<figcaption>${inline(fig[1])}</figcaption></figure>`); i++; continue; }
    if (/^#{1,4} /.test(l)) { const level = l.match(/^#+/)[0].length; const text = l.replace(/^#+ /, ''); out.push(`<h${level + 1} id="${slug(text)}">${inline(text)}</h${level + 1}>`); i++; continue; }
    if (/^\|/.test(l)) { const rows = []; while (i < lines.length && /^\|/.test(lines[i])) rows.push(lines[i++]); const cells = r => r.replace(/^\||\|$/g, '').split('|').map(c => inline(c.trim())); const body = rows.filter(r => !/^\|(\s*:?-+:?\s*\|)+$/.test(r.trim())); out.push('<table><thead><tr>' + cells(body[0]).map(c => `<th>${c}</th>`).join('') + '</tr></thead><tbody>' + body.slice(1).map(r => '<tr>' + cells(r).map(c => `<td>${c}</td>`).join('') + '</tr>').join('') + '</tbody></table>'); continue; }
    if (/^- /.test(l)) { const items = []; while (i < lines.length && /^- /.test(lines[i])) { let item = lines[i++].slice(2); while (i < lines.length && /^  \S/.test(lines[i])) item += ' ' + lines[i++].trim(); items.push(item); } out.push('<ul>' + items.map(x => `<li>${inline(x)}</li>`).join('') + '</ul>'); continue; }
    if (/^\s*$/.test(l)) { i++; continue; }
    const para = []; while (i < lines.length && !/^\s*$/.test(lines[i]) && !/^(```|#|\||- |!\[)/.test(lines[i])) para.push(lines[i++]);
    out.push(`<p>${inline(para.join(' '))}</p>`);
  }
  return out.join('\n');
}
const slug = (s) => s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

// ---------------------------------------------------------------- the pages

const CSS = `
:root{--page:#eaf0f7;--surface:#f4f7fb;--card:#fff;--ink:#24364c;--ink2:#6c7e93;--muted:#a3b1c2;--line:#dbe4f0;--accent:#4268b2;--accent-soft:#e6edf9;--accent-soft2:#d3def2;--code:#f1f5fa;--canvas:linear-gradient(135deg,#dfe7f3,#eef3f9)}
:root[data-theme=dark]{--page:#0f1419;--surface:#151b23;--card:#1a212b;--ink:#e6ebf2;--ink2:#9aa8b8;--muted:#5c6b7d;--line:#263040;--accent:#8aa6e6;--accent-soft:#1f2a3d;--accent-soft2:#283553;--code:#111821;--canvas:linear-gradient(135deg,#1b2430,#141b24)}
*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:var(--page);color:var(--ink);font:15px/1.6 Inter,system-ui,sans-serif;-webkit-font-smoothing:antialiased}
a{color:var(--accent);text-decoration:none}a:hover{text-decoration:underline}
code{font:13px/1.5 "IBM Plex Mono",ui-monospace,monospace;background:var(--code);padding:1px 6px;border-radius:6px;color:var(--ink)}
pre{background:var(--code);padding:16px 18px;border-radius:14px;overflow:auto;border:1px solid var(--line);position:relative;margin:12px 0}pre code{background:none;padding:0;font-size:13px}pre code.hljs{background:transparent;padding:0}
pre .copy{position:absolute;top:10px;right:10px;border:1px solid var(--line);background:var(--card);color:var(--ink2);border-radius:8px;font:12px Inter,system-ui;padding:3px 10px;cursor:pointer}pre .copy:hover{color:var(--ink)}
table{border-collapse:collapse;width:100%;margin:10px 0;font-size:14px}th,td{text-align:left;padding:9px 12px;border-bottom:1px solid var(--line);vertical-align:top}th{color:var(--ink2);font-weight:600}
h1{font-size:34px;line-height:1.15;margin:0 0 8px;letter-spacing:-.01em}h2{font-size:22px;margin:36px 0 12px}h3{font-size:16px;margin:22px 0 8px;color:var(--ink)}
.muted{color:var(--ink2);font-size:13.5px}.pill{display:inline-block;background:var(--accent-soft);color:var(--accent);border-radius:999px;padding:2px 10px;font-size:12.5px;font-weight:600;margin:0 6px 6px 0}
/* frame */
.top{position:sticky;top:0;z-index:5;background:color-mix(in srgb,var(--page) 85%,transparent);backdrop-filter:blur(10px);border-bottom:1px solid var(--line)}
.top .in{max-width:1600px;margin:0 auto;display:flex;align-items:center;gap:18px;padding:12px 24px}
.top .brand{font-weight:700;font-size:16px;color:var(--ink)}.top .brand span{color:var(--ink2);font-weight:400;margin-left:8px;font-size:13.5px}
.top .search{margin-left:auto;display:flex;align-items:center;gap:8px}.top input{width:260px;padding:8px 12px;border:1px solid var(--line);border-radius:999px;background:var(--card);color:var(--ink);font:14px Inter,system-ui}
.top button{border:1px solid var(--line);background:var(--card);color:var(--ink);border-radius:999px;padding:7px 12px;cursor:pointer;font:13px Inter,system-ui}
.wrap{max-width:1600px;margin:0 auto;display:grid;grid-template-columns:210px 1fr;gap:28px;padding:24px}
nav{position:sticky;top:76px;align-self:start;max-height:calc(100vh - 100px);overflow:auto;padding-right:8px}
nav .group{font-size:11.5px;letter-spacing:.08em;text-transform:uppercase;color:var(--ink2);margin:16px 12px 6px}
nav a{display:block;padding:7px 12px;border-radius:10px;color:var(--ink);font-size:14px}nav a.on,nav a:hover{background:var(--card);text-decoration:none;box-shadow:0 1px 0 var(--line)}
nav .keyfield{margin:8px 12px}nav .keyfield input{width:100%;padding:7px 10px;border:1px solid var(--line);border-radius:10px;background:var(--card);color:var(--ink);font:13px Inter,system-ui}nav .keyfield .muted{font-size:12px;margin:4px 0 0 2px}
main{min-width:0}
/* hero */
.hero{display:grid;grid-template-columns:1fr 340px;gap:24px;background:var(--card);border:1px solid var(--line);border-radius:24px;padding:24px;margin:8px 0 24px}
.canvas{background:var(--canvas);border-radius:18px;min-height:300px;display:flex;align-items:center;justify-content:center;padding:28px;overflow:hidden}.canvas svg{max-width:100%;max-height:440px;width:auto;height:auto;filter:drop-shadow(0 8px 24px rgba(36,54,76,.10))}
.aside h1{font-size:30px}.aside .role{color:var(--ink2);font-size:16px;line-height:1.5;margin:4px 0 16px}
.ctl{margin:0 0 12px}.ctl label{display:block;font-size:12.5px;color:var(--ink2);margin-bottom:5px}.controls .group{font-size:11.5px;letter-spacing:.08em;text-transform:uppercase;color:var(--ink2);margin:18px 0 8px}
.seg{display:inline-flex;background:var(--accent-soft);border-radius:999px;padding:3px;flex-wrap:wrap}.seg button{border:0;background:none;color:var(--ink);font:13.5px Inter,system-ui;padding:6px 13px;border-radius:999px;cursor:pointer}.seg button.on{background:var(--accent);color:#fff}
.controls input[type=range]{width:100%;accent-color:var(--accent)}.controls input[type=text]{width:100%;padding:7px 10px;border:1px solid var(--line);border-radius:10px;background:var(--card);color:var(--ink);font:13.5px Inter,system-ui}.controls .sw{display:flex;gap:8px;align-items:center;color:var(--ink);font-size:13.5px}
/* tabs */
.tabs{display:flex;gap:6px;border-bottom:1px solid var(--line);margin:8px 0 18px}.tabs button{border:0;background:none;color:var(--ink2);font:15px/1 Inter,system-ui;font-weight:600;padding:12px 16px;border-bottom:2px solid transparent;cursor:pointer;margin-bottom:-1px}.tabs button.on{color:var(--accent);border-color:var(--accent)}
.tab{display:none}.tab.on{display:block}
.card{background:var(--card);border:1px solid var(--line);border-radius:20px;padding:22px 24px;margin:0 0 18px}
.steps{padding-left:22px;margin:8px 0}.steps li{margin:12px 0}.steps li strong{color:var(--ink)}
details.more{background:var(--card);border:1px solid var(--line);border-radius:20px;padding:0 24px;margin:0 0 14px}details.more summary{cursor:pointer;font-weight:600;padding:16px 0;list-style:none;display:flex;align-items:center;gap:10px}details.more summary::before{content:"";width:8px;height:8px;border-right:2px solid var(--ink2);border-bottom:2px solid var(--ink2);transform:rotate(-45deg);transition:transform .15s}details.more[open] summary::before{transform:rotate(45deg)}details.more .body{padding:0 0 18px}
.usages{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:16px;margin:8px 0 6px}.usage{margin:0;background:var(--surface);border:1px solid var(--line);border-radius:16px;padding:12px}.usage svg{width:100%;height:auto;display:block;border-radius:10px}.usage figcaption{font-size:13px;color:var(--ink2);margin-top:8px;line-height:1.5}
.diagram{width:100%;height:auto;margin:6px 0 10px}
.previews{display:flex;flex-wrap:wrap;gap:12px;margin:12px 0}.preview{background:var(--surface);border:1px solid var(--line);border-radius:14px;padding:12px;text-align:center;min-width:160px;cursor:pointer}.preview:hover{border-color:var(--accent)}.preview svg{max-width:240px;max-height:150px;width:auto;height:auto;display:block;margin:0 auto 6px}.preview .name{font:12px ui-monospace,monospace;color:var(--ink2)}.preview .ph{height:56px;display:flex;align-items:center;justify-content:center;color:var(--ink2);font-size:13px}
.api .fn{margin:10px 0}.api .sig{display:block}.api .doc{color:var(--ink2);font-size:14px;margin:2px 0 0}
.where th{width:130px;white-space:nowrap}
/* index */
.intro{display:grid;grid-template-columns:1.2fr 1fr;gap:24px;align-items:center;background:var(--card);border:1px solid var(--line);border-radius:24px;padding:28px;margin:8px 0 24px}.intro .canvas{min-height:260px}
.cards{display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:16px;margin:16px 0 28px}
.cards .card{padding:0;overflow:hidden;color:var(--ink);display:flex;flex-direction:column}.cards .card:hover{border-color:var(--accent);text-decoration:none;box-shadow:0 8px 24px rgba(36,54,76,.08)}
.cards .thumb{background:var(--canvas);height:150px;display:flex;align-items:center;justify-content:center;padding:18px}.cards .thumb svg{max-width:100%;max-height:110px;width:auto;height:auto}
.cards .txt{padding:14px 18px 18px}.cards .set{font-family:ui-monospace,monospace;font-size:13px;color:var(--accent);font-weight:600}.cards p{color:var(--ink2);font-size:13.5px;margin:6px 0 0;line-height:1.5}

.split{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:20px;align-items:start;margin:8px 0 20px}
.world{background:var(--card);border:1px solid var(--line);border-radius:24px;padding:20px 22px;min-width:0}
.world>h2{margin:0 0 12px;font-size:13px;letter-spacing:.08em;text-transform:uppercase;color:var(--ink2);display:flex;align-items:center;gap:8px}.world>h2 b{color:var(--ink);font-size:15px;letter-spacing:0;text-transform:none}
.world .canvas{min-height:240px;cursor:pointer}.world .canvas svg{max-height:300px;user-select:none}
.hint{font-size:12.5px;color:var(--ink2);text-align:center;margin:6px 0 10px}.concept{background:var(--card);border:1px solid var(--line);border-radius:24px;padding:20px 22px;margin:0 0 20px}.concept h2{margin:0 0 6px}.badge{display:inline-block;font-size:11px;letter-spacing:.1em;text-transform:uppercase;font-weight:600;color:var(--accent);background:var(--accent-soft);border-radius:999px;padding:3px 10px;margin-bottom:10px}.ai-steps{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:10px;margin:14px 0}.ai-step{display:flex;gap:12px;background:var(--surface);border:1px solid var(--line);border-radius:14px;padding:12px 14px}.ai-step .k{flex:none;width:24px;height:24px;border-radius:50%;background:var(--accent);color:#fff;font-weight:600;font-size:12px;display:flex;align-items:center;justify-content:center}.ai-step b{display:block;font-size:14px;margin-bottom:4px}.ai-step code{font-size:12px;display:inline-block;margin:0 6px 4px 0}.ai-step p{margin:4px 0 0;font-size:13px;color:var(--ink2);line-height:1.5}.principles{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;margin-top:12px}.principle{background:var(--surface);border:1px solid var(--line);border-radius:14px;padding:12px 14px 12px}.principle .pd{display:block;width:100%;height:auto;max-height:130px;margin:0 0 10px}.principle h4{margin:0 0 4px;font-size:14.5px;display:flex;align-items:center;gap:8px}.principle .n{flex:none;width:22px;height:22px;border-radius:50%;background:var(--accent-soft);color:var(--accent);font-weight:600;font-size:12px;display:flex;align-items:center;justify-content:center}.principle p{margin:0;font-size:13.5px;color:var(--ink2);line-height:1.5}.pd{color:var(--ink2);font:11px/1 Inter,system-ui,sans-serif}.pd text{fill:var(--ink)}.pd text.sub{fill:var(--ink2);font-size:9.5px}.pd .b{fill:var(--card);stroke:var(--line);stroke-width:1}.pd .b.acc{stroke:var(--accent);stroke-width:1.5}.pd .b.fig{stroke:#a259ff;stroke-width:1.5}.pd .b.bad{stroke:#e5484d;stroke-dasharray:3 3}.pd .ar{stroke:currentColor;stroke-width:1.5;fill:none}.pd .ar.acc{stroke:var(--accent)}.pd .ar.no{stroke:#e5484d;stroke-dasharray:4 3}.pd text.no{fill:#e5484d;font-weight:600}.pd text.ok{fill:#0a9d6c;font-weight:600}@media (max-width:900px){.principles{grid-template-columns:1fr}}.pipe{display:flex;flex-direction:column;margin:16px 0 14px}.lane{display:grid;grid-template-columns:104px minmax(0,1fr);gap:10px;border-radius:18px;padding:12px 14px 12px 0;border-left:4px solid var(--lane)}.lane.figma{--lane:#a259ff;background:rgba(162,89,255,.07)}.lane.app{--lane:var(--accent);background:var(--accent-soft)}.lane.harry{--lane:#0a9d6c;background:rgba(10,157,108,.08)}.lane-label{padding:8px 0 0 14px;font-size:12px;line-height:1.35;color:var(--ink2)}.lane-label b{display:block;color:var(--lane);font-size:12.5px;letter-spacing:.04em;text-transform:uppercase}.lane-rows{display:flex;flex-direction:column}.lane-rows>i,.pipe>i.between{display:block;color:var(--ink2);font-style:normal;font-size:18px;line-height:26px;width:380px;text-align:center}.pipe>i.between{margin-left:118px}.row{display:grid;grid-template-columns:380px minmax(0,1fr);gap:22px;align-items:center;padding:6px 0;cursor:pointer}.node{background:var(--surface);border:1px solid var(--line);border-radius:14px;padding:12px 14px;min-width:0;transition:border-color .2s,box-shadow .2s,background .2s}.row:hover .node{border-color:var(--accent)}.row.lit .node{border-color:var(--accent);box-shadow:0 0 0 3px var(--accent-soft);background:var(--card)}.node .t{font-weight:600;font-size:15px;display:inline}.node .f{font-size:12px;color:var(--ink2);display:inline;margin-left:8px}.node code{display:block;font-size:11.5px;white-space:pre-wrap;word-break:break-word;background:var(--code);border-radius:8px;padding:6px 8px;margin-top:8px}.why{font-size:14px;line-height:1.6;color:var(--ink2);transition:color .2s}.row.lit .why{color:var(--ink)}@media (max-width:1000px){.row{grid-template-columns:1fr}.lane-rows>i,.pipe>i.between{width:auto;text-align:left;padding-left:20px}}
.flow{display:flex;align-items:center;gap:4px;flex-wrap:wrap;font-size:12px;color:var(--ink2);margin:0 0 14px}.flow span{background:var(--accent-soft);color:var(--ink);border-radius:999px;padding:3px 10px;font-weight:500;cursor:pointer;transition:background .15s,color .15s,box-shadow .15s}.flow span:hover{box-shadow:0 0 0 2px var(--accent)}.flow span.lit{background:var(--accent);color:#fff}.flow i{font-style:normal;color:var(--muted)}
.path{counter-reset:step}.path .step{position:relative;padding:10px 12px 10px 44px;border-radius:14px;border:1px solid transparent;transition:background .2s,border-color .2s;margin:0 0 6px}.path .step::before{counter-increment:step;content:counter(step);position:absolute;left:12px;top:12px;width:22px;height:22px;border-radius:50%;background:var(--accent-soft);color:var(--accent);font:600 12px/22px Inter,system-ui;text-align:center}
.path .step.lit{background:var(--accent-soft);border-color:var(--accent-soft2)}.path .step.lit::before{background:var(--accent);color:#fff}
.path .step h4{margin:0 0 4px;font-size:14px}.path .step .muted{font-size:12.5px}.path .step pre{margin:8px 0 0;padding:10px 12px;border-radius:10px}.world pre code{white-space:pre-wrap;word-break:break-word;font-size:12.5px}.world pre{overflow:visible}.path .step pre code{font-size:12.5px}
.event{font:13px ui-monospace,monospace;color:var(--accent);min-height:20px;margin:8px 0 0 44px}
.world h2 a.ai{float:right;font-size:12px;font-weight:500;color:var(--ink2);border:1px solid var(--line);border-radius:999px;padding:2px 10px;text-decoration:none;margin-top:4px}.world h2 a.ai:hover{color:var(--accent);border-color:var(--accent)}.live{margin:8px 0}.live-h{font-size:11.5px;color:var(--ink2);font-family:ui-monospace,monospace;margin:0 0 -6px 4px}.slotname{display:flex;align-items:center;gap:10px;margin:0 0 12px;font-size:13px;color:var(--ink2)}.slotname input{flex:1;padding:7px 10px;border:1px solid var(--line);border-radius:10px;background:var(--surface);color:var(--ink);font:13px ui-monospace,monospace}
@media (max-width:1000px){.split{grid-template-columns:1fr}}
.hidden{display:none!important}
@media (max-width:900px){.wrap{grid-template-columns:1fr}nav{position:static;max-height:none}.hero,.intro{grid-template-columns:1fr}.top input{width:160px}}
`;

const PLAY = `
(() => {
  // theme: the reader's choice, kept in this browser
  const root = document.documentElement, themeBtn = document.getElementById('theme');
  const setTheme = t => { root.dataset.theme = t; try { localStorage.setItem('theme', t); } catch (e) {} const dark = t === 'dark'; const l = document.getElementById('hl-light'), d = document.getElementById('hl-dark'); if (l && d) { l.disabled = dark; d.disabled = !dark; } };
  // light like the product; dark only when the reader chose it
  let theme = 'light'; try { theme = localStorage.getItem('theme') || 'light'; } catch (e) {}
  setTheme(theme);
  if (themeBtn) themeBtn.addEventListener('click', () => setTheme(root.dataset.theme === 'dark' ? 'light' : 'dark'));
  // search: filters the menu and the cards
  const search = document.getElementById('search');
  if (search) search.addEventListener('input', () => { const q = search.value.trim().toLowerCase(); document.querySelectorAll('[data-key]').forEach(el => el.classList.toggle('hidden', !!q && !el.textContent.toLowerCase().includes(q) && !el.dataset.key.includes(q.replace(/-/g, '_')))); });
  // tabs
  document.querySelectorAll('#tabs button').forEach(b => b.addEventListener('click', () => { document.querySelectorAll('#tabs button').forEach(x => x.classList.toggle('on', x === b)); document.querySelectorAll('.tab').forEach(t => t.classList.toggle('on', t.dataset.tab === b.dataset.tab)); try { localStorage.setItem('tab', b.dataset.tab); } catch (e) {} }));
  try { const t = localStorage.getItem('tab'); if (t) document.querySelector('#tabs button[data-tab="' + t + '"]')?.click(); } catch (e) {}
  // the playground: the property controls pick the variant, the part controls move the parts
  const data = document.getElementById('variants'); const hero = document.getElementById('hero');
  if (data && hero) {
    const svgs = JSON.parse(data.textContent); const props = {};
    document.querySelectorAll('.seg').forEach(seg => { props[seg.dataset.prop] = seg.querySelector('button.on').dataset.value; });
    const apply = (ctl) => {
      const el = hero.querySelector('[id="' + ctl.dataset.part + '"]'); if (!el) return;
      if (ctl.dataset.kind === 'opacity') el.style.opacity = ctl.checked ? 1 : 0;
      if (ctl.dataset.kind === 'text' && ctl.value) el.textContent = ctl.value;
      if (ctl.dataset.kind === 'meter') {
        const v = ctl.value / 100, box = el.getBBox(), parent = el.parentNode.getBBox ? el.parentNode.getBBox() : box;
        if (/fill|progress|bar/.test(ctl.dataset.part)) el.style.transform = 'scaleX(' + v + ')';
        else el.style.transform = 'translateX(' + ((parent.width - box.width) * v - (box.x - parent.x)) + 'px)';
      }
    };
    const show = () => {
      const name = Object.entries(props).map(([p, v]) => p + '=' + v).join(', ') || 'default';
      hero.innerHTML = svgs[name] || '<div class="muted">no preview of ' + name + '</div>';
      document.querySelectorAll('[data-part]').forEach(apply);
    };
    document.querySelectorAll('.seg button').forEach(b => b.addEventListener('click', () => { const seg = b.parentNode; seg.querySelectorAll('button').forEach(x => x.classList.remove('on')); b.classList.add('on'); props[seg.dataset.prop] = b.dataset.value; show(); }));
    document.querySelectorAll('[data-part]').forEach(ctl => ctl.addEventListener('input', () => apply(ctl)));
    document.querySelectorAll('.gallery .preview').forEach(p => p.addEventListener('click', () => {
      for (const kv of p.dataset.variant.split(', ')) { const [k, v] = kv.split('='); const seg = document.querySelector('.seg[data-prop="' + k + '"]'); if (!seg) continue; seg.querySelectorAll('button').forEach(x => x.classList.toggle('on', x.dataset.value === v)); props[k] = v; }
      show(); window.scrollTo({ top: 0, behavior: 'smooth' });
    }));
    show();
  }
  // the slot name the reader types goes into every code line
  const slotField = document.getElementById('slotname');
  const applySlot = () => {
    const v = (slotField && slotField.value.trim()) || '#home/x';
    document.querySelectorAll('.slotref').forEach(el => el.textContent = v);
    document.querySelectorAll('.path pre code').forEach(code => { if (!code.dataset.raw) code.dataset.raw = code.textContent; code.textContent = code.dataset.raw.replace(/SLOT/g, v); if (window.hljs) hljs.highlightElement(code); });
  };
  // the start page: the tap runs through the pipeline by itself; a click on a stage stops it there
  const rows = [...document.querySelectorAll('.concept .row')];
  if (rows.length) {
    const show = i => rows.forEach(r => r.classList.toggle('lit', Number(r.dataset.stage) === i));
    let i = 0; show(0);
    let auto = setInterval(() => { i = (i + 1) % rows.length; show(i); }, 3000);
    rows.forEach(r => r.addEventListener('click', () => { clearInterval(auto); i = Number(r.dataset.stage); show(i); }));
  }
  // the simulation: a tap on the component on the left runs the event through the app on the right
  const bh = document.getElementById('behaviour');
  if (bh && hero) {
    const b = JSON.parse(bh.textContent);
    const steps = [...document.querySelectorAll('.path .step')], flow = [...document.querySelectorAll('#flow span')], eventEl = document.getElementById('event');
    const part = id => id && hero.querySelector('[id="' + id + '"]');
    const light = (i) => { steps.forEach(s => s.classList.toggle('lit', Number(s.dataset.stage) === i)); flow.forEach(f => f.classList.toggle('lit', Number(f.dataset.stage) === i)); };
    let timers = []; const later = (ms, fn) => timers.push(setTimeout(fn, ms));
    const clear = () => { timers.forEach(clearTimeout); timers = []; };
    const toggled = {};
    const down = () => { clear(); light(0); const p = part(b.pressed); if (p) { p.style.transition = 'opacity .08s'; p.style.opacity = 1; } if (eventEl) eventEl.textContent = 'Gesture::Down'; later(250, () => light(1)); };
    const up = () => {
      clear(); light(1);
      const p = part(b.pressed); if (p) { p.style.transition = 'opacity .16s'; p.style.opacity = 0; }
      if (eventEl) eventEl.textContent = 'Gesture::Up { inside: true } → ' + (b.event ? 'Some(' + b.event + ')' : 'the component changes');
      // what the component changes in itself: a toggle flips, a selection moves
      for (const id of b.toggle || []) { const el = part(id); if (!el) continue; toggled[id] = !toggled[id]; if (id.endsWith('/on')) el.style.opacity = toggled[id] ? 1 : 0; if (/thumb/.test(id)) { const box = el.getBBox(), parent = el.parentNode.getBBox(); el.style.transition = 'transform .2s'; el.style.transform = toggled[id] ? 'translateX(' + (parent.width - box.width - (box.x - parent.x)) + 'px)' : ''; } }
      if (b.focus) { const el = part(b.focus); if (el) el.style.opacity = 1; }
      later(350, () => run(2, false));
    };
    // the pass from a stage to the end: one stage every 500 ms; scrolled, the current step stays in view
    const run = (from, scroll) => {
      clear(); let t = 0;
      for (let i = from; i < 6; i++) later(t += (i === from ? 0 : 500), () => { light(i); if (scroll) { const st = steps.find(x => Number(x.dataset.stage) === i); st && st.scrollIntoView({ block: 'nearest', behavior: 'smooth' }); } });
      later(t + 1000, () => light(-1));
    };
    // a click on a chip of the strip runs the pass from that stage
    flow.forEach(f => f.addEventListener('click', () => {
      const i = Number(f.dataset.stage);
      if (eventEl) eventEl.textContent = i <= 1 ? 'Gesture::Down … Gesture::Up { inside: true } → ' + (b.event ? 'Some(' + b.event + ')' : 'the component changes') : '';
      run(i, true);
    }));
    if (b.kind !== 'display' && b.kind !== 'sheet') {
      hero.addEventListener('pointerdown', e => { e.preventDefault(); down(); });
      hero.addEventListener('pointerup', up);
      hero.addEventListener('pointerleave', () => { const p = part(b.pressed); if (p) p.style.opacity = 0; });
    }
  }
  // the Figma links: the file key lives in this browser (never in the repo)
  const keyField = document.getElementById('figma-key');
  const figmaKey = () => { try { return localStorage.getItem('figmaKey') || ''; } catch (e) { return ''; } };
  const linkFigma = () => { const k = figmaKey(); document.querySelectorAll('a.figma').forEach(a => { a.href = k ? 'https://www.figma.com/design/' + k + '/?node-id=' + a.dataset.node : '#'; a.title = k ? '' : 'set the Figma file key in the menu'; if (!k) a.addEventListener('click', e => { e.preventDefault(); keyField && keyField.focus(); }); }); };
  if (keyField) { keyField.value = figmaKey(); keyField.addEventListener('input', () => { try { localStorage.setItem('figmaKey', keyField.value.trim()); } catch (e) {} linkFigma(); }); }
  linkFigma();
  if (window.hljs) hljs.highlightAll();
  if (slotField) { slotField.addEventListener('input', applySlot); applySlot(); }
  // copy buttons on the code blocks
  document.querySelectorAll('pre').forEach(pre => { const b = document.createElement('button'); b.className = 'copy'; b.textContent = 'copy'; b.addEventListener('click', () => { navigator.clipboard.writeText(pre.querySelector('code').textContent); b.textContent = 'copied'; setTimeout(() => b.textContent = 'copy', 1200); }); pre.appendChild(b); });
})();
`;

const GROUPS = [
  ['Actions', ['button', 'icon_button', 'segmented', 'tabs']],
  ['Inputs', ['toggle', 'radio', 'slider', 'dropdown', 'input_field', 'keyboard', 'keyboard_dock']],
  ['Lists and cards', ['list_item', 'scrollbar', 'tile']],
  ['Over the screen', ['sheet', 'overlay', 'form']],
  ['Display', ['icon', 'symbol', 'bar_chart']],
];
const labelOf = (key, c) => c.set.startsWith('#') ? c.set : `${key} (code)`;

function page(title, nav, body, depth) {
  const base = depth ? '../' : '';
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>${esc(title)}</title><link rel="preconnect" href="https://fonts.googleapis.com"><link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"><link rel="stylesheet" href="${base}style.css"><link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css" id="hl-light"><link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css" id="hl-dark" disabled></head><body>
<header class="top"><div class="in"><a class="brand" href="${base}index.html">EveryWare UI<span>components for harry apps</span></a><div class="search"><input id="search" placeholder="Find a component…" autocomplete="off"><button id="theme" title="light / dark">◐</button></div></div></header>
<div class="wrap"><nav>${nav}</nav><main>${body}</main></div>
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script><script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/rust.min.js"></script><script src="${base}play.js"></script></body></html>`;
}

function navHtml(current, depth) {
  const base = depth ? '../' : '';
  const link = (href, text, on, key) => `<a href="${href}"${on ? ' class="on"' : ''}${key ? ` data-key="${key}"` : ''}>${esc(text)}</a>`;
  let html = link(`${base}index.html`, 'Start here', current === 'index');
  const placed = new Set();
  for (const [group, keys] of GROUPS) {
    const present = keys.filter(k => SPEC.components[k]);
    if (!present.length) continue;
    html += `<div class="group">${group}</div>`;
    for (const k of present) { placed.add(k); html += link(`${base}components/${k}.html`, labelOf(k, SPEC.components[k]), current === k, k); }
  }
  const rest = Object.keys(SPEC.components).filter(k => !placed.has(k));
  if (rest.length) { html += `<div class="group">More</div>`; for (const k of rest) html += link(`${base}components/${k}.html`, labelOf(k, SPEC.components[k]), current === k, k); }
  html += `<div class="group">For AI</div>` + link(`${base}llms.txt`, 'llms.txt') + link(`${base}llms-full.txt`, 'llms-full.txt');
  html += `<div class="group">Figma</div><div class="keyfield"><input id="figma-key" placeholder="file key from the URL" title="figma.com/design/&lt;key&gt;/…: kept in this browser only"><div class="muted">links open the current file</div></div>`;
  return html;
}

// The variant SVGs of a set (docs/assets/<set>/<variant>.svg, render-previews.js), by variant name.
function variantSvgs(c) {
  const set = c.set.replace(/^#/, '');
  const out = {};
  for (const v of variantsOf(c.props || {})) {
    const file = path.join(ASSETS, set, `${v.replace(/[^a-z0-9=,-]/g, '') || 'default'}.svg`);
    if (fs.existsSync(file)) out[v || 'default'] = fs.readFileSync(file, 'utf8').replace(/<\?xml[^>]*>/, '').trim();
  }
  return out;
}

// The playground: the chosen variant drawn large, a control per property, a control per part
// the code sets (an opacity toggles, a meter slides, a text is typed), the gallery of all variants.
function hero(key, c, parts, title, role) {
  const svgs = variantSvgs(c);
  const names = Object.keys(svgs);
  const props = Object.entries(c.props || {});
  const sp = parts[c.set];
  const partNames = sp ? sp.names.filter(n => !n.includes('<')) : [];
  const kindOf = n => /pressed|\/on$|focus|selected|caret|ring|\/open$|\/list$|hover/.test(n) ? 'opacity' : /thumb|fill|marker|progress|\/bar$/.test(n) ? 'meter' : /label|title|subtitle|text|value|helper|placeholder/.test(n) ? 'text' : '';
  const controls = props.map(([p, vals]) => `<div class="ctl"><label>${p}</label><div class="seg" data-prop="${p}">${vals.map((v, i) => `<button${i === 0 ? ' class="on"' : ''} data-value="${v}">${v}</button>`).join('')}</div></div>`).join('');
  const partControls = partNames.map(n => { const k = kindOf(n); if (!k) return ''; const id = esc(n);
    if (k === 'opacity') return `<div class="ctl"><label><code>${id}</code></label><label class="sw"><input type="checkbox" data-part="${id}" data-kind="opacity"> shown</label></div>`;
    if (k === 'meter') return `<div class="ctl"><label><code>${id}</code></label><input type="range" min="0" max="100" value="60" data-part="${id}" data-kind="meter"></div>`;
    return `<div class="ctl"><label><code>${id}</code></label><input type="text" placeholder="type a text" data-part="${id}" data-kind="text"></div>`; }).join('');
  const canvas = names.length ? svgs[names[0]] : `<div class="muted">no preview yet: run render-previews.js</div>`;
  return `<section class="hero">
  <div class="canvas" id="hero">${canvas}</div>
  <div class="aside"><h1>${esc(title)}</h1><div class="role">${inline(role)}</div><div class="controls">${controls}${partControls ? `<div class="group">What the code sets</div>${partControls}` : ''}</div></div>
</section>
<script type="application/json" id="variants">${JSON.stringify(svgs).replace(/</g, '\\u003c')}</script>`;
}
function gallery(c) {
  const svgs = variantSvgs(c);
  return `<div class="previews gallery">${Object.entries(svgs).map(([v, svg]) => `<div class="preview" data-variant="${esc(v)}">${svg}<div class="name">${esc(v)}</div></div>`).join('')}</div>`;
}

// The path of an event through a harry app, as a strip of stages the page lights up.
const FLOW = ['gesture', 'actions', 'state', 'presenter', 'UI model', 'harry draws'];
// Example arguments for a constructor, from its signature: a bool is false, a number 0, a text its name.
const newArgsExample = (sig) => sig.replace(/^.*\(|\).*$/g, '').split(',').map(a => a.trim()).filter(Boolean).map(a => {
  const [n, t] = a.split(':').map(x => x.trim());
  if (t === 'bool') return 'false';
  if (/f32|f64/.test(t)) return '0.0';
  if (/usize|u32|i32/.test(t)) return '0';
  if (/Vec|\[/.test(t)) return 'vec![]';
  if (/str|String/.test(t)) return `"${n}"`;
  return n;
}).join(', ');
const flowStrip = () => `<div class="flow" id="flow">${FLOW.map((f, i) => `<span data-stage="${i}">${f}</span>${i < FLOW.length - 1 ? '<i>→</i>' : ''}`).join('')}</div>`;

// The mechanism, as a picture: the slot in the design, the preview, harry's clone, the app's field.
function slotDiagram(set, variant, name) {
  const box = (x, y, w, h, title, lines, accent) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="12" fill="${accent ? 'var(--accent-soft)' : 'var(--card)'}" stroke="${accent ? 'var(--accent)' : 'var(--line)'}" stroke-width="1.5"/><text x="${x + 14}" y="${y + 26}" font-family="Inter, system-ui" font-size="14" font-weight="600" fill="var(--ink)">${esc(title)}</text>${lines.map((l, i) => `<text x="${x + 14}" y="${y + 48 + i * 18}" font-family="ui-monospace, monospace" font-size="12" fill="var(--ink2)">${esc(l)}</text>`).join('')}`;
  const arrow = (x1, y, x2, label) => `<line x1="${x1}" y1="${y}" x2="${x2}" y2="${y}" stroke="var(--ink2)" stroke-width="1.5" marker-end="url(#arr)"/><text x="${(x1 + x2) / 2}" y="${y - 8}" text-anchor="middle" font-family="Inter, system-ui" font-size="11.5" fill="var(--ink2)">${esc(label)}</text>`;
  return `<svg class="diagram" viewBox="0 0 980 150" xmlns="http://www.w3.org/2000/svg"><defs><marker id="arr" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto"><path d="M0 0 L8 4 L0 8 z" fill="var(--ink2)"/></marker></defs>
${box(10, 20, 230, 110, 'Figma: the slot', ['frame  #<screen>/' + name, 'child  preview instance', '       ' + set + ' ' + variant.split(',')[0] + '…'], true)}
${arrow(240, 75, 290, 'fetch → .dcf')}
${box(290, 20, 210, 110, 'harry: the clone', ['the variant the app picks', 'as a list item of its own', 'texts · opacities · meters'])}
${arrow(500, 75, 550, 'SlotCustomizer')}
${box(550, 20, 200, 110, 'app: the field', [name.replace(/-/g, '_') + ': Slot', '= slot("#…/' + name + '",', '       &state.' + name.replace(/-/g, '_') + ')'])}
${arrow(750, 75, 800, 'gesture by node')}
${box(800, 20, 170, 110, 'app: the event', [set.replace(/^#/, '') + '.on(gesture)', '→ Some(Event)'])}
</svg>`;
}

function componentPage(key, c, content, parts) {
  const types = content && content.meta.types ? content.meta.types.split(',').map(x => x.trim()).filter(Boolean) : [];
  const where = locate(key, c, parts, content);
  const rustFile = (content && content.meta.rust) || where.file;
  const api = rustFile ? rustApi(rustFile, types.length ? types : (where.type ? [where.type] : [])) : [];
  const r = recipeApi(where);
  const props = Object.entries(c.props || {});
  const defaultVariant = props.map(([p, v]) => `${p}=${v[0]}`).join(', ');
  const title = c.set.startsWith('#') ? c.set : key;
  const name = key.replace(/_/g, '-');
  const field = name.replace(/-/g, '_');
  const kind = kindOf(key, content);
  const sections = content ? Object.fromEntries(content.body.split(/\n(?=## )/).map(sec => [(sec.match(/^## (.*)/) || [, 'intro'])[1].trim(), sec.replace(/^## .*\n/, '')])) : {};
  const sec = (h) => sections[h] ? markdown(sections[h]) : '';
  const fileLink = (w) => w ? `<code>${w.rel}${w.line > 1 ? ':' + w.line : ''}</code>` : '<span class="muted">not found</span>';
  const role = content ? content.meta.role || '' : (c.value || '').split(/[.;]/)[0];
  const svgs = variantSvgs(c); const names = Object.keys(svgs);
  const sp = parts[c.set];
  const partNames = sp ? sp.names.filter(n => !n.includes('<')) : [];
  const T = where.type || 'Component';
  // the pattern the app matches: the last variant is the completed one (Changed after Moving),
  // its payload named by what it is
  const PAYLOAD_NAME = { bool: 'on', f32: 'value', usize: 'index', String: 'text' };
  const done = r.events && r.events.length ? r.events[r.events.length - 1] : null;
  const ev = done ? `${r.eventEnum}::${done.name}${done.payload ? `(${PAYLOAD_NAME[done.payload] || 'value'})` : ''}` : null;
  // the arguments of on() besides the gesture, from its signature
  const onArgs = (r.on || 'gesture: Gesture').replace(/^.*\(|\).*$/g, '').split(',').map(a => a.trim()).filter(a => a && !/^&mut self$/.test(a)).map(a => {
    const [n, t] = a.split(':').map(x => x.trim());
    if (t === 'Gesture') return '*gesture';
    if (t === 'Instant') return 'Instant::now()';
    if (/Keyboard/.test(t)) return '&mut state.osk.keyboard';
    if (t === '&str') return 'node';
    return n;   // index, row: the app knows which one from the node name
  });
  const extraArgs = onArgs.filter(a => a !== '*gesture');
  const SLOT = 'SLOT';   // the page replaces it with the name the reader types

  // what the simulation does with the parts (by their names), the same rule harry's app code follows
  const behaviour = {
    kind, event: ev, type: T,
    pressed: partNames.find(n => /\/pressed$/.test(n)) || null,
    toggle: partNames.filter(n => /\/(thumb|on)$/.test(n) && /toggle/.test(c.set)),
    focus: partNames.find(n => /focus/.test(n)) || null,
    selected: partNames.filter(n => /selected$/.test(n)),
  };

  // ---- left: the designer's world
  const controls = props.map(([p, vals]) => `<div class="ctl"><label>${p}</label><div class="seg" data-prop="${p}">${vals.map((v, i) => `<button${i === 0 ? ' class="on"' : ''} data-value="${v}">${v}</button>`).join('')}</div></div>`).join('');
  const usage = (where.figma && where.figma.usage) || [];
  let left = `<h2><b>${esc(title)}</b> in Figma</h2>
<div class="canvas" id="hero">${names.length ? svgs[names[0]] : `<div class="muted">no preview yet</div>`}</div>
<div class="hint">${kind === 'display' ? 'display only: nothing to tap' : 'tap or drag the component: the right side shows what happens in the app'}</div>
<div class="controls">${controls}</div>
<p class="role">${inline(role)}</p>`;
  if (kind !== 'sheet') {
    left += `<h3>Put it on a screen</h3><ol class="steps">
<li>Assets → the library → drag <code>${esc(c.set)}</code> onto the screen${props.length ? ' and pick any variant (the app sets the real one)' : ''}.</li>
<li>Wrap it in a frame (Ctrl/Cmd + Alt + G) of the same size and name the frame <code class="slotref">${SLOT}</code>: that name is the contract with the app.${kind === 'list' ? ' A list is a clipping frame with as many preview rows as it shows; the scrollbar is a slot of its own beside it.' : ''}</li>
<li>Texts in the preview are placeholders: the app writes into the parts named <code>${esc(c.set)}/…</code>. Nothing hidden.</li>
</ol>`;
    if (usage.length) left += `<h3>On the screens now</h3><div class="usages">` + usage.map(u => `<figure class="usage">${fs.readFileSync(path.join(ASSETS, c.set.replace(/^#/, ''), u.file), 'utf8')}<figcaption>slot <code>${esc(u.slot)}</code> in ${esc(u.in)}<br><a class="figma" data-node="${u.id}" href="#">open in Figma</a></figcaption></figure>`).join('') + `</div>`;
  } else left += `<p>A sheet is an instance of the library's <code>#sheet</code> with the app's content component swapped into its slot: see <a href="sheet.html">#sheet</a>.</p>`;
  left += sec('In Figma');

  const pr = recipe(key, SLOT, field, 'list');
  // ---- right: the developer's world: the path of an event, lit by the simulation
  let right = `<h2><b>${esc(title)}</b> in the app <a class="ai" href="../../ai/${key}.md" title="this component as one Markdown file for a program">for AI: docs/ai/${key}.md</a></h2>${flowStrip()}`;
  if (kind === 'sheet') right += sec('In the app') || `<p>See <a href="sheet.html">#sheet</a>.</p>`;
  else {
    right += `<div class="slotname">slot name <input id="slotname" value="#home/${name}" spellcheck="false"></div><div class="path" id="path">
<div class="step" data-stage="0"><h4>harry reports a gesture</h4><div class="muted">The finger is on the node named <code class="slotref">${SLOT}</code>: harry hit-tests the drawn tree and hands the app <code>Gesture::Down</code>, moves, <code>Gesture::Up</code> with that name.</div></div>
<div class="step" data-stage="1"><h4>actions.rs: ${pr && pr.act ? 'one match arm, the component decides' : 'nothing: display only'}</h4><div class="muted">${pr && pr.act ? 'harry\'s gesture is an action of the app like every other (the listener below sends it). The app\'s <code>apply</code> matches the node name: <b>you add this arm</b>. The component takes the gesture, updates itself (pressed, on or off, the finger\'s position) and answers with an event when the gesture completed something, <code>None</code> otherwise.' : 'This component shows data; it takes no gestures.'}</div>
${pr && pr.act ? `<pre><code class="language-rust">${esc(pr.act.join('\n'))}</code></pre>` : ''}${r.on ? `<div class="muted">the component's side: <code>${esc(r.on)}</code></div>` : ''}<div class="event" id="event"></div>
<details class="fold"><summary>once per app: the listener that turns harry's input into an action (ui_event_listener.rs)</summary>
<pre><code class="language-rust">impl UiEventListener for UiListener {
    fn handle_event(&amp;mut self, event: UserInputEvent) -> HandleEventStatus {
        // everyware_ui::Input groups harry's touch events into Gesture::Down / Move / Up per node
        for (node, gesture) in self.input.feed(&amp;event) {
            let _ = self.actions.send(Actions::CustomAction(HomeAction::Gesture(node, gesture)));
        }
        HandleEventStatus::Consumed
    }
}</code></pre></details></div>
<div class="step" data-stage="2"><h4>state.rs: the component lives in the app's state</h4><div class="muted">One value per place on the screen; it holds what is pressed, chosen, typed. The app owns it, so it survives between frames and the presenter can read it.</div>
<pre><code class="language-rust">${esc([...(pr ? pr.use : []), '', 'pub struct State {', ...(pr ? pr.state : []).map(l => '    ' + l), '}', '', '// in State::new', ...(pr ? pr.init : [])].join('\n'))}</code></pre>${pr && pr.note ? `<div class="muted">${inline(pr.note)}</div>` : ''}</div>
<div class="step" data-stage="3"><h4>presenter.rs: state → UI model</h4><div class="muted">After every action the presenter writes the component into its slot: variant, texts, opacities, the transition of the last change.</div>
<pre><code class="language-rust">${esc((pr ? pr.present : []).join('\n'))}</code></pre></div>
<div class="step" data-stage="4"><h4>ui_model.rs: what to show, by node name</h4><div class="muted">One field per node in <code>everyware_ui::ui_model! { ... }</code>; the <code>SlotCustomizer</code> turns it into harry's customizations.</div>
<pre><code class="language-rust">${esc((pr ? pr.model : []).join('\n'))}</code></pre></div>
<div class="step" data-stage="5"><h4>harry draws</h4><div class="muted">The clone of the chosen variant in the slot, with the parts set${behaviour.pressed ? ` (<code>${esc(behaviour.pressed)}</code> fades in while pressed)` : ''}: the picture on the left is what the device shows.</div></div>
</div>`;
    const live = usage.length ? liveExample(usage[0].slot) : null;
    if (live) {
      const block = (stage, file) => live[stage] ? `<div class="live"><div class="live-h">${file}:${live[stage].line}</div><pre><code class="language-rust">${esc(live[stage].code)}</code></pre></div>` : '';
      right += `<h3>Live: <code>${esc(live.slot)}</code> in ${esc(live.app)}</h3><div class="muted">The lines that drive this slot today, cut from the app's sources when these pages were built.</div>${block('state', 'state.rs')}${block('ui_model', 'ui_model.rs')}${block('presenter', 'presenter.rs')}${block('actions', 'actions.rs')}`;
    }
    right += sec('In the app');
  }

  let body = `<div class="split"><section class="world">${left}</section><section class="world">${right}</section></div>
<script type="application/json" id="variants">${JSON.stringify(svgs).replace(/</g, '\\u003c')}</script>
<script type="application/json" id="behaviour">${JSON.stringify(behaviour).replace(/</g, '\\u003c')}</script>`;
  if (sections.intro) body += `<div class="card">${markdown(sections.intro)}</div>`;

  // ---- details, folded
  const more = (t, inner) => inner ? `<details class="more"><summary>${t}</summary><div class="body">${inner}</div></details>` : '';
  body += more('All variants', `<p class="muted">${props.length ? `${props.map(([p, v]) => `<span class="pill">${p}</span>${v.map(x => `<code>${x}</code>`).join(' ')}`).join('<br>')}<br>The variant string, as harry expects it: <code>${defaultVariant}</code> (properties in this order). Click one to see it above.` : 'One variant.'}</p>${gallery(c)}`);
  const tr = transitionsOf(key);
  body += more('What the code sets, and how it moves', `${c.value ? `<p>${inline(c.value)}</p>` : ''}${sp ? `<table><thead><tr><th>Node</th><th></th></tr></thead><tbody>${sp.names.map(n => `<tr><td><code>${esc(n)}</code></td><td>${sp.optional.includes(n) ? 'not in every variant' : ''}</td></tr>`).join('')}</tbody></table>` : ''}${tr.length ? `<h3>Transitions</h3><table><thead><tr><th>Change</th><th>Duration</th><th>Easing</th></tr></thead><tbody>${tr.map(t => `<tr><td>${t.change}</td><td>${t.ms} ms</td><td>${t.easing}</td></tr>`).join('')}</tbody></table><p class="muted">from <code>components.json</code>; the app passes them to harry as <code>AnimationOverride</code></p>` : ''}${partNames.length ? `<h3>Try the parts</h3><div class="controls parts">${partNames.map(n => { const k = /pressed|\/on$|focus|selected|caret|ring|\/open$|\/list$/.test(n) ? 'opacity' : /thumb|fill|marker|progress|\/bar$/.test(n) ? 'meter' : /label|title|subtitle|text|value|helper|placeholder/.test(n) ? 'text' : ''; const id = esc(n); if (!k) return ''; if (k === 'opacity') return `<div class="ctl"><label><code>${id}</code></label><label class="sw"><input type="checkbox" data-part="${id}" data-kind="opacity"> shown</label></div>`; if (k === 'meter') return `<div class="ctl"><label><code>${id}</code></label><input type="range" min="0" max="100" value="60" data-part="${id}" data-kind="meter"></div>`; return `<div class="ctl"><label><code>${id}</code></label><input type="text" placeholder="type a text" data-part="${id}" data-kind="text"></div>`; }).join('')}</div>` : ''}`);
  body += more('Rust API', api.length ? `<p class="muted"><code>${esc(rustFile)}</code></p><div class="api">${api.map(t => `<h3><code>pub ${t.kind} ${t.type}</code></h3><p class="doc">${inline(t.doc)}</p>${t.variants.length ? `<table><tbody>${t.variants.map(v => `<tr><td><code>${v.name}</code></td><td>${inline(v.doc)}</td></tr>`).join('')}</tbody></table>` : ''}${t.fns.map(f => `<div class="fn"><code class="sig">${esc(f.sig)}</code><p class="doc">${inline(f.doc)}</p></div>`).join('')}`).join('')}</div>` : '');
  body += more('Where it lives, how to change it', `<table class="where"><tbody>
<tr><th>Figma</th><td>${where.figma ? `<a class="figma" data-node="${where.figma.id}" href="#">${esc(c.set)} on the Components page</a> <span class="muted">(set the file key in the menu)</span>` : '<span class="muted">no set in the design (code only)</span>'}</td></tr>
<tr><th>Spec</th><td>${fileLink(where.spec)} <span class="muted">properties, values, transitions</span></td></tr>
<tr><th>Rust</th><td>${fileLink(where.rust)} ${where.type ? `<span class="muted"><code>${where.type}</code>, the state machine</span>` : ''}</td></tr>
<tr><th>harry binding</th><td>${fileLink(where.slot)} <span class="muted">the parts of the clone the code sets</span></td></tr>
</tbody></table><ol class="steps">
<li>A property, a value or a transition: <code>components.json</code>; the order of <code>props</code> is the order of the variant name.</li>
<li>The drawing: the component set <code>${esc(c.set)}</code> in the Figma library, every variant; the layers the code sets keep their names <code>${esc(c.set)}/&lt;part&gt;</code>, nothing hidden.</li>
<li>The behaviour: the Rust module${where.type ? ` (<code>${where.type}</code>; <code>variant()</code> produces the set's variant string)` : ''}; the parts: the harry binding.</li>
<li>Fetch the design into the app and run <code>everyware_ui_test</code>: it fails when a variant or a part the code uses is missing in the design.</li>
</ol>`);
  if (sections.Traps) body += more('Traps', markdown(sections.Traps));
  return page(`${title} · EveryWare UI`, navHtml(key, 1), body, 1);
}

// The one concept behind every page: a tap's six stages through a harry app, drawn as a pipeline.
// [owner, title, file, one line of code, the explanation]. Stages 0 and 5 are Figma's and harry's,
// 1 to 4 the app's four files.
const STAGES = [
  ['figma', 'gesture', 'ui_event_listener.rs', 'HomeAction::Gesture(node, gesture)',
   'The slot frame is named in Figma (<code>#entryway/light</code>). harry hit-tests the drawn tree under the finger and reports Down, Move, Up with that name; the app\'s listener (written once) turns it into an action of the app. Data from the bundles arrives as actions the same way.'],
  ['app', 'actions', 'actions.rs', 'state.light.on(gesture)  →  Some(ToggleEvent::Toggled(on))',
   'One <code>apply()</code> takes every action. For a gesture it matches the node name and hands the gesture to the component living there; the component updates itself (pressed, on or off) and returns an event when something completed. On the event the app acts: sends the command, opens the sheet.'],
  ['app', 'state', 'state.rs', 'pub light: Toggle,',
   'One struct owns everything the screen knows: one component value per place on the screen, and the data. Nothing else remembers anything; a test builds a State, applies actions and reads it.'],
  ['app', 'presenter', 'presenter.rs', 'model.light = slot("#entryway/light", &amp;state.light);',
   'After every action a pure function turns the state into the UI model: each component into its slot, with the variant string the component chose, its texts, the values of its parts and the transition of the last change (components.json).'],
  ['app', 'UI model', 'ui_model.rs', 'light: Slot = "#entryway/light" ["SlotCustomizer"];',
   'One field per #name the app touches. The names are the contract with the design: rename both or neither. The <code>SlotCustomizer</code> turns the field into harry\'s customizations: which variant to clone, which text, which opacity, which meter.'],
  ['harry', 'harry draws', 'the framework, the .dcf', 'clone the variant → apply the parts → animate → render',
   'harry clones the chosen variant from the Library into the slot, applies the parts, animates by the transition and renders with Impeller. The picture is Figma\'s (fetched into the .dcf), the decision is the code\'s. Figma\'s Play prototype is for people looking at the design: harry never reads it.'],
];
// The principles the pipeline follows: [title, one sentence, a small diagram]. The diagrams are
// inline SVG in the theme's colours; shown on the start page, the texts in llms.txt.
const D = {
  svg: (w, h, body) => `<svg class="pd" viewBox="0 0 ${w} ${h}" preserveAspectRatio="xMidYMid meet"><defs><marker id="ah" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" fill="currentColor"/></marker></defs>${body}</svg>`,
  // sub: a second line under the title; two lines with '|' between them
  box: (x, y, w, h, t, cls = '', sub = '') => { const lines = sub ? sub.split('|') : []; const ty = y + h / 2 + (lines.length ? 4 - 6 * lines.length : 4);
    return `<rect class="b ${cls}" x="${x}" y="${y}" width="${w}" height="${h}" rx="7"/><text x="${x + w / 2}" y="${ty}" text-anchor="middle">${t}</text>${lines.map((l, i) => `<text class="sub" x="${x + w / 2}" y="${ty + 13 + 11 * i}" text-anchor="middle">${l}</text>`).join('')}`; },
  ar: (x1, y1, x2, y2, cls = '', label = '') => `<line class="ar ${cls}" x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" marker-end="url(#ah)"/>${label ? `<text class="sub" x="${(x1 + x2) / 2}" y="${Math.min(y1, y2) - 4}" text-anchor="middle">${label}</text>` : ''}`,
  no: (x, y) => `<text class="no" x="${x}" y="${y}" text-anchor="middle">✕</text>`,
};
const PRINCIPLES = [
  ['One truth, one direction',
   'Action → state → UI model → pixels. Nothing on the screen writes back: a tap is an action like a message from a bundle, so a frame is a function of the state, testable on the host.',
   D.svg(340, 100, D.box(4, 30, 70, 34, 'action', '', 'tap, data') + D.ar(74, 47, 92, 47) + D.box(92, 30, 70, 34, 'State', 'acc') + D.ar(162, 47, 180, 47) + D.box(180, 30, 76, 34, 'UI model') + D.ar(256, 47, 274, 47) + D.box(274, 30, 62, 34, 'pixels') +
     `<path class="ar no" d="M305 64 Q305 92 127 92 L127 68" fill="none" marker-end="url(#ah)"/>` + D.no(216, 96))],
  ['Figma decides how it looks, code decides what happens',
   'Geometry, colours, variants and names come from the .fig; which variant shows, what a tap does and the data come from Rust. Figma\'s Play prototype is for people looking at the design: harry never reads it.',
   D.svg(340, 100, D.box(4, 6, 156, 62, 'Figma: how it looks', 'fig', 'geometry · colours|variants · #names') + D.box(180, 6, 156, 62, 'Rust: what happens', 'acc', 'which variant shows|what a tap does · the data') +
     D.box(4, 76, 110, 20, 'Play prototype', 'fig') + D.ar(114, 86, 176, 86, 'no') + D.no(145, 82) + D.box(180, 76, 156, 20, 'harry', ''))],
  ['Names are the contract',
   'A #name on a Figma layer is a field in ui_model.rs. Rename both or neither: everyware_ui_test fails when they drift apart.',
   D.svg(340, 100, D.box(4, 20, 148, 46, '#entryway/light', 'fig', 'a layer in Figma') + `<text x="170" y="46" text-anchor="middle" font-size="22">=</text>` + D.box(188, 20, 148, 46, 'light: Slot', 'acc', '"#entryway/light"|in ui_model.rs') +
     `<text class="sub" x="170" y="86" text-anchor="middle">renamed on one side → everyware_ui_test fails</text>`)],
  ['Components are state machines, owned by the library',
   'Pressed, value, focus, variant and transition live in everyware_ui_core. The app routes a gesture by name and acts on the event, so a button behaves the same in every EveryWare app.',
   D.svg(340, 100, D.box(4, 30, 96, 40, 'the app', '', 'match name, act') + D.ar(100, 40, 150, 40, '', 'gesture') + D.ar(150, 60, 100, 60) + `<text class="sub" x="122" y="76" text-anchor="middle">event</text>` +
     D.box(150, 12, 186, 76, 'library: Toggle', 'acc', 'pressed · value · variant|transition'))],
  ['A component on a screen is a slot',
   'A named frame with a preview instance inside; harry clones the chosen variant into it. One component, many places, each with its own texts and values.',
   D.svg(340, 100, D.box(2, 30, 72, 40, '#button', 'fig', 'the Library') + `<rect class="b" x="104" y="4" width="232" height="92" rx="9"/><text class="sub" x="220" y="16" text-anchor="middle">the screen</text>` +
     D.box(114, 24, 64, 26, 'Connect', 'acc') + D.box(188, 24, 64, 26, 'Cancel', 'acc') + D.box(262, 24, 64, 26, 'Rescan', 'acc') + `<text class="sub" x="220" y="68" text-anchor="middle">#wifi/connect · #wifi/cancel · #wifi/rescan</text><text class="sub" x="220" y="84" text-anchor="middle">three clones, each with its own text</text>` +
     D.ar(74, 42, 112, 38, '', 'clone'))],
  ['Variants for rare changes, parts for smooth ones',
   'A variant change rebuilds the node (~50 ms on the RB3): kind, disabled, selected. Pressed, fades, thumbs and carets are opacities and meters, display-only, every frame.',
   D.svg(340, 100, `<text class="sub" x="84" y="14" text-anchor="middle">variant: a jump, the node rebuilt</text><polyline class="ar" points="14,72 80,72 80,32 150,32" fill="none"/><text class="sub" x="84" y="92" text-anchor="middle">~50 ms</text>` +
     `<text class="sub" x="256" y="14" text-anchor="middle">part: a curve, drawn every frame</text><path class="ar acc" d="M186 72 C 230 72 240 32 322 32" fill="none"/><text class="sub" x="256" y="92" text-anchor="middle">opacity, meter, position</text>`)],
  ['One spec, two readers',
   'components.json names the sets, their variants and transitions. The Figma generator and the Rust library read the same file, so they cannot disagree.',
   D.svg(340, 100, D.box(110, 6, 120, 34, 'components.json', 'acc') + D.ar(150, 40, 70, 62) + D.ar(190, 40, 270, 62) + D.box(4, 62, 130, 34, 'Figma generator', 'fig', 'component sets') + D.box(206, 62, 130, 34, 'Rust library', '', 'variant(), transition()'))],
  ['Roles, not contexts',
   'A component is named by its role (button, list item, tile: Material 3), never by what the app puts in it. A new need extends a property or a part before a new component is made.',
   D.svg(340, 100, `<text class="ok" x="84" y="20" text-anchor="middle">✓ by role</text>` + D.box(14, 32, 140, 24, '#button', 'acc') + D.box(14, 66, 140, 24, '#list-item · #tile', 'acc') +
     `<text class="no" x="256" y="20" text-anchor="middle">✕ by use</text>` + D.box(186, 32, 140, 24, '#fan-selector', 'bad') + D.box(186, 66, 140, 24, '#room-tile · #wifi-row', 'bad'))],
];
// How a program works with the library: the entry files, in the order it reads them.
const aiFirstBlock = () => `<section class="concept aifirst"><h2>For a program</h2><p class="role">Give an agent the task and these files; it does not need this site. Each file is small, generated from the sources, and checked before every commit.</p>
<div class="ai-steps">
<div class="ai-step"><div class="k">1</div><div><b>the task map</b><code>AGENTS.md</code><p>which file to read for which task: put a component on a screen, a sheet, a new component, a screen in Figma, the app</p></div></div>
<div class="ai-step"><div class="k">2</div><div><b>what the library is</b><code>docs/llms.txt</code><p>the principles, the six stages, the rules, the list of components (2 KB)</p></div></div>
<div class="ai-step"><div class="k">3</div><div><b>the recipe</b><code>skills/new-app/SKILL.md</code> <code>skills/use-component/SKILL.md</code> <code>skills/sheet/SKILL.md</code> <code>skills/ui-component/SKILL.md</code><p>the steps of one task with the exact files and commands (the open Agent Skills format)</p></div></div>
<div class="ai-step"><div class="k">4</div><div><b>one component's facts</b><code>docs/ai/&lt;component&gt;.md</code><p>variants, parts, transitions, the Rust API and events, the four lines the app writes, the live lines of the smarthome app, the traps (1-3 KB; all of them: <code>docs/llms-full.txt</code>)</p></div></div>
<div class="ai-step"><div class="k">5</div><div><b>the check</b><code>everyware_ui_test</code> <code>scripts/check.sh</code><p>the code against the design, the tests and their coverage, the generated files against the sources; a failure says what to change</p></div></div>
</div>
<p class="role">The spec (<code>components.json</code>) is read by the Figma generator and the Rust library alike; the docs are generated from the spec, the sources, the binding and the live app. Nothing is written twice, so nothing can disagree.</p></section>`;
const principlesBlock = () => `<section class="concept"><h2>Principles</h2><div class="principles">${PRINCIPLES.map(([t, x, d], i) => `<div class="principle">${d}<h4><span class="n">${i + 1}</span>${t}</h4><p>${x}</p></div>`).join('')}</div></section>`;
const conceptBlock = () => `<section class="concept" id="concept"><h2>One tap, six stages</h2><p class="role">The architecture of every harry app: the strip on the right of each component page is this pipeline. A tap runs through it by itself.</p>
<div class="pipe">${[['figma', 'Design', 'the Figma file', [0]], ['app', 'The app', 'four Rust files', [1, 2, 3, 4]], ['harry', 'harry', 'the framework', [5]]].map(([own, who, what, idx], g) => `<div class="lane ${own}"><div class="lane-label"><b>${who}</b>${what}</div><div class="lane-rows">${idx.map((i, k) => { const [, title, file, code, text] = STAGES[i]; return `<div class="row" data-stage="${i}"><div class="node"><div><span class="t">${title}</span><span class="f">${file}</span></div><code>${code}</code></div><div class="why">${text}</div></div>${k < idx.length - 1 ? '<i>↓</i>' : ''}`; }).join('')}</div></div>${g < 2 ? '<i class="between">↓</i>' : ''}`).join('')}</div>
<p class="role">The Figma file holds the first and the last stage: the names, the variants, the pictures. The app holds the four files between: one match arm, one field, one slot line per component on the screen, whoever writes them, a person or an AI.</p></section>`;

function indexPage() {
  const intro = fs.existsSync(path.join(CONTENT, 'index.md')) ? markdown(readContent('index').body) : '';
  const card = (key) => { const c = SPEC.components[key]; const content = readContent(key); const first = Object.values(variantSvgs(c))[0] || '';
    return `<a class="card" href="components/${key}.html" data-key="${key}"><div class="thumb">${first}</div><div class="txt"><div class="set">${esc(labelOf(key, c))}</div><p>${inline(content ? content.meta.role || '' : (c.value || '').split(/[.;:]/)[0])}</p></div></a>`; };
  const placed = new Set();
  let groups = '';
  for (const [group, keys] of GROUPS) { const present = keys.filter(k => SPEC.components[k]); if (!present.length) continue; present.forEach(k => placed.add(k)); groups += `<h2>${group}</h2><div class="cards">${present.map(card).join('')}</div>`; }
  const rest = Object.keys(SPEC.components).filter(k => !placed.has(k));
  if (rest.length) groups += `<h2>More</h2><div class="cards">${rest.map(card).join('')}</div>`;
  const sample = variantSvgs(SPEC.components.button)['kind=primary, state=default, size=regular'] || '';
  return page('EveryWare UI', navHtml('index', 0), `<section class="intro"><div><h1>The UI of the next embedded device</h1><p class="role">Designed in Figma. Driven by Rust. On Android's core, without its weight. The same button on a home panel, a kitchen appliance, a robot. And an agent builds the next screen alone.</p></div><div class="canvas">${sample}</div></section>${aiFirstBlock()}${principlesBlock()}${conceptBlock()}<details class="fold card"><summary>In depth: what lives where (Figma, the code, the spec), compared with DesignCompose</summary>${intro}</details>${groups}`, 0);
}

// ---------------------------------------------------------------- llms

// The live example: the lines of the smarthome app that drive a slot, found by the slot's name
// in the app's sources (no markers in the app): the UI model field, the presenter line, the
// state field it reads and its constructor, the match arm in actions.rs. The app is the one in
// the tree (EVERYWARE_APP overrides); without it the pages show only the template.
const APP = process.env.EVERYWARE_APP || path.join(TREE, 'packages/services/display_safety/utils/smarthome-app');
const appSrc = (f) => { const file = path.join(APP, 'src', f); return fs.existsSync(file) ? fs.readFileSync(file, 'utf8').split('\n') : null; };
function liveExample(slot) {
  const ui = appSrc('ui_model.rs'), pr = appSrc('presenter.rs'), st = appSrc('state.rs'), ac = appSrc('actions.rs');
  if (!ui || !pr || !st || !ac) return null;
  const q = `"${slot}"`;
  const out = { slot, app: path.basename(APP) };
  const uiI = ui.findIndex(l => l.includes(q) && /:\s*Slot\s*=/.test(l));
  if (uiI >= 0) out.ui_model = { line: uiI + 1, code: ui[uiI].trim() };
  const prI = pr.findIndex(l => l.includes(`(${q}`) && /\b(slot|list|scrollbar)\(/.test(l));
  if (prI >= 0) {
    out.presenter = { line: prI + 1, code: pr[prI].trim() };
    // the state field the presenter reads: &d.light, &wifi.connect, &state.osk.keyboard
    const m = pr[prI].match(/&(?:mut )?(?:[a-z_]+\.)*([a-z_]+)\)/);
    if (m) {
      const field = m[1];
      const stI = st.findIndex(l => new RegExp(`^\\s*pub ${field}:`).test(l));
      if (stI >= 0) {
        const doc = []; for (let j = stI - 1; j >= 0 && /^\s*\/\/\//.test(st[j]); j--) doc.unshift(st[j].trim());
        const init = st.find(l => new RegExp(`^\\s*${field}:\\s*[A-Z]`).test(l));
        out.state = { line: stI + 1, code: [...doc, st[stI].trim(), ...(init ? [init.trim().replace(/,$/, '') + ','] : [])].join('\n') };
      }
    }
  }
  // the match arm: from the line with "slot" => to its closing brace at the same indent
  const acI = ac.findIndex(l => l.includes(q) && /=>/.test(l));
  if (acI >= 0) {
    const indent = ac[acI].match(/^\s*/)[0].length;
    let end = acI;
    if (/\{\s*$/.test(ac[acI])) { for (let j = acI + 1; j < ac.length; j++) { if (ac[j].match(/^\s*/)[0].length === indent && /^\s*}/.test(ac[j])) { end = j; break; } } }
    const block = ac.slice(acI, end + 1).map(l => l.slice(Math.min(indent, l.match(/^\s*/)[0].length)));
    out.actions = { line: acI + 1, code: block.join('\n') };
  }
  return out.ui_model || out.presenter || out.actions ? out : null;
}

// One Markdown file per component for a program: the facts (Figma set and variants, the parts
// the code sets, the Rust API, the transitions, where it sits on the screens), the four lines
// the app writes, the written guide. docs/ai/<component>.md; llms-full.txt is their concatenation.
function componentMd(key, c, parts) {
  const content = readContent(key);
  const where = locate(key, c, parts, content);
  const r = recipeApi(where);
  const sp = parts[c.set];
  const kind = kindOf(key, content);
  const usage = (where.figma && where.figma.usage) || [];
  const slot = usage.length ? usage[0].slot : `#home/${key}`;
  const field = slot.replace(/^#/, '').replace(/[^a-z0-9]+/gi, '_').toLowerCase();
  const T = where.type || 'Component';
  const L = [];
  L.push(`# ${c.set}`, '');
  L.push(content && content.meta.role ? content.meta.role : (c.value || '').split(/[.;]/)[0], '');
  L.push('## Figma', '');
  L.push(`- set: \`${c.set}\`${where.figma ? ` (node ${where.figma.id} in the Library)` : ''}`);
  for (const [p, v] of Object.entries(c.props || {})) L.push(`- property \`${p}\`: ${v.map(x => `\`${x}\``).join(' | ')}`);
  L.push(`- variant string: \`${Object.entries(c.props || {}).map(([p, v]) => `${p}=${v[0]}`).join(', ')}\` (the properties in this order; \`${T}::variant()\` produces it, never typed by hand)`);
  if (c.value) L.push(`- parts the code sets: ${c.value}`);
  if (sp) L.push(`- nodes set by code (\`${sp.type}\`, harry/src/slot.rs): ${sp.names.map(n => `\`${n}\``).join(', ')}`);
  if (usage.length) L.push(`- on the screens now: ${usage.map(u => `\`${u.slot}\` in ${u.in}${u.id ? ` (node ${u.id})` : ''}`).join('; ')}`);
  else if (kind !== 'sheet') L.push('- on the screens now: not used yet');
  L.push(`- to place one: a frame named \`#<screen>/<name>\` with an instance of \`${c.set}\` inside (the slot); texts in it are placeholders, nothing hidden`);
  const tr = transitionsOf(key);
  if (tr.length) L.push('', '## Transitions (components.json, through `transition()`)', '', ...tr.map(t => `- ${t.change}: ${t.ms} ms ${t.easing}`));
  L.push('', '## Rust', '');
  if (where.file) {
    L.push(`- module: \`${where.file}\`, type \`${T}\`${where.slot ? `; binding: \`harry/src/slot.rs\` (\`impl Slotted for ${sp.type}\`)` : ''}`);
    if (r.newSig) L.push(`- \`${r.newSig}\``);
    if (r.on) L.push(`- \`${r.on}\``);
    if (r.eventEnum) L.push(`- events \`${r.eventEnum}\`: ${r.events.map(e => `\`${e.name}${e.payload ? `(${e.payload})` : ''}\``).join(', ')}`);
    const types = content && content.meta.types ? content.meta.types.split(',').map(x => x.trim()).filter(Boolean) : [T];
    for (const t of rustApi(where.file, types)) {
      L.push(`- pub ${t.kind} \`${t.type}\`: ${t.doc}`);
      for (const v of t.variants) L.push(`  - \`${v.name}\`: ${v.doc}`);
      for (const f of t.fields || []) L.push(`  - field \`pub ${f.name}: ${f.type}\`: ${f.doc}`);
      for (const f of t.fns) if (!['new', 'on'].includes(f.name)) L.push(`  - \`${f.sig}\`: ${f.doc}`);
    }
  } else L.push('- no Rust state machine: the app sets the parts directly (texts, opacities, meters)');
  if (kind !== 'sheet') {
    const rc = recipe(key, slot, field, 'list');
    if (rc) {
      L.push('', '## The app: the lines per component on the screen (compiled by everyware_ui_recipes_test)', '', '```rust',
        '// imports', ...rc.use, '',
        '// state.rs: in struct State, and its first value in State::new', ...rc.state, ...rc.init.map(l => '// ' + l), ...(rc.state.length ? [''] : []),
        '// ui_model.rs: in everyware_ui::ui_model! { ... } (the contract with the design)', ...rc.model, '',
        '// presenter.rs: after every action', ...rc.present, '',
        ...(rc.act ? ['// actions.rs: in the match on the gesture\'s node name', ...rc.act] : ['// actions.rs: nothing, display only']), '```');
      if (rc.note) L.push('', rc.note);
    } else L.push('', '## The app', '', `No Rust binding yet: the app cannot put \`${c.set}\` in a slot (no \`impl Slotted\` in harry/src/slot.rs).`);
  }
  const live = usage.length ? liveExample(usage[0].slot) : null;
  if (live) {
    L.push('', `## Live: \`${live.slot}\` in ${live.app} (cut from the sources at build time)`, '', '```rust');
    for (const [stage, file] of [['state', 'state.rs'], ['ui_model', 'ui_model.rs'], ['presenter', 'presenter.rs'], ['actions', 'actions.rs']]) if (live[stage]) L.push(`// ${file}:${live[stage].line}`, live[stage].code, '');
    if (L[L.length - 1] === '') L.pop();
    L.push('```');
  }
  if (content && content.body.trim()) L.push('', content.body.trim());
  L.push('', '## Check', '', '- `everyware_ui_test` (host): the variants and parts the code uses exist in the app\'s .dcf', '- `scripts/check.sh` before the commit: tests, coverage, this file regenerated (`node docs/build-docs.js`)');
  return L.join('\n') + '\n';
}

// Every recipe of docs/recipes.js in one small app, compiled and run by the host test
// everyware_ui_recipes_test: the lines the docs show are the lines that build.
function recipesRs() {
  const keys = Object.keys(RECIPES);
  const rs = keys.map(k => [k, recipe(k, `#doc/${k}`, k, 'list_item')]);
  // the imports merged by module (two recipes may name the same type); design and model come
  // with the header
  const byMod = new Map();
  // names at the crate's top (part_of, ScreenView) join the header's own line
  const top = new Set(['list', 'row_of', 'scrollbar', 'slot', 'Gesture']);
  for (const u of rs.flatMap(([, r]) => r.use)) {
    const t = u.match(/^use everyware_ui::(\w+);$/);
    if (t) { top.add(t[1]); continue; }
    const m = u.match(/^use everyware_ui::(\w+)::(?:\{(.*)\}|(\w+));$/);
    if (!m || m[1] === 'design') continue;
    const set = byMod.get(m[1]) || new Set();
    for (const n of (m[2] || m[3]).split(',').map(x => x.trim()).filter(Boolean)) set.add(n);
    byMod.set(m[1], set);
  }
  const uses = [...byMod].sort().map(([m, names]) => `use everyware_ui::${m}::{${[...names].sort().join(', ')}};`);
  const ind = (l, n) => l.map(x => ' '.repeat(n) + x);
  const gestures = rs.filter(([, r]) => r.act).map(([k]) => k === 'segmented' ? '#segmented/segment-2' : k === 'tabs' ? '#doc/tabs/#tabs/tab-2' : k === 'dropdown' ? '#dropdown/item/1' : k === 'keyboard' ? '#kbd/r1/1' : k === 'radio' ? `#doc/${k}/2` : k === 'list_item' ? `#doc/${k}/0` : `#doc/${k}`);
  return [
    '// Generated by docs/build-docs.js from docs/recipes.js: every component\'s recipe, the lines the',
    '// docs show, in one app. Do not edit; everyware_ui_recipes_test compiles and runs it.',
    '#![allow(dead_code, unused_imports, unused_variables)]',
    '',
    'use everyware_ui::design::{Frame, Sizes};',
    'use everyware_ui::model::prelude::*;',
    `use everyware_ui::{${[...top].sort().join(', ')}};`,
    'use std::time::Instant;',
    ...uses,
    '',
    'pub struct State {',
    ...ind(rs.flatMap(([, r]) => r.state), 4),
    '}',
    '',
    'impl State {',
    '    pub fn new(sizes: &Sizes) -> Self {',
    '        let frame = sizes.frame("#doc/slider").unwrap_or(Frame { x: 0.0, y: 0.0, width: 243.0, height: 56.0 });',
    '        State {',
    ...ind(rs.flatMap(([, r]) => r.init), 12),
    '        }',
    '    }',
    '}',
    '',
    'everyware_ui::ui_model! {',
    ...ind(rs.flatMap(([, r]) => r.model), 4),
    '}',
    '',
    'pub fn present(model: &mut UiModel, state: &State) {',
    ...ind(rs.flatMap(([, r]) => r.present), 4),
    '}',
    '',
    'pub fn on_gesture(state: &mut State, node: &str, gesture: Gesture) {',
    '    match node {',
    ...ind(rs.flatMap(([, r]) => r.act || []), 8),
    '        _ => {}',
    '    }',
    '}',
    '',
    '#[test]',
    'fn every_recipe_builds_a_model_and_takes_a_tap() {',
    '    let mut state = State::new(&Sizes::none());',
    '    let mut model = UiModel::new(HashMap::new());',
    '    present(&mut model, &state);',
    `    for node in [${gestures.map(g => JSON.stringify(g)).join(', ')}] {`,
    '        on_gesture(&mut state, node, Gesture::DOWN);',
    '        on_gesture(&mut state, node, Gesture::UP);',
    '    }',
    '    present(&mut model, &state);',
    '    // a tap on the toggle switched it, a tap on the second segment chose it',
    '    assert!(state.toggle.value);',
    '    assert_eq!(state.segmented.value(), "Med");',
    '}',
    '',
  ].join('\n');
}

function llmsFull(parts) {
  const lines = ['# EveryWare UI: every component, for a program that writes harry apps', '', 'One section per component, the same text as docs/ai/<component>.md: Figma set and variants, the parts the code sets, transitions, the Rust API, the four lines the app writes, the guide. The principles and rules: docs/llms.txt.', ''];
  for (const [key, c] of Object.entries(SPEC.components)) lines.push(componentMd(key, c, parts).replace(/^# /, '## ').replace(/\n## /g, '\n### '), '');
  return lines.join('\n');
}

function llmsIndex() {
  const rules = fs.existsSync(path.join(CONTENT, 'rules.md')) ? fs.readFileSync(path.join(CONTENT, 'rules.md'), 'utf8').trim() : '';
  const list = Object.entries(SPEC.components).map(([key, c]) => { const content = readContent(key); return `- ${c.set}: ${content && content.meta.role ? content.meta.role : (c.value || '').split(/[.;]/)[0]} (docs/ai/${key}.md)`; }).join('\n');
  return `# EveryWare UI

The UI component library of SDV EveryWare: Figma component sets (one per role, Material 3 names), Rust state machines (everyware_ui_core), the binding to the harry UI framework (everyware_ui), used by harry apps on embedded devices (no Android framework).

## Principles
${PRINCIPLES.map(([t, x], i) => `${i + 1}. ${t}: ${x}`).join('\n')}

## One tap, six stages (the architecture of a harry app; every component page follows it)
${STAGES.map(([own, title, file, code, text], i) => `${i + 1}. ${title} (${file}, ${own === 'app' ? 'the app' : own}): ${text.replace(/<[^>]+>/g, '').replace(/&amp;/g, '&')} Code: ${code.replace(/&amp;/g, '&')}`).join('\n')}

## Read next
- docs/ai/<component>.md: one component (1-3 KB): Figma set and variants, parts, transitions, Rust API, the four lines the app writes, guide, check; docs/llms-full.txt is all of them in one file
- docs/site/: the same content rendered for people (pictures, the tap simulation)
- components.json: the machine-readable spec (sets, properties, transitions); the Rust library and the Figma generator read it
- skills/new-app/SKILL.md: a new board as an app of its own from one sentence (scripts/new-app.sh, the design patch, fig-to-dcf.sh, the RB3)
- skills/ui-component/SKILL.md and traps.md: how to add or change a component (spec, Figma generator, Rust, binding, app)
- ../everyware/skills/figma-design/SKILL.md: screens, sheets and the designer's prototype
- skills/harry-app/SKILL.md, reference.md and traps.md: a harry app end to end (state, actions, presenter, UI model, build, run, debug)

${rules}

## Components
${list}
`;
}

// ---------------------------------------------------------------- main

fs.mkdirSync(path.join(SITE, 'components'), { recursive: true });
fs.writeFileSync(path.join(SITE, 'style.css'), CSS.trim() + '\n');
fs.writeFileSync(path.join(SITE, 'play.js'), PLAY.trim() + '\n');
const parts = slotParts();
let written = 0;
for (const [key, c] of Object.entries(SPEC.components)) {
  fs.writeFileSync(path.join(SITE, 'components', `${key}.html`), componentPage(key, c, readContent(key), parts));
  written++;
}
fs.writeFileSync(path.join(SITE, 'index.html'), indexPage());
fs.mkdirSync(path.join(__dirname, 'recipes'), { recursive: true });
fs.writeFileSync(path.join(__dirname, 'recipes', 'recipes.rs'), recipesRs());
const AI = path.join(__dirname, 'ai');
fs.mkdirSync(AI, { recursive: true });
for (const f of fs.readdirSync(AI)) if (f.endsWith('.md') && !SPEC.components[f.replace(/\.md$/, '')]) fs.unlinkSync(path.join(AI, f));
for (const [key, c] of Object.entries(SPEC.components)) fs.writeFileSync(path.join(AI, `${key}.md`), componentMd(key, c, parts));
fs.writeFileSync(path.join(__dirname, 'llms-full.txt'), llmsFull(parts));
fs.writeFileSync(path.join(__dirname, 'llms.txt'), llmsIndex());
// the AI files next to the site too, so one folder can be served
fs.copyFileSync(path.join(__dirname, 'llms-full.txt'), path.join(SITE, 'llms-full.txt'));
fs.copyFileSync(path.join(__dirname, 'llms.txt'), path.join(SITE, 'llms.txt'));
const withContent = fs.readdirSync(CONTENT).filter(f => f.endsWith('.md') && !['index.md', 'rules.md'].includes(f)).length;
console.log(`docs: ${written} component pages (${withContent} with written content), index, ${written} docs/ai/*.md, llms.txt, llms-full.txt -> ${path.relative(ROOT, SITE)}`);
