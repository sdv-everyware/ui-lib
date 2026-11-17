// render-previews.js: the previews of every variant of every component set as SVG, drawn from
// the Figma file's own data (the kiwi message), not by Figma's servers: their render drops the
// texts of a generated file (no derived text layout in it). Run through the kiwi round trip:
//
//   node ../everyware/design/figma/roundtrip.js design/library.fig /tmp/x.fig docs/render-previews.js
//   (from ui-lib, after design/build-library.sh; docs/build-docs.js runs both when the SVGs are missing)
//
// Draws what the generators draw: frames and symbols (fill, corners, shadow, clip), rectangles,
// ellipses, texts (Inter: the docs page loads the font), vectors (the icons: the path commands
// from the file's blobs), instances (the symbol's children with the instance's overrides: name,
// text, opacity, visible, swapped symbol, nested). Every element carries its layer name as id,
// so the playground can move the parts the code sets. Output: docs/assets/<set>/<variant>.svg.
'use strict';
const fs = require('fs');
const path = require('path');

const ASSETS = path.join(__dirname, 'assets');
const SPEC = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'components.json'), 'utf8'));
const SETS = new Set(Object.values(SPEC.components).map(c => c.set).filter(s => s.startsWith('#')));
const key = g => `${g.sessionID}:${g.localID}`;
const safe = v => v.replace(/[^a-z0-9=,-]/g, '');
const esc = s => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');
const num = n => Math.round(n * 100) / 100;

module.exports = function (msg) {
  const nodes = msg.nodeChanges;
  const byKey = new Map(nodes.map(n => [key(n.guid), n]));
  const children = new Map();
  for (const n of nodes) if (n.parentIndex) { const k = key(n.parentIndex.guid); if (!children.has(k)) children.set(k, []); children.get(k).push(n); }
  for (const list of children.values()) list.sort((a, b) => (a.parentIndex.position < b.parentIndex.position ? -1 : 1));
  const kids = n => children.get(key(n.guid)) || [];
  const blobs = msg.blobs || [];

  // ---- the path commands of a geometry blob: 1 byte command, float32 coordinates
  const pathData = (blobIndex) => {
    const b = blobs[blobIndex]; if (!b || !b.bytes) return '';
    const bytes = b.bytes instanceof Uint8Array ? b.bytes : Uint8Array.from(b.bytes);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    let i = 0; const out = [];
    const f = () => { const v = view.getFloat32(i, true); i += 4; return num(v); };
    while (i < bytes.length) {
      const cmd = bytes[i++];
      if (cmd === 0) out.push('Z');
      else if (cmd === 1) out.push(`M${f()} ${f()}`);
      else if (cmd === 2) out.push(`L${f()} ${f()}`);
      else if (cmd === 3) out.push(`Q${f()} ${f()} ${f()} ${f()}`);
      else if (cmd === 4) out.push(`C${f()} ${f()} ${f()} ${f()} ${f()} ${f()}`);
      else break;
    }
    return out.join(' ');
  };

  const color = (c, extra = 1) => `rgba(${Math.round(c.r * 255)},${Math.round(c.g * 255)},${Math.round(c.b * 255)},${num((c.a === undefined ? 1 : c.a) * extra)})`;
  const solid = paints => (paints || []).find(p => p.visible !== false && p.type === 'SOLID');
  const fillAttr = n => { const p = solid(n.fillPaints); return p ? `fill="${color(p.color, p.opacity === undefined ? 1 : p.opacity)}"` : 'fill="none"'; };
  const strokeAttr = n => {
    const p = solid(n.strokePaints); if (!p || !n.strokeWeight) return '';
    const dash = n.dashPattern && n.dashPattern.length ? ` stroke-dasharray="${n.dashPattern.join(' ')}"` : '';
    return ` stroke="${color(p.color, p.opacity === undefined ? 1 : p.opacity)}" stroke-width="${n.strokeWeight}"${dash}`;
  };
  const transform = n => { const t = n.transform || {}; const [a, b, c, d, e, f] = [t.m00 ?? 1, t.m10 ?? 0, t.m01 ?? 0, t.m11 ?? 1, t.m02 ?? 0, t.m12 ?? 0]; return (a === 1 && b === 0 && c === 0 && d === 1) ? (e || f ? ` transform="translate(${num(e)} ${num(f)})"` : '') : ` transform="matrix(${[a, b, c, d, e, f].map(num).join(' ')})"`; };
  let defs = []; let uid = 0;
  const shadow = n => {
    const e = (n.effects || []).find(x => x.visible !== false && x.type === 'DROP_SHADOW'); if (!e) return '';
    const id = `sh${++uid}`;
    defs.push(`<filter id="${id}" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="${num(e.offset.x)}" dy="${num(e.offset.y)}" stdDeviation="${num(e.radius / 2)}" flood-color="${color(e.color)}"/></filter>`);
    return ` filter="url(#${id})"`;
  };
  const weight = style => ({ Thin: 100, Light: 300, Regular: 400, Medium: 500, 'Semi Bold': 600, SemiBold: 600, Bold: 700 })[style] || 400;

  // ---- one node, inside an instance's override scope (overrides: Map<guidKey, override[]>)
  const render = (n, ov, depth) => {
    const o = (ov.get(key(n.guid)) || []).reduce((acc, x) => Object.assign(acc, x), {});
    if (n.visible === false || o.visible === false) return '';
    const name = o.name || n.name || '';
    const opacity = (o.opacity !== undefined ? o.opacity : n.opacity) ?? 1;
    const idAttr = name ? ` id="${esc(name)}"` : '';
    const op = opacity !== 1 ? ` opacity="${num(opacity)}"` : '';
    const w = num(n.size ? n.size.x : 0), h = num(n.size ? n.size.y : 0);
    switch (n.type) {
      case 'ROUNDED_RECTANGLE': case 'RECTANGLE':
        return `<rect${idAttr} width="${w}" height="${h}" rx="${num(n.cornerRadius || 0)}" ${fillAttr(n)}${strokeAttr(n)}${op}${shadow(n)}${transform(n)}/>`;
      case 'ELLIPSE':
        return `<ellipse${idAttr} cx="${w / 2}" cy="${h / 2}" rx="${w / 2}" ry="${h / 2}" ${fillAttr(n)}${strokeAttr(n)}${op}${transform(n)}/>`;
      case 'LINE':
        return `<line${idAttr} x1="0" y1="0" x2="${w}" y2="0"${strokeAttr(n)}${op}${transform(n)}/>`;
      case 'TEXT': {
        const text = o.textData && o.textData.characters !== undefined ? o.textData.characters : (n.textData && n.textData.characters) || '';
        const size = n.fontSize || 16, lh = (n.lineHeight && n.lineHeight.units === 'PIXELS' && n.lineHeight.value) || Math.round(size * 1.3);
        const ah = n.textAlignHorizontal || 'LEFT', av = n.textAlignVertical || 'TOP';
        const x = ah === 'CENTER' ? w / 2 : ah === 'RIGHT' ? w : 0, anchor = ah === 'CENTER' ? 'middle' : ah === 'RIGHT' ? 'end' : 'start';
        // one line: the baseline from the vertical alignment (Inter's ascent is about 0.73 em)
        const y = av === 'CENTER' ? h / 2 + size * 0.36 : av === 'BOTTOM' ? h - (lh - size * 0.73) / 2 - size * 0.27 + size * 0.36 - size * 0.36 : (lh - size) / 2 + size * 0.73;
        const style = (n.fontName && n.fontName.style) || 'Regular';
        return `<text${idAttr} x="${num(x)}" y="${num(y)}" text-anchor="${anchor}" font-family="Inter, system-ui, sans-serif" font-size="${size}" font-weight="${weight(style)}" ${fillAttr(n)}${op}${transform(n)}>${esc(text)}</text>`;
      }
      case 'VECTOR': case 'STAR': case 'REGULAR_POLYGON': case 'BOOLEAN_OPERATION': {
        let d = '';
        for (const g of n.fillGeometry || []) d += `<path d="${pathData(g.commandsBlob)}" ${fillAttr(n)} fill-rule="${g.windingRule === 'EVENODD' ? 'evenodd' : 'nonzero'}"/>`;
        // Figma stores a stroke as the outline of the stroke: filled with the stroke paint
        const sp = solid(n.strokePaints);
        for (const g of n.strokeGeometry || []) if (sp) d += `<path d="${pathData(g.commandsBlob)}" fill="${color(sp.color, sp.opacity === undefined ? 1 : sp.opacity)}"/>`;
        return `<g${idAttr}${op}${transform(n)}>${d}</g>`;
      }
      case 'INSTANCE': {
        const symbolId = o.overriddenSymbolID || (n.symbolData && n.symbolData.symbolID);
        const symbol = symbolId && byKey.get(key(symbolId));
        if (!symbol) return `<g${idAttr}${op}${transform(n)}/>`;
        // the overrides of this instance, by the guid path of the node inside the symbol; a
        // path of two guids belongs to a nested instance: kept for its scope
        const inner = new Map();
        const add = (g, ovr) => { const k = key(g); if (!inner.has(k)) inner.set(k, []); inner.get(k).push(ovr); };
        for (const ovr of (n.symbolData && n.symbolData.symbolOverrides) || []) {
          const guids = (ovr.guidPath && ovr.guidPath.guids) || [];
          if (guids.length === 1) add(guids[0], ovr);
          else if (guids.length > 1) add(guids[0], { nested: { ...ovr, guidPath: { guids: guids.slice(1) } } });
        }
        // overrides from an outer instance aimed at this nested one
        for (const x of (ov.get(key(n.guid)) || [])) if (x.nested) { const guids = x.nested.guidPath.guids; if (guids.length === 1) add(guids[0], x.nested); else add(guids[0], { nested: { ...x.nested, guidPath: { guids: guids.slice(1) } } }); }
        const clip = symbol.frameMaskDisabled === false ? clipOf(symbol, w, h) : '';
        return `<g${idAttr}${op}${transform(n)}${clip}>${kids(symbol).map(c => render(c, inner, depth + 1)).join('')}</g>`;
      }
      case 'FRAME': case 'SYMBOL': case 'GROUP': {
        const fill = solid(n.fillPaints) ? `<rect width="${w}" height="${h}" rx="${num(n.cornerRadius || 0)}" ${fillAttr(n)}${strokeAttr(n)}${shadow(n)}/>` : (n.strokeWeight && solid(n.strokePaints) ? `<rect width="${w}" height="${h}" rx="${num(n.cornerRadius || 0)}" fill="none"${strokeAttr(n)}/>` : '');
        const clip = n.frameMaskDisabled === false ? clipOf(n, w, h) : '';
        return `<g${idAttr}${op}${transform(n)}${clip}>${fill}${kids(n).map(c => render(c, ov, depth + 1)).join('')}</g>`;
      }
      default:
        return `<!-- ${n.type} ${esc(name)} -->`;
    }
  };
  const clipOf = (n, w, h) => { const id = `clip${++uid}`; defs.push(`<clipPath id="${id}"><rect width="${w}" height="${h}" rx="${num(n.cornerRadius || 0)}"/></clipPath>`); return ` clip-path="url(#${id})"`; };

  // ---- every variant of every set of the library
  let written = 0;
  // the Figma node ids (sessionID-localID in a figma.com URL) of the sets and their variants
  const ids = {};
  for (const set of nodes.filter(n => n.isStateGroup && SETS.has(n.name))) {
    const dir = path.join(ASSETS, set.name.replace(/^#/, ''));
    fs.mkdirSync(dir, { recursive: true });
    ids[set.name] = { id: `${set.guid.sessionID}-${set.guid.localID}`, variants: Object.fromEntries(kids(set).map(v => [v.name, `${v.guid.sessionID}-${v.guid.localID}`])) };
    for (const variant of kids(set)) {
      defs = [];  // uid keeps counting: the SVGs are inlined into one page, their def ids must not repeat
      const w = num(variant.size.x), h = num(variant.size.y);
      // the variant drawn at the origin: its own transform is its place in the set
      const body = kids(variant).map(c => render(c, new Map(), 1)).join('');
      const bg = solid(variant.fillPaints) ? `<rect width="${w}" height="${h}" rx="${num(variant.cornerRadius || 0)}" ${fillAttr(variant)}/>` : '';
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">${defs.length ? `<defs>${defs.join('')}</defs>` : ''}<g id="${esc(variant.name)}">${bg}${body}</g></svg>\n`;
      fs.writeFileSync(path.join(dir, `${safe(variant.name) || 'default'}.svg`), svg);
      written++;
    }
  }
  fs.writeFileSync(path.join(ASSETS, 'nodes.json'), JSON.stringify(ids, null, 1));
  console.log(`  [previews] ${written} variant SVGs and nodes.json -> ${path.relative(process.cwd(), ASSETS)}`);
  return msg;
};
