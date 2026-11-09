#!/usr/bin/env node
// fetch-library.js: what the designers have in the Library in Figma, into git as
// design/source/library.json; fetch-library.sh calls it with Figma's answer. It keeps
// - every component set drawn in Figma in full: the sets of components.json with "figma": true,
//   and the sets components.json does not know (new, drawn by a designer);
// - of the sets the code draws only a fingerprint, to notice that a designer changed one;
// - the components' ids (set and variant names), for the instances inside the kept sets.
// and reports what is new, what changed and what has two owners. No file key is written.
//
//   node fetch-library.js --rest <nodes.json>     # GET /v1/files/<key>/nodes?ids=<Library>&geometry=paths&plugin_data=shared
//   node fetch-library.js --local <file.json>     # the same from a .fig: fig2json.js's file.json (no Figma)
//   node fetch-library.js --accept '#button' ...  # a code-drawn set as Figma has it now is fine: keep the code
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const HERE = __dirname;
const OUT = process.env.EVERYWARE_LIBRARY_JSON || path.join(HERE, 'source/library.json');
const SPEC = JSON.parse(fs.readFileSync(process.env.EVERYWARE_COMPONENTS_JSON || path.join(HERE, '../components.json'), 'utf8'));
const CODE_HASHES = path.join(HERE, 'build/library.code.json');
const [, , mode, ...args] = process.argv;
const usage = () => { console.error('usage: fetch-library.js --rest <nodes.json> | --local <file.json> | --accept <#set> ...'); process.exit(2); };
if (!mode) usage();

const prev = fs.existsSync(OUT) ? JSON.parse(fs.readFileSync(OUT, 'utf8')) : { components: {}, code: {}, sets: {} };
const codeNow = fs.existsSync(CODE_HASHES) ? JSON.parse(fs.readFileSync(CODE_HASHES, 'utf8')) : {};
const write = lib => { fs.mkdirSync(path.dirname(OUT), { recursive: true }); fs.writeFileSync(OUT, JSON.stringify(lib, null, 1) + '\n'); };

if (mode === '--accept') {
  if (!args.length) usage();
  for (const set of args) {
    const c = prev.code[set];
    if (!c) { console.error(`${set}: not a code-drawn set seen in Figma (${Object.keys(prev.code).join(' ') || 'none'})`); process.exit(1); }
    c.accepted = c.figma;
    if (codeNow[set]) c.code = codeNow[set];
    console.log(`${set}: accepted as Figma has it now; the code keeps drawing it`);
  }
  write(prev);
  process.exit(0);
}
if ((mode !== '--rest' && mode !== '--local') || !args[0]) usage();

// ---- the Library frame in the answer
const answer = JSON.parse(fs.readFileSync(args[0], 'utf8'));
const find = (n, test) => { if (test(n)) return n; for (const c of n.children || []) { const r = find(c, test); if (r) return r; } return null; };
let library, file;
if (mode === '--rest') {
  const node = Object.values(answer.nodes || {})[0];
  if (!node || !node.document) { console.error(`no node in Figma's answer: ${JSON.stringify(answer).slice(0, 200)}`); process.exit(1); }
  library = node.document;
  file = { name: answer.name, version: answer.version, lastModified: answer.lastModified };
} else {
  const page = (answer.document.children || []).find(p => p.name === 'Components');
  library = page && (page.children || []).find(n => n.name === 'Library');
  file = { name: answer.name, version: answer.version, lastModified: answer.lastModified };
}
if (!library) { console.error('no frame "Library" on the page "Components"'); process.exit(1); }
const sets = (library.children || []).filter(n => n.type === 'COMPONENT_SET');
if (!sets.length) { console.error('the Library frame has no component sets'); process.exit(1); }

// ---- a set's fingerprint: its look and structure, without what changes when nothing was edited
// (ids, positions on the page)
const DROP = new Set(['id', 'componentId', 'absoluteBoundingBox', 'absoluteRenderBounds', 'key', 'componentSetId']);
const fingerprint = (set) => {
  const top = { ...set, relativeTransform: undefined };
  return crypto.createHash('sha1').update(JSON.stringify(top, (k, v) => (DROP.has(k) ? undefined : v))).digest('hex').slice(0, 16);
};

const owner = {};   // set name -> 'figma' | 'code'
for (const c of Object.values(SPEC.components)) {
  owner[c.set] = c.figma ? 'figma' : 'code';
  // (a set a component draws beside its own, the placeholder content of #sheet: the code's)
  for (const h of c.helper_sets || []) owner[h] = 'code';
}

const lib = { comment: 'The Library as the designers have it in Figma, written by design/fetch-library.sh: do not edit. sets: the component sets drawn in Figma, in full; code: fingerprints of the sets the code draws (figma: as fetched, accepted: as agreed, code: the code\'s drawing then).',
  file, components: {}, code: {}, sets: {} };
const report = [];
for (const set of sets) {
  for (const v of (set.children || []).filter(k => k.type === 'COMPONENT')) lib.components[v.id] = { set: set.name, name: v.name };
  const fp = fingerprint(set), variants = (set.children || []).filter(k => k.type === 'COMPONENT').length;
  if (owner[set.name] === 'code') {
    const was = prev.code[set.name];
    lib.code[set.name] = was ? { ...was, figma: fp } : { figma: fp, accepted: fp, ...(codeNow[set.name] ? { code: codeNow[set.name] } : {}) };
    if (!was) report.push(`   ${set.name}: drawn by code, now also in Figma (${variants} variants)`);
    else if (fp !== was.accepted) report.push(`!! ${set.name}: drawn by code, and CHANGED in Figma. Decide: hand it over ("figma": true in components.json, then fetch again), or keep the code (fetch-library.sh --accept '${set.name}')`);
  } else {
    lib.sets[set.name] = set;
    const was = prev.sets[set.name];
    const state = !was ? 'new' : fingerprint(was) !== fp ? 'changed' : 'unchanged';
    if (owner[set.name] === 'figma') { if (state !== 'unchanged') report.push(`   ${set.name}: drawn in Figma, ${state} (${variants} variants)`); }
    else report.push(`++ ${set.name}: ${state === 'unchanged' ? 'still ' : ''}new in Figma, not in components.json (${variants} variants): adopt it (ui-component skill, "A component drawn in Figma")`);
  }
}
for (const [name, o] of Object.entries(owner)) if (o === 'figma' && !lib.sets[name]) report.push(`!! ${name}: "figma": true in components.json and not in the Library in Figma`);
write(lib);
console.log(`library.json: ${sets.length} sets in Figma (${Object.keys(lib.sets).length} kept in full, ${Object.keys(lib.code).length} drawn by code), file "${file.name || '?'}" changed ${file.lastModified || '?'}`);
console.log(report.length ? report.join('\n') : '   nothing new, nothing changed');
