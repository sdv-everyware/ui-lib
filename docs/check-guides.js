#!/usr/bin/env node
// The written guides and the app cannot name things that do not exist:
//
//   node docs/check-guides.js            # the guides; exit 1 with one line per problem
//   node docs/check-guides.js --app      # the app's slots against its .dcf (another app:
//                                          EVERYWARE_APP=<dir> EVERYWARE_APP_DCF=<file>)
//
// 1. Every `#name` in docs/content/*.md and every `Type::item` where Type is a library type must
//    exist: the type and the item in core/src or harry/src (pub), the name as a node in the
//    app's .dcf (the fetched design) or as a part a component sets (harry/src/slot.rs).
// 2. Every slot name the app binds (`= "#..."` in its ui_model.rs) must be a node of its .dcf:
//    a renamed layer in Figma fails here, not on the device.
// The .dcf is a binary proto; a node name is a string in it, so the check is a byte search.
const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const TREE = path.resolve(ROOT, '../../..');
const APP = process.env.EVERYWARE_APP || path.join(TREE, 'packages/services/display_safety/utils/smarthome-app');
// the guides' examples are smarthome's: they are checked against its design; --app checks an
// app's own names against that app's design
const APP_MODE = process.argv.includes('--app');
const DCF = APP_MODE ? (process.env.EVERYWARE_APP_DCF || process.env.SMARTHOME_DCF || path.join(APP, 'data/smarthome.dcf'))
  : (process.env.SMARTHOME_DCF || path.join(TREE, 'packages/services/display_safety/utils/smarthome-app/data/smarthome.dcf'));

const problems = [];
const rustFiles = ['core/src', 'harry/src'].flatMap(d => fs.readdirSync(path.join(ROOT, d)).filter(f => f.endsWith('.rs')).map(f => path.join(ROOT, d, f)));
const rust = rustFiles.map(f => fs.readFileSync(f, 'utf8')).join('\n');
// pub types of the library and the items declared anywhere (fns, consts, variants): a coarse index
const types = new Set([...rust.matchAll(/^pub (?:struct|enum|trait|type) (\w+)/gm)].map(m => m[1]));
const items = new Set([...rust.matchAll(/^\s*pub (?:const )?fn (\w+)|^\s*pub const (\w+)|^\s{4}(\w+)\s*[,({]/gm)].map(m => m[1] || m[2] || m[3]));
const dcf = fs.existsSync(DCF) ? fs.readFileSync(DCF) : null;
const dcfHas = (name) => dcf && dcf.indexOf(Buffer.from(name, 'utf8')) >= 0;
// names the code sets on parts (library sets), from the binding
const partNames = new Set([...rust.matchAll(/"(#[\w-]+\/[\w\/-]+)"/g)].map(m => m[1]));
// a name in a guide may hold a placeholder: #<screen>/<name>, #x/..., #kbd/<row>/<i>
const placeholder = (n) => /[<>…]|^#x\b|\/x\b|\/N\b|<i>|^#(name|names|node)$/.test(n) || /\/\.\.\./.test(n);

// 1. the guides
const content = path.join(ROOT, 'docs/content');
for (const f of APP_MODE ? [] : fs.readdirSync(content).filter(f => f.endsWith('.md'))) {
  const text = fs.readFileSync(path.join(content, f), 'utf8');
  const code = [...text.matchAll(/```[a-z]*\n([\s\S]*?)```|`([^`\n]+)`/g)].map(m => m[1] || m[2]).join('\n');
  for (const [, type, item] of code.matchAll(/\b([A-Z]\w+)::(\w+)/g)) {
    if (!types.has(type)) continue;   // not a library type (Option::Some, HomeAction::Gesture)
    if (!items.has(item)) problems.push(`docs/content/${f}: ${type}::${item} is not in the library`);
  }
  for (const [name] of code.matchAll(/#[a-z][\w-]*(?:\/[\w<>…\.-]+)*/g)) {
    if (placeholder(name)) continue;
    if (name.includes('/') ? (partNames.has(name) || dcfHas(name)) : (dcfHas(name) || partNames.has(name) || [...partNames].some(p => p.startsWith(name + '/')))) continue;
    if (/^#(kbd|chart|dropdown\/item|segmented\/segment|tabs\/tab|list-item\/leading|tile\/symbol)/.test(name)) continue;   // indexed parts
    problems.push(`docs/content/${f}: ${name} is neither a node of the design (${path.relative(TREE, DCF)}) nor a part the code sets`);
  }
}

// 2. the app's slots (--app; the app has its own working tree, so ui-lib's check reports it as a note)
const uiModel = path.join(APP, 'src/ui_model.rs');
if (!process.argv.includes('--app')) { /* guides only */ } else if (fs.existsSync(uiModel) && dcf) {
  for (const [, name] of fs.readFileSync(uiModel, 'utf8').matchAll(/=\s*"(#[^"]+)"/g)) {
    if (!dcfHas(name)) problems.push(`${path.relative(TREE, uiModel)}: the app binds ${name} and the design has no such node: rename the layer or the field (they are the contract), then fetch`);
  }
} else problems.push(`no app or .dcf to check (${path.relative(TREE, uiModel)}, ${path.relative(TREE, DCF)})`);

if (problems.length) { console.log(problems.join('\n')); process.exit(1); }
console.log(process.argv.includes('--app') ? `app: every slot of ui_model.rs is a node of ${path.basename(DCF)}` : `guides: ${fs.readdirSync(content).filter(f => f.endsWith('.md')).length} files, every library item and #name they name exists`);
