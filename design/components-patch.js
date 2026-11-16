// components-patch.js: the UI component library of the Smart HOme Figma file, at the kiwi level.
//
//   node $FIGMA/roundtrip.js <in.fig> <out.fig> $UI/design/components-patch.js     # FIGMA: everyware/design/figma
//
// Adds a page "Components" with one component set per component (variants = states), and on
// Page 1 the frame #ui-gallery with an instance of each, which the app can show and which
// makes the fetch bring every set into the .dcf.
//
// harry switches variants at runtime (RenderCustomization::Variant) and animates the change;
// it does not read Figma prototype transitions (squoosh view.rs, TODO b/431281126), so the
// transitions are written on the page and taken by the Rust component library from
// TRANSITIONS below.
//
// Layer names starting with # are the contract with the Rust component library.

const path = require('path');
// fig-helpers.js (the node authoring) comes from everyware's roundtrip.js, the patch's second argument
let { key, C, FONT, authoring } = {};

// Transitions per change of state, from the UI library's spec (the same file the Rust library
// compiles in), rendered as the table at the bottom of the page.
const fs = require('fs');
const crypto = require('crypto');
// (the two paths can be given: a test builds the Library with every set taken from JSON)
const SPEC = JSON.parse(fs.readFileSync(process.env.EVERYWARE_COMPONENTS_JSON || path.join(__dirname, '../components.json'), 'utf8'));
// What the designers have in Figma (design/fetch-library.sh): the sets drawn there, and the
// fingerprints of the sets the code draws. Absent: nothing is drawn in Figma.
const LIBRARY_JSON = process.env.EVERYWARE_LIBRARY_JSON || path.join(__dirname, 'source/library.json');
// The code's changes on top of Figma's sets (figma-tweaks.js has the rules).
const TWEAKS = require(process.env.EVERYWARE_FIGMA_TWEAKS || path.join(__dirname, 'figma-tweaks.js'));
const FIGMA = fs.existsSync(LIBRARY_JSON) ? JSON.parse(fs.readFileSync(LIBRARY_JSON, 'utf8')) : { components: {}, code: {}, sets: {} };
const NAMES = { button: 'Button', icon_button: 'Icon button', toggle: 'Toggle', radio: 'Radio', dropdown: 'Dropdown', list_item: 'List item', slider: 'Slider', input_field: 'Input field', keyboard: 'Keyboard keys', bar_chart: 'Bar chart', overlay: 'Overlay', form: 'Form', keyboard_dock: 'Keyboard dock', icon: 'Icon', scrollbar: 'Scrollbar', sheet: 'Sheet', skeleton: 'Skeleton', segmented: 'Segmented button', tile: 'Tile', tabs: 'Tabs', pages: 'Pages' };
const CHANGE = { press: 'default → pressed', release: 'pressed → default', value: 'off ↔ on', open: 'closed → open', close: 'open → closed', select: '→ selected', focus: '→ focused', blur: 'focused →', sweep: 'loading: one pass, repeated', switch: 'screen → screen (the screens\' effect)' };
const TRANSITIONS = SPEC.transitions.map(t => [t.components.map(c => NAMES[c]).join(', '), CHANGE[t.change], `${t.ms} ms`, t.easing])
  .concat([['all', '↔ disabled', 'none', 'changes at once']]);

module.exports = function patch(msg, helpers) {
  ({ key, C, FONT, authoring } = helpers);
  const { fromRest } = helpers;
  const A = authoring(msg, 'components');
  const { nodes, kids, log, base, frame, rect, ellipse, text, stroke, shadow, chevron, plus, componentSet: drawSet, instance, meter, FILL, copyNode, arc, at } = A;
  const doc = nodes.find(n => n.type === 'DOCUMENT');
  const page1 = nodes.find(n => n.type === 'CANVAS' && n.name === 'Page 1');
  if (nodes.some(n => n.type === 'CANVAS' && n.name === 'Components')) throw new Error('page Components already present');

  // ---- the page, between Page 1 ("!") and the internal canvas ("~")
  const page = base(doc, 'CANVAS', 'Components', 0, 0, 0, 0, {
    strokeAlign: 'CENTER', strokeJoin: 'BEVEL', backgroundOpacity: 1, backgroundEnabled: true,
    backgroundColor: { r: 0.118, g: 0.118, b: 0.118, a: 1 },
  }, '"');
  delete page.size;
  // A top-level frame of the page: the fetch resolves names one level below the pages only
  // (depth=2), so scripts/fetch-dcf.sh fetches "Library" to bring every set along.
  const lib = frame(page, 'Library', 0, 0, 3140, 8700, { fillPaints: [A.solid(C.page)], cornerRadius: 0 });
  text(lib, 'Title', 80, 48, 1200, 44, 'Smart HOme · Components', 34, FONT.semibold, C.ink);
  text(lib, 'Subtitle', 80, 96, 1600, 24, 'Component sets for harry: variants are states, switched by the Rust component library. Transitions: table at the bottom.', 17, FONT.regular, C.ink2);
  const heading = (x, y, s) => text(lib, 'Heading', x, y - 40, 800, 28, s, 21, FONT.semibold, C.ink);

  // ---- One owner per component set. The code draws a set (drawSet) unless its component has
  // "figma": true in components.json: then the set is the one the designers drew in Figma
  // (source/library.json, fetched), built here from its JSON at the same place and checked
  // against components.json, so the code can drive it. Everything below calls componentSet and
  // gets { set, symbols } either way.
  const specOf = name => Object.entries(SPEC.components).find(([, c]) => c.set === name) || [null, null];
  const fromFigma = new Map();   // set name -> { set, symbols }, built once (an instance in another set may need it first)
  const kiwiSymbol = (setName, variant) => {
    if (specOf(setName)[1] && specOf(setName)[1].figma) figmaSet(setName);
    const st = nodes.find(n => n.isStateGroup && n.name === setName);
    return st && kids(key(st.guid)).find(n => n.type === 'SYMBOL' && n.name === variant);
  };
  const resolve = id => { const c = FIGMA.components[id]; return c && kiwiSymbol(c.set, c.name); };
  const below = n => kids(key(n.guid)).flatMap(k => [k, ...below(k)]);
  // a set's drawing by the code as a fingerprint: to notice later that the code changed a set the
  // designers have a copy of (fetch-library.js keeps it next to Figma's)
  const codeHashes = {};
  const hashOf = set => {
    const blob = i => crypto.createHash('sha1').update(Buffer.from(msg.blobs[i].bytes)).digest('hex').slice(0, 8);
    const skip = new Set(['guid', 'id', 'propDefId', 'symbolID', 'guids', 'overriddenSymbolID']);
    // (the set's own place in the Library is not its drawing)
    // (numbers to four decimals: a value that went through Figma's JSON is the same value)
    const body = [{ ...set, transform: undefined }, ...below(set)].map(n => JSON.stringify(n, (k, v) => (skip.has(k) ? undefined : /Blob$/.test(k) && typeof v === 'number' ? blob(v) : typeof v === 'number' ? Math.round(v * 1e4) / 1e4 : v)));
    return crypto.createHash('sha1').update(body.join('\n')).digest('hex').slice(0, 16);
  };
  // what a tweak gets (figma-tweaks.js): the set, its variants and layers, setters that assign
  // only what differs, so a tweak applied to its own result changes nothing
  const sortedKids = n => kids(key(n.guid)).slice().sort((a, b) => (a.parentIndex.position < b.parentIndex.position ? -1 : a.parentIndex.position > b.parentIndex.position ? 1 : 0));
  const near = (a, b) => (typeof a === 'number' && typeof b === 'number' ? Math.abs(a - b) < 1e-4
    : a && b && typeof a === 'object' && typeof b === 'object' ? Object.keys({ ...a, ...b }).every(k => near(a[k], b[k])) : a === b);
  const tweakTools = set => {
    const t = {
      set: null, A, C, FONT,
      variants: () => sortedKids(set).filter(n => n.type === 'SYMBOL'),
      variant: name => kids(key(set.guid)).find(n => n.type === 'SYMBOL' && n.name === name) || null,
      all: v => below(v),
      find: (v, layer) => below(v).find(n => n.name === layer) || null,
      fill: (node, rgb) => t.set(node, { fillPaints: [A.solid(rgb)] }),
      moveBelow: (node, sibling) => {
        const parent = nodes.find(n => key(n.guid) === key(node.parentIndex.guid));
        const order = sortedKids(parent).filter(n => n !== node);
        order.splice(order.indexOf(sibling), 0, node);
        if (order.length > 90) throw new Error(`${set.name}: more than 90 layers in ${parent.name}`);
        order.forEach((n, i) => { n.parentIndex = { ...n.parentIndex, position: String.fromCharCode(0x21 + i) }; });
      },
      // a new variant as a copy of another, below the others; nothing when the set has it
      addVariant: (from, name, edit) => {
        if (t.variant(name)) return t.variant(name);
        const src = t.variant(from);
        if (!src) throw new Error(`${set.name}: no variant "${from}" to copy`);
        const bottom = Math.max(...t.variants().map(v => v.transform.m12 + v.size.y));
        // (the layers in their stacking order: copyNode takes them as they were made)
        const dup = (n, parent, x, y) => {
          const c = JSON.parse(JSON.stringify(n));
          for (const f of ['guid', 'parentIndex', 'phase', 'transform', 'size', 'type', 'name']) delete c[f];
          const made = base(parent, n.type, n.name, x, y, n.size.x, n.size.y, c);
          made.transform = { ...n.transform, m02: x, m12: y };
          for (const k of sortedKids(n)) dup(k, made, k.transform.m02, k.transform.m12);
          return made;
        };
        const copy = dup(src, set, src.transform.m02, bottom + 24);
        copy.name = name;
        const values = name.split(',').map(x => x.trim().split('='));
        copy.variantPropSpecs = values.map(([p, value]) => {
          const def = (set.componentPropDefs || []).find(d => d.name === p);
          if (!def) throw new Error(`${set.name}: "${name}" names the property ${p}, which the set does not have`);
          const order = set.stateGroupPropertyValueOrders.find(o => o.property === p);
          if (!order.values.includes(value)) order.values.push(value);
          return { propDefId: def.id, value };
        });
        set.size = { x: set.size.x, y: Math.max(set.size.y, copy.transform.m12 + copy.size.y + 24) };
        if (edit) edit(copy);
        return copy;
      },
    };
    t.set = (node, props) => {
      if (!node) throw new Error(`${set.name}: a tweak sets a layer that is not there`);
      for (const [k, v] of Object.entries(props)) if (!near(node[k], v)) node[k] = v;
      return node;
    };
    t.node = set;
    return t;
  };
  function figmaSet(name) {
    if (fromFigma.has(name)) return fromFigma.get(name);
    const [id, c] = specOf(name);
    const rest = FIGMA.sets[name];
    if (!rest) throw new Error(`${name} has "figma": true in components.json and is not in design/source/library.json: design/fetch-library.sh`);
    const set = fromRest(A, msg, lib, rest, { x: 0, y: 0, resolve, where: 'Figma' });
    if (TWEAKS[name]) {
      const before = hashOf(set);
      TWEAKS[name](tweakTools(set));
      log(hashOf(set) !== before
        ? `note: ${name}: the code changes Figma's drawing (design/figma-tweaks.js) and the designers do not have the change: paste the set from design/build/library.fig into their Figma file`
        : `note: ${name}: its tweak changes nothing, Figma has the change: delete it from design/figma-tweaks.js`);
    }
    const variants = kids(key(set.guid)).filter(n => n.type === 'SYMBOL');
    const props = Object.entries(c.props || {});
    const names = variants.map(v => v.name);
    const problems = [];
    // (which combinations of the props a set has is its own matter: #list-item has no image row
    // with one line; a variant the code asks for and the design lacks fails everyware_ui_test)
    if (!variants.length) problems.push('no variants');
    // (a component without props in components.json has one variant, whatever its name)
    for (const nm of props.length ? names : []) {
      const pv = nm.split(',').map(x => x.trim().split('='));
      if (pv.length !== props.length || pv.some(([k, v], i) => !props[i] || props[i][0] !== k || !props[i][1].includes(v))) problems.push(`variant "${nm}" does not fit the props of components.json (${props.map(([k, vs]) => `${k}: ${vs.join('|')}`).join(', ')})`);
    }
    for (const v of variants) {
      const all = below(v);
      for (const part of c.parts || []) if (!all.some(n => n.name === part)) problems.push(`"${v.name}" has no layer ${part}`);
      for (const n of [v, ...all]) if (n.visible === false) problems.push(`"${v.name}": ${n.name} is hidden (harry would never show it: use opacity 0)`);
    }
    if (problems.length) throw new Error(`${name}, drawn in Figma, does not match components.json "${id}":\n  ${problems.slice(0, 12).join('\n  ')}${problems.length > 12 ? `\n  ... ${problems.length - 12} more` : ''}`);
    log(`${name}: drawn in Figma, ${variants.length} variants${(c.parts || []).length ? `, parts ${c.parts.join(' ')}` : ''}`);
    const made = { set, symbols: Object.fromEntries(variants.map(v => [v.name, v])), placed: false };
    fromFigma.set(name, made);
    return made;
  }
  const componentSet = (parent, name, x, y, ...rest) => {
    const [, c] = specOf(name);
    if (c && c.figma) {
      const made = figmaSet(name);
      made.set.transform = { ...made.set.transform, m02: x, m12: y };
      made.placed = true;
      return made;
    }
    const seen = FIGMA.code[name];
    if (seen && seen.figma !== seen.accepted) throw new Error(`${name} has two owners: the code draws it, and a designer changed it in Figma. Decide: hand it over ("figma": true in components.json, then design/fetch-library.sh), or keep the code (design/fetch-library.sh --accept '${name}')`);
    const made = drawSet(parent, name, x, y, ...rest);
    codeHashes[name] = hashOf(made.set);
    if (seen && seen.code && seen.code !== codeHashes[name]) log(`note: ${name} is drawn by code and its drawing changed since the designers got it: paste the new set from design/build/library.fig into their Figma file, then design/fetch-library.sh --accept '${name}'`);
    return made;
  };

  // ---- Button with text
  heading(80, 180, 'Button');
  const btnColors = {
    primary: { default: [C.accent, C.white], pressed: [C.accentPressed, C.white], disabled: [C.accentDisabled, C.white] },
    secondary: { default: [C.accentSoft, C.ink], pressed: [C.accentSoftPressed, C.ink], disabled: [C.surface, C.muted] },
    ghost: { default: [null, C.accent], pressed: [C.accentSoft, C.accent], disabled: [null, C.accentDisabled] },
  };
  // Pressed is not a variant (a variant change rebuilds the tree, ~50 ms on the RB3): the
  // #<set>/pressed layer in the pressed colour fades in by opacity (display-only).
  // As Google draws a button (ListTestUI.fig "Button"): a fixed size per variant, the fill and
  // the radius on the variant itself, the label a box across it with the text centred. The
  // width is the design's (size=regular 200, compact 116), never the label's: a clone in a
  // slot is exactly the variant, a longer label stays centred.
  const BTN_W = { regular: 200, compact: 116 }, BTN_H = 56;
  componentSet(lib, '#button', 80, 180, [['kind', ['primary', 'secondary', 'ghost']], ['state', ['default', 'disabled']], ['size', ['regular', 'compact']]], 200, BTN_H, 24, 24, (s, v) => {
    const [bg, ink] = btnColors[v.kind][v.state];
    const w = BTN_W[v.size];
    s.size = { x: w, y: BTN_H };
    s.fillPaints = bg ? [A.solid(bg)] : [];
    s.cornerRadius = BTN_H / 2;
    s.frameMaskDisabled = false;
    rect(s, '#button/pressed', 0, 0, w, BTN_H, btnColors[v.kind].pressed[0] || C.accentSoft, BTN_H / 2, { opacity: 0, ...FILL });
    text(s, '#button/label', 0, 0, w, BTN_H, 'Button', 17, v.kind === 'primary' ? FONT.semibold : FONT.medium, ink, 'CENTER', 'CENTER', FILL);
  }, { cols: 4 });

  // ---- Button with icon: as on the dashboard (Smart HOme.fig): a circle in the widget's
  // colour with one of the design's icons. The icons are copies of the dashboard's vectors
  // (stroke 1.7); wifi is drawn here, the design has none. kind: tonal (the widgets), glass
  // (the status row: white 44 % with a white line, as Presence), scrim (white 15 % on the
  // camera). size: regular 56, compact 48 (Adjust humidity, the Wi-Fi indicator).
  // the right column, from under the bar chart down to the list items: the set grows with the
  // spec's icons (components.json), 13 of them are 984 high
  heading(2400, 1300, 'Icon button');
  const ICON_SIZE = { regular: 56, compact: 48 };
  const TARGET = 72;
  const iconKinds = {
    tonal: { bg: [C.accentSoft, 1], pressed: C.accentSoftPressed, ink: C.ink, line: null },
    glass: { bg: [C.white, 0.44], pressed: C.white, ink: C.ink, line: C.white },
    scrim: { bg: [C.white, 0.15], pressed: C.white, ink: C.white, line: null },
  };
  // the icons are the spec's (ui-lib/components.json), the same list as the Rust enum Icon
  const ICONS = SPEC.components.icon_button.props.icon;
  // the dashboard's icon frames (a frame named after the icon with a vector inside)
  const designIcon = name => {
    const f = nodes.find(n => n.type === 'FRAME' && n.name === name && kids(key(n.guid)).some(c => c.type === 'VECTOR'));
    if (!f) throw new Error(`icon ${name}: not in the design`);
    return f;
  };
  const wifiIcon = (parent, x, y, ink) => {
    // 24 x 24 in the style of the design's icons: three arcs around a dot
    const g = frame(parent, 'wifi', x, y, 24, 24);
    [4.5, 8.5, 12.5].forEach((r, i) => arc(g, `arc-${i + 1}`, 12, 20, r, 1.7, -3 * Math.PI / 4, -Math.PI / 4, ink));
    ellipse(g, 'dot', 10.3, 18.3, 3.4, 3.4, ink);
    return g;
  };
  // 24 x 24, the design's has none: two strokes from corner to corner (Lucide "x": 6,6 to 18,18),
  // the line weight of the design's icons
  const closeIcon = (parent, x, y, ink) => {
    const g = frame(parent, 'x', x, y, 24, 24);
    const len = 12 * Math.SQRT2, th = 1.7;
    [45, -45].forEach((deg, i) => {
      const r = deg * Math.PI / 180;
      // the bar's centre on (12, 12) after rotating about its top-left corner
      const ox = 12 - (Math.cos(r) * len / 2 - Math.sin(r) * th / 2), oy = 12 - (Math.sin(r) * len / 2 + Math.cos(r) * th / 2);
      base(g, 'ROUNDED_RECTANGLE', `stroke-${i + 1}`, 0, 0, len, th, { fillPaints: [A.solid(ink)], cornerRadius: th / 2, transform: at(ox, oy, deg) });
    });
    return g;
  };
  const icon = (parent, name, S, ink) => {
    if (name === 'wifi') return wifiIcon(parent, (S - 24) / 2, (S - 24) / 2, ink);
    if (name === 'x') return closeIcon(parent, (S - 24) / 2, (S - 24) / 2, ink);
    const f = designIcon(name);
    return copyNode(f, parent, (S - f.size.x) / 2, (S - f.size.y) / 2, n => { if (n.type === 'VECTOR') n.strokePaints = [A.solid(ink)]; });
  };
  // The variant is TARGET x TARGET: the touch target (harry hits the clone's bounds), the
  // circle of ICON_SIZE is the visual, centred (Material: 40 in 48; AAOS asks for 76).
  componentSet(lib, '#icon-button', 2400, 1300, [['kind', ['tonal', 'glass', 'scrim']], ['state', ['default', 'disabled']], ['icon', ICONS], ['size', ['regular', 'compact']]], TARGET, TARGET, 24, 24, (s, v) => {
    const k = iconKinds[v.kind], S = ICON_SIZE[v.size], dis = v.state === 'disabled', o = (TARGET - S) / 2;
    ellipse(s, 'bg', o, o, S, S, k.bg[0], { opacity: k.bg[1], ...(k.line ? stroke(k.line, 1) : {}) });
    ellipse(s, '#icon-button/pressed', o, o, S, S, k.pressed, { opacity: 0 });
    const i = icon(s, v.icon, S, dis && v.kind === 'tonal' ? C.muted : k.ink);
    i.name = '#icon-button/icon';
    i.transform = at(i.transform.m02 + o, i.transform.m12 + o);
    if (dis && v.kind !== 'tonal') i.opacity = 0.45;
  }, { cols: 16 });

  // ---- Toggle
  heading(1080, 180, 'Toggle');
  // The value is not a variant: a variant change rebuilds the tree and lays the screen out
  // again (~60 ms on the RB3). The thumb is a progress marker meter and the blue track a
  // layer whose opacity follows the value; both are display-only, drawn from the cache.
  const toggle = componentSet(lib, '#toggle', 1080, 180, [['state', ['default', 'pressed', 'disabled']]], 68, 40, 32, 0, (s, v) => {
    const op = v.state === 'disabled' ? 0.45 : 1;
    rect(s, 'track', 0, 0, 68, 40, C.track, 20, { opacity: op });
    rect(s, '#toggle/on', 0, 0, 68, 40, C.accent, 20, { opacity: op });
    // pressed: the thumb stretches, as on phones; the marker's travel shrinks with it
    const tw = v.state === 'pressed' ? 40 : 32;
    // At x = 0: harry draws a horizontal marker at node x + startX..endX (it cancels only the
    // y of a vertical one, generation.rs), DesignCompose reads startX..endX in the parent; at
    // x = 0 both are the same. The meter places it, also in the preview of the plugin.
    const thumb = rect(s, '#toggle/thumb', 0, 4, tw, 32, C.white, 16, { opacity: op, effects: shadow(1, 3, 0.15) });
    meter(thumb, 'progressMarkerData', { startX: 4, endX: 68 - 4 - tw });
  });

  // ---- Radio
  heading(80, 540, 'Radio');
  componentSet(lib, '#radio', 80, 540, [['value', ['off', 'on']], ['state', ['default', 'pressed', 'disabled']]], 240, 48, 24, 16, (s, v) => {
    const on = v.value === 'on', dis = v.state === 'disabled';
    if (v.state === 'pressed') ellipse(s, 'halo', 4, 4, 40, 40, C.accentSoft);
    const ring = dis ? C.muted : on ? C.accent : C.ink2;
    ellipse(s, 'ring', 12, 12, 24, 24, C.white, stroke(ring, 2));
    if (on) ellipse(s, 'dot', 18, 18, 12, 12, dis ? C.accentDisabled : C.accent);
    text(s, '#radio/label', 52, 0, 188, 48, 'Option', 17, FONT.regular, dis ? C.muted : C.ink, 'LEFT', 'CENTER');
  });

  // ---- Dropdown: as Presence on the dashboard: a glass pill (white 44 % with a white line)
  // with a status lamp, the value and the design's chevron; the list opens below it. open is
  // not a variant either: the list is always in the component, over what lies below, and
  // fades in and out by opacity (display-only); the selection mark of an item is an opacity
  // too. Shown open here; the app starts it closed.
  heading(960, 540, 'Dropdown');
  const DD_W = 153, DD_H = 48, ITEM_H = 44, LIST_Y = DD_H + 8, LIST_H = 8 + 3 * ITEM_H + 8, OPEN_H = LIST_Y + LIST_H;
  const ddSet = componentSet(lib, '#dropdown', 960, 540, [['state', ['default', 'disabled']]], DD_W, OPEN_H, 32, 0, (s, v) => {
    const dis = v.state === 'disabled';
    rect(s, 'field', 0, 0, DD_W, DD_H, C.white, DD_H / 2, { opacity: 0.44, ...stroke(C.white, 1) });
    ellipse(s, 'lamp', 20, (DD_H - 7) / 2, 7, 7, dis ? C.muted : C.accent);
    text(s, '#dropdown/value', 35, 0, DD_W - 35 - 40, DD_H, 'At home', 17, FONT.regular, dis ? C.muted : C.ink, 'LEFT', 'CENTER');
    const chev = designIcon('chevron-down');
    copyNode(chev, s, DD_W - 20 - chev.size.x, (DD_H - chev.size.y) / 2, n => { if (n.type === 'VECTOR') n.strokePaints = [A.solid(dis ? C.muted : C.ink2)]; });
    const list = frame(s, '#dropdown/list', 0, LIST_Y, DD_W, LIST_H);
    rect(list, 'bg', 0, 0, DD_W, LIST_H, C.white, 16, { effects: shadow(8, 24, 0.10) });
    ['At home', 'Away', 'Asleep'].forEach((label, i) => {
      const item = frame(list, `#dropdown/item/${i + 1}`, 0, 8 + i * ITEM_H, DD_W, ITEM_H);
      rect(item, `#dropdown/item/${i + 1}/selected`, 8, 4, DD_W - 16, ITEM_H - 8, C.accentSoft, 10, { opacity: i === 0 ? 1 : 0 });
      text(item, `#dropdown/item/${i + 1}/label`, 20, 0, DD_W - 40, ITEM_H, label, 17, FONT.regular, C.ink, 'LEFT', 'CENTER');
    });
  });
  ddSet.set.size = { x: ddSet.set.size.x, y: 24 * 2 + OPEN_H };

  // ---- List item
  // ---- Icon: a glyph for the slots of other components (the list item's leading and
  // trailing). Lucide, 24 px, ink: the design's own icon frames where it has them, the
  // wifi family (wifi / wifi-high / wifi-low / wifi-zero: the signal level, 3..0 arcs lit
  // over the full fan, as Android draws it, so every level has the same outline) and the
  // lock drawn here.
  heading(1080, 900, 'Icon');
  const ICON_GLYPHS = ['wifi', 'wifi-high', 'wifi-low', 'wifi-zero', 'lock', 'chevron-right', 'minus', 'plus', 'settings-2', 'sliders-horizontal', 'maximize-2'];
  const SIGNAL_OFF = 0.25;
  const signalIcon = (parent, x, y, arcs, ink) => {
    const g = frame(parent, 'glyph', x, y, 24, 24);
    // the arcs above the level stay, dimmed: the outline is the same at every level
    [4.5, 8.5, 12.5].forEach((r, i) => arc(g, `arc-${i + 1}`, 12, 20, r, 1.7, -3 * Math.PI / 4, -Math.PI / 4, ink, i < arcs ? undefined : { opacity: SIGNAL_OFF }));
    ellipse(g, 'dot', 10.3, 18.3, 3.4, 3.4, ink);
    return g;
  };
  const lockIcon = (parent, x, y, ink) => {
    const g = frame(parent, 'glyph', x, y, 24, 24);
    rect(g, 'body', 4, 11, 16, 10, null, 2, { ...stroke(ink, 1.7) });
    arc(g, 'shackle', 12, 11, 5, 1.7, Math.PI, 2 * Math.PI, ink);
    return g;
  };
  const iconSet = componentSet(lib, '#icon', 1080, 900, [['icon', ICON_GLYPHS]], 24, 24, 16, 16, (s, v) => {
    const arcs = { wifi: 3, 'wifi-high': 2, 'wifi-low': 1, 'wifi-zero': 0 }[v.icon];
    if (arcs !== undefined) return signalIcon(s, 0, 0, arcs, C.ink);
    if (v.icon === 'lock') return lockIcon(s, 0, 0, C.ink);
    const f = designIcon(v.icon);
    return copyNode(f, s, (24 - f.size.x) / 2, (24 - f.size.y) / 2, n => { if (n.type === 'VECTOR') n.strokePaints = [A.solid(C.ink)]; });
  }, { cols: 11 });
  const glyphOf = name => iconSet.symbols[`icon=${name}`];

  // ---- Symbol: the tile with a room's or a device's symbol (the dashboard's "Room symbol":
  // 48 x 48, the soft accent, the glyph in ink). The variant is the symbol; a slot takes the
  // variant the code picks.
  heading(2200, 180, 'Symbol');
  const SYMBOLS = SPEC.components.symbol.props.icon;
  componentSet(lib, '#symbol', 2200, 180, [['icon', SYMBOLS]], 48, 48, 16, 16, (s, v) => {
    rect(s, 'bg', 0, 0, 48, 48, C.accentSoft, 16);
    const f = designIcon(v.icon);
    const g = copyNode(f, s, (48 - f.size.x) / 2, (48 - f.size.y) / 2, n => { if (n.type === 'VECTOR') n.strokePaints = [A.solid(C.ink)]; });
    g.name = '#symbol/icon';
  }, { cols: SYMBOLS.length });

  // ---- Segmented button (Material 3): one choice of 2 to 4, as the dashboard's fan speed
  // selector draws it: a pill (white 52 %), the segments in it, the chosen one filled with the
  // accent and its label white. The choice is not a variant (a variant change rebuilds the tree,
  // see #toggle): every segment has its highlight and two labels (ink2 and white) and the code
  // lights the chosen one by opacity; it sets the labels. Each segment is a touch target.
  heading(2200, 420, 'Segmented button');
  const SEG_W = 243, SEG_H = 56, SEG_PAD = 5, SEG_GAP = 4, SEG_LABELS = ['Low', 'Med', 'High', 'Max'];
  componentSet(lib, '#segmented', 2200, 420, [['segments', ['2', '3', '4']], ['state', ['default', 'disabled']]], SEG_W, SEG_H, 24, 16, (s, v) => {
    const n = Number(v.segments), op = v.state === 'disabled' ? 0.45 : 1;
    const w = (SEG_W - 2 * SEG_PAD - (n - 1) * SEG_GAP) / n, h = SEG_H - 2 * SEG_PAD;
    rect(s, 'bg', 0, 0, SEG_W, SEG_H, C.white, SEG_H / 2, { opacity: 0.52 });
    for (let i = 1; i <= n; i++) {
      const on = i === 1;
      const seg = frame(s, `#segmented/segment-${i}`, SEG_PAD + (i - 1) * (w + SEG_GAP), SEG_PAD, w, h, { opacity: op });
      rect(seg, `#segmented/segment-${i}/selected`, 0, 0, w, h, C.accent, h / 2, { opacity: on ? 1 : 0 });
      text(seg, `#segmented/segment-${i}/label`, 0, 0, w, h, SEG_LABELS[i - 1], 14, FONT.medium, C.ink2, 'CENTER', 'CENTER', { opacity: on ? 0 : 1, maxLines: 1, textTruncation: 'ENDING' });
      text(seg, `#segmented/segment-${i}/label-selected`, 0, 0, w, h, SEG_LABELS[i - 1], 14, FONT.medium, C.white, 'CENTER', 'CENTER', { opacity: on ? 1 : 0, maxLines: 1, textTruncation: 'ENDING' });
    }
  }, { rows: true });

  // ---- Scrollbar: one component for a list of any height, as Android draws a View's
  // scrollbar from the list's extent (View.onDrawScrollBars). It is as high as the screen;
  // the code sets the track (a vertical progress bar: it grows from the bottom) to the list's
  // height and moves the thumb (a vertical progress marker) along it, and puts the clone with
  // its bottom at the list's bottom (ui-lib slot.rs scrollbar(); an app's slot: fig-helpers
  // scrollbarSlot). The code shows it while the content moves and fades it 300 ms after
  // (components.json hide). SB_MAX and SB_THUMB are the code's too (slot.rs).
  heading(2200, 1000, 'Scrollbar');
  const SB_W = 6, SB_THUMB = 48, SB_MAX = 1080;
  componentSet(lib, '#scrollbar', 2200, 1000, [['state', ['default']]], SB_W, SB_MAX, 24, 24, (s) => {
    s.size = { x: SB_W, y: SB_MAX };
    meter(rect(s, '#scrollbar/track', 0, 0, SB_W, SB_MAX, C.line, SB_W / 2, { opacity: 0.5 }), 'progressBarData', { vertical: true, endY: SB_MAX });
    meter(rect(s, '#scrollbar/thumb', 0, 0, SB_W, SB_THUMB, C.ink2, SB_W / 2), 'progressMarkerData', { vertical: true, startY: 0, endY: SB_MAX - SB_THUMB });
  }, { cols: 1 });

  // on their own below the transitions table, not over the other sets (the Tile below them)
  heading(80, 4000, 'List item');
  // The roles of Material 3's list item: a leading slot (none, an #icon glyph, an image), the
  // headline (#list-item/title), the supporting text (#list-item/subtitle) and the trailing
  // end (an #icon glyph, a toggle, a chevron, a trailing text with an image, or with kind text
  // the trailing supporting text alone: a value on the right of its label). lines: one
  // (headline only), two (headline and one line of supporting text), three (a tall row:
  // headline and supporting text up to two lines each, the image as a thumbnail). size: the
  // width (regular 440: a card's column; column 490: a dashboard column; panel 856: the content
  // of a left or right #sheet beside a scrollbar; wide 960: a dialog sheet). Kind leading-text:
  // a short text of two lines at the start and a thin bar in a colour the code sets; kind
  // subheader: the label of the group of rows below it, lower and without a card.
  const LI_SIZE = { regular: 440, column: 490, panel: 856, wide: 960 }, LI_HEIGHT = { one: 72, two: 72, three: 128 };
  const LI_KINDS = ['plain', 'icon', 'image', 'toggle', 'chevron', 'text', 'leading-text', 'subheader'];
  // Material's list subheader (48): the label at the bottom, nearer the rows it heads than the
  // rows above it. In a list of rows it takes its own pitch (ui-lib ListItem::pitch).
  const SUBHEADER_H = 48;
  // the leading text's column ("12:30 PM" in 15 px semi bold), the bar after it
  const LEAD_W = 80, ACCENT_W = 4;
  // kind text: the trailing supporting text is the row's value ("Firmware · 0.1.0"), in one line;
  // a subheader has one line, a leading text two
  const liCombo = v => v.kind === 'image' ? v.lines !== 'one'
    : v.kind === 'text' || v.kind === 'subheader' ? v.lines === 'one'
    : v.kind === 'leading-text' ? v.lines === 'two' : v.lines !== 'three';
  const liSet = componentSet(lib, '#list-item', 80, 4000, [['state', ['default', 'selected', 'disabled']], ['kind', LI_KINDS], ['lines', ['one', 'two', 'three']], ['size', ['regular', 'column', 'panel', 'wide']]], 960, LI_HEIGHT.three, 24, 24, (s, v) => {
    const op = v.state === 'disabled' ? 0.45 : 1, LI_W = LI_SIZE[v.size], LI_H = v.kind === 'subheader' ? SUBHEADER_H : LI_HEIGHT[v.lines], tall = v.lines === 'three';
    s.size = { x: LI_W, y: LI_H };
    // a text longer than its lines ends in an ellipsis (harry: max lines and ellipsis of Impeller)
    const lines = n => ({ maxLines: n, textTruncation: 'ENDING' });
    if (v.kind === 'subheader') {
      // no card and no selected look: a label on the list's background. The pressed layer
      // every row has, without a fill: a tap on a subheader shows nothing.
      const VALUE_W = Math.round(LI_W * 0.45);
      rect(s, '#list-item/pressed', 0, 0, LI_W, LI_H, null, 16, { opacity: 0 });
      text(s, '#list-item/title', 20, 20, LI_W - 40 - VALUE_W - 12, 20, 'SECTION', 14, FONT.semibold, C.ink2, 'LEFT', 'TOP', { opacity: op, ...lines(1) });
      text(s, '#list-item/trailing-text', LI_W - 20 - VALUE_W, 20, VALUE_W, 20, '3 ITEMS', 14, FONT.medium, C.ink2, 'RIGHT', 'TOP', { opacity: op, ...lines(1) });
      return;
    }
    const bg = v.state === 'selected' ? C.accentSoft : C.white;
    rect(s, 'bg', 0, 0, LI_W, LI_H, bg, 16);
    rect(s, '#list-item/pressed', 0, 0, LI_W, LI_H, v.state === 'selected' ? C.accentSoftPressed : C.accentSoft, 16, { opacity: 0 });
    if (v.state === 'selected') rect(s, 'mark', 0, 16, 4, LI_H - 32, C.accent, 2);
    let tx = 20;
    // An icon slot: every glyph of #icon stacked at opacity 0, the code lights one per row
    // (harry cannot swap a variant inside a list item; opacity it can, per item).
    const iconSlot = (name, x) => {
      const slot = frame(s, name, x, (LI_H - 24) / 2, 24, 24, { opacity: op });
      ICON_GLYPHS.forEach(g => instance(slot, `${name}/${g}`, 0, 0, glyphOf(g), { opacity: 0 }));
      return slot;
    };
    if (v.kind === 'icon') {
      iconSlot('#list-item/leading', 20);
      tx = 64;
    }
    // the image: a placeholder the code replaces by a registered picture (everyware_ui::images).
    // harry draws a picture on a plain rectangle only (pre_renderer/generation.rs, Shape::Rect),
    // so the rounded corners are a frame that clips around it.
    if (v.kind === 'image') {
      const [iw, ih] = tall ? [120, 90] : [56, 56];
      const clip = frame(s, 'image-clip', 16, (LI_H - ih) / 2, iw, ih, { cornerRadius: 12, frameMaskDisabled: false, opacity: op });
      // a frame without corners: a plain rectangle in harry (a ROUNDED_RECTANGLE node may not be)
      frame(clip, '#list-item/image', 0, 0, iw, ih, { fillPaints: [A.solid(C.line)] });
      tx = 16 + iw + 16;
    }
    // the leading text: a strong line over a dim one, and the bar in the colour the code sets: a
    // plain frame in a clip with the corners (harry draws a picture, the colour, on a plain
    // rectangle only); the design's accent until the code sets one
    if (v.kind === 'leading-text') {
      text(s, '#list-item/leading-title', 20, 14, LEAD_W, 24, '12:30', 15, FONT.semibold, C.ink, 'LEFT', 'TOP', { opacity: op, ...lines(1) });
      text(s, '#list-item/leading-subtitle', 20, 40, LEAD_W, 20, '30 min', 13, FONT.regular, C.ink2, 'LEFT', 'TOP', { opacity: op, ...lines(1) });
      const clip = frame(s, 'accent-clip', 20 + LEAD_W + 12, 14, ACCENT_W, LI_H - 28, { cornerRadius: ACCENT_W / 2, frameMaskDisabled: false, opacity: op });
      frame(clip, '#list-item/accent', 0, 0, ACCENT_W, LI_H - 28, { fillPaints: [A.solid(C.accent)] });
      tx = 20 + LEAD_W + 12 + ACCENT_W + 12;
    }
    // the trailing end: an icon slot (shown or hidden by code) before the toggle / chevron, or
    // with an image the trailing text (a source and an age) at the headline's line; a leading
    // text has none, its texts reach the row's end
    const TT_W = 56, VALUE_W = Math.round(LI_W * 0.45);
    const trailingX = v.kind === 'text' ? LI_W - 20 - VALUE_W : v.kind === 'toggle' ? LI_W - 16 - 68 - 12 - 24 : v.kind === 'chevron' ? LI_W - 28 - 12 - 24 : v.kind === 'image' ? LI_W - 16 - TT_W : LI_W - 20 - 24;
    const textW = (v.kind === 'leading-text' ? LI_W - 20 + 12 : trailingX) - 12 - tx;
    if (v.lines === 'one') {
      text(s, '#list-item/title', tx, 0, textW, LI_H, 'Living room lights', 17, FONT.medium, C.ink, 'LEFT', 'CENTER', { opacity: op, ...lines(1) });
    } else if (tall) {
      text(s, '#list-item/title', tx, 16, textW, 44, 'A headline that may take two lines', 17, FONT.medium, C.ink, 'LEFT', 'TOP', { opacity: op, lineHeight: { value: 22, units: 'PIXELS' }, ...lines(2) });
      text(s, '#list-item/subtitle', tx, 66, LI_W - 16 - tx, 40, 'Supporting text, up to two lines under the headline', 15, FONT.regular, C.ink2, 'LEFT', 'TOP', { opacity: op, lineHeight: { value: 20, units: 'PIXELS' }, ...lines(2) });
    } else {
      text(s, '#list-item/title', tx, 14, textW, 24, 'Living room lights', 17, FONT.medium, C.ink, 'LEFT', 'TOP', { opacity: op, ...lines(1) });
      text(s, '#list-item/subtitle', tx, 40, textW, 20, 'On · 65 %', 14, FONT.regular, C.ink2, 'LEFT', 'TOP', { opacity: op, ...lines(1) });
    }
    if (v.kind === 'image') text(s, '#list-item/trailing-text', trailingX, tall ? 18 : 16, TT_W, 20, '2 h', 13, FONT.regular, C.ink2, 'RIGHT', 'TOP', { opacity: op });
    if (v.kind === 'text') text(s, '#list-item/trailing-text', trailingX, 0, VALUE_W, LI_H, '21.4 °C', 17, FONT.regular, C.ink2, 'RIGHT', 'CENTER', { opacity: op, maxLines: 1, textTruncation: 'ENDING' });
    if (v.kind === 'plain' || v.kind === 'icon') iconSlot('#list-item/trailing', trailingX);
    if (v.kind === 'toggle') instance(s, '#list-item/toggle', LI_W - 16 - 68, (LI_H - 40) / 2, toggle.symbols['state=default'], { opacity: op });
    if (v.kind === 'chevron') chevron(s, 'chevron', LI_W - 28, LI_H / 2, 16, C.ink2, 'right');
  }, { only: liCombo });

  // ---- Slider: as the dimmer on the dashboard: a pill (white 52 %), the track inset 20,
  // the thumb a white disc with the accent line. The variant is the state. The value moves
  // #slider/fill (progress bar meter) and #slider/thumb (progress marker meter); drawn here at
  // 0 % (thumb at x = 0, see #toggle).
  heading(80, 1900, 'Slider');
  const SL_W = 243, SL_H = 56, SL_PAD = 20, TRACK = 8, THUMB = 28, TRACK_W = SL_W - 2 * SL_PAD;
  componentSet(lib, '#slider', 80, 1900, [['state', ['default', 'pressed', 'disabled']]], SL_W, SL_H, 32, 0, (s, v) => {
    const dis = v.state === 'disabled';
    rect(s, 'bg', 0, 0, SL_W, SL_H, C.white, SL_H / 2, { opacity: 0.52 });
    // The pressed halo: a marker under the track with the thumb's travel (above the track it
    // would cover the fill next to the thumb). A marker is moved by its own shape (see the
    // thumb below).
    const HALO = THUMB + 16;
    if (v.state === 'pressed') {
      meter(rect(s, '#slider/halo', 0, (SL_H - HALO) / 2, HALO, HALO, C.accentSoft, HALO / 2), 'progressMarkerData', { startX: SL_PAD - HALO / 2, endX: SL_PAD + TRACK_W - HALO / 2 });
    }
    rect(s, 'track', SL_PAD, (SL_H - TRACK) / 2, TRACK_W, TRACK, C.track, TRACK / 2);
    // the fill runs from the track's start to the thumb's centre
    meter(rect(s, '#slider/fill', SL_PAD, (SL_H - TRACK) / 2, TRACK / 2, TRACK, dis ? C.accentDisabled : C.accent, TRACK / 2),
      'progressBarData', { startX: 0, endX: TRACK_W });
    // The thumb is the marker itself: harry moves a marker's own shape (a rounded rect, as
    // the toggle's thumb), not the children of a frame.
    meter(rect(s, '#slider/thumb', 0, (SL_H - THUMB) / 2, THUMB, THUMB, C.white, THUMB / 2, { ...stroke(dis ? C.accentDisabled : C.accent, 3), effects: shadow(1, 4, 0.15) }),
      'progressMarkerData', { startX: SL_PAD - THUMB / 2, endX: SL_PAD + TRACK_W - THUMB / 2 });
  });

  // ---- Bar chart: a row of 24 bars. Each bar is a vertical progress bar meter (its level,
  // 0..100) with a twin in the accent colour above it, which the code shows by opacity. harry
  // places a vertical bar from the bottom of endY in the parent (meters.rs), so the bars stand
  // at y = 0. What a bar means is the app's.
  heading(2400, 1000, 'Bar chart');
  const CH_W = 304, CH_H = 54, BARS = 24, BAR_GAP = 5, BAR_W = (CH_W - (BARS - 1) * BAR_GAP) / BARS;
  componentSet(lib, '#bar-chart', 2400, 1000, [['bars', [String(BARS)]]], CH_W, CH_H, 0, 0, (s) => {
    for (let i = 0; i < BARS; i++) {
      const x = i * (BAR_W + BAR_GAP);
      meter(rect(s, `#bar-chart/bar/${i + 1}`, x, 0, BAR_W, CH_H, C.line, 3), 'progressBarData', { vertical: true, endY: CH_H });
      meter(rect(s, `#bar-chart/bar/${i + 1}/accent`, x, 0, BAR_W, CH_H, C.accent, 3, { opacity: i >= 14 && i <= 17 ? 1 : 0 }), 'progressBarData', { vertical: true, endY: CH_H });
    }
  });

  // ---- Input field: the variant is the state (default, disabled, error). Focus is not a
  // variant: #input-field/focus is a ring the code fades in (opacity), the caret blinks by
  // opacity. The text hugs its content in an auto-layout row, so the caret follows the text
  // (harry measures text in layout); the row clips a text longer than the field.
  heading(1000, 1900, 'Input field');
  const IF_W = 880, IF_H = 76, IF_TOTAL = 108, PAD = 24;
  componentSet(lib, '#input-field', 1000, 1900, [['state', ['default', 'disabled', 'error']]], IF_W, IF_TOTAL, 0, 24, (s, v) => {
    const dis = v.state === 'disabled', err = v.state === 'error';
    rect(s, 'bg', 0, 0, IF_W, IF_H, dis ? C.surface : C.white, 20, stroke(err ? C.error : C.line, err ? 2 : 1));
    // the ring: accent, red in the error state; shown in the preview of the error state only
    rect(s, '#input-field/focus', 0, 0, IF_W, IF_H, null, 20, { ...stroke(err ? C.error : C.accent, 2), opacity: err ? 1 : 0 });
    text(s, '#input-field/placeholder', PAD, 0, IF_W - 2 * PAD, IF_H, 'Enter password', 25, FONT.regular, C.muted, 'LEFT', 'CENTER', { opacity: v.state === 'error' ? 0 : 1 });
    // a space, not '': Figma may drop an empty text on import, and the app needs the layer
    const row = frame(s, '#input-field/row', PAD, 0, IF_W - 2 * PAD, IF_H, {
      frameMaskDisabled: false, bordersTakeSpace: false, stackMode: 'HORIZONTAL', stackSpacing: 2,
      stackCounterAlignItems: 'CENTER', stackPrimarySizing: 'FIXED', stackLayoutVersion: 1,
    });
    text(row, '#input-field/text', 0, 22, 90, 33, err ? '••••' : ' ', 25, FONT.regular, dis ? C.muted : C.ink, 'LEFT', 'CENTER', { textAutoResize: 'WIDTH_AND_HEIGHT' });
    rect(row, '#input-field/caret', 92, 22, 2, 32, C.accent, 1, { opacity: err ? 1 : 0 });
    text(s, '#input-field/helper', 4, IF_H + 8, IF_W - 8, 22, err ? 'At least 8 characters' : 'Helper text', 15, FONT.regular, err ? C.error : C.ink2);
  }, { rows: true });

  // ---- Keyboard: one variant (the layout); the labels come from the code per mode (abc, ABC,
  // 123, #+=). Each key is a frame #kbd/<id> (the tap target) with a pressed layer the code
  // fades in and the label. In an app it sits in a slot like any component (ui-lib slot.rs).
  heading(80, 2420, 'Keyboard');
  const KBD_W = 1920, KBD_H = 420, KEY_W = 150, KEY_H = 84, GAP = 12, PAD_TOP = 24, WIDE = 228;
  const kbdSet = componentSet(lib, '#keyboard', 80, 2420, [['state', ['default']]], KBD_W, KBD_H, 0, 0, (s) => {
    // Opaque, no shadow on the dock or the keys: the dock moves over sheets, and every
    // translucent layer and blurred shadow is drawn again on each frame of the slide (#1).
    rect(s, 'bg', 0, 0, KBD_W, KBD_H, C.panel, 32);
    const keyNode = (id, x, y, w, label, kind = 'letter') => {
      const fill = kind === 'letter' || kind === 'space' ? C.white : kind === 'done' ? C.accent : C.special;
      const ink = kind === 'done' ? C.white : C.ink;
      const k = frame(s, `#kbd/${id}`, x, y, w, KEY_H);
      rect(k, 'bg', 0, 0, w, KEY_H, fill, 18);
      const down = kind === 'done' ? C.accentPressed : kind === 'special' ? C.line : C.special;
      rect(k, `#kbd/${id}/pressed`, 0, 0, w, KEY_H, down, 18, { opacity: 0 });
      const letter = kind === 'letter';
      text(k, `#kbd/${id}/label`, 0, 0, w, KEY_H, label, letter ? 32 : 21, letter ? FONT.regular : FONT.medium, ink, 'CENTER', 'CENTER');
    };
    const rowY = i => PAD_TOP + i * (KEY_H + GAP);
    const rowX = n => Math.round((KBD_W - (n * KEY_W + (n - 1) * GAP)) / 2);
    [...'qwertyuiop'].forEach((c, i) => keyNode(`r1/${i + 1}`, rowX(10) + i * (KEY_W + GAP), rowY(0), KEY_W, c));
    [...'asdfghjkl'].forEach((c, i) => keyNode(`r2/${i + 1}`, rowX(9) + i * (KEY_W + GAP), rowY(1), KEY_W, c));
    // row 3: Shift, 7 letters, Delete, as wide as row 1
    const row1W = 10 * KEY_W + 9 * GAP, left = rowX(10), midW = 7 * KEY_W + 6 * GAP;
    const sideW = Math.round((row1W - midW - 2 * GAP) / 2);
    keyNode('shift', left, rowY(2), sideW, 'Shift', 'special');
    [...'zxcvbnm'].forEach((c, i) => keyNode(`r3/${i + 1}`, left + sideW + GAP + i * (KEY_W + GAP), rowY(2), KEY_W, c));
    keyNode('delete', left + row1W - sideW, rowY(2), sideW, 'Delete', 'special');
    // row 4: 123, space, Done
    keyNode('mode', left, rowY(3), WIDE, '123', 'special');
    keyNode('space', left + WIDE + GAP, rowY(3), row1W - 2 * (WIDE + GAP), 'space', 'space');
    keyNode('done', left + row1W - WIDE, rowY(3), WIDE, 'Done', 'done');
  });

  // ---- Transitions
  const TY = 2980;
  // one line: wider than heading()'s 800
  text(lib, 'Heading', 80, TY - 40, 1600, 28, 'Transitions (from ui-lib/components.json; harry takes them from the Rust library, not from the Figma prototype)', 21, FONT.semibold, C.ink);
  const col = [80, 440, 760, 920];
  ['Component', 'Change', 'Duration', 'Easing'].forEach((h, i) => text(lib, 'th', col[i], TY, 340, 22, h, 15, FONT.semibold, C.ink2));
  TRANSITIONS.forEach((row, r) => row.forEach((cell, i) => text(lib, 'td', col[i], TY + 30 + r * 28, i === 3 ? 900 : 340, 22, cell, 15, FONT.regular, C.ink)));

  // ---- #ui-gallery on Page 1, next to the dashboard: one instance per component
  const sets = Object.fromEntries(nodes.filter(n => n.isStateGroup && n.name.startsWith('#') && n.parentIndex && n.parentIndex.guid === lib.guid).map(n => [n.name, n]));
  const variant = (set, name) => kids(`${sets[set].guid.sessionID}:${sets[set].guid.localID}`).find(n => n.name === name);
  const gal = frame(page1, '#ui-gallery', 2100, 0, 1920, 1080, { fillPaints: [A.solid(C.page)], frameMaskDisabled: false });
  text(gal, 'Title', 80, 56, 1200, 44, 'Components', 34, FONT.semibold, C.ink);
  const g = (set, vname, name, x, y) => instance(gal, name, x, y, variant(set, vname));
  g('#button', 'kind=primary, state=default, size=regular', '#ui/button-primary', 80, 160);
  g('#button', 'kind=secondary, state=default, size=regular', '#ui/button-secondary', 304, 160);
  g('#button', 'kind=ghost, state=default, size=regular', '#ui/button-ghost', 528, 160);
  g('#button', 'kind=primary, state=default, size=compact', '#ui/button-compact', 752, 160);
  g('#icon-button', 'kind=tonal, state=default, icon=plus, size=regular', '#ui/icon-tonal', 80, 260);
  g('#icon-button', 'kind=glass, state=default, icon=wifi, size=compact', '#ui/icon-glass', 160, 260);
  g('#icon-button', 'kind=scrim, state=default, icon=maximize-2, size=regular', '#ui/icon-scrim', 240, 260);
  g('#toggle', 'state=default', '#ui/toggle-1', 80, 360);
  g('#toggle', 'state=pressed', '#ui/toggle-2', 180, 360);
  g('#radio', 'value=on, state=default', '#ui/radio-1', 80, 440);
  g('#radio', 'value=off, state=default', '#ui/radio-2', 80, 496);
  g('#dropdown', 'state=default', '#ui/dropdown', 80, 600);
  g('#dropdown', 'state=disabled', '#ui/dropdown-disabled', 440, 600);
  g('#slider', 'state=default', '#ui/slider', 80, 880);
  g('#slider', 'state=pressed', '#ui/slider-pressed', 440, 880);
  g('#input-field', 'state=default', '#ui/input', 900, 540);
  g('#input-field', 'state=error', '#ui/input-error', 900, 680);
  ['plain', 'icon', 'toggle', 'chevron'].forEach((kind, i) => g('#list-item', `state=default, kind=${kind}, lines=two, size=regular`, `#ui/list-item-${i + 1}`, 900, 160 + i * 88));
  g('#list-item', 'state=default, kind=image, lines=three, size=regular', '#ui/list-item-image', 1380, 160);
  g('#list-item', 'state=default, kind=subheader, lines=one, size=regular', '#ui/list-item-subheader', 1380, 300);
  g('#list-item', 'state=default, kind=leading-text, lines=two, size=regular', '#ui/list-item-leading-text', 1380, 356);

  ICON_GLYPHS.forEach((name, i) => g('#icon', `icon=${name}`, `#ui/icon-${name}`, 80 + i * 40, 1000));
  g('#scrollbar', 'state=default', '#ui/scrollbar', 1860, 0);

  g('#bar-chart', 'bars=24', '#ui/bar-chart', 960, 900);
  // A variant of a set defined above, by its name ("prop=value, prop=value").
  const variantNamed = (set, name) => {
    const s = nodes.find(n => n.isStateGroup && n.name === set);
    const v = s && kids(key(s.guid)).find(n => n.name === name);
    if (!v) throw new Error(`${set}: no variant ${name}`);
    return v;
  };

  // ---- Tabs (Material 3 "Tabs"): the screen's navigation, as the user's header draws it (Smart
  // HOme.fig, Home status / View selector): a track (white 32 %), the tabs in it (154 x 46, 4
  // apart, 5 inside), the selected tab a white pill (72 %) with its label in the accent, semi
  // bold, the others ink2, regular. Tabs switch screens; a segmented button (#segmented) picks
  // a value. The selected tab is not a variant (a variant change rebuilds the tree): every tab
  // has its pill and two labels and the code lights one by opacity; it sets the labels. Each
  // tab is a touch target (#tabs/tab-<i>). The width follows the number of tabs.
  // in the right column: the Transitions table takes the left one from y 2980 down
  heading(2200, 3000, 'Tabs');
  const TAB_W = 154, TAB_H = 46, TAB_PAD = 5, TAB_GAP = 4, TABS_H = 56, TAB_LABELS = ['Overview', 'Rooms', 'Automations', 'Energy'];
  const tabsWidth = n => 2 * TAB_PAD + n * TAB_W + (n - 1) * TAB_GAP;
  componentSet(lib, '#tabs', 2200, 3000, [['tabs', ['2', '3', '4']], ['state', ['default', 'disabled']]], tabsWidth(4), TABS_H, 24, 16, (s, v) => {
    const n = Number(v.tabs), op = v.state === 'disabled' ? 0.45 : 1, W = tabsWidth(n);
    s.size = { x: W, y: TABS_H };
    rect(s, 'bg', 0, 0, W, TABS_H, C.white, TABS_H / 2, { opacity: 0.32 });
    for (let i = 1; i <= n; i++) {
      const on = i === 1;
      const tab = frame(s, `#tabs/tab-${i}`, TAB_PAD + (i - 1) * (TAB_W + TAB_GAP), TAB_PAD, TAB_W, TAB_H, { opacity: op });
      rect(tab, `#tabs/tab-${i}/selected`, 0, 0, TAB_W, TAB_H, C.white, TAB_H / 2, { opacity: on ? 0.72 : 0 });
      text(tab, `#tabs/tab-${i}/label`, 0, 0, TAB_W, TAB_H, TAB_LABELS[i - 1], 16, FONT.regular, C.ink2, 'CENTER', 'CENTER', { opacity: on ? 0 : 1, maxLines: 1, textTruncation: 'ENDING' });
      text(tab, `#tabs/tab-${i}/label-selected`, 0, 0, TAB_W, TAB_H, TAB_LABELS[i - 1], 16, FONT.semibold, C.accent, 'CENTER', 'CENTER', { opacity: on ? 1 : 0, maxLines: 1, textTruncation: 'ENDING' });
    }
  }, { rows: true });

  // ---- Tile (Material 3 card for one device): the room card's look (white 44 %, radius 32),
  // a leading symbol, the headline and the supporting text, and at the bottom the device's
  // control drawn with the library's own components (like #list-item's toggle): kind toggle
  // (#toggle and the state), slider (#slider and the level), segmented (#segmented, 3), stepper
  // (#icon-button minus and plus around the value), image (a picture area, a camera). The
  // symbol slot holds every #symbol glyph stacked at opacity 0, the code lights one (harry does
  // not swap a variant inside a clone). The whole tile is a touch target: it opens the device.
  // below the list items, whose set grows with their kinds
  const TILE_Y = Math.ceil(liSet.set.transform.m12 + liSet.set.size.y) + 140;
  heading(80, TILE_Y, 'Tile');
  const TILE_W = 438, TILE_H = 248, TP = 24, TILE_KINDS = ['toggle', 'slider', 'segmented', 'stepper', 'image'];
  componentSet(lib, '#tile', 80, TILE_Y, [['kind', TILE_KINDS], ['state', ['default', 'disabled']]], TILE_W, TILE_H, 24, 24, (s, v) => {
    const op = v.state === 'disabled' ? 0.45 : 1, CY = TILE_H - TP - 56;
    rect(s, 'bg', 0, 0, TILE_W, TILE_H, C.white, 32, { opacity: 0.44 });
    rect(s, '#tile/pressed', 0, 0, TILE_W, TILE_H, C.accentSoft, 32, { opacity: 0 });
    const sym = frame(s, '#tile/symbol', TP, TP, 48, 48, { opacity: op });
    SYMBOLS.forEach((icon, i) => instance(sym, `#tile/symbol/${icon}`, 0, 0, variantNamed('#symbol', `icon=${icon}`), { opacity: i === 0 ? 1 : 0 }));
    const lines = n => ({ maxLines: n, textTruncation: 'ENDING' });
    text(s, '#tile/title', TP + 64, TP + 2, TILE_W - 2 * TP - 64, 24, 'Ceiling light', 18, FONT.medium, C.ink, 'LEFT', 'TOP', { opacity: op, ...lines(1) });
    text(s, '#tile/subtitle', TP + 64, TP + 28, TILE_W - 2 * TP - 64, 20, 'On · 65 %', 15, FONT.regular, C.ink2, 'LEFT', 'TOP', { opacity: op, ...lines(1) });
    if (v.kind === 'toggle') {
      text(s, '#tile/value', TP, CY, 200, 56, 'On', 28, FONT.medium, C.ink, 'LEFT', 'CENTER', { opacity: op, ...lines(1) });
      instance(s, '#tile/toggle', TILE_W - TP - 68, CY + 8, variantNamed('#toggle', 'state=default'), { opacity: op });
    }
    if (v.kind === 'slider') {
      const sl = variantNamed('#slider', 'state=default');
      instance(s, '#tile/slider', TP, CY, sl, { opacity: op });
      text(s, '#tile/value', TP + sl.size.x + 16, CY, TILE_W - 2 * TP - sl.size.x - 16, 56, '65 %', 28, FONT.medium, C.ink, 'RIGHT', 'CENTER', { opacity: op, ...lines(1) });
    }
    if (v.kind === 'segmented') instance(s, '#tile/segmented', TP, CY, variantNamed('#segmented', 'segments=3, state=default'), { opacity: op });
    if (v.kind === 'stepper') {
      const minus = variantNamed('#icon-button', 'kind=tonal, state=default, icon=minus, size=regular');
      const plus = variantNamed('#icon-button', 'kind=tonal, state=default, icon=plus, size=regular');
      instance(s, '#tile/minus', TP - 8, CY - 8, minus, { opacity: op });
      text(s, '#tile/value', TP + 72, CY, TILE_W - 2 * TP - 144, 56, '22°', 40, FONT.medium, C.ink, 'CENTER', 'CENTER', { opacity: op, ...lines(1) });
      instance(s, '#tile/plus', TILE_W - TP - 72 + 8, CY - 8, plus, { opacity: op });
    }
    if (v.kind === 'image') {
      // a plain frame inside a rounded clip: harry draws a picture on a plain rectangle only
      const clip = frame(s, 'image-clip', TP, TP + 64, TILE_W - 2 * TP, TILE_H - 2 * TP - 64, { cornerRadius: 20, frameMaskDisabled: false, opacity: op });
      frame(clip, '#tile/image', 0, 0, TILE_W - 2 * TP, TILE_H - 2 * TP - 64, { fillPaints: [A.solid([0x29 / 255, 0x2b / 255, 0x28 / 255])] });
    }
  }, { rows: true });

  // ---- Sheet: the container for anything over the screen (Material 3 dialog or side sheet):
  // layout dialog (1120 wide in the middle, in from the bottom, as high as its content: the
  // card is an auto layout that hugs the content slot, 40 around it; 840 with the placeholder)
  // or a panel along an edge (left, right: half the width; top, bottom: half the height; the
  // card fixed, the content component drawn at that size), state closed (the card 40 px behind its edge,
  // the scrim clear) or open. The app reads a content's height from the .dcf to put a dialog's
  // card in the middle (ui-lib design.rs). #sheet/content is a nested instance of #sheet-content,
  // an empty placeholder: a sheet on the screen swaps it for its content component (Figma's
  // instance swap, sheetInstance() in fig-helpers.js) and renames the parts for itself. The
  // app keeps a sheet in its open variant and moves the card and fades the scrim (ui-lib
  // sheet.rs: the same numbers; change both).
  const SCREEN_W = 1920, SCREEN_H = 1080;
  const SHEET_PAD = 40, SHEET_RADIUS = 32, CLOSED_GAP = 40, DIALOG_W = 1120, DIALOG_H = 840;
  const SHEET_LAYOUTS = ['dialog', 'left', 'right', 'top', 'bottom'];
  const sheetCard = layout => ({
    dialog: { x: (SCREEN_W - DIALOG_W) / 2, y: (SCREEN_H - DIALOG_H) / 2, w: DIALOG_W, h: DIALOG_H },
    left: { x: 0, y: 0, w: SCREEN_W / 2, h: SCREEN_H }, right: { x: SCREEN_W / 2, y: 0, w: SCREEN_W / 2, h: SCREEN_H },
    top: { x: 0, y: 0, w: SCREEN_W, h: SCREEN_H / 2 }, bottom: { x: 0, y: SCREEN_H / 2, w: SCREEN_W, h: SCREEN_H / 2 },
  })[layout];
  const sheetClosed = layout => {
    const c = sheetCard(layout);
    return { dialog: { x: c.x, y: SCREEN_H + CLOSED_GAP }, bottom: { x: c.x, y: SCREEN_H + CLOSED_GAP }, top: { x: c.x, y: -c.h - CLOSED_GAP },
      left: { x: -c.w - CLOSED_GAP, y: c.y }, right: { x: SCREEN_W + CLOSED_GAP, y: c.y } }[layout];
  };
  // the content area of the dialog is the placeholder's size; a panel's content is resized
  const CONTENT_W = DIALOG_W - 2 * SHEET_PAD, CONTENT_H = DIALOG_H - 2 * SHEET_PAD;
  let sheetY = Math.ceil(Math.max(...kids(key(lib.guid)).map(n => n.transform.m12 + n.size.y))) + 80;
  heading(80, sheetY, 'Sheet content (placeholder)');
  const placeholder = componentSet(lib, '#sheet-content', 80, sheetY, [['state', ['default']]], CONTENT_W, CONTENT_H, 0, 0, (s) => { A.hugBox(s, CONTENT_W, CONTENT_H); }, { rows: true });
  sheetY += placeholder.set.size.y + 120;
  heading(80, sheetY, 'Sheet');
  const sheetSet = componentSet(lib, '#sheet', 80, sheetY, [['layout', SHEET_LAYOUTS], ['state', ['closed', 'open']]], SCREEN_W, SCREEN_H, 48, 48, (s, v) => {
    const open = v.state === 'open', dialog = v.layout === 'dialog';
    const c = sheetCard(v.layout), at = open ? c : sheetClosed(v.layout);
    rect(s, '#sheet/scrim', 0, 0, SCREEN_W, SCREEN_H, C.ink, 0, { opacity: open ? 0.35 : 0 });
    // Opaque (a translucent card lets the screen show through the content): the card's own
    // fill, shadow and corners, so its auto layout holds the content alone. A dialog's card
    // hugs the content's height (auto layout, 40 around it); a panel's card is fixed and the
    // content slot is resized to it.
    const card = frame(s, '#sheet/card', at.x, at.y, c.w, c.h, {
      fillPaints: [A.solid(C.white)], cornerRadius: SHEET_RADIUS, effects: shadow(8, 32, 0.06),
      stackMode: 'VERTICAL', stackPrimarySizing: dialog ? 'RESIZE_TO_FIT' : 'FIXED', stackCounterSizing: 'FIXED',
      stackHorizontalPadding: SHEET_PAD, stackVerticalPadding: SHEET_PAD, stackPaddingRight: SHEET_PAD, stackPaddingBottom: SHEET_PAD,
    });
    // the slot: an instance of the hugging placeholder, so a swapped content component sizes
    // it (a panel's content component is drawn at the panel's content size)
    instance(card, '#sheet/content', SHEET_PAD, SHEET_PAD, placeholder.symbols['state=default']);
  }, { cols: 2 });
  // two screens side by side are wider than the Library: widen it
  lib.size = { x: Math.max(lib.size.x, 80 + sheetSet.set.size.x + 80), y: lib.size.y };

  // ---- Keyboard dock: the screen's one keyboard (as a system keyboard, not a part of each
  // sheet), closed below the screen or open at the bottom; #keyboard sits in its slot #osk/keys
  // (osk: on-screen keyboard). The app keeps its instance closed and moves #osk/keys while a
  // field has the focus (core/src/dock.rs); the open variant is the designer's view and the
  // prototype's.
  const dockY = sheetY + sheetSet.set.size.y + 120;
  heading(80, dockY, 'Keyboard dock');
  const dockSet = componentSet(lib, '#keyboard-dock', 80, dockY, [['state', ['closed', 'open']]], SCREEN_W, SCREEN_H, 0, 48, (s, v) => {
    const keys = kbdSet.symbols['state=default'];
    const slot = frame(s, '#osk/keys', 0, v.state === 'open' ? SCREEN_H - keys.size.y : SCREEN_H + 40, keys.size.x, keys.size.y);
    instance(slot, '#osk/keys/preview', 0, 0, keys);
  }, { rows: true });

  // ---- Skeleton: a placeholder while content loads (Material's skeleton loader). Grey shapes
  // where the content will be: a heading bar, four text lines (the last shorter) or a 16:9
  // picture, 440 or 832 wide (a side panel's 880 less 24 a side). The shimmer: three white bands (soft, bright, soft) with one
  // travel, horizontal progress markers at x = 0 (traps.md); the variant clips them, so at
  // meter 0 they wait left of it and nothing shows; over white they are invisible, over the
  // grey shapes they lighten them. The code sweeps them while loading (slot.rs sweep).
  const skY = dockY + dockSet.set.size.y + 120;
  heading(80, skY, 'Skeleton');
  const SK_W = { regular: 440, wide: 832 }, SK_BAND = [[60, 0.35], [80, 0.7], [60, 0.35]];
  const skH = (kind, w) => kind === 'heading' ? 36 : kind === 'text' ? 100 : Math.round(w * 9 / 16);
  componentSet(lib, '#skeleton', 80, skY, [['kind', ['heading', 'text', 'media']], ['size', ['regular', 'wide']]], SK_W.wide, skH('media', SK_W.wide), 48, 48, (s, v) => {
    const w = SK_W[v.size], h = skH(v.kind, w);
    s.size = { x: w, y: h };
    s.frameMaskDisabled = false;
    if (v.kind === 'heading') rect(s, 'bar', 0, 4, Math.round(w * 0.7), 28, C.panel, 6);
    if (v.kind === 'text') [1, 0.96, 0.92, 0.6].forEach((f, i) => rect(s, `line-${i + 1}`, 0, i * 28, Math.round(w * f), 16, C.panel, 6));
    if (v.kind === 'media') rect(s, 'picture', 0, 0, w, h, C.panel, 12);
    const band = SK_BAND.reduce((a, [bw]) => a + bw, 0);
    let off = 0;
    SK_BAND.forEach(([bw, op], i) => {
      meter(rect(s, `#skeleton/shimmer/${i + 1}`, 0, 0, bw, h, C.white, 0, { opacity: op }), 'progressMarkerData', { startX: off - band, endX: w + off });
      off += bw;
    });
  });

  // ---- The sets born in Figma: "figma": true and no place given by the code above. Below
  // the others, each with its heading.
  let drawnY = Math.max(...kids(key(lib.guid)).map(n => n.transform.m12 + n.size.y)) + 120;
  for (const [id, c] of Object.entries(SPEC.components).filter(([, c]) => c.figma)) {
    const made = figmaSet(c.set);
    if (made.placed) continue;
    heading(80, drawnY + 40, `${NAMES[id] || id} (drawn in Figma)`);
    made.set.transform = { ...made.set.transform, m02: 80, m12: drawnY + 40 };
    made.placed = true;
    drawnY += 40 + made.set.size.y + 120;
  }
  for (const name of Object.keys(TWEAKS)) if (!(specOf(name)[1] && specOf(name)[1].figma)) throw new Error(`design/figma-tweaks.js changes ${name}, which is not a set drawn in Figma: a code-drawn set is changed in components-patch.js`);
  for (const name of Object.keys(FIGMA.sets)) if (!specOf(name)[1]) log(`note: ${name} is new in Figma and not in components.json: not in the Library until it is adopted (ui-component skill, "A component drawn in Figma")`);
  if (process.env.LIBRARY_CODE_OUT) fs.writeFileSync(process.env.LIBRARY_CODE_OUT, JSON.stringify(codeHashes, null, 1) + '\n');

  // The Library as tall as what is in it: the sheets of keyboard-patch.js and sheet-patch.js
  // go below it (lib.size.y + 80); a fixed height let #list-item grow into #wifi-sheet.
  const bottom = Math.max(...kids(key(lib.guid)).map(n => n.transform.m12 + n.size.y));
  lib.size = { x: lib.size.x, y: Math.ceil(bottom) + 80 };
  log(`added ${A.count()} nodes: page Components (${nodes.filter(n => n.isStateGroup).length} component sets), #ui-gallery on Page 1; Library ${lib.size.x}x${lib.size.y}`);
  return msg;
};
