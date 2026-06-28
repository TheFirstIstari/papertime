#!/usr/bin/env python3
"""
generate_naptan_cache.py — Download NaPTAN data and create a TIPLOC → metadata cache.

Run this once to create static/naptan-cache.json, which build-index.js uses.
The naptan pip package downloads ~400k UK stop points. We filter to railway stations
and map ATCO codes (9100 + TIPLOC) to station names and coordinates.
"""

import json
import re
import sys

try:
    import naptan
    import pandas as pd
except ImportError:
    print("Installing naptan...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "naptan", "pandas"])
    import naptan
    import pandas as pd

print("Downloading NaPTAN data...")
stops = naptan.get_all_stops()
print(f"Total stops: {len(stops)}")

# Filter to railway stations
rail = stops[stops['StopType'] == 'RLY'].copy()
print(f"Railway stations: {len(rail)}")

# Build TIPLOC → metadata mapping
cache = {}
for _, row in rail.iterrows():
    atco = str(row['ATCOCode']) if pd.notna(row['ATCOCode']) else ''
    
    # ATCO codes for rail: 9100 + TIPLOC code
    if atco.startswith('9100') and len(atco) > 4:
        tiploc = atco[4:].strip()
        name = str(row['CommonName']) if pd.notna(row['CommonName']) else ''
        name = re.sub(r'\s+Rail\s+Station$', '', name)
        name = re.sub(r'\s+Station$', '', name)
        
        lat = float(row['Latitude']) if pd.notna(row.get('Latitude')) else None
        lng = float(row['Longitude']) if pd.notna(row.get('Longitude')) else None
        
        # Determine station type
        station_type = 'minor'
        name_lower = name.lower()
        if any(w in name_lower for w in ['international', 'airport']):
            station_type = 'airport'
        elif any(w in name_lower for w in ['central', 'junction', 'cross', 'square', 'gardens',
                                           'euston', 'kings cross', 'paddington', 'waterloo',
                                           'victoria', 'liverpool street', 'fenchurch', 'moorgate',
                                           'stratford', 'canary wharf', 'marylebone']):
            station_type = 'terminal'
        elif '&' in name or ' and ' in name_lower:
            station_type = 'interchange'
        elif any(w in name_lower for w in ['main', 'central', 'high street', 'road']):
            station_type = 'major'
        
        if tiploc and name:
            cache[tiploc] = {
                'name': name,
                'lat': lat,
                'lng': lng,
                'type': station_type
            }

print(f"Cached {len(cache)} TIPLOC entries")

# Save
output_path = 'static/naptan-cache.json'
with open(output_path, 'w') as f:
    json.dump(cache, f, indent=2)

print(f"Saved to {output_path}")

# Stats
with_coords = sum(1 for v in cache.values() if v.get('lat'))
by_type = {}
for v in cache.values():
    by_type[v.get('type', 'minor')] = by_type.get(v.get('type', 'minor'), 0) + 1
print(f"  With coordinates: {with_coords}")
print(f"  By type: {by_type}")
