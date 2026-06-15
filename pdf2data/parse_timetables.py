#!/usr/bin/env python3
"""PaperTime Timetable Parser — Phase M1"""
import json
import re
from collections import defaultdict
from pathlib import Path

RAW_DIR = Path(__file__).parent.parent / "raw-text"
TT_DIR = RAW_DIR / "timetable"
RM_DIR = RAW_DIR / "route-maps"
OUT_DIR = Path(__file__).parent.parent / "static" / "data"
SERVICES_DIR = OUT_DIR / "services"

CRS_RE = re.compile(r'\(([A-Z]{3})\)')
TABLE_NUM_RE = re.compile(r'Table (\d{3})')
PAGE_SPLIT_RE = re.compile(r'=== PAGE \d+ ===\n')
DAYS_MAP = {'Mondays to Fridays': 'MF', 'Saturdays': 'SAT', 'Sundays': 'SUN'}


def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    SERVICES_DIR.mkdir(parents=True, exist_ok=True)

    print("Phase 1: Route maps...")
    route_maps = parse_route_maps()
    print(f"  {len(route_maps)} route maps")

    print("Phase 2: Timetables...")
    tables = parse_all_timetables()
    n_services = sum(len(t["services"]) for t in tables if not t.get("gap"))
    n_stations = len(set(s for t in tables for s in t["stations"]))
    print(f"  {len(tables)} tables, {n_services} services, {n_stations} unique stations")

    print("Phase 3: Building indexes...")
    stations = build_station_index(tables)
    table_index = build_table_index(tables, route_maps)
    route_index = build_route_index(route_maps, tables, stations)

    print("Phase 4: Writing JSON...")
    write_json(OUT_DIR / "stations.json", stations)
    write_json(OUT_DIR / "table-index.json", table_index)
    write_json(OUT_DIR / "route-index.json", route_index)

    print(f"\nDone! Output: {OUT_DIR}")

    # Quick validation
    print(f"\nValidation:")
    print(f"  stations.json: {len(stations)} entries")
    print(f"  table-index.json: {len(table_index)} entries")
    print(f"  route-index.json: {len(route_index['routes'])} routes")
    print(f"  services/*.json: {len(list(SERVICES_DIR.glob('*.json')))} files")


def parse_route_maps():
    route_maps = []
    for path in sorted(RM_DIR.glob("*.txt")):
        text = path.read_text().replace('\xa0', ' ').replace('\u2009', ' ')
        m = TABLE_NUM_RE.search(path.stem)
        table_num = m.group(1) if m else ""
        stations = []
        for line in text.split("\n"):
            s = line.strip()
            if not s or len(s) < 3 or len(s) > 60:
                continue
            if not all(c.isalpha() or c.isspace() or c in "&'-" for c in s):
                continue
            lower = s.lower()
            if any(lower.startswith(w) for w in [
                "legend", "version", "scale", "produced", "official",
                "use type", "date", "miles", "system operator", "national rail",
                "timetable route", "corporate gis", "network rail",
            ]):
                continue
            cleaned = s.strip("!(").strip()
            if len(cleaned) >= 3:
                stations.append(cleaned)
        if stations:
            route_maps.append({"table": table_num, "region": "Route Map", "stations": stations})
    return route_maps


def parse_all_timetables():
    tables = []
    for path in sorted(TT_DIR.glob("*.txt")):
        text = path.read_text().replace('\xa0', ' ').replace('\u2009', ' ')
        m = TABLE_NUM_RE.search(path.stem)
        table_num = m.group(1) if m else ""

        pages = PAGE_SPLIT_RE.split(text)
        name = ""
        operators = []
        all_stations = []
        station_set = set()
        all_services = []
        day_set = set()

        for page_text in pages[1:]:  # skip text before first PAGE marker
            lines = page_text.split('\n')
            i = 0
            while i < len(lines):
                # Find day-period header
                t = lines[i].strip()
                if t in DAYS_MAP:
                    dp = DAYS_MAP[t]
                    day_set.add(dp)
                    i += 1
                    # Skip blanks
                    while i < len(lines) and not lines[i].strip():
                        i += 1
                    # Read operator line + codes
                    if i < len(lines) and lines[i].strip().startswith('Operator'):
                        i += 1
                        while i < len(lines):
                            code_line = lines[i].strip()
                            if not code_line or code_line in DAYS_MAP or code_line.startswith('Operator'):
                                break
                            if re.match(r'^[A-Z]{2,4}$', code_line) and code_line not in [o['code'] for o in operators]:
                                operators.append({"code": code_line, "name": OP_NAMES.get(code_line, "Unknown"), "color": OP_COLORS.get(code_line, "#999")})
                                i += 1
                            else:
                                break
                    # Skip metadata rows
                    while i < len(lines) and lines[i].strip().startswith(('Days of operation', '1st Class', 'Catering')):
                        i += 1
                    # Parse station rows
                    while i < len(lines):
                        trimmed = lines[i].strip()
                        if not trimmed or trimmed in DAYS_MAP or trimmed.startswith('Operator'):
                            break
                        m = CRS_RE.search(trimmed)
                        if m:
                            crs = m.group(1)
                            name_end = trimmed.find("(")
                            stn_name = trimmed[:name_end].strip() if name_end > 0 else ""
                            if stn_name:
                                all_stations.append(crs)
                                station_set.add(crs)
                                after = trimmed[trimmed.find(")") + 1:] if ")" in trimmed else ""
                                direction, times = parse_times(after)
                                for col, mins in enumerate(times):
                                    if mins is None:
                                        continue
                                    if col >= len(all_services):
                                        op_code = operators[col]['code'] if col < len(operators) else ""
                                        all_services.append({"id": f"{dp}_{col}", "headcode": "", "operator": op_code, "days": [dp], "direction": "", "stops": []})
                                    if col < len(all_services):
                                        all_services[col]["stops"].append({
                                            "station": crs,
                                            "arr": mins if "a" in direction else None,
                                            "dep": mins if "d" in direction else None,
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
                                        if mins is not None and col < len(all_services):
                                            all_services[col]["stops"].append({
                                                "station": crs,
                                                "arr": mins if "a" in direction else None,
                                                "dep": mins if "d" in direction else None,
                                            })
                                    i += 1
                                continue
                        i += 1
                else:
                    i += 1

        table_data = {
            "table": table_num,
            "name": name,
            "period": "",
            "operators": operators,
            "days": sorted(day_set),
            "stations": all_stations,
            "services": all_services,
            "gap": len(all_services) == 0 and len(all_stations) == 0,
        }
        if not table_data["gap"] and all_services:
            write_json(SERVICES_DIR / f"{table_num}.json", table_data)
        tables.append(table_data)
    return tables


def parse_times(line):
    t = line.strip()
    if not t:
        return "", []
    s = t.lstrip("".join(chr(c) for c in range(0x80, 0x10FFFF)))
    parts = s.split()
    if not parts:
        return "", []
    direction = ""
    start = 0
    if parts[0] in ("d", "a"):
        direction = parts[0]
        start = 1
    times = []
    for p in parts[start:]:
        m = re.match(r'^(\d{3,4})$', p)
        if m:
            v = int(m.group(1))
            times.append(v // 100 * 60 + v % 100)
        elif p in ("—", "-", ".."):
            times.append(None)
    return direction, times


def build_station_index(tables):
    stn_tabs = defaultdict(list)
    for t in tables:
        if t.get("gap"):
            continue
        for crs in t["stations"]:
            if t["table"] not in stn_tabs[crs]:
                stn_tabs[crs].append(t["table"])
    stations = []
    for crs, tabs in sorted(stn_tabs.items()):
        stype = "major" if len(tabs) > 5 else "interchange" if len(tabs) > 2 else "minor"
        stations.append({"id": crs, "name": crs, "aliases": [], "tables": sorted(tabs), "routes": [], "lat": None, "lng": None, "type": stype})
    return stations


def build_table_index(tables, route_maps):
    rm_set = {r["table"] for r in route_maps}
    entries = []
    for t in tables:
        entries.append({
            "table": t["table"], "name": t["name"] or None, "region": None,
            "operators": [o["code"] for o in t["operators"]], "stations": t["stations"],
            "n_services": len(t["services"]), "days": t["days"],
            "file": f"services/{t['table']}.json" if not t.get("gap") and t["services"] else None,
            "routes": [], "has_route_map": t["table"] in rm_set, "gap": t.get("gap", False),
        })
    entries.sort(key=lambda e: e["table"])
    return entries


def build_route_index(route_maps, tables, stations):
    routes = []
    used = set()
    rid = 0
    by_region = defaultdict(list)
    for rm in route_maps:
        by_region[rm["region"]].append(rm["table"])
    for region, tnums in by_region.items():
        stn_list = []
        for tn in sorted(set(tnums)):
            if tn in used:
                continue
            t = next((t for t in tables if t["table"] == tn), None)
            if t:
                for s in t["stations"]:
                    if s not in stn_list:
                        stn_list.append(s)
                used.add(tn)
        if stn_list:
            routes.append({"id": f"r{rid}", "name": region, "region": region, "tables": sorted(set(tnums)), "stations": stn_list, "station_order_source": "route_map"})
            rid += 1
    # Unmapped tables by Jaccard
    unmapped = [t for t in tables if t["table"] not in used and not t.get("gap")]
    if unmapped:
        tsets = {t["table"]: set(t["stations"]) for t in unmapped}
        grouped = [False] * len(unmapped)
        for i in range(len(unmapped)):
            if grouped[i]:
                continue
            grp = [unmapped[i]["table"]]
            grouped[i] = True
            si = tsets[unmapped[i]["table"]]
            for j in range(i + 1, len(unmapped)):
                if grouped[j]:
                    continue
                sj = tsets[unmapped[j]["table"]]
                inter, union = len(si & sj), len(si | sj)
                if union > 0 and inter / union > 0.5 and len(si) > 2:
                    grp.append(unmapped[j]["table"])
                    grouped[j] = True
            if grp:
                routes.append({"id": f"r{rid}", "name": f"Route {rid}", "region": "Derived", "tables": sorted(grp), "stations": [], "station_order_source": "inferred"})
                rid += 1
    return {"routes": routes}


OP_NAMES = {"CC":"CrossCountry","XC":"CrossCountry","EM":"East Midlands Railway","LO":"London Overground","VT":"Avanti West Coast","GW":"Great Western Railway","TP":"TransPennine Express","NT":"Northern Trains","SN":"Southern","SE":"Southeastern","SW":"South Western Railway","TL":"Thameslink","GR":"Grand Central","CS":"Caledonian Sleeper","HE":"Hull Trains","LD":"Lumo","ME":"Merseyrail","AW":"Transport for Wales","SR":"ScotRail","LE":"Greater Anglia","HX":"Heathrow Express","IL":"Island Line","CH":"Chiltern Railways","LM":"West Midlands Trains"}
OP_COLORS = {"CC":"#009E73","XC":"#009E73","SE":"#009E73","LE":"#009E73","EM":"#CC79A7","GR":"#CC79A7","AW":"#CC79A7","LO":"#E86A10","ME":"#E86A10","IL":"#E86A10","VT":"#E32636","HE":"#E32636","HX":"#E32636","GW":"#56B4E9","SR":"#56B4E9","CS":"#56B4E9","TP":"#D55E00","TL":"#D55E00","LM":"#D55E00","NT":"#0072B2","SW":"#0072B2","CH":"#0072B2","LD":"#0072B2","SN":"#F0E442"}


def write_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w") as f:
        json.dump(data, f, indent=2, default=str)


if __name__ == "__main__":
    main()
