"""Composes the component gallery of the README (docs/readme/components.png) from the Library's
preview SVGs (docs/assets/<set>/<variant>.svg), with the values the code would set: a toggle on,
a slider at 65 %, a dropdown closed, a tile's title, subtitle and symbol. The previews are
see-through glass and are drawn on the light theme's background.

From ui-lib's folder:

    python3 docs/readme/gallery.py docs/assets /tmp/gallery.html
    google-chrome --headless=new --hide-scrollbars --force-device-scale-factor=2 \
        --window-size=1600,520 --screenshot=docs/readme/components.png file:///tmp/gallery.html
"""
import os, re, sys
A, OUT = sys.argv[1:3]

def load(path):
    return open(os.path.join(A, path)).read()

def on(s):
    s = re.sub(r'(id="#toggle/thumb"[^>]*?)transform="translate\(0 4\)"', r'\1transform="translate(32 4)"', s)
    s = re.sub(r'(id="#slider/fill" )width="4"', r'\1width="132"', s)
    return re.sub(r'(id="#slider/thumb"[^>]*?)transform="translate\(0 14\)"', r'\1transform="translate(138 14)"', s)

def tile(path, title, subtitle, icon):
    s = on(load(path))
    s = re.sub(r'(id="#tile/title"[^>]*>)[^<]*', lambda m: m.group(1) + title, s)
    s = re.sub(r'(id="#tile/subtitle"[^>]*>)[^<]*', lambda m: m.group(1) + subtitle, s)
    if icon != 'house':
        s = s.replace('id="#tile/symbol/house"', 'id="#tile/symbol/house" opacity="0"', 1)
        s = s.replace(f'id="#tile/symbol/{icon}" opacity="0"', f'id="#tile/symbol/{icon}"', 1)
    return s

def dropdown_closed():
    s = load('dropdown/state=default.svg')
    s = s.replace('<g id="#dropdown/list"', '<g id="#dropdown/list" opacity="0"', 1)
    # the pill only: the root svg keeps room for the open list below it
    m = re.match(r'<svg([^>]*?)width="(\d+)" height="(\d+)" viewBox="([\d.]+) ([\d.]+) ([\d.]+) ([\d.]+)"', s)
    w = m.group(2)
    return s.replace(m.group(0), f'<svg{m.group(1)}width="{w}" height="56" viewBox="{m.group(4)} {m.group(5)} {w} 56"', 1)

tiles = [tile('tile/kind=toggle,state=default.svg', 'Ceiling light', 'Living room', 'house'),
         tile('tile/kind=slider,state=default.svg', 'Dimmer', 'Bedroom', 'bed-double'),
         tile('tile/kind=segmented,state=default.svg', 'Fan', 'Office', 'armchair'),
         tile('tile/kind=stepper,state=default.svg', 'Climate', 'Whole house', 'droplets')]
row1 = [load('button/kind=primary,state=default,size=regular.svg'), load('segmented/segments=3,state=default.svg'),
        load('tabs/tabs=3,state=default.svg'), on(load('toggle/state=default.svg')), load('radio/value=on,state=default.svg')]
row2 = [('w440', load('list-item/state=default,kind=chevron,lines=two,size=regular.svg')), ('', dropdown_closed()),
        ('', load('icon-button/kind=glass,state=default,icon=armchair,size=regular.svg')), ('', load('symbol/icon=house.svg')),
        ('', load('button/kind=ghost,state=default,size=regular.svg'))]
cell = lambda s, c='': f'<div class="{c}">{s}</div>'
page = f'''<!doctype html><html><head><meta charset="utf-8"><style>
body{{margin:0;font-family:Inter,system-ui,sans-serif}}
.c{{width:1600px;height:520px;padding:64px;box-sizing:border-box;background:linear-gradient(135deg,#dfe7f3,#eef3f9);display:flex;flex-direction:column;gap:36px}}
svg{{display:block;height:auto;filter:drop-shadow(0 8px 24px rgba(36,54,76,.12))}}
.tiles{{display:grid;grid-template-columns:repeat(4,1fr);gap:24px}} .tiles svg{{width:100%}}
.row{{display:flex;gap:24px;align-items:center}} .w440 svg{{width:440px}}
</style></head><body><div class="c">
<div class="tiles">{"".join(cell(t) for t in tiles)}</div>
<div class="row">{"".join(cell(s) for s in row1)}</div>
<div class="row">{"".join(cell(s, c) for c, s in row2)}</div>
</div></body></html>'''
open(OUT, 'w').write(page)
