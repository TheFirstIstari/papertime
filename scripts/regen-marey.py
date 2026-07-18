import json, os
from collections import defaultdict

# Load station data
stations_idx = json.load(open('static/station-index.json'))

# Build CRS->TIPLOC map (some stations have CRS != TIPLOC like EUS->EUSTON)
name_to_ids = defaultdict(set)
for s in stations_idx:
    name_to_ids[s.get('name','')].add(s.get('id',''))
    name_to_ids[s.get('name','')].add(s.get('tiploc',''))
crs_tiploc = {}
for s in stations_idx:
    name = s.get('name', '')
    sid = s.get('id', '')
    ids = name_to_ids.get(name, {sid})
    longer = max(ids, key=len) if len(ids) > 1 else sid
    crs_tiploc[sid] = longer

service_dir = 'static/services'
marey_dir = 'static/marey'
os.makedirs(marey_dir, exist_ok=True)

def get_tiploc_variants(crs):
    """Get all possible TIPLOC codes that could match this station in calls."""
    # Most stations: TIPLOC == CRS (identity)
    variants = {crs, crs_tiploc.get(crs, crs)}
    # Also try name-based matching
    for s in stations_idx:
        if s.get('id') == crs:
            name = s.get('name', '')
            for other in name_to_ids.get(name, set()):
                variants.add(other)
            break
    return variants

def normalize_calls(services):
    """Convert old-format (stops) to new-format (calls)."""
    result = []
    for svc in services:
        if 'calls' in svc:
            result.append(svc)
        elif 'stops' in svc:
            new_calls = []
            for stop in svc.get('stops', []):
                crs = stop.get('station', '')
                # Find name from station-index if possible
                name = crs
                for s in stations_idx:
                    if s.get('id') == crs:
                        name = s.get('name', crs)
                        break
                new_calls.append({
                    'crs': crs,
                    'name': name,
                    'arr': stop.get('arr'),
                    'dep': stop.get('dep'),
                })
            result.append({
                'id': svc['id'],
                'operator': svc.get('operator', ''),
                'days': svc.get('days', []),
                'destination_name': svc.get('direction', ''),
                'origin_name': '',
                'calls': new_calls,
            })
    return result

def get_route_key(calls):
    """Generate a route key from calls. Normalize to remove duplicates and TIPLOC variants."""
    # Get sequence of distinct stations
    seen = set()
    route = []
    for c in calls:
        crs = c['crs']
        if crs not in seen:
            route.append(crs)
            seen.add(crs)
    return tuple(route)

def find_best_route(services, focal_variants):
    """Find the route through focal station with best (num_services * num_stations) score."""
    from collections import Counter
    route_counts = Counter()
    route_svcs = defaultdict(list)
    route_station_count = {}

    for svc in services:
        calls = svc.get('calls', [])
        if not calls:
            continue
        # Check if this service passes through focal station
        if not any(c['crs'] in focal_variants for c in calls):
            continue
        key = get_route_key(calls)
        route_counts[key] += 1
        route_svcs[key].append(svc)
        route_station_count[key] = len(key)

    if not route_counts:
        return None, None, []

    # Score: prefer routes with many services AND many stations
    # For terminal stations (focal at end), prefer the longer routes
    best_key = max(route_counts.keys(),
        key=lambda k: route_counts[k] * max(route_station_count[k], 10))

    return best_key, route_svcs[best_key], list(best_key)


def generate_marey(station_id, services, station_name=''):
    unif_svcs = normalize_calls(services)
    has_calls = sum(1 for s in unif_svcs if s.get('calls'))
    if not has_calls:
        return None

    # Get all possible TIPLOC variants for focal station
    focal_variants = get_tiploc_variants(station_id)

    # Find the best route through this station
    route_key, route_services, route_stations = find_best_route(unif_svcs, focal_variants)

    if not route_services:
        return None

    # Build stations array from ordered route stations
    marey_stations = []
    for i, crs in enumerate(route_stations):
        name = None
        for svc in route_services:
            for c in svc.get('calls', []):
                if c['crs'] == crs:
                    name = c.get('name')
                    break
            if name:
                break
        if not name:
            for s in stations_idx:
                if s.get('id') == crs or s.get('tiploc') == crs:
                    name = s.get('name', crs)
                    break
        marey_stations.append({
            'name': name or crs,
            'crs': crs,
            'mileage': i,
            'type': 'minor'
        })

    # Build marey services (only from the chosen route)
    marey_services = []
    for svc in route_services:
        calls = svc.get('calls', [])
        stops = []
        for call in calls:
            if call.get('arr') is not None or call.get('dep') is not None:
                stops.append({
                    'station': call['crs'],
                    'arr': call.get('arr'),
                    'dep': call.get('dep')
                })
        if stops:
            marey_services.append({
                'id': svc['id'],
                'operator': svc['operator'],
                'direction': svc.get('destination_name', ''),
                'days': svc.get('days', []),
                'stops': stops
            })

    if not marey_services:
        return None

    route_name = f"{marey_stations[0]['name']} → {marey_stations[-1]['name']}"

    return {
        'route': route_name,
        'route_id': station_id,
        'stations': marey_stations,
        'services': marey_services
    }

# Regenerate
count = 0
for entry in stations_idx:
    sid = entry['id']
    try:
        services_data = json.load(open(f'static/services/{sid}.json'))
        marey = generate_marey(sid, services_data.get('services', []), services_data.get('name', sid))
        out_path = f'{marey_dir}/{sid}.json'
        if marey:
            json.dump(marey, open(out_path, 'w'))
            count += 1
        elif os.path.exists(out_path):
            os.remove(out_path)
    except:
        pass

print(f'Regenerated {count} marey files')
