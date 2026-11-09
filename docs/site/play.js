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
