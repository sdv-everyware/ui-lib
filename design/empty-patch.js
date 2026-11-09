// empty-patch.js: the base of the Library's .fig, made once from the designer's export
// (Smart HOme.fig): the document and its pages, with nothing on them but the icon sources. The
// Library (components-patch.js) draws its sets on this, so no app's screen is in it.
//
//   build-library.sh --base     # library-base.fig from smarthome-app's design/source/Smart HOme.fig
//                               # (this patch, then the export's images and thumbnail dropped)
//
// The icon sources: components-patch.js copies the designer's icons (a frame named after the
// icon with a vector inside) into #icon, #icon-button and #symbol. Each such frame is kept, the
// first of its name (the one the generator finds), moved out of the dashboard onto Page 1 in a
// row above the Library's place; everything else of the drawing is gone.
module.exports = function (msg, { key }) {
  const nodes = msg.nodeChanges;
  // an icon is named as in the icon set the design uses ("chevron-down"); a frame of the
  // dashboard with a vector in it ("Front door") is not one
  const ICON = /^[a-z0-9]+(-[a-z0-9]+)*$/;
  const kidsOf = new Map();
  for (const n of nodes) if (n.parentIndex) {
    const p = key(n.parentIndex.guid);
    if (!kidsOf.has(p)) kidsOf.set(p, []);
    kidsOf.get(p).push(n);
  }
  const kids = n => kidsOf.get(key(n.guid)) || [];
  const page1 = nodes.find(n => n.type === 'CANVAS' && n.name === 'Page 1');
  if (!page1) throw new Error('no Page 1');
  const keep = new Set(nodes.filter(n => n.type === 'DOCUMENT' || n.type === 'CANVAS').map(n => key(n.guid)));
  const subtree = n => { keep.add(key(n.guid)); kids(n).forEach(subtree); };
  const seen = new Set();
  let i = 0;
  for (const n of nodes) {
    if (n.type !== 'FRAME' || !ICON.test(n.name) || seen.has(n.name) || !kids(n).some(c => c.type === 'VECTOR')) continue;
    seen.add(n.name);
    subtree(n);
    n.parentIndex = { guid: page1.guid, position: String.fromCharCode(0x21 + i) };
    n.transform = { ...n.transform, m02: i * 48, m12: -200 };
    i++;
  }
  if (i > 93) throw new Error(`${i} icons: more than the positions used here`);
  const before = nodes.length;
  msg.nodeChanges = nodes.filter(n => keep.has(key(n.guid)));
  // the vectors' geometry is in msg.blobs, referred to by index (vectorNetworkBlob,
  // commandsBlob, ...): keep the blobs the kept nodes use, renumbered
  const used = new Map();
  const remap = o => {
    if (Array.isArray(o)) return o.forEach(remap);
    if (!o || typeof o !== 'object') return;
    for (const [k, v] of Object.entries(o)) {
      if (/Blob$/.test(k) && typeof v === 'number') {
        if (!used.has(v)) used.set(v, used.size);
        o[k] = used.get(v);
      } else remap(v);
    }
  };
  remap(msg.nodeChanges);
  const blobs = msg.blobs || [];
  msg.blobs = [...used.keys()].map(i => blobs[i]);
  console.log(`  empty: ${msg.nodeChanges.length} of ${before} nodes kept, ${msg.blobs.length} of ${blobs.length} blobs, ${i} icon sources: ${[...seen].join(', ')}`);
  return msg;
};
