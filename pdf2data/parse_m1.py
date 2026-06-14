#!/usr/bin/env python3
"""PaperTime M1 Parser — v3, handling day sections correctly"""
import json, re
from collections import defaultdict
from pathlib import Path

BASE = Path("/Users/frobinson/Documents/ObsidianVault/Programming/SideProjects/PaperTime")
TT = BASE / "raw-text/timetable"
RM = BASE / "raw-text/route-maps"
OUT = BASE / "static/data"
SVC = OUT / "services"

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    SVC.mkdir(parents=True, exist_ok=True)
    
    rms = parse_rms()
    print(f"Route maps: {len(rms)}")
    
    tables, stations = parse_all_tts()
    print(f"Tables: {len(tables)}, Unique stations: {len(stations)}")
    
    # Validate Table 002
    t002 = next((t for t in tables if t["table"] == "002"), None)
    if t002:
        print(f"\nTable 002: {len(t002['services'])} services, {len(t002['stations'])} station entries")
        for svc in t002['services'][:3]:
            print(f"  {svc['id']}: {len(svc['stops'])} stops, op={svc['operator']}")
    
    write_output(tables, rms, stations)
    print(f"\nDone! {len(list(SVC.glob('*.json')))} service files written")

def parse_rms():
    result = []
    for p in sorted(RM.glob("*.txt")):
        txt = p.read_text().replace('\xa0',' ').replace('\u2009',' ')
        m = re.search(r'Table (\d{3})', p.stem)
        tn = m.group(1) if m else ""
        stns = []
        for line in txt.split('\n'):
            s = line.strip()
            if 3 <= len(s) <= 60 and all(c.isalpha() or c.isspace() or c in "&'-" for c in s):
                sl = s.lower()
                if not any(sl.startswith(w) for w in ["legend","version","scale","produced","official","use type","date","miles","system operator","national rail","timetable route","corporate gis","network rail"]):
                    stns.append(s.strip("!(").strip())
        if stns:
            result.append({"table": tn, "region": "Route Map", "stations": stns})
    return result

def parse_all_tts():
    tables = []
    station_map = defaultdict(list)
    
    for p in sorted(TT.glob("*.txt")):
        txt = p.read_text().replace('\xa0',' ').replace('\u2009',' ')
        m = re.search(r'Table (\d{3})', p.stem)
        tn = m.group(1) if m else ""
        
        pages = re.split(r'=== PAGE \d+ ===\n', txt)
        name = ""
        all_ops = []
        all_stations = []
        all_services = []
        all_days = set()
        
        for page in pages[1:]:
            lines = page.split('\n')
            i = 0
            while i < len(lines):
                t = lines[i].strip()
                if t not in ('Mondays to Fridays','Saturdays','Sundays'):
                    i += 1
                    continue
                
                # Found a day section
                dp = {'Mondays to Fridays':'MF','Saturdays':'SAT','Sundays':'SUN'}[t]
                all_days.add(dp)
                i += 1
                while i < len(lines) and not lines[i].strip():
                    i += 1
                
                # Read operator codes for this section
                section_ops = []
                if i < len(lines) and lines[i].strip().startswith('Operator'):
                    i += 1
                    while i < len(lines):
                        cl = lines[i].strip()
                        if not cl or cl in ('Mondays to Fridays','Saturdays','Sundays') or cl.startswith('Operator'):
                            break
                        if re.match(r'^[A-Z]{2,4}$', cl):
                            section_ops.append(cl)
                            i += 1
                        else:
                            break
                
                # Add to global ops list
                for code in section_ops:
                    if code not in [o['code'] for o in all_ops]:
                        all_ops.append({'code':code,'name':code,'color':'#999'})
                
                # Skip metadata
                while i < len(lines) and lines[i].strip().startswith(('Days','1st','Catering')):
                    i += 1
                
                # Parse stations for this day section
                section_stations = []
                base_col = len(all_services)  # Service columns start here
                
                while i < len(lines):
                    trimmed = lines[i].strip()
                    if not trimmed or trimmed in ('Mondays to Fridays','Saturdays','Sundays') or trimmed.startswith('Operator'):
                        break
                    cm = re.search(r'\(([A-Z]{3})\)', trimmed)
                    if not cm:
                        i += 1
                        continue
                    
                    crs = cm.group(1)
                    section_stations.append(crs)
                    all_stations.append(crs)
                    
                    # Read time data
                    time_lines = []
                    after = trimmed[trimmed.find(')')+1:].strip() if ')' in trimmed else ''
                    if after:
                        time_lines.append(after)
                    i += 1
                    while i < len(lines):
                        nl = lines[i].strip()
                        if not nl or nl in ('Mondays to Fridays','Saturdays','Sundays') or nl.startswith('Operator') or re.search(r'\(([A-Z]{3})\)', nl):
                            break
                        time_lines.append(nl)
                        i += 1
                    
                    # Parse times into service columns
                    direction = ''
                    tcol = 0
                    for tl in time_lines:
                        parts = tl.split()
                        if not parts: continue
                        if parts[0] in ('d','a'):
                            direction = parts[0]
                            parts = parts[1:]
                        for p in parts:
                            m2 = re.match(r'^(\d{3,4})$', p)
                            if m2:
                                v = int(m2.group(1))
                                mins = v // 100 * 60 + v % 100
                                col = base_col + tcol
                                while len(all_services) <= col:
                                    op_idx = len(all_services) - base_col
                                    op = section_ops[op_idx] if op_idx < len(section_ops) else ''
                                    all_services.append({'id':f'{dp}_{len(all_services)}','headcode':'','operator':op,'days':[dp],'direction':'','stops':[]})
                                all_services[col]['stops'].append({'station':crs,'arr':mins if 'a' in direction else None,'dep':mins if 'd' in direction else None})
                                tcol += 1
                            elif p == 'T':
                                tcol = 0
                    continue
                # End of day section
            # End of page
        # End of file
        
        td = {"table":tn,"name":name,"period":"","operators":all_ops,"days":sorted(all_days),"stations":all_stations,"services":all_services,"gap":len(all_services)==0 and len(all_stations)==0}
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
        stations.append({"id":crs,"name":crs,"aliases":[],"tables":sorted(set(tabs)),"routes":[],"lat":None,"lng":None,"type":st})
    return tables, stations

def write_output(tables, rms, stations):
    rm_set = {r["table"] for r in rms}
    tidx = []
    for t in tables:
        tidx.append({"table":t["table"],"name":t["name"] or None,"region":None,"operators":[o["code"] for o in t["operators"]],"stations":t["stations"],"n_services":len(t["services"]),"days":t["days"],"file":f"services/{t['table']}.json" if not t.get("gap") and t["services"] else None,"routes":[],"has_route_map":t["table"] in rm_set,"gap":t.get("gap",False)})
    tidx.sort(key=lambda e: e["table"])
    (OUT / "table-index.json").write_text(json.dumps(tidx, indent=2))
    (OUT / "stations.json").write_text(json.dumps(stations, indent=2))
    routes = []; used = set(); rid = 0
    by_region = defaultdict(list)
    for rm in rms: by_region[rm["region"]].append(rm["table"])
    for region, tnums in by_region.items():
        stn_list = []; stn_set = set()
        for tn in sorted(set(tnums)):
            if tn in used: continue
            t = next((t for t in tables if t["table"]==tn), None)
            if t:
                for s in t["stations"]:
                    if s not in stn_set: stn_list.append(s); stn_set.add(s)
                used.add(tn)
        if stn_list:
            routes.append({"id":f"r{rid}","name":region,"region":region,"tables":sorted(set(tnums)),"stations":stn_list,"station_order_source":"route_map"})
            rid += 1
    (OUT / "route-index.json").write_text(json.dumps({"routes":routes}, indent=2))

if __name__ == "__main__":
    main()
