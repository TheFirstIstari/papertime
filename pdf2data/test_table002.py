#!/usr/bin/env python3
"""Minimal PaperTime parser — test on Table 002"""
import json, re
from pathlib import Path

def parse_table_002():
    txt = Path("/Users/frobinson/Documents/ObsidianVault/Programming/SideProjects/PaperTime/raw-text/timetable/Table 002.txt").read_text()
    txt = txt.replace('\xa0', ' ').replace('\u2009', ' ')
    
    pages = re.split(r'=== PAGE \d+ ===\n', txt)
    
    all_stations = []
    all_services = []
    day_set = set()
    operators = []
    
    for page in pages[1:]:
        lines = page.split('\n')
        i = 0
        while i < len(lines):
            t = lines[i].strip()
            if t in ('Mondays to Fridays', 'Saturdays', 'Sundays'):
                dp = {'Mondays to Fridays': 'MF', 'Saturdays': 'SAT', 'Sundays': 'SUN'}[t]
                day_set.add(dp)
                i += 1
                while i < len(lines) and not lines[i].strip():
                    i += 1
                # Operator codes
                if i < len(lines) and lines[i].strip().startswith('Operator'):
                    i += 1
                    while i < len(lines):
                        cl = lines[i].strip()
                        if not cl or cl in ('Mondays to Fridays', 'Saturdays', 'Sundays') or cl.startswith('Operator'):
                            break
                        if re.match(r'^[A-Z]{2,4}$', cl):
                            if cl not in [o['code'] for o in operators]:
                                operators.append({'code': cl, 'name': cl, 'color': '#999'})
                            i += 1
                        else:
                            break
                # Skip metadata
                while i < len(lines) and lines[i].strip().startswith(('Days', '1st', 'Catering')):
                    i += 1
                # Parse station + time data
                while i < len(lines):
                    trimmed = lines[i].strip()
                    if not trimmed or trimmed in ('Mondays to Fridays', 'Saturdays', 'Sundays') or trimmed.startswith('Operator'):
                        break
                    cm = re.search(r'\(([A-Z]{3})\)', trimmed)
                    if not cm:
                        i += 1
                        continue
                    
                    crs = cm.group(1)
                    all_stations.append(crs)
                    
                    # Get times from this line + continuation lines
                    time_lines = []
                    after = trimmed[trimmed.find(')')+1:].strip() if ')' in trimmed else ''
                    if after:
                        time_lines.append(after)
                    i += 1
                    while i < len(lines):
                        nl = lines[i].strip()
                        if not nl or nl in ('Mondays to Fridays', 'Saturdays', 'Sundays') or nl.startswith('Operator') or re.search(r'\(([A-Z]{3})\)', nl):
                            break
                        time_lines.append(nl)
                        i += 1
                    
                    # Parse time lines: each line may start with 'd' or 'a' direction
                    # Then times follow. Each time = one service column.
                    direction = ''
                    time_col = 0
                    for tl in time_lines:
                        parts = tl.split()
                        if not parts:
                            continue
                        # Check for direction marker
                        if parts[0] in ('d', 'a'):
                            direction = parts[0]
                            parts = parts[1:]
                        for p in parts:
                            m2 = re.match(r'^(\d{3,4})$', p)
                            if m2:
                                v = int(m2.group(1))
                                mins = v // 100 * 60 + v % 100
                                while len(all_services) <= time_col:
                                    op = operators[time_col]['code'] if time_col < len(operators) else ''
                                    all_services.append({
                                        'id': f'{dp}_{time_col}',
                                        'headcode': '',
                                        'operator': op,
                                        'days': [dp],
                                        'direction': '',
                                        'stops': []
                                    })
                                all_services[time_col]['stops'].append({
                                    'station': crs,
                                    'arr': mins if 'a' in direction else None,
                                    'dep': mins if 'd' in direction else None
                                })
                                time_col += 1
                            elif p == 'T':
                                time_col = 0  # Terminus marker resets column
                    continue
            else:
                i += 1
    
    return {
        'table': '002',
        'name': 'Romford to Upminster',
        'period': '',
        'operators': operators,
        'days': sorted(day_set),
        'stations': all_stations,
        'services': all_services,
        'gap': False
    }

if __name__ == '__main__':
    result = parse_table_002()
    print(f"Stations: {len(result['stations'])}")
    print(f"Services: {len(result['services'])}")
    print(f"Days: {result['days']}")
    print(f"Operators: {[o['code'] for o in result['operators']]}")
    
    if result['services']:
        print(f"\nFirst 3 services:")
        for svc in result['services'][:3]:
            print(f"  {svc['id']} op={svc['operator']} stops={len(svc['stops'])}")
            for s in svc['stops'][:5]:
                print(f"    {s['station']}: arr={s['arr']} dep={s['dep']}")
    
    out = Path("/Users/frobinson/Documents/ObsidianVault/Programming/SideProjects/PaperTime/static/data/services")
    out.mkdir(parents=True, exist_ok=True)
    (out / "002.json").write_text(json.dumps(result, indent=2))
