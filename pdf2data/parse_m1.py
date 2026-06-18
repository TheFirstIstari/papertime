#!/usr/bin/env python3
"""PaperTime M1 Parser — v4, with real station names, table names, operator data"""
import json, re
from collections import defaultdict
from pathlib import Path

BASE = Path("/Users/frobinson/Documents/ObsidianVault/Programming/SideProjects/PaperTime")
TT = BASE / "raw-text/timetable"
RM = BASE / "raw-text/route-maps"
OUT = BASE / "static"
SVC = OUT / "services"

# Operator metadata
OP_NAMES = {
    "CC": "CrossCountry", "XC": "CrossCountry", "EM": "East Midlands Railway",
    "LO": "London Overground", "VT": "Avanti West Coast", "GW": "Great Western Railway",
    "TP": "TransPennine Express", "NT": "Northern Trains", "SN": "Southern",
    "SE": "Southeastern", "SW": "South Western Railway", "TL": "Thameslink",
    "GR": "Grand Central", "CS": "Caledonian Sleeper", "HE": "Hull Trains",
    "LD": "Lumo", "ME": "Merseyrail", "AW": "Transport for Wales",
    "SR": "ScotRail", "LE": "Greater Anglia", "HX": "Heathrow Express",
    "IL": "Island Line", "CH": "Chiltern Railways", "LM": "West Midlands Trains",
}
OP_COLORS = {
    "CC": "#009E73", "XC": "#009E73", "SE": "#009E73", "LE": "#009E73",
    "EM": "#CC79A7", "GR": "#CC79A7", "AW": "#CC79A7",
    "LO": "#E86A10", "ME": "#E86A10", "IL": "#E86A10",
    "VT": "#E32636", "HE": "#E32636", "HX": "#E32636",
    "GW": "#56B4E9", "SR": "#56B4E9", "CS": "#56B4E9",
    "TP": "#D55E00", "TL": "#D55E00", "LM": "#D55E00",
    "NT": "#0072B2", "SW": "#0072B2", "CH": "#0072B2", "LD": "#0072B2",
    "SN": "#F0E442",
}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    SVC.mkdir(parents=True, exist_ok=True)

    rms = parse_rms()
    print(f"Route maps: {len(rms)}")

    tables, stations, crs_to_name = parse_all_tts()
    print(f"Tables: {len(tables)}, Unique stations: {len(stations)}")
    print(f"Station names captured: {sum(1 for v in crs_to_name.values() if v != v.upper())} proper names")
    print(f"crs_to_name sample: {dict(list(crs_to_name.items())[:3])}")

    # Update station names
    for s in stations:
        if s["id"] in crs_to_name:
            s["name"] = crs_to_name[s["id"]]
    
    # Verify
    named = sum(1 for s in stations if s["name"] != s["id"])
    print(f"Stations after name update: {named} with real names")

    # Write everything
    write_output(tables, rms, stations)
    print(f"\nDone! {len(list(SVC.glob('*.json')))} service files written")


def parse_rms():
    result = []
    for p in sorted(RM.glob("*.txt")):
        txt = p.read_text().replace('\xa0', ' ').replace('\u2009', ' ')
        m = re.search(r'Table (\d{3})', p.stem)
        tn = m.group(1) if m else ""
        stns = []
        for line in txt.split('\n'):
            s = line.strip()
            if 3 <= len(s) <= 60 and all(c.isalpha() or c.isspace() or c in "&'-" for c in s):
                sl = s.lower()
                if not any(sl.startswith(w) for w in ["legend", "version", "scale", "produced", "official",
                                                       "use type", "date", "miles", "system operator",
                                                       "national rail", "timetable route", "corporate gis",
                                                       "network rail"]):
                    stns.append(s.strip("!(").strip())
        if stns:
            result.append({"table": tn, "region": "Route Map", "stations": stns})
    return result


def clean_name(raw: str) -> str:
    """Clean a station name extracted from the raw text."""
    name = re.sub(r'\s+', ' ', raw).strip()
    # Remove trailing/leading non-alpha cruft
    name = name.strip('.,;:!?()')
    # Remove OCR artifacts: leading/trailing garbage
    name = re.sub(r'^[^A-Za-z0-9]+', '', name)
    name = re.sub(r'[^A-Za-z0-9\s\'\-\.&]+$', '', name)
    # Clean known OCR issues
    name = name.replace(' EL', '')  # Elizabeth line marker
    name = name.replace('\x10', '').replace('\x18', '').replace('\x0f', '').replace('\x1b', '')
    # Apply OCR corrections from table names (subset relevant to station names)
    name = name.replace('*', 'G').replace('3', 'P').replace('<', 'Y')
    name = name.replace('+', 'H').replace(':', 'W')
    name = name.replace('8', 'U').replace('9', 'V')
    name = name.strip()
    return name


def extract_table_name(lines, start_idx):
    """Extract table name from lines after 'TABLE XXX' on page 1, before day sections."""
    # Day section headers
    day_headers = {'Mondays to Fridays', 'Saturdays', 'Sundays'}
    name_parts = []
    for i in range(start_idx, min(start_idx + 10, len(lines))):
        t = lines[i].strip()
        if not t or t in day_headers or t.startswith('Operator') or t.startswith('Page'):
            break
        # Skip lines that are just numbers or metadata
        if re.match(r'^\d+$', t) or t.startswith('Days of operation'):
            continue
        name_parts.append(t)
    # Join, clean up OCR artifacts
    full = ' '.join(name_parts)
    full = clean_table_name(full)
    return full.strip()


# Comprehensive OCR correction for table name text
_OCR_CORRECTIONS = {
    # Special/symbol replacements (applied first)
    '+': 'H', '&': 'C', ':': 'W', '*': 'G',
    '[': '', ']': '', '\u0011': '-super-',
    # Character replacements (applied per-character)
    '8': 'U', '3': 'P', '<': 'Y', '9': 'V', '4': 'Q',
    # Note: ',' and 'N' NOT in per-char repl — handled via _KNOWN_WORD_CORRECTIONS
}

_KNOWN_WORD_CORRECTIONS = {
    '8pminster': 'Upminster', '&entral': 'Central',
    'WestonS\u0011M': 'Weston-super-Mare',
    '+ighbury': 'Highbury', '+uddersfield': 'Huddersfield',
    '+ull': 'Hull', '+astings': 'Hastings', '+eath': 'Heath',
    '+ill': 'Hill', '+orne': 'Horne', '+aywards': 'Haywards',
    '+orsham': 'Horsham', '+elston': 'Helston', '+avant': 'Havant',
    '-unction': 'Junction',
    ',slington': 'Islington', ',lford': 'Ilford',
    ',nternational': 'International', ',nverness': 'Inverness',
    ',pswich': 'Ipswich', ',psZich': 'Ipswich',
    ',lNley': 'Ilkley', ',nverness': 'Inverness',
    '.ings': 'Kings', '.naresborough': 'Knaresborough',
    '.nutsford': 'Knutsford', '.irkcaldy': 'Kirkcaldy',
    '.ilkwinning': 'Kilwinning', '.ilmarnock': 'Kilmarnock',
    '.yle': 'Kyle', '.irby': 'Kirby',
    'StoNeonTrent': 'Stoke-on-Trent',
    'StocNport': 'Stockport', 'PaddocN': 'Paddock',
    'PecNham': 'Peckham', 'MatlocN': 'Matlock',
    'CannocN': 'Cannock', 'MancKester': 'Manchester',
    'GlenrotKes': 'Glenrothes', 'GlenrotNes': 'Glenrothes',
    'CowdenbeatK': 'Cowdenbeath',
    'FaYersham': 'Faversham',
    'MineKead': 'Minehead', 'LoZestoft': 'Lowestoft',
    '3reston': 'Preston', '3eterborough': 'Peterborough',
    '3lymouth': 'Plymouth', '3aisley': 'Paisley',
    '3en]ance': 'Penzance', '3ortsmouth': 'Portsmouth',
    '3otters': 'Potters', '3erth': 'Perth', '3artick': 'Partick',
    'HeathroZ': 'Heathrow', 'WoolZich': 'Woolwich',
    'NorZicK': 'Norwich', 'NorZich': 'Norwich',
    'CreZe': 'Crewe', 'ShreZsbury': 'Shrewsbury',
    'AberystZyth': 'Aberystwyth', 'PZllheli': 'Pwllheli',
    'CraZley': 'Crawley', 'LeZes': 'Lewes',
    'edZay': 'edway', 'atZicN': 'atwick',
    'K': 'K', 'LoZestoft': 'Lowestoft',
    '3eterborougK': 'Peterborough', 'armoutK': 'Yarmouth',
    'SKeringKam': 'Sheringham', 'PitlocKry': 'Pitlochry',
    'EdinburgK': 'Edinburgh', 'PertK': 'Perth',
    '<ork': 'York', '<eovil': 'Yeovil',
    '<armouth': 'Yarmouth', '<orkshire': 'Yorkshire',
    '*reat': 'Great', '*rimsby': 'Grimsby',
    '*uide': 'Guide', '*lossop': 'Glossop',
    '*alashiels': 'Galashiels', '*lasgow': 'Glasgow',
    '*atwick': 'Gatwick', '*reenford': 'Greenford',
    '*reenock': 'Greenock', '*ourock': 'Gourock',
    '*ainsborough': 'Gainsborough', '*unnislaNe': 'Gunnislake',
    '*oole': 'Goole', '*irvan': 'Girvan',
    '*loucester': 'Gloucester', '*rove': 'Grove',
    '&anary': 'Canary', '&ardiff': 'Cardiff',
    '&anterbury': 'Canterbury', '&astleford': 'Castleford',
    '&oatbridge': 'Coatbridge', '&romer': 'Cromer',
    '&rewe': 'Crewe', '&ambridge': 'Cambridge',
    '&roydon': 'Croydon', '&heltenham': 'Cheltenham',
    ':itham': 'Witham', ':illesden': 'Willesden',
    ':alton': 'Walton', ':indsor': 'Windsor',
    ':igan': 'Wigan', ':estbury': 'Westbury',
    ':ood': 'Wood', ':harf': 'Wharf',
    ':ealdstone': 'Wealdstone', ':atford': 'Watford',
    ':est': 'West', ':rexham': 'Wrexham',
    ':eovil': 'Yeovil', ':atcKet': 'Watchet',
    ':imble': 'Wimble', ':eston': 'Weston',
    ':oolwich': 'Woolwich', ':imbledon': 'Wimbledon',
    ':estonsuperMare': 'Weston-super-Mare',
    ':ater': 'Water', ':hifflet': 'Whifflet',
    ':ick': 'Wick', ':orcester': 'Worcester',
    ':orNsop': 'Worksop', ':akefield': 'Wakefield',
    '8cNfield': 'Uckfield', 'Bank 4uay': 'Bank Quay',
    'Ebbw 9ale': 'Ebbw Vale',
    '9ale': 'Vale', '9ictoria': 'Victoria',
    '9alley': 'Valley', '9al': 'Val',
    'Ha]el': 'Hazel',
    'ononSea': '-on-Sea', 'onontheNa': '-on-the-Naze',
    'onHumber': '-on-Humber', 'inFurness': '-in-Furness',
    'LeWillows': 'le-Willows',
    'M1': 'M1', 'M25': 'M25', 'M40': 'M40',
    'StoNeonTrent': 'Stoke-on-Trent',
    'LiYerpool': 'Liverpool',
    'BlacNpool': 'Blackpool',
    '*lasgoZ': 'Glasgow',
    '*uildford': 'Guildford',
    'tKis': 'this', 'otKer': 'other', 'all otKer': 'all other',
    'PenartK': 'Penarth', 'CaerpKilly': 'Caerphilly',
    'RKymney': 'Rhymney', 'TreKerbert': 'Treherbert',
    'MertKyr': 'Merthyr', 'PencN': 'PencK',  # Keep as is
    'edway': 'edway',  # Don't change
}

def clean_table_name(name: str) -> str:
    """Apply OCR corrections to a table name."""
    # First strip control characters
    name = name.replace('\x18', '').replace('\x10', '').replace('\x0f', '').replace('\x1b', '').replace('\t', ' ')
    # Then apply known word corrections
    for wrong, right in _KNOWN_WORD_CORRECTIONS.items():
        name = name.replace(wrong, right)
    # Apply per-character OCR corrections
    chars = []
    for c in name:
        if c in _OCR_CORRECTIONS:
            corrected = _OCR_CORRECTIONS[c]
            if corrected:
                chars.append(corrected)
        else:
            chars.append(c)
    name = ''.join(chars)
    # Collapse multiple spaces
    name = re.sub(r'\s+', ' ', name).strip()
    # Fix common multi-word patterns
    name = name.replace('ononSea', '-on-Sea')
    name = name.replace('onontheNa', '-on-the-Naze')
    return name.strip()


def parse_all_tts():
    tables = []
    station_map = defaultdict(list)
    crs_to_name = {}  # crs -> best station name
    day_headers = {'Mondays to Fridays', 'Saturdays', 'Sundays'}

    for p in sorted(TT.glob("*.txt")):
        txt = p.read_text().replace('\xa0', ' ').replace('\u2009', ' ')
        m = re.search(r'Table (\d{3})', p.stem)
        tn = m.group(1) if m else ""

        pages = re.split(r'=== PAGE \d+ ===\n', txt)

        # Extract table name from page 1 header
        name = ""
        if len(pages) > 1:
            page1_lines = pages[1].split('\n')
            for idx, line in enumerate(page1_lines):
                if line.strip().startswith('TABLE'):
                    name = extract_table_name(page1_lines, idx + 1)
                    break

        all_ops = []
        all_stations = []
        all_services = []
        all_days = set()

        for page in pages[1:]:
            lines = page.split('\n')
            i = 0
            while i < len(lines):
                t = lines[i].strip()
                if t not in day_headers:
                    i += 1
                    continue

                # Found a day section
                dp = {'Mondays to Fridays': 'MF', 'Saturdays': 'SAT', 'Sundays': 'SUN'}[t]
                all_days.add(dp)
                i += 1
                while i < len(lines) and not lines[i].strip():
                    i += 1

                # Read operator codes
                section_ops = []
                if i < len(lines) and lines[i].strip().startswith('Operator'):
                    i += 1
                    while i < len(lines):
                        cl = lines[i].strip()
                        if not cl or cl in day_headers or cl.startswith('Operator'):
                            break
                        if re.match(r'^[A-Z]{2,4}$', cl):
                            section_ops.append(cl)
                            i += 1
                        else:
                            break

                # Add to global ops list with enriched metadata
                for code in section_ops:
                    if code not in [o['code'] for o in all_ops]:
                        all_ops.append({
                            'code': code,
                            'name': OP_NAMES.get(code, code),
                            'color': OP_COLORS.get(code, '#999'),
                        })

                # Skip metadata
                while i < len(lines) and lines[i].strip().startswith(('Days', '1st', 'Catering')):
                    i += 1

                # Parse stations for this day section
                section_stations = []
                base_col = len(all_services)

                while i < len(lines):
                    trimmed = lines[i].strip()
                    if not trimmed or trimmed in day_headers or trimmed.startswith('Operator'):
                        break
                    cm = re.search(r'\(([A-Z]{3})\)', trimmed)
                    if not cm:
                        i += 1
                        continue

                    crs = cm.group(1)

                    # Extract station name
                    name_end = trimmed.find('(')
                    raw_name = trimmed[:name_end].strip() if name_end > 0 else crs
                    stn_name = clean_name(raw_name)

                    # Store best name for this CRS (prefer longer, more descriptive name)
                    if stn_name:
                        # Skip if the 'name' is just the CRS code (all caps, 3-4 chars)
                        is_crs_like = len(stn_name) <= 4 and stn_name == stn_name.upper() and stn_name.isalpha()
                        if not is_crs_like:
                            if crs not in crs_to_name or len(stn_name) > len(crs_to_name.get(crs, '')):
                                crs_to_name[crs] = stn_name

                    section_stations.append(crs)
                    all_stations.append(crs)

                    # Read time data
                    time_lines = []
                    after = trimmed[trimmed.find(')') + 1:].strip() if ')' in trimmed else ''
                    if after:
                        time_lines.append(after)
                    i += 1
                    while i < len(lines):
                        nl = lines[i].strip()
                        if not nl or nl in day_headers or nl.startswith('Operator') or re.search(r'\(([A-Z]{3})\)', nl):
                            break
                        time_lines.append(nl)
                        i += 1

                    # Parse times into service columns
                    direction = ''
                    tcol = 0
                    for tl in time_lines:
                        parts = tl.split()
                        if not parts:
                            continue
                        # Scan ALL parts for direction markers, not just parts[0]
                        # This handles OCR noise like "ö d" where 'd' is the direction
                        # but parts[0] is a garbage character
                        dir_markers = [p for p in parts if p in ('d', 'a')]
                        if dir_markers:
                            direction = dir_markers[0]
                            parts = [p for p in parts if p not in ('d', 'a')]
                        for p in parts:
                            m2 = re.match(r'^(\d{3,4})$', p)
                            if m2:
                                v = int(m2.group(1))
                                mins = v // 100 * 60 + v % 100
                                col = base_col + tcol
                                while len(all_services) <= col:
                                    op_idx = len(all_services) - base_col
                                    op = section_ops[op_idx] if op_idx < len(section_ops) else ''
                                    all_services.append({
                                        'id': f'{dp}_{len(all_services)}',
                                        'headcode': '', 'operator': op,
                                        'days': [dp], 'direction': '', 'stops': [],
                                    })
                                all_services[col]['stops'].append({
                                    'station': crs,
                                    'arr': mins if 'a' in direction else None,
                                    'dep': mins if 'd' in direction else None,
                                })
                                tcol += 1
                            elif p == 'T':
                                tcol = 0
                    continue
                # End of day section
            # End of page
        # End of file

        # Deduplicate stations list (keep order, remove consecutive duplicates)
        # This keeps the station order for the PaperTable component
        seen = set()
        deduped_stations = []
        for crs in all_stations:
            if crs not in seen:
                seen.add(crs)
                deduped_stations.append(crs)
            else:
                # If same station appears later with different context, keep one
                # (handles case where day sections have different station orders)
                pass

        td = {
            "table": tn,
            "name": name,
            "period": "",
            "operators": all_ops,
            "days": sorted(all_days),
            "stations": deduped_stations,  # Use deduped list
            "services": all_services,
            "gap": len(all_services) == 0 and len(all_stations) == 0,
        }
        if not td["gap"] and all_services:
            (SVC / f"{tn}.json").write_text(json.dumps(td, indent=2))
        tables.append(td)

        # Update station map
        for crs in set(all_stations):
            station_map[crs].append(tn)

    # Build station index
    stations = []
    for crs, tabs in sorted(station_map.items()):
        st = "major" if len(tabs) > 5 else "interchange" if len(tabs) > 2 else "minor"
        stations.append({
            "id": crs,
            "name": crs,  # Will be updated with real names from crs_to_name
            "aliases": [],
            "tables": sorted(set(tabs)),
            "routes": [],
            "lat": None,
            "lng": None,
            "type": st,
        })
    return tables, stations, crs_to_name


def write_output(tables, rms, stations):
    rm_set = {r["table"] for r in rms}
    tidx = []
    for t in tables:
        tidx.append({
            "table": t["table"],
            "name": t["name"] or None,
            "region": None,
            "operators": [o["code"] for o in t["operators"]],
            "stations": t["stations"],
            "n_services": len(t["services"]),
            "days": t["days"],
            "file": f"services/{t['table']}.json" if not t.get("gap") and t["services"] else None,
            "routes": [],
            "has_route_map": t["table"] in rm_set,
            "gap": t.get("gap", False),
        })
    tidx.sort(key=lambda e: e["table"])
    # Deduplicate: keep first entry for each table number
    seen_tables = set()
    deduped = []
    for entry in tidx:
        if entry["table"] not in seen_tables:
            seen_tables.add(entry["table"])
            deduped.append(entry)
    tidx = deduped
    (OUT / "table-index.json").write_text(json.dumps(tidx, indent=2))
    (OUT / "stations.json").write_text(json.dumps(stations, indent=2))
    routes = []
    used = set()
    rid = 0
    by_region = defaultdict(list)
    for rm in rms:
        by_region[rm["region"]].append(rm["table"])
    for region, tnums in by_region.items():
        stn_list = []
        stn_set = set()
        for tn in sorted(set(tnums)):
            if tn in used:
                continue
            t = next((t for t in tables if t["table"] == tn), None)
            if t:
                for s in t["stations"]:
                    if s not in stn_set:
                        stn_list.append(s)
                        stn_set.add(s)
                used.add(tn)
        if stn_list:
            routes.append({
                "id": f"r{rid}", "name": region, "region": region,
                "tables": sorted(set(tnums)), "stations": stn_list,
                "station_order_source": "route_map",
            })
            rid += 1
    (OUT / "route-index.json").write_text(json.dumps({"routes": routes}, indent=2))


if __name__ == "__main__":
    main()
