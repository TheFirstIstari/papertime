#!/usr/bin/env python3
"""Merge NaPTAN station metadata into station-index.json"""
import json, re
import naptan
import pandas as pd

# Build TIPLOC metadata from NaPTAN
stops = naptan.get_all_stops()
rail_stops = stops[stops['StopType'] == 'RLY'].copy()
tiploc_meta = {}
for _, row in rail_stops.iterrows():
    atco = str(row['ATCOCode']) if pd.notna(row['ATCOCode']) else ''
    if atco.startswith('9100') and len(atco) > 4:
        tiploc = atco[4:].strip()
        name = str(row['CommonName']) if pd.notna(row['CommonName']) else ''
        name = re.sub(r'\s+Rail\s+Station$', '', name)
        name = re.sub(r'\s+Station$', '', name)
        lat = float(row['Latitude']) if pd.notna(row.get('Latitude')) else None
        lng = float(row['Longitude']) if pd.notna(row.get('Longitude')) else None
        tiploc_meta[tiploc] = {'name': name, 'lat': lat, 'lng': lng}

print(f"NaPTAN TIPLOC mappings: {len(tiploc_meta)}")

# Load current station-index
with open('static/station-index.json') as f:
    idx = json.load(f)

print(f"Before: {len(idx)} entries")

# Remove numeric IDs (PDF table artifacts)
idx = [e for e in idx if not e.get('id', '').isdigit()]
print(f"After removing numeric: {len(idx)}")

# Apply NaPTAN metadata
updated = 0
for entry in idx:
    tiploc = entry.get('id', '')
    if tiploc in tiploc_meta:
        meta = tiploc_meta[tiploc]
        if meta.get('lat'):
            entry['lat'] = meta['lat']
            entry['lng'] = meta['lng']
        if meta.get('name') and (entry.get('name', '').upper() == tiploc or entry.get('name', '') == tiploc):
            entry['name'] = meta['name']
        updated += 1

print(f"Updated with NaPTAN: {updated}")

# Count results
has_coords = sum(1 for e in idx if e.get('lat'))
proper_names = sum(1 for e in idx if e.get('name', '') not in [e.get('id', ''), e.get('id', '').upper()])
print(f"With coordinates: {has_coords}")
print(f"With proper names: {proper_names}")

# Save
with open('static/station-index.json', 'w') as f:
    json.dump(idx, f, indent=2)
print("Saved!")
