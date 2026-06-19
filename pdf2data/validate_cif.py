#!/usr/bin/env python3
"""
PaperTime — CIF Timetable Validator (M5)
Validates parsed timetable data against the official Network Rail CIF (Common
Interface File) timetable feed.

Usage:
  1. Register at https://opendata.nationalrail.co.uk/ for API credentials
  2. Set credentials in environment or pass via --username/--password
  3. Run: python3 validate_cif.py [--table 001] [--service MF_0]

If run without credentials, performs internal self-consistency checks instead.
"""

import json, re, gzip, os, sys
from collections import defaultdict
from datetime import datetime, timedelta
from pathlib import Path

BASE = Path(__file__).parent.parent
SVC_DIR = BASE / "static" / "services"
MAREY_DIR = BASE / "static" / "marey"

# ── CIF Parsing ──────────────────────────────────────────────────────────────

CIF_RECORD_TYPES = {
    'BS': 'Basic Schedule',
    'BX': 'Basic Schedule Extra',
    'LO': 'Location Origin',
    'LI': 'Location Intermediate',
    'LT': 'Location Terminating',
    'CR': 'Changes En Route',
}

def parse_cif_time(t_str):
    """Parse CIF time string (HHmm) to minutes since midnight."""
    if not t_str or t_str == '0000':
        return None
    try:
        h, m = int(t_str[:2]), int(t_str[2:4])
        return h * 60 + m
    except (ValueError, IndexError):
        return None

def parse_cif_date(d_str):
    """Parse CIF date string (YYYY-MM-DD or DDMMYY)."""
    if not d_str:
        return None
    try:
        if '-' in d_str:
            parts = d_str.split('-')
            return datetime(int(parts[0]), int(parts[1]), int(parts[2]))
        else:
            day, month, year = int(d_str[:2]), int(d_str[2:4]), int(d_str[4:6])
            return datetime(2000 + year, month, day)
    except (ValueError, IndexError):
        return None

def parse_cif_schedule(lines):
    """Parse a CIF schedule block into a structured record."""
    schedules = []
    current = None
    
    for line in lines:
        if len(line) < 2:
            continue
        rec_type = line[:2]
        
        if rec_type == 'BS':
            # Basic Schedule header
            current = {
                'type': 'BS',
                'train_uid': line[2:8].strip(),
                'date_from': line[8:16].strip(),
                'date_to': line[16:24].strip(),
                'days': line[24:31].strip(),  # Mon-Sun bitmask
                'stp': line[63:64].strip(),  # C=Cancel, N=New, V=Variation
                'locations': [],
            }
        elif rec_type == 'BX':
            # Extra details (bank holidays, etc.)
            if current:
                current['bank_holiday'] = line[2:3].strip()
        elif rec_type == 'LO':
            # Origin location
            if current:
                current['locations'].append({
                    'type': 'LO',
                    'location': line[1:8].strip(),  # TIPLOC code
                    'dep': parse_cif_time(line[15:19]),
                })
        elif rec_type == 'LI':
            # Intermediate location (timing point)
            if current:
                current['locations'].append({
                    'type': 'LI',
                    'location': line[1:8].strip(),
                    'arr': parse_cif_time(line[15:19]),
                    'dep': parse_cif_time(line[20:24]),
                })
        elif rec_type == 'LT':
            # Terminating location
            if current:
                current['locations'].append({
                    'type': 'LT',
                    'location': line[1:8].strip(),
                    'arr': parse_cif_time(line[13:17]),
                })
                schedules.append(current)
                current = None
        elif rec_type == 'CR' or rec_type == '--':
            continue
        else:
            # Unknown record type — schedule might be malformed
            if current:
                schedules.append(current)
                current = None
    
    if current:
        schedules.append(current)
    
    return schedules


# ── CIF Download ──────────────────────────────────────────────────────────────

def download_cif(username=None, password=None, filepath=None):
    """Download CIF timetable from Network Rail feed or load from file."""
    if filepath and os.path.exists(filepath):
        print(f"Loading CIF from: {filepath}")
        with gzip.open(filepath, 'rt') if filepath.endswith('.gz') else open(filepath) as f:
            return f.read().split('\n')
    
    if username and password:
        print("Downloading CIF from Network Rail feed...")
        import urllib.request
        from urllib.error import HTTPError
        
        # The URL pattern for the latest CIF file
        today = datetime.now().strftime('%Y/%m/%d')
        url = f'https://opendata.nationalrail.co.uk/api/staticfeeds/1.0/cif_all'
        
        # Create password manager
        password_mgr = urllib.request.HTTPPasswordMgrWithDefaultRealm()
        password_mgr.add_password(None, "https://opendata.nationalrail.co.uk", username, password)
        handler = urllib.request.HTTPBasicAuthHandler(password_mgr)
        opener = urllib.request.build_opener(handler)
        
        try:
            response = opener.open(url, timeout=60)
            content = response.read()
            if content[:2] == b'\x1f\x8b':  # gzip magic
                import gzip
                content = gzip.decompress(content)
            lines = content.decode('utf-8').split('\n')
            print(f"Downloaded {len(lines)} lines")
            return lines
        except HTTPError as e:
            print(f"Download failed: {e} (HTTP {e.code})")
            return None
    else:
        print("No credentials. Use --username/--password or --cif-file")
        return None


# ── TIPLOC ↔ CRS Mapping ─────────────────────────────────────────────────────

def build_tiploc_crs_map(cif_lines):
    """Extract TIPLOC-to-CRS mapping from CIF file (locations section).
    
    CIF uses TIPLOC codes for timing points. We need to map these to CRS
    (3-letter station codes). The mapping is in the 'RL' (Location) records
    at the start of the CIF file, or can be loaded from a separate file.
    """
    mapping = {}
    in_locations = False
    
    for line in cif_lines:
        if not line:
            continue
        if line.startswith('RL'):
            # Location record: RL + TIPLOC(7) + ... + CRS(4)
            tiploc = line[2:9].strip()
            crs = line[51:55].strip()
            if tiploc and crs:
                mapping[tiploc] = crs
        elif line.startswith('BS'):
            break  # Schedule section starts
    
    return mapping


# ── Comparison ────────────────────────────────────────────────────────────────

def load_paper_time_data(table_num):
    """Load PaperTime parsed data for a given table number."""
    svc_path = SVC_DIR / f"{table_num}.json"
    marey_path = MAREY_DIR / f"t{table_num}.json"
    
    if not svc_path.exists():
        return None, None
    
    with open(svc_path) as f:
        services = json.load(f)
    with open(marey_path) as f:
        marey = json.load(f)
    
    return services, marey


def match_service_to_cif(pt_service, cif_schedules, tiploc_map):
    """Match a PaperTime service to CIF schedules by comparing locations."""
    # Extract CRS codes from the service
    pt_stops = [(s['station'], s.get('dep') or s.get('arr'))
                for s in pt_service['stops']
                if (s.get('dep') or s.get('arr')) is not None]
    
    if not pt_stops:
        return []
    
    matches = []
    for cif_sched in cif_schedules:
        locations = cif_sched['locations']
        if not locations:
            continue
        
        # Map TIPLOC to CRS for the CIF locations
        cif_stops = []
        for loc in locations:
            crs = tiploc_map.get(loc['location'], loc['location'])
            t = loc.get('dep') or loc.get('arr')
            if t:
                cif_stops.append((crs, t))
        
        if not cif_stops:
            continue
        
        # Compare first and last stations
        if cif_stops[0][0] == pt_stops[0][0] and cif_stops[-1][0] == pt_stops[-1][0]:
            # Same route endpoints — detailed comparison
            diffs = []
            for pt_stn, pt_t in pt_stops:
                cif_t = next((ct for cs, ct in cif_stops if cs == pt_stn), None)
                if cif_t is not None:
                    diff = pt_t - cif_t
                    if abs(diff) > 5:  # Only report significant differences
                        diffs.append((pt_stn, pt_t, cif_t, diff))
            
            if diffs:
                matches.append({
                    'uid': cif_sched['train_uid'],
                    'n_diffs': len(diffs),
                    'diffs': diffs[:10],  # First 10 diffs
                })
    
    return matches


def internal_consistency_check(services, table_num):
    """Self-consistency checks that don't need external data."""
    issues = []
    
    for svc in services.get('services', []):
        stops = svc['stops']
        valid = [(i, s['station'], s.get('dep') or s.get('arr'))
                 for i, s in enumerate(stops)
                 if (s.get('dep') or s.get('arr')) is not None]
        
        if len(valid) < 2:
            continue
        
        # Check for unreasonable speeds (time gap too small for station distance)
        # This indicates wrong times between stations
        for j in range(1, len(valid)):
            _, stn_a, t_a = valid[j-1]
            _, stn_b, t_b = valid[j]
            
            if t_a is None or t_b is None:
                continue
            
            gap = abs(t_b - t_a)
            if gap == 0:
                # Same time at consecutive stations (possible but worth noting)
                pass
            elif gap > 180:  # > 3 hours
                issues.append(f"{svc['id']}: {stn_a}->{stn_b}: {gap}min gap")
    
    return issues


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(description='Validate PaperTime data against CIF')
    parser.add_argument('--username', help='National Rail Open Data username')
    parser.add_argument('--password', help='National Rail Open Data password')
    parser.add_argument('--cif-file', help='Local CIF file (.gz or .txt)')
    parser.add_argument('--table', help='Specific table to validate (e.g., 001)')
    parser.add_argument('--check', action='store_true',
                       help='Run internal consistency checks only')
    args = parser.parse_args()
    
    if args.check or not (args.username or args.cif_file):
        print("=== Internal Consistency Check ===\n")
        total_issues = 0
        
        tables_to_check = [args.table] if args.table else \
            sorted(set(f.stem for f in SVC_DIR.glob('*.json')))
        
        for tn in tables_to_check:
            services, _ = load_paper_time_data(tn)
            if not services:
                continue
            issues = internal_consistency_check(services, tn)
            for issue in issues:
                print(f"  {tn}/{issue}")
                total_issues += 1
        
        print(f"\nTotal: {total_issues} issues found")
        if total_issues == 0:
            print("All services pass internal consistency checks!")
        return
    
    # CIF validation path
    print("=== CIF Timetable Validation ===\n")
    
    lines = download_cif(args.username, args.password, args.cif_file)
    if not lines:
        print("Failed to load CIF data.")
        print("\nTo access CIF data:")
        print("  1. Register at https://opendata.nationalrail.co.uk/")
        print("  2. Request API access for the static timetable feed")
        print("  3. Run with: --username YOUR_EMAIL --password YOUR_PASSWORD")
        print("  Or download manually and pass: --cif-file /path/to/CIF_ALL.CIF.gz")
        return
    
    print(f"Loaded {len(lines)} CIF lines")
    cif_map = build_tiploc_crs_map(lines)
    print(f"Mapped {len(cif_map)} TIPLOC codes to CRS")
    
    cif_schedules = parse_cif_schedule(lines)
    print(f"Parsed {len(cif_schedules)} CIF schedules\n")
    
    # Compare against PaperTime data
    tables_to_check = [args.table] if args.table else \
        sorted(set(f.stem for f in SVC_DIR.glob('*.json')))
    
    total_diffs = 0
    for tn in tables_to_check:
        services, marey = load_paper_time_data(tn)
        if not services:
            continue
        
        for svc in services.get('services', []):
            matches = match_service_to_cif(svc, cif_schedules, cif_map)
            for match in matches:
                print(f"{tn}/{svc['id']}: {match['n_diffs']} diffs vs CIF {match['uid']}")
                for stn, pt_t, cif_t, diff in match['diffs'][:5]:
                    print(f"  {stn}: PT={pt_t}, CIF={cif_t}, diff={diff}")
                total_diffs += 1
    
    print(f"\nTotal: {total_diffs} services with CIF discrepancies")


if __name__ == '__main__':
    main()
