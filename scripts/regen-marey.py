import json, os

# Build marey data from existing services files
stations_idx = json.load(open('static/station-index.json'))
service_dir = 'static/services'
marey_dir = 'static/marey'
os.makedirs(marey_dir, exist_ok=True)

def generate_marey(station_id, services, station_name=''):
    # Skip old-format files (Service with 'stops' not ServiceRef with 'calls')
    if not services or not services[0] or 'calls' not in services[0]:
        return None
    # Build station order + positions from all calls
    station_order = []
    station_positions = {}
    for svc in services:
        for call in svc.get('calls', []):
            crs = call['crs']
            if crs not in station_positions:
                station_positions[crs] = len(station_order)
                station_order.append(crs)

    marey_stations = [
        {'name': next((c['name'] for s in services for c in s.get('calls', []) if c['crs'] == crs), crs),
         'crs': crs, 'mileage': i, 'type': 'minor'}
        for i, crs in enumerate(station_order)
    ]

    marey_services = []
    for svc in services:
        stops = []
        for call in svc.get('calls', []):
            if call.get('arr') is not None or call.get('dep') is not None:
                stops.append({'station': call['crs'], 'arr': call.get('arr'), 'dep': call.get('dep')})
        if stops:
            marey_services.append({
                'id': svc['id'], 'operator': svc['operator'],
                'direction': svc.get('destination_name', ''), 'days': svc.get('days', []),
                'stops': stops
            })

    if not marey_services:
        return None

    return {
        'route': f"{station_name or station_id} services",
        'route_id': station_id,
        'stations': marey_stations,
        'services': marey_services
    }

# Regenerate only stations that have valid format
count = 0
for entry in stations_idx:
    sid = entry['id']
    try:
        services_data = json.load(open(f'{service_dir}/{sid}.json'))
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