# PaperTime — Specification

> May 2026 National Rail timetable explorer. Station-centric architecture. Rust data pipeline consuming Darwin Timetable Feed (S3). SvelteKit SSG frontend. Render Static Site.
>
> Domain: `papertime.tweak.wiki`

---

## 1. Overview

**PaperTime** serves the May 2026 National Rail timetable data in three distinct views:

1. **Station Timetable View** — pick a station, see all services passing through it in a classic timetable format
2. **iBRY (Marey) Traffic Flow Graphs** — time–distance diagrams per station showing service patterns
3. **Service Pattern Diagrams** — schematic diagrams showing how services diverge from a station

**Key architectural decisions:**
- **Data source: Darwin Timetable Feed via S3** — official GB rail industry timetable data. No PDF parsing. No OCR. Authoritative source.
- **Station-centric data model** — services indexed by station, not by arbitrary "table" groupings. Simpler, more flexible.
- **Build pipeline: Pure Rust** — single binary (`darwin2data/`) downloads from S3, parses XML, outputs structured JSON.
- **Hosting: Render Static Site** — free, CDN-served, no cold starts, always on, $0/mo.
- **Data strategy: Pre-computed JSON committed to git.** Render only runs `npm run build`.
- **No backend, no database, no API routes.** All logic runs client-side.
- **PDF source deprecated** — `Timetable PDFs/` and `pdf2data/` kept for reference only.

---

## 2. Data Source: Darwin Timetable Feed

### 2.1 What is Darwin?

Darwin is the GB rail industry's official train running information engine. It takes feeds from every TOC's customer information system, combining it with train location data from Network Rail.

### 2.2 S3 Static Feed (Recommended)

The Darwin Timetable Feed is available as a daily ZIP file on S3:

- **URL**: `https://s3.{region}.amazonaws.com/{bucket}/{prefix}timetable.zip`
- **Credentials**: Access Key + Secret Key from Rail Data Marketplace
- **Format**: ZIP containing XML schedule files + reference data
- **Update frequency**: Daily
- **Cost**: Free

### 2.3 Darwin XML Format

The ZIP contains:
- `schedule/*.xml` — Individual schedule files, each containing multiple `<schedule>` elements
- `ref/*.xml` — Reference data (station names, TIPLOC→CRS mapping)

**Schedule element:**
```xml
<Pport xmlns="http://www.thalesgroup.com/rtti/PushPort/v16" ts="..." version="16.0">
  <schedule rid="..." uid="C12345" trainId="1A01" ssd="2026-06-19" toc="VT"
            status="P" isPassengerSvc="true" isActive="true">
    <OR tpl="EUSTON" wtd="06:13" pta="06:13" ptd="06:13"/>
    <IP tpl="WFJ" wta="06:30" wtd="06:31" pta="06:30" ptd="06:31"/>
    <DT tpl="GLASGOW" wta="09:42" pta="09:42"/>
  </schedule>
</Pport>
```

**Location types**: OR (origin), IP (intermediate), DT (destination), PP (passing)
**Key attributes**: tpl (TIPLOC), pta/ptd (public times), wta/wtd (working times), act (activity)

### 2.4 TIPLOC → CRS Mapping

Darwin uses TIPLOC codes (e.g., "EUSTON"); the frontend uses CRS codes (e.g., "EUS"). Mapping is in the `ref/` files within the ZIP.

---

## 3. Station-Centric Data Model

### 3.1 Core Concept

Services are indexed by station, not by "timetable tables". Each station has a file containing all services that pass through it.

**Advantages over table-centric model:**
- Simpler data pipeline (no need to reconstruct table groupings)
- More flexible UI (filter by station, then by destination/operator/time)
- No "missing table" gaps — every station has complete data
- Natural fit for Darwin's data model (schedules reference stations)

### 3.2 Output Files

| File | Description | Count | Size |
|------|-------------|-------|------|
| `station-index.json` | Master station list with metadata | 1 | ~300 KB |
| `services/{crs}.json` | Services through each station | ~2,500 | ~8 MB total |
| `marey/{crs}.json` | Marey chart data per station | ~2,500 | ~3 MB total |

### 3.3 Station Index Format

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
      "destinations": ["GLC", "MAN", "BHM", "LIV"],
      "file": "services/EUS.json"
    }
  ]
}
```

### 3.4 Station Service File Format

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

### 3.5 Marey Data Format (Per Station)

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
        {"crs": "MKC", "arr": 412, "dep": 439}
      ]
    }
  ]
}
```

---

## 4. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Darwin S3 Static Feed                      │
│                  (National Rail, daily ZIP)                   │
└──────────────────────────┬───────────────────────────────────┘
                           │ (HTTP GET + basic auth)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                   Rust: darwin2data                           │
│                                                               │
│  1. Download ZIP from S3                                      │
│  2. Extract XML files                                         │
│  3. Parse reference data (TIPLOC→CRS mapping)                 │
│  4. Parse schedule XML → Service structs                      │
│  5. Index services by station                                 │
│  6. Generate station-index.json                               │
│  7. Generate services/{crs}.json per station                  │
│  8. Generate marey/{crs}.json per station                     │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼  (committed to git)
┌──────────────────────────────────────────────────────────────┐
│  static/                                                      │
│  ├── station-index.json                                       │
│  ├── services/{crs}.json   (~2,500 files)                     │
│  └── marey/{crs}.json      (~2,500 files)                     │
└──────────────────────────┬───────────────────────────────────┘
                           │  (npm run build)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  SvelteKit SSG + adapter-static                              │
│  └→ prerenders station pages (/station/{crs})                │
└──────────────────────────┬───────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  Render Static Site (CDN)                                    │
│  papertime.tweak.wiki                                        │
└──────────────────────────────────────────────────────────────┘
```

---

## 5. Tech Stack

| Layer | Technology |
|---|---|
| **Data source** | Darwin Timetable Feed (S3, XML) |
| **Data pipeline** | Rust — `darwin2data` binary |
| **Hosting** | Render Static Site |
| **Frontend** | SvelteKit + `adapter-static` |
| **CSS** | Tailwind v4 |
| **Charts** | D3.js (lazy-loaded) |
| **Search** | Fuse.js (client-side) |

---

## 6. Rust Pipeline: darwin2data

### 6.1 Project Structure

```
darwin2data/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point, orchestration
│   ├── feed.rs          # S3 download
│   ├── parse.rs         # XML parsing (quick-xml)
│   ├── stations.rs      # Station index + service indexing
│   └── types.rs         # Shared data structures
```

### 6.2 Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["blocking"] }
zip = "2"
quick-xml = "0.37"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
chrono = "0.4"
dotenvy = "0.15"
env_logger = "0.11"
log = "0.4"
```

### 6.3 Processing Steps

1. **Download**: HTTP GET to S3 with basic auth → `timetable.zip`
2. **Extract**: Unzip to temp directory
3. **Reference data**: Parse `ref/*.xml` → TIPLOC→CRS mapping + station names
4. **Schedules**: Parse `schedule/*.xml` → `Vec<Service>`
5. **Index**: For each service, for each calling station → add to station's service list
6. **Output**: Write `station-index.json`, `services/{crs}.json`, `marey/{crs}.json`

### 6.4 Credentials

```bash
# .env (gitignored)
DARWIN_S3_BUCKET=darwin.xmltimetable
DARWIN_S3_PREFIX=PPTimetable/
DARWIN_S3_ACCESS_KEY=...
DARWIN_S3_SECRET_KEY=...
DARWIN_S3_REGION=eu-west-1
```

---

## 7. Frontend

### 7.1 Routes

| Route | Description |
|---|---|
| `/` | Landing page with station search |
| `/station/{crs}` | Station timetable page |
| `/station/{crs}/marey` | Station Marey chart |

### 7.2 Search Flow

1. User types station name → Fuse.js autocomplete on `station-index.json`
2. Select station → load `services/{crs}.json`
3. Display services in timetable format
4. Filter by: operator, destination, time of day, direction

### 7.3 Station Page

- **Timetable view**: Classic paper-style table showing all services
- **Marey view**: Time–distance diagram for the station
- **Pattern view**: Service pattern diagram

---

## 8. Migration Plan

1. ✅ Research Darwin feed format and XML schema
2. ✅ Scaffold `darwin2data` Rust project
3. ✅ Design station-centric data model
4. ⬜ Test S3 download with real credentials
5. ⬜ Implement full XML parsing (schedule locations)
6. ⬜ Implement station indexing
7. ⬜ Update frontend for station-centric views
8. ⬜ Validate output against current data
9. ⬜ Deprecate PDF pipeline

---

## 9. Open Questions

1. **Service filtering**: Pre-filter by day (MF/SAT/SUN) or include all and filter client-side?
2. **Time range**: Full day or split by time period for large stations?
3. **Portion handling**: How to display trains that split/join?
4. **Historical data**: Keep previous timetable versions or always show current?
