#!/usr/bin/env python3
"""Re-parse and merge multi-part tables (029, 073, 152, 153) from raw text.

The parser processes files alphabetically, so 'Table 073a.txt' overwrites 'Table 073.txt'.
This script parses both files independently and merges the results.

Usage: python3 pdf2data/merge_a_tables.py
"""

import json, re, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from parse_timetables import (
    CRS_RE, TABLE_NUM_RE, PAGE_SPLIT_RE, DAYS_MAP,
    parse_times, OP_NAMES, OP_COLORS
)

TT_DIR = Path(__file__).parent.parent / "raw-text" / "timetable"
SERVICES_DIR = Path(__file__).parent.parent / "static" / "services"


def parse_file(path: Path) -> dict:
    """Parse a single raw text file, returning partial table data."""
    text = path.read_text().replace('\xa0', ' ').replace('\u2009', ' ')
    pages = PAGE_SPLIT_RE.split(text)

    operators = []
    all_stations = []
    station_set = set()
    all_services = []
    day_set = set()

    for page_text in pages[1:]:
        lines = page_text.split('\n')
        i = 0
        while i < len(lines):
            t = lines[i].strip()
            if t in DAYS_MAP:
                dp = DAYS_MAP[t]
                day_set.add(dp)
                i += 1
                while i < len(lines) and not lines[i].strip():
                    i += 1
                # Read operator codes
                if i < len(lines) and lines[i].strip().startswith('Operator'):
                    i += 1
                    while i < len(lines):
                        code_line = lines[i].strip()
                        if not code_line or code_line in DAYS_MAP or code_line.startswith('Operator'):
                            break
                        if code_line.startswith(('Days of operation', '1st Class', 'Catering', 'NOTES')):
                            break
                        if '(' in code_line and CRS_RE.search(code_line):
                            break
                        if re.match(r'^[A-Z]{2,4}$', code_line) and code_line not in [o['code'] for o in operators]:
                            operators.append({"code": code_line, "name": OP_NAMES.get(code_line, "Unknown"), "color": OP_COLORS.get(code_line, "#999")})
                            i += 1
                        else:
                            break
                # Skip metadata section headers
                while i < len(lines) and lines[i].strip().startswith(('Days of operation', '1st Class', 'Catering')):
                    i += 1
                # Skip data lines under those headers (1Y, ], S, etc.)
                while i < len(lines):
                    sl = lines[i].strip()
                    if not sl:
                        break
                    if sl in DAYS_MAP or sl.startswith('Operator') or CRS_RE.search(sl):
                        break
                    # Check if it's a metadata data-line (short non-time values)
                    if re.match(r'^[A-Z0-9*]{1,3}$', sl) and not re.match(r'^\d{4}$', sl):
                        i += 1
                    else:
                        break
                # Parse station rows
                while i < len(lines):
                    trimmed = lines[i].strip()
                    if not trimmed or trimmed in DAYS_MAP or trimmed.startswith('Operator'):
                        break
                    m = CRS_RE.search(trimmed)
                    if m:
                        crs = m.group(1)
                        stn_name = trimmed[:trimmed.find('(')].strip() if '(' in trimmed else ''
                        if stn_name and crs not in station_set:
                            all_stations.append(crs)
                            station_set.add(crs)
                        after = trimmed[trimmed.find(')')+1:] if ')' in trimmed else ''
                        direction, times = parse_times(after)
                        for col, mins in enumerate(times):
                            if mins is None:
                                continue
                            if col >= len(all_services):
                                op_code = operators[col]['code'] if col < len(operators) else ''
                                all_services.append({'id': f'{dp}_{col}', 'headcode': '', 'operator': op_code, 'days': [dp], 'direction': '', 'stops': []})
                            if col < len(all_services):
                                all_services[col]['stops'].append({
                                    'station': crs,
                                    'arr': mins if 'a' in direction else None,
                                    'dep': mins if 'd' in direction else None,
                                })
                        # Continuation lines
                        i += 1
                        while i < len(lines):
                            nl = lines[i].strip()
                            if not nl or nl in DAYS_MAP or nl.startswith('Operator') or CRS_RE.search(nl):
                                break
                            _, ct = parse_times(nl)
                            if not ct:
                                break
                            for col, mins in enumerate(ct):
                                if mins is None:
                                    continue
                                if col >= len(all_services):
                                    op_code = operators[col]['code'] if col < len(operators) else ''
                                    all_services.append({'id': f'{dp}_{col}', 'headcode': '', 'operator': op_code, 'days': [dp], 'direction': '', 'stops': []})
                                if col < len(all_services):
                                    all_services[col]['stops'].append({
                                        'station': crs,
                                        'arr': mins if 'a' in direction else None,
                                        'dep': mins if 'd' in direction else None,
                                    })
                            i += 1
                        continue
                    i += 1
            else:
                i += 1

    return {
        'stations': all_stations,
        'services': all_services,
        'days': sorted(day_set),
        'operators': operators,
    }


def merge(base: dict, extra: dict) -> dict:
    """Merge extra into base (mutates base)."""
    # Merge stations (preserve order, deduplicate)
    seen = set(base['stations'])
    for s in extra['stations']:
        if s not in seen:
            base['stations'].append(s)
            seen.add(s)
    # Merge services
    base['services'].extend(extra['services'])
    # Merge days
    base['days'] = sorted(set(base['days']) | set(extra['days']))
    # Merge operators
    seen_ops = {o['code'] for o in base['operators']}
    for o in extra['operators']:
        if o['code'] not in seen_ops:
            base['operators'].append(o)
            seen_ops.add(o['code'])
    return base


def main():
    affected = {
        '029': ('Table 029.txt', 'Table 029a.txt'),
        '073': ('Table 073.txt', 'Table 073a.txt'),
        '152': ('Table 152.txt', 'Table 152a.txt'),
        '153': ('Table 153.txt', 'Table 153a.txt'),
    }

    for tn, (main_name, a_name) in affected.items():
        print(f'\n=== Table {tn} ===')
        main_path = TT_DIR / main_name
        a_path = TT_DIR / a_name

        if not main_path.exists():
            print(f'  Main file missing: {main_path}')
            continue

        # Parse main file
        main_data = parse_file(main_path)
        print(f'  Main: {len(main_data["services"])} services, {len(main_data["stations"])} stations, days={main_data["days"]}')

        result = {
            'table': tn,
            'name': '',
            'period': '',
            'operators': main_data['operators'],
            'days': main_data['days'],
            'stations': main_data['stations'],
            'services': main_data['services'],
            'gap': len(main_data['services']) == 0 and len(main_data['stations']) == 0,
        }

        # Parse and merge 'a' file if it exists
        if a_path.exists():
            a_data = parse_file(a_path)
            print(f'  "a" file: {len(a_data["services"])} services, {len(a_data["stations"])} stations, days={a_data["days"]}')
            result = merge(result, a_data)
        else:
            print(f'  No "a" file')

        # Re-number service IDs
        for i, s in enumerate(result['services']):
            prefix = s['days'][0] if s['days'] else 'XX'
            s['id'] = f'{prefix}_{i}'

        # Write
        out_file = SERVICES_DIR / f'{tn}.json'
        with open(out_file, 'w') as f:
            json.dump(result, f, indent=2)
        print(f'  Written: {len(result["services"])} services, {len(result["stations"])} stations')


if __name__ == '__main__':
    main()
