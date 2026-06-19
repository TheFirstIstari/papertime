# Station-Centric Timetable Architecture — Research & Best Practices

## 1. Core Concept

Instead of grouping services by National Rail "timetable tables" (an artifact of the PDF format),
we index services by station. This is a fundamental simplification:

**Before (table-centric):**
- Services grouped by arbitrary table numbers (001, 002, etc.)
- Tables are a PDF publishing concept, not a data concept
- User must know origin + destination to find the right table
- 191 tables to manage, some with corrupted data

**After (station-centric):**
- Services indexed by the stations they pass through
- Each station has a complete list of all services calling there
- User picks any station → sees all services → filters as needed
- ~2,500 stations, each with a manageable service list

## 2. Data Structure

### Station Service File (`services/{crs}.json`)
```json
{
  "station": "EUS",
  "name": "London Euston",
  "services": [
    {
      "id": "1A01",
      "operator": "VT",
      "origin": {"crs": "EUS", "name": "London Euston", "dep": 373},
      "destination": {"crs": "GLC", "name": "Glasgow Central", "arr": 582},
      "calls": [
        {"crs": "EUS", "dep": 373},
        {"crs": "WFJ", "arr": 390, "dep": 391},
        {"crs": "MKC", "arr": 412, "dep": 413},
        {"crs": "CRE", "arr": 437, "dep": 439},
        {"crs": "GLC", "arr": 582}
      ],
      "days": ["MF"],
      "direction": "northbound"
    }
  ]
}
```

### Station Index (`station-index.json`)
```json
{
  "stations": [
    {
      "id": "EUS",
      "name": "London Euston",
      "crs": "EUS",
      "tiploc": "EUSTON",
      "lat": 51.5286,
      "lng": -0.1337,
      "type": "terminal",
      "n_services": 1250,
      "operators": ["VT", "LM", "XC"],
      "destinations": ["GLC", "MAN", "BHM", "LIV", "EDI"],
      "file": "services/EUS.json"
    }
  ]
}
```

### Marey Data per Station (`marey/{crs}.json`)
```json
{
  "station": "EUS",
  "name": "London Euston",
  "services": [
    {
      "id": "1A01",
      "operator": "VT",
      "direction": "northbound",
      "departure": 373,
      "destination": "GLC",
      "calls": [
        {"crs": "WFJ", "arr": 390, "dep": 391},
        {"crs": "MKC", "arr": 412, "dep": 413}
      ]
    }
  ]
}
```

## 3. Data Flow

```
Darwin S3 Feed (ZIP)
  ↓
Extract XML schedules
  ↓
Parse each schedule → Service { id, operator, origin, destination, calls[], days }
  ↓
For each service, for each calling station:
  Add service to station's service list
  Update station metadata (operators, destinations, count)
  ↓
Output:
  services/{crs}.json     — per-station service lists
  station-index.json      — station metadata
  marey/{crs}.json        — per-station Marey data
```

## 4. Optimization Strategies

### 4.1 Service Deduplication
- Darwin may send multiple schedule updates for the same service
- Use `rid` (RTTI unique ID) as the dedup key
- Only keep the most recent version of each schedule

### 4.2 Station Service Limits
- Major stations (Euston, Victoria) may have 1000+ services per day
- Consider splitting by time period (morning/afternoon/evening) for very large stations
- Or use pagination in the frontend

### 4.3 Incremental Updates
- Darwin S3 feed updates daily
- Only re-process changed schedules (compare `rid` + timestamp)
- Station files only need updating if their services changed

### 4.4 Data Size Estimates
- ~69,000 services nationally
- Average station: ~50-100 services
- Major station: ~500-1000 services
- Total JSON: ~8-12 MB (similar to current)
- Per-station files: ~10-50 KB each

## 5. Frontend Implications

### 5.1 Search Flow
1. User types station name → Fuse.js autocomplete on `station-index.json`
2. Select station → load `services/{crs}.json`
3. Display services in timetable format
4. Filter by: operator, destination, time of day, direction

### 5.2 Marey Chart per Station
- Each station gets its own Marey chart
- Y-axis: time of day
- X-axis: distance from station (or just sequential stops)
- Each service = a line showing when it arrives/departs
- Much simpler than route-based Marey charts

### 5.3 Service Pattern Diagrams
- Per-station pattern diagrams show all services through that station
- Branch detection: where do services diverge?
- Simpler than route-based patterns since we only care about one station

## 6. Comparison with Other Projects

### National Rail Enquiries (nationalrail.co.uk)
- Station-centric: pick a station → see departures/arrivals
- Uses Darwin data directly
- Our approach mirrors this but with full timetable (not just live)

### Realtime Trains (realtimetrains.co.uk)
- Station-centric with historical data
- Shows all services through a station with full calling patterns
- Uses Darwin data via RTTI API
- Good reference for UI/UX

### OpenRailData community
- Most projects use station-centric views
- Darwin data is naturally station-centric (schedules reference stations)
- Table grouping is a PDF artifact, not a data model

## 7. Migration Path

1. **Phase 1**: Build station-centric indexing in `darwin2data`
2. **Phase 2**: Generate `services/{crs}.json` and `station-index.json`
3. **Phase 3**: Update frontend to use station-centric data
4. **Phase 4**: Remove table-centric code and data files
5. **Phase 5**: Deprecate PDF pipeline entirely

## 8. Open Questions

1. **Service filtering**: Should we pre-filter services by day (MF/SAT/SUN) or include all and filter client-side?
2. **Time range**: Should station files include a full day or be split by time period?
3. **Historical data**: Do we keep previous timetable versions or always show current?
4. **Portion handling**: Some trains run in portions (split/join). How do we display these?
