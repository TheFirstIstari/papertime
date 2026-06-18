#!/usr/bin/env python3
"""
PaperTime M3: Marey Chart Computation
Reads parsed timetable data, computes Marey chart coordinates per table.
Output: static/data/marey/{table-num}.json
"""
import json, math, re
from collections import defaultdict
from pathlib import Path

BASE = Path(__file__).parent.parent
STATIC = BASE / "static"
OUT = STATIC / "marey"
STATIONS_FILE = STATIC / "stations.json"
TABLES_FILE = STATIC / "table-index.json"
SERVICES_DIR = STATIC / "services"

def haversine_miles(lat1, lon1, lat2, lon2):
    R = 3958.8
    phi1, phi2 = math.radians(lat1), math.radians(lat2)
    dphi = math.radians(lat2 - lat1)
    dlam = math.radians(lon2 - lon1)
    a = math.sin(dphi/2)**2 + math.cos(phi1)*math.cos(phi2)*math.sin(dlam/2)**2
    return R * 2 * math.atan2(math.sqrt(a), math.sqrt(1-a))

STATION_COORDS = {
    "EUS": (51.5282, -0.1337), "WAT": (51.6635, -0.3037), "WFJ": (51.6786, -0.3506),
    "MKC": (52.0340, -0.7220), "CRE": (53.0887, -2.3958), "MAN": (53.4773, -2.2307),
    "LIV": (53.4076, -2.9779), "BHM": (52.4778, -1.9083), "COV": (52.4000, -1.5800),
    "BAS": (51.3780, -2.3605), "BRI": (51.4490, -2.5790), "EXD": (50.7260, -3.5200),
    "TAU": (51.0210, -3.1030), "PLY": (50.3770, -4.1430), "TRU": (50.4100, -5.0400),
    "PNZ": (50.1210, -5.5320), "SWA": (51.6170, -3.9430), "CDF": (51.4758, -3.1792),
    "NWP": (51.5880, -2.9970), "OXF": (51.7530, -1.2700), "RDG": (51.4580, -0.9710),
    "PBO": (52.2060, 0.1240), "CAM": (52.2050, 0.1440), "BIM": (52.9020, -1.1400),
    "DBY": (52.9150, -1.4640), "LEI": (52.6300, -1.1250), "NOT": (52.9470, -1.1460),
    "SHE": (53.3800, -1.4700), "DON": (53.5220, -1.1400), "YRK": (53.9580, -1.0920),
    "LDS": (53.7950, -1.5480), "NCL": (54.9780, -1.6180), "DAR": (54.5210, -1.5510),
    "EDI": (55.9520, -3.1880), "GLA": (55.8580, -4.2580), "HUL": (53.7440, -0.3420),
    "MIA": (51.2710, -0.5110), "WOK": (51.3820, -0.8140), "HAV": (50.8430, 0.1040),
    "BEX": (51.4400, 0.1080), "POT": (51.5910, -0.3350), "SLO": (51.5150, -0.1280),
    "BHI": (52.4810, -1.8490), "STA": (52.8040, -2.1200), "SHR": (52.7120, -2.7530),
    "CNG": (51.4810, -2.5830), "GLC": (55.8620, -4.2510), "ABD": (57.1430, -2.0980),
    "INV": (57.4800, -4.2250), "CAR": (54.8900, -2.9300), "PRE": (53.7590, -2.7000),
    "LAN": (53.8300, -2.7000), "NUN": (52.6260, -1.1960), "TAM": (53.0000, -1.6700),
    "BUX": (53.2620, -1.9100), "MAT": (53.1380, -1.4100), "COT": (52.9090, -1.4700),
    "RMF": (51.5830, 0.2500), "UPM": (51.5590, 0.2500), "EMP": (51.7380, -0.0180),
}

def get_coords(crs):
    return STATION_COORDS.get(crs)

def estimate_mileages(station_order):
    known = [(i, crs, get_coords(crs)) for i, crs in enumerate(station_order) if get_coords(crs)]
    if len(known) < 2:
        mileages = {crs: i * 5.0 for i, crs in enumerate(station_order)}
    else:
        mileages = {}
        prev_mileage = 0.0
        prev_idx, _, prev_coords = known[0]
        mileages[station_order[prev_idx]] = 0.0
        for i in range(1, len(known)):
            idx, crs, coords = known[i]
            dist = haversine_miles(prev_coords[0], prev_coords[1], coords[0], coords[1])
            gap = idx - prev_idx
            if gap > 0:
                step = dist / gap
                for j in range(1, gap):
                    gi = prev_idx + j
                    if gi < len(station_order):
                        mileages[station_order[gi]] = prev_mileage + step * j
            prev_mileage += dist
            mileages[crs] = prev_mileage
            prev_idx, prev_coords = idx, coords
        for crs in station_order:
            if crs not in mileages:
                mileages[crs] = 0.0
    # Enforce minimum vertical gap for chart readability
    # Prevents station labels overlapping when stations are very close
    MIN_GAP = 1.0  # miles
    adjusted = {}
    prev_m = None
    for crs in station_order:
        m = mileages.get(crs, 0)
        if prev_m is not None and m - prev_m < MIN_GAP:
            m = prev_m + MIN_GAP
        adjusted[crs] = round(m, 1)
        prev_m = m
    return adjusted


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    
    stations = json.load(open(STATIONS_FILE))
    tables = json.load(open(TABLES_FILE))
    station_lookup = {s["id"]: s for s in stations}
    
    written = 0
    for t in tables:
        if t.get("gap"):
            continue
        table_num = t["table"]
        stations_ordered = t["stations"]
        
        # Load services
        svc_file = SERVICES_DIR / f"{table_num}.json"
        if not svc_file.exists():
            continue
        table_data = json.load(open(svc_file))
        services = table_data.get("services", [])
        if not services:
            continue
        
        # Get unique station order (deduplicate while preserving order)
        seen = set()
        unique_stations = []
        for crs in stations_ordered:
            if crs not in seen:
                seen.add(crs)
                unique_stations.append(crs)
        
        mileages = estimate_mileages(unique_stations)
        
        marey_data = {
            "table": table_num,
            "route": f"Route {table_num}",
            "route_id": f"t{table_num}",
            "stations": [
                {"name": station_lookup.get(crs, {}).get("name", crs), "crs": crs, "mileage": round(mileages.get(crs, 0), 1),
                 "type": station_lookup.get(crs, {}).get("type", "minor")}
                for crs in unique_stations
            ],
            "services": services
        }
        
        with open(OUT / f"t{table_num}.json", "w") as f:
            json.dump(marey_data, f, indent=2)
        written += 1
    
    print(f"Wrote {written} Marey JSON files")
    
    # Show examples
    for f in sorted(OUT.glob("*.json"))[:5]:
        d = json.load(open(f))
        print(f"  {d['route_id']}: {len(d['stations'])} stations, {len(d['services'])} services")


if __name__ == "__main__":
    main()
