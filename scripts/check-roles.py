#!/usr/bin/env python3
"""The library knows no use case: no name of its API says what an app is about.

Checked: the modules of core/src and harry/src (file names), the public items in them (struct,
enum, fn, const, trait, type), the components and their sets in components.json. A name is split
into its words (WeatherIcon, from_wmo, #weather-icon) and each word looked up in USE_CASE_WORDS.
Not checked, on purpose: the glyph names of the icon sets (wifi, house, bed-double are pictures),
the texts of tests and examples.

A new domain an app brings: add its words to USE_CASE_WORDS. KNOWN lists what is in the library
against this rule, each with what replaces it; it is empty and stays empty.

  scripts/check-roles.py     # prints the violations; exit 1 on one that is not in KNOWN
"""
import json, os, re, sys

USE_CASE_WORDS = {
    # weather
    'weather', 'forecast', 'wmo', 'meteo', 'temperature', 'rain', 'precipitation',
    # network
    'wifi', 'network', 'ssid', 'bluetooth',
    # a house and what is in it
    'smarthome', 'room', 'bedroom', 'kitchen', 'hallway', 'climate', 'thermostat', 'intercom', 'doorbell',
    # content
    'news', 'article', 'music', 'song', 'playlist', 'album', 'podcast',
    # a vehicle
    'vehicle', 'parking', 'speedometer',
}

# what breaks the rule today: name -> what replaces it. Remove an entry with its fix.
KNOWN = {
}

def words(name):
    name = re.sub(r'\.rs$', '', os.path.basename(name)) if name.endswith('.rs') else name
    parts = re.split(r'[^A-Za-z0-9]+', name)
    out = []
    for p in parts:
        out += [w.lower() for w in re.findall(r'[A-Z]+(?![a-z])|[A-Z]?[a-z]+|[0-9]+', p)]
    return out

def main():
    lib = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
    names = []   # (name, where)
    for d in ('core/src', 'harry/src'):
        for f in sorted(os.listdir(os.path.join(lib, d))):
            if not f.endswith('.rs'):
                continue
            rel = f'{d}/{f}'
            names.append((rel, 'a module'))
            for i, line in enumerate(open(os.path.join(lib, rel)), 1):
                m = re.match(r'\s*pub (?:struct|enum|fn|const|trait|type) ([A-Za-z0-9_]+)', line)
                if m:
                    names.append((m.group(1), f'{rel}:{i}'))
    spec = json.load(open(os.path.join(lib, 'components.json')))
    for key, c in spec['components'].items():
        names.append((key, 'components.json'))
        for s in [c.get('set')] + c.get('helper_sets', []):
            if s and s.startswith('#'):
                names.append((s, f'components.json, {key}'))
    new, known = [], []
    for name, where in names:
        hit = [w for w in words(name) if w in USE_CASE_WORDS]
        if not hit:
            continue
        (known if name in KNOWN else new).append((name, where, hit))
    for name, where, hit in known:
        print(f'  note {name} ({where}): known, to become {KNOWN[name]}')
    stale = [k for k in KNOWN if k not in {n for n, _, _ in known}]
    for k in stale:
        print(f'  FAIL {k} is fixed: remove it from KNOWN in scripts/check-roles.py')
    for name, where, hit in new:
        print(f'  FAIL {name} ({where}): "{hit[0]}" is a use case. Name the component by its role (AGENTS.md, "The library knows no use case")')
    if not new and not stale:
        print(f'  ok   {len(names)} names of the API: no new use case ({len(known)} known)')
    return 1 if new or stale else 0

sys.exit(main())
