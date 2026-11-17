"""Composes the component gallery of the README (docs/readme/components.png) from the Library's
preview SVGs (docs/assets/<set>/<variant>.svg, built by docs/build-docs.js), with the values the
code would set: a toggle on, a slider at 65 %, a dropdown closed, a tile's title and symbol. Every
component with a Figma set is in it, grouped as the README's table; scripts/check.sh fails when a
set is missing here or when the Library changed and the picture was not built again.

From ui-lib's folder:

    node docs/build-docs.js
    python3 docs/readme/gallery.py docs/assets /tmp/gallery.html \
        ../../../packages/services/display_safety/utils/tv-app/data
    google-chrome --headless=new --hide-scrollbars --allow-file-access-from-files \
        --force-device-scale-factor=1.5 --window-size=1600,2600 \
        --screenshot=/tmp/gallery.png file:///tmp/gallery.html
    python3 docs/readme/gallery.py --crop /tmp/gallery.png docs/readme/components.png
"""
import os, re, sys

if sys.argv[1] == '--crop':
    # the page is taller than the gallery: cut the white below it
    from PIL import Image
    img = Image.open(sys.argv[2]).convert('RGB')
    w, h = img.size
    bottom = h
    while bottom > 1 and img.getpixel((w // 2, bottom - 1)) == (255, 255, 255):
        bottom -= 1
    img.crop((0, 0, w, bottom)).save(sys.argv[3], optimize=True)
    sys.exit(0)

A, OUT = sys.argv[1:3]
# the device's font (regular and bold; a medium weight is drawn regular), so that the picture
# does not fall back to the host's system font
FONTS = os.path.abspath(sys.argv[3]) if len(sys.argv) > 3 else None

def load(path):
    return open(os.path.join(A, path)).read()

def on(s):
    s = re.sub(r'(id="#toggle/thumb"[^>]*?)transform="translate\(0 4\)"', r'\1transform="translate(32 4)"', s)
    s = re.sub(r'(id="#slider/fill" )width="4"', r'\1width="132"', s)
    return re.sub(r'(id="#slider/thumb"[^>]*?)transform="translate\(0 14\)"', r'\1transform="translate(138 14)"', s)

def off(s):
    # the preview draws the on-track at full opacity: the code fades it out for an off toggle
    return s.replace('<rect id="#toggle/on" ', '<rect id="#toggle/on" opacity="0" ', 1)

def with_icon(s, icon):
    # the row's leading slot holds every glyph at opacity 0: the code shows one
    return s.replace(f'id="#list-item/leading/{icon}" opacity="0"', f'id="#list-item/leading/{icon}"', 1)

def scrollbar():
    # the thumb in a short piece of its track: the preview's track is a screen high
    s = load('scrollbar/state=default.svg')
    s = s.replace('width="6" height="1080" viewBox="0 0 6 1080"', 'width="6" height="200" viewBox="0 0 6 200"', 1)
    s = s.replace('id="#scrollbar/track" width="6" height="1080"', 'id="#scrollbar/track" width="6" height="200"', 1)
    return s.replace('id="#scrollbar/thumb" ', 'id="#scrollbar/thumb" transform="translate(0 40)" ', 1)

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

def item(svg, name, width=None, cls=''):
    # width: the preview scaled to this width; cls 'screen': a whole screen, on the screen's ground
    style = f' class="w {cls}" style="width:{width}px"' if width else ''
    return f'<figure{style}><div class="p">{svg}</div><figcaption>{name}</figcaption></figure>'

B = 'button/kind={},state=default,size=regular.svg'
IB = 'icon-button/kind={},state=default,icon={},size=regular.svg'
LI = 'list-item/state=default,kind={},lines={},size=regular.svg'
# (group, [items]): the README's groups; Pages, Overlay and Form have no set of their own
GROUPS = [
    ('Actions', [
        item(load(B.format('primary')), 'Button'),
        item(load(B.format('secondary')), 'Button · secondary'),
        item(load(B.format('ghost')), 'Button · ghost'),
        item(load(B.format('primary-success')), 'tone success'),
        item(load(B.format('primary-warning')), 'tone warning'),
        item(load(B.format('primary-error')), 'tone error'),
        item(load(IB.format('glass', 'armchair')) + load(IB.format('tonal', 'bed-double')), 'Icon button'),
    ]),
    ('Selection', [
        item(on(load('toggle/state=default.svg')) + off(load('toggle/state=default.svg')), 'Toggle · on, off'),
        item(load('radio/value=on,state=default.svg') + load('radio/value=off,state=default.svg'), 'Radio'),
        item(load('segmented/segments=3,state=default.svg'), 'Segmented button'),
        item(dropdown_closed(), 'Dropdown'),
        item(on(load('slider/state=default.svg')), 'Slider'),
        item(load('tabs/tabs=3,state=default.svg'), 'Tabs'),
    ]),
    ('Content', [
        item(tile('tile/kind=toggle,state=default.svg', 'Ceiling light', 'Living room', 'house'), 'Tile · toggle', 340),
        item(tile('tile/kind=slider,state=default.svg', 'Dimmer', 'Bedroom', 'bed-double'), 'Tile · slider', 340),
        item(tile('tile/kind=segmented,state=default.svg', 'Fan', 'Office', 'armchair'), 'Tile · segmented', 340),
        item(tile('tile/kind=stepper,state=default.svg', 'Climate', 'Whole house', 'droplets'), 'Tile · stepper', 340),
        item(load(LI.format('chevron', 'two')), 'List item', 440),
        item(on(load(LI.format('toggle', 'one'))), 'List item · toggle', 440),
        item(with_icon(load(LI.format('icon', 'two')), 'settings-2'), 'List item · icon', 440),
        # the image row at the panel's width: its preview texts are cut in a narrower size
        item(load('list-item/state=default,kind=image,lines=three,size=panel.svg'), 'List item · image', 856),
        item(load('bar-chart/bars=24.svg'), 'Bar chart'),
        item(''.join(load(f'symbol/icon={i}.svg') for i in ('house', 'bed-double', 'droplets')), 'Symbol'),
        item(''.join(load(f'icon/icon={i}.svg') for i in ('settings-2', 'sliders-horizontal', 'maximize-2', 'plus', 'minus', 'chevron-right')), 'Icon'),
        item(load('skeleton/kind=text,size=regular.svg'), 'Skeleton', 340, cls='flat'),
        item(scrollbar(), 'Scrollbar'),
    ]),
    ('Over the screen and input', [
        item(load('sheet/layout=dialog,state=open.svg'), 'Sheet · dialog', 460),
        item(load('sheet/layout=right,state=open.svg'), 'Sheet · side', 460),
        item(load('keyboard-dock/state=open.svg'), 'Keyboard dock · the keyboard on the screen', 460, cls='screen'),
        item(load('input-field/state=default.svg'), 'Input field', 440),
        item(load('input-field/state=error.svg'), 'Input field · error', 440),
        item(load('keyboard/state=default.svg'), 'Keyboard · the keys', 900),
    ]),
]
faces = ''.join(
    f"@font-face{{font-family:Inter;font-weight:{w};src:url('file://{FONTS}/inter_{n}.otf')}}"
    for n, w in (('regular', '100 599'), ('bold', '600 900'))) if FONTS else ''
body = ''.join(f'<section><h2>{g}</h2><div class="row">{"".join(items)}</div></section>' for g, items in GROUPS)
page = f'''<!doctype html><html><head><meta charset="utf-8"><style>{faces}
body{{margin:0;font-family:Inter,system-ui,sans-serif;background:#fff}}
.c{{width:1600px;padding:56px 64px 64px;box-sizing:border-box;background:linear-gradient(135deg,#dfe7f3,#eef3f9);display:flex;flex-direction:column;gap:40px}}
h2{{margin:0 0 18px;font-size:15px;font-weight:600;letter-spacing:.12em;text-transform:uppercase;color:#5b6b80}}
.row{{display:flex;flex-wrap:wrap;gap:28px 28px;align-items:flex-end}}
figure{{margin:0;display:flex;flex-direction:column;gap:10px}}
.p{{display:flex;gap:14px;align-items:center}}
figcaption{{font-size:14px;color:#5b6b80}}
svg{{height:auto;max-width:100%;filter:drop-shadow(0 8px 24px rgba(36,54,76,.12))}}
figure.w svg{{width:100%}} figure.flat svg{{filter:none}} figure.screen .p{{background:#9fa9b6;border-radius:6px;overflow:hidden}}
</style></head><body><div class="c">{body}</div></body></html>'''
open(OUT, 'w').write(page)
