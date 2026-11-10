// dark-theme-patch.js: the dark theme of the EveryWare Library (dark navy surfaces, light type,
// one cyan accent). Applied with everyware's roundtrip.js to a .fig built on the Library, as the
// last step of an app's design (an app's README has the chain):
//
//   node $FIGMA/roundtrip.js design/build/<app>-light.fig design/<app>.fig $UI/design/dark-theme-patch.js
//   (an app's design/build.sh runs it)
//
// Only the look changes: every #name, every frame and every variant stays, so an app draws it
// unchanged (fig-to-dcf.sh, then --data-path). The whole file is recoloured, the screen and the
// Library alike, so the toggles, sliders, buttons and rows in the slots are dark too. The colours
// are a table from the Library's palette, not a formula: a palette of ~25 colours maps by hand to
// one that is designed.

// light colour -> dark colour, for surfaces (frames, rectangles, instances)
const SURFACE = {
  '#ffffff': '#1b2531',  // cards and sheets
  '#f4f7fb': '#141d27', '#eaf0f7': '#162029', '#dfe7f3': '#1f2c3a',
  '#e6edf9': '#253443',  // soft accent: icon buttons, symbols, selected rows
  '#d3def2': '#2a3b4d', '#c9d6ea': '#2c3d50', '#b9cbe5': '#2f4a61',
  '#bdcdde': '#344556', '#bccddd': '#344556', '#a9bad9': '#3c5670',
  '#a3b1c2': '#3f4c5b',  // dividers, tracks
  '#6c7e93': '#8f9eae', '#24364c': '#e8eef4',
  '#4268b2': '#1ec0f0',  // the accent
  '#35558f': '#0f9bc6',  // the accent pressed
  '#c0392b': '#ef5a43',
};
// design colour -> hub colour, for marks (texts, glyphs, thumbs): white stays white
const MARK = { ...SURFACE, '#ffffff': '#ffffff', '#24364c': '#f3f6f9', '#6c7e93': '#94a3b3', '#a3b1c2': '#6b7a8a' };
const MARK_TYPES = new Set(['TEXT', 'VECTOR', 'ELLIPSE', 'LINE', 'STAR', 'REGULAR_POLYGON', 'BOOLEAN_OPERATION']);

// the screen's background: a deep navy gradient instead of the light one
const BACKGROUND = ['#0b121a', '#0e1822', '#131b2a'];

const hex = c => '#' + [c.r, c.g, c.b].map(v => Math.round(v * 255).toString(16).padStart(2, '0')).join('');
const rgb = (h, a) => ({ r: parseInt(h.slice(1, 3), 16) / 255, g: parseInt(h.slice(3, 5), 16) / 255, b: parseInt(h.slice(5, 7), 16) / 255, a });

function recolour(paints, table) {
  if (!paints) return paints;
  return paints.map(p => {
    if (p.type === 'SOLID' && p.color) {
      const to = table[hex(p.color)];
      return to ? { ...p, color: rgb(to, p.color.a ?? 1), colorVar: undefined } : p;
    }
    if (p.stops) {
      const stops = p.stops.map(s => { const to = table[hex(s.color)]; return to ? { ...s, color: rgb(to, s.color.a ?? 1) } : s; });
      return { ...p, stops, stopsVar: undefined };
    }
    return p;
  });
}

module.exports = function (msg) {
  const key = g => `${g.sessionID}:${g.localID}`;
  // by role where the type does not tell: a round button's background is an ellipse (a
  // surface), a thumb is a rectangle (a white mark), a scrim darkens (black)
  const tableOf = n => {
    const name = n.name || '';
    if (/\/thumb$/.test(name)) return { ...MARK, '#ffffff': '#ffffff', '#1b2531': '#ffffff' };
    if (/\/scrim$/.test(name)) return Object.fromEntries(Object.keys(SURFACE).map(k => [k, '#000000']));
    if (name === 'bg') return SURFACE;
    return MARK_TYPES.has(n.type) ? MARK : SURFACE;
  };
  for (const n of msg.nodeChanges) {
    const table = tableOf(n);
    n.fillPaints = recolour(n.fillPaints, table);
    n.strokePaints = recolour(n.strokePaints, table);
    // shadows: black and a little stronger, light shadows vanish on a dark ground
    if (n.effects) n.effects = n.effects.map(e => e.type === 'DROP_SHADOW' ? { ...e, color: { r: 0, g: 0, b: 0, a: Math.min(0.5, (e.color?.a ?? 0.1) * 2.5) } } : e);
    // overrides inside instances carry their own paints
    for (const o of (n.symbolData && n.symbolData.symbolOverrides) || []) {
      if (o.fillPaints) o.fillPaints = recolour(o.fillPaints, MARK);
      if (o.strokePaints) o.strokePaints = recolour(o.strokePaints, MARK);
    }
  }
  // the panel's root (smarthome-app's #smarthome-root, when the file has it): its light gradient
  // -> the dark one; an app's root has a palette colour, mapped above
  const root = msg.nodeChanges.find(n => n.name === '#smarthome-root' && n.fillPaints && n.fillPaints.length);
  if (root) root.fillPaints = root.fillPaints.map(p => p.stops
    ? { ...p, stops: p.stops.map((s, i) => ({ ...s, color: rgb(BACKGROUND[Math.min(i, BACKGROUND.length - 1)], 1) })), stopsVar: undefined }
    : { ...p, color: rgb(BACKGROUND[0], 1) });
  // larger rounding on the dashboard's tiles and cards (16..28 -> +4)
  for (const n of msg.nodeChanges) if (n.type !== 'TEXT' && typeof n.cornerRadius === 'number' && n.cornerRadius >= 16 && n.cornerRadius <= 28) n.cornerRadius += 4;
};
