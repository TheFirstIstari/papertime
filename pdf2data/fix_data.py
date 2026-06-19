#!/usr/bin/env python3
"""PaperTime Data Fixer — post-processing for known data quality issues.

Usage: python3 pdf2data/fix_data.py

Fixes:
1. Station names: extracts proper names from raw text (was using CRS codes as names)
2. Operator codes: best-effort recovery for OCR-corrupted operator lines
3. Service-level operator assignment for tables with missing data
"""
import json, re, os
from collections import defaultdict
from pathlib import Path

BASE = Path(__file__).parent.parent
SERVICES_DIR = BASE / "static" / "services"
STATIONS_FILE = BASE / "static" / "stations.json"
RAW_DIR = BASE / "raw-text" / "timetable"
MAREY_DIR = BASE / "static" / "marey"

KNOWN_OPS = {
    'CC': 'c2c', 'XC': 'CrossCountry', 'EM': 'East Midlands Railway',
    'LO': 'London Overground', 'VT': 'Avanti West Coast', 'GW': 'Great Western Railway',
    'TP': 'TransPennine Express', 'NT': 'Northern Trains', 'SN': 'Southern',
    'SE': 'Southeastern', 'SW': 'South Western Railway', 'TL': 'Thameslink',
    'GR': 'Grand Central', 'CS': 'Caledonian Sleeper', 'HE': 'Hull Trains',
    'LD': 'Lumo', 'ME': 'Merseyrail', 'AW': 'Transport for Wales', 'SR': 'ScotRail',
    'LE': 'Greater Anglia', 'HX': 'Heathrow Express', 'IL': 'Island Line',
    'CH': 'Chiltern Railways', 'LM': 'West Midlands Trains', 'GN': 'Govia Thameslink',
    'GC': 'Grand Central', 'GX': 'Gatwick Express', 'LF': 'London Northwestern',
    'XR': 'Elizabeth Line', 'HT': 'Grand Central', 'TW': 'Transport for Wales',
    'NY': 'Northern Trains',
}

# Known operator assignments for tables where OCR corrupts the operator line
# Verified via route knowledge (which operator runs services on each route)
TABLE_OPERATORS = {
    '003': 'LO', '010': 'LO', '011': 'SR',
    '016': 'EM', '017': 'EM',
    '024': 'NT', '026': 'NT',
    '038': 'NT', '039': 'NT',
    '044': 'NT', '061': 'NT',
    '065': 'NT',
    '072': 'SR', '075': 'SR',
    '077': 'SR', '085': 'SR',
    '090': 'SR', '114': 'SR',
    '115': 'NT',
    '122': 'NT', '124': 'NT',
    '127': 'SW', '128': 'SW',
    '130': 'SW', '131': 'SW',
    '135': 'SW', '138': 'SW',
    '141': 'AW', '145': 'AW',
    '150': 'NT', '153': 'NT',
    '158': 'SR', '160': 'SR',
    '163': 'SR', '173': 'SR',
    '181': 'AW', '186': 'AW',
    '190': 'NT', '200': 'NT',
    '201': 'NT', '202': 'NT',
    '203': 'NT', '204': 'NT',
    '206': 'NT', '207': 'NT',
    '210': 'NT',
    # Remaining OCR-corrupted tables
    '093': 'NT',
    '209': 'SR', '212': 'SR', '213': 'SR',
    '215': 'SR', '216': 'SR', '217': 'SR',
    '218': 'SR', '219': 'SR',
}


def extract_station_names():
    """Extract station names from raw timetable text files.

    Returns: dict mapping CRS -> most common name found
    """
    crs_names = defaultdict(list)
    crs_re = re.compile(r'\(([A-Z]{3})\)')
    page_split = re.compile(r'=== PAGE \d+ ===')

    for path in sorted(RAW_DIR.glob("*.txt")):
        text = path.read_text().replace('\xa0', ' ')
        pages = page_split.split(text)

        for page in pages[1:]:
            lines = page.split('\n')
            for line in lines:
                # Match "Name (CRS)" pattern
                m = crs_re.search(line)
                if m:
                    crs = m.group(1)
                    # Extract everything before the paren as the name
                    name = line[:m.start()].strip()
                    # Clean up OCR artifacts
                    name = re.sub(r'[\u2000-\u200f\u2028-\u202f\u205f-\u206f]', ' ', name)
                    name = re.sub(r'\s+', ' ', name).strip()
                    if name and len(name) >= 2 and not name.startswith(('Operator', 'Days', '1st', 'Catering', 'NOTES')):
                        crs_names[crs].append(name)

    # For each CRS, pick the most common name
    result = {}
    for crs, names in crs_names.items():
        # Filter out names that are just the CRS code
        real_names = [n for n in names if n != crs and len(n) > 3]
        if real_names:
            # Pick most common
            from collections import Counter
            result[crs] = Counter(real_names).most_common(1)[0][0]
        else:
            result[crs] = crs

    return result


def fix_operators(station_names):
    """Fix operator codes in service files.

    For tables with corrupted operator lines, assigns the known operator.
    Also checks raw text for any parseable operator codes.
    """
    fixed_tables = 0
    fixed_services = 0

    for path in sorted(SERVICES_DIR.glob("*.json")):
        tn = path.stem
        with open(path) as f:
            data = json.load(f)

        services = data.get('services', [])
        if not services:
            continue

        # Check if any service has an operator
        has_any_op = any(s.get('operator') for s in services)

        # Determine operator to use
        target_op = None

        # First: check raw text for parseable operator
        raw_path = RAW_DIR / f"Table {tn}.txt"
        if raw_path.exists():
            text = raw_path.read_text()
            for line in text.split('\n'):
                if 'Operator' in line and not line.strip().startswith('NOTES'):
                    parts = line.strip().split()
                    for p in parts[1:]:
                        # Check for known operator codes in the raw text
                        # Handle OCR confusions: /2->LO, *1->EM, etc.
                        clean = p.strip('*/:.$%#@')
                        if clean in KNOWN_OPS:
                            target_op = clean
                            break
                    if target_op:
                        break

        # Second: if raw text didn't yield a parseable code, use table override
        if not target_op and tn in TABLE_OPERATORS:
            target_op = TABLE_OPERATORS[tn]

        # Apply the operator
        if target_op:
            for svc in services:
                if not svc.get('operator'):
                    svc['operator'] = target_op
                    fixed_services += 1

        if fixed_services > 0:
            with open(path, 'w') as f:
                json.dump(data, f, indent=2)
            fixed_tables += 1

    return fixed_tables, fixed_services


def main():
    print("PaperTime Data Fixer")
    print("=" * 50)

    # 1. Extract station names
    print("\n1. Extracting station names from raw text...")
    station_names = extract_station_names()
    print(f"   Found names for {len(station_names)} CRS codes")

    # Load current stations
    with open(STATIONS_FILE) as f:
        stations = json.load(f)

    # Update station names
    renamed = 0
    for s in stations:
        crs = s['id']
        if crs in station_names and station_names[crs] != crs:
            old_name = s['name']
            new_name = station_names[crs]
            if old_name != new_name:
                s['name'] = new_name
                renamed += 1

    if renamed:
        with open(STATIONS_FILE, 'w') as f:
            json.dump(stations, f, indent=2)
        print(f"   Updated {renamed} station names in stations.json")

    # Sample
    print("\n   Sample names:")
    for s in stations[:10]:
        print(f"     {s['id']}: \"{s['name']}\"")

    # 2. Fix operators
    print("\n2. Fixing operator codes...")
    fixed_tables, fixed_services = fix_operators(station_names)
    print(f"   Fixed {fixed_services} services across {fixed_tables} tables")

    # 3. Verify results
    print("\n3. Verification:")
    total_svcs = 0
    total_with_op = 0
    for path in sorted(SERVICES_DIR.glob("*.json")):
        with open(path) as f:
            data = json.load(f)
        for svc in data.get('services', []):
            total_svcs += 1
            if svc.get('operator'):
                total_with_op += 1
    print(f"   Services: {total_with_op}/{total_svcs} have operators ({total_with_op*100//total_svcs}%)")

    # 4. Report remaining tables with operator gaps
    print("\n4. Tables still missing operators:")
    for path in sorted(SERVICES_DIR.glob("*.json")):
        with open(path) as f:
            data = json.load(f)
        svcs = data.get('services', [])
        empty = sum(1 for s in svcs if not s.get('operator'))
        if empty > 0:
            print(f"   Table {path.stem}: {empty}/{len(svcs)} services missing operator")
    print("\nDone!")


if __name__ == '__main__':
    main()
