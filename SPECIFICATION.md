# PaperTime — Specification

> July 2026 National Rail timetable explorer. Station-centric architecture. Rust data pipeline consuming Darwin Timetable Feed (S3). SvelteKit CSR frontend. Render Static Site.

> Domain: `papertime-hngn.onrender.com`

---

## 1. Overview

**PaperTime** serves the July 2026 National Rail timetable data in three distinct views:

1. **Station Timetable View** — pick a station, see all services passing through it in a classic timetable format with operator/destination/time-of-day filters
2. **Marey Traffic Flow Graphs** — time–distance diagrams per station showing service patterns (D3.js SVG)
3. **Service Pattern Diagrams** — schematic diagrams showing how services diverge from a station, grouped by branch/destination

**Key architectural decisions:**
- **Data source: Darwin Timetable Feed via S3** — official GB rail industry timetable data. No PDF parsing. No OCR. Authoritative source.
- **Station-centric data model** — services indexed by station, not by arbitrary "table" groupings. Simpler, more flexible.
- **Build pipeline: Pure Rust** — single binary (`darwin2data/`) downloads from S3, parses XML, outputs structured JSON.
- **Hosting: Render Static Site** — free, CDN-served, no cold starts, always on, $0/mo.
- **Data strategy: Pre-computed JSON committed to git.** Large data files tracked with Git LFS. Render pulls LFS files during build.
- **No backend, no database, no API routes.** All logic runs client-side. CSR only (`export const csr = true`).

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
- `schedule/*.xml` — Individual schedule files, each containing multiple `<Journey>` elements
- `ref/*.xml` — Reference data (station names, TIPLOC→CRS mapping — often absent/empty)

**Schedule element:**
```xml
<Pport xmlns="http://www.thalesgroup.com/rtti/PushPort/v16" ts="..." version="16.0">
  <Journey rid="..." uid="C12345" trainId="1A01" ssd="2026-06-19" toc="VT"
            status="P" isPassengerSvc="true" isActive="true">
    <OR tpl="EUSTON" wtd="06:13" pta="06:13" ptd="06:13"/>
    <IP tpl="WFJ" wta="06:30" wtd="06:31" pta="06:30" ptd="06:31"/>
    <DT tpl="GLASGOW" wta="09:42" pta="09:42"/>
  </Journey>
</Pport>
```

**Location types**: OR (origin), OPOR (operational origin), IP (intermediate), OPIP (operational intermediate), PP (passing), DT (destination), OPDT (operational destination)
**Key attributes**: tpl (TIPLOC), pta/ptd (public times), wta/wtd (working times)

### 2.4 TIPLOC → CRS Mapping

The Darwin reference data (`ref/*.xml` `<LocationRef tpl="..." crs="..." locname="..."/>`) provides TIPLOC→CRS mapping, but in practice the CRS field is often empty. The pipeline stores whatever code is available (falling back to TIPLOC) in the field named `crs`. The frontend matches by station name instead of CRS code to handle this.

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
| `station-index.json` | Master station list with metadata | 1 | ~1.6 MB |
| `services/{crs}.json` | Services through each station | 5,804 | ~8 MB total |
| `marey/{crs}.json` | Marey chart data per station | ~2,100 | ~3 MB total |
| `patterns/{crs}.json` | Pattern diagram data per station | ~1,500 | ~2 MB total |
| `naptan-cache.json` | Station coordinates/names from NaPTAN | 1 | ~350 KB |

### 3.3 Station Index Format (`station-index.json`)

```json
{
  "id": "EUS",
  "name": "London Euston",
  "tiploc": "EUS",
  "lat": 51.5281,
  "lng": -0.1339,
  "type": "terminal",
  "n_services": 1377,
  "operators": ["CS", "LF", "LM", "LO", "VT"],
  "destinations": ["Glasgow Central", "Liverpool Lime Street", ...],
  "file": "services/EUS.json"
}
```

Note: `id` and `tiploc` are the same value (the Darwin feed populates neither reliably as a CRS code). The same physical station may appear under multiple IDs (e.g., "EUS" and "EUSTON" both map to London Euston with 1377 services each).

### 3.4 Station Service File Format (`services/{crs}.json`)

```json
{
  "station": "EUS",
  "name": "London Euston",
  "services": [
    {
      "id": "20260704123456",
      "headcode": "1A01",
      "operator": "VT",
      "origin": "EUS",
      "origin_name": "London Euston",
      "destination": "GLC",
      "destination_name": "Glasgow Central",
      "calls": [
        {"crs": "EUS", "name": "London Euston", "dep": 373},
        {"crs": "WFJ", "name": "Watford Junction", "arr": 390, "dep": 391},
        {"crs": "MKC", "name": "Milton Keynes Central", "arr": 412, "dep": 413},
        {"crs": "CRE", "name": "Crewe", "arr": 437, "dep": 439},
        {"crs": "GLC", "name": "Glasgow Central", "arr": 582}
      ],
      "days": ["MF"]
    }
  ]
}
```

Times are stored as minutes past midnight (e.g., 373 = 06:13). The `crs` field in calls may contain TIPLOC codes in practice; the frontend matches by `name` field for reliability.

### 3.5 Marey Data Format (`marey/{crs}.json`)

Simplified: station ordering and services with stop-level arr/dep times. Used by MareyChart.svelte (D3.js SVG rendering).

### 3.6 Pattern Data Format (`patterns/{crs}.json`)

```json
{
  "station": "HTF",
  "station_name": "Hartford",
  "n_services": 76,
  "branches": [
    {
      "next_stop": "Liverpool South Parkway",
      "destination": "London Euston",
      "destination_tiploc": "EUSTON",
      "services": [
        {"id": "...", "operator": "VT", "headcode": "1A18", "dep": 490, "arr": 582, "days": ["MF"]}
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
│  3. Parse reference data (TIPLOC→CRS mapping, station names)  │
│  4. Parse schedule XML → DarwinSchedule structs                │
│  5. Index services by station                                 │
│  6. Generate station-index.json                               │
│  7. Generate services/{crs}.json per station                  │
│  8. Generate marey/{crs}.json per station                     │
│  9. Generate patterns/{crs}.json per station                  │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼  (committed to git)
┌──────────────────────────────────────────────────────────────┐
│  static/                                                      │
│  ├── station-index.json    (5,804 stations)                   │
│  ├── naptan-cache.json     (coordinates from NaPTAN)          │
│  ├── services/{crs}.json   (5,804 files)                      │
│  ├── marey/{crs}.json      (~2,100 files)                     │
│  └── patterns/{crs}.json   (~1,500 files)                     │
└──────────────────────────┬───────────────────────────────────┘
                           │  (npm run build)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  SvelteKit + adapter-static (CSR)                            │
│  └→ all route logic client-side via fetch()                  │
└──────────────────────────┬───────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  Render Static Site (CDN)                                    │
│  papertime-hngn.onrender.com                                 │
└──────────────────────────────────────────────────────────────┘
```

---

## 5. Tech Stack

| Layer | Technology |
|---|---|
| **Data source** | Darwin Timetable Feed (S3, XML) |
| **Data pipeline** | Rust — `darwin2data` binary (edition 2024) |
| **Hosting** | Render Static Site |
| **Frontend** | SvelteKit + `adapter-static` (CSR) |
| **CSS** | Tailwind v4 |
| **Charts** | D3.js (Marey charts) |
| **Search** | Fuse.js (client-side, fuzzy) |

---

## 6. Rust Pipeline: darwin2data

### 6.1 Project Structure

```
darwin2data/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point, orchestration
│   ├── feed.rs          # S3 download (S3 SDK)
│   ├── parse.rs         # XML parsing (quick-xml)
│   ├── stations.rs      # Station index + service indexing
│   ├── patterns.rs      # Pattern diagram generation
│   └── types.rs         # Shared data structures
```

### 6.2 Processing Steps

1. **Download**: S3 GET with access/secret key → `timetable.zip`
2. **Extract**: Unzip to temp directory
3. **Reference data**: Parse `ref/*.xml` → TIPLOC→CRS mapping + station names
4. **Schedules**: Parse `schedule/*.xml` → `Vec<DarwinSchedule>`
5. **Index**: For each service, for each calling station → add to station's service list
6. **Output**: Write `station-index.json`, `services/{crs}.json`, `marey/{crs}.json`, `patterns/{crs}.json`

### 6.3 Credentials

```bash
# .env (gitignored)
DARWIN_S3_BUCKET=...
DARWIN_S3_PREFIX=...
DARWIN_S3_ACCESS_KEY=...
DARWIN_S3_SECRET_KEY=...
DARWIN_S3_REGION=eu-west-1
```

---

## 7. Frontend

### 7.1 Routes

| Route | Description | Render mode |
|---|---|---|
| `/` | Landing page with station search (Fuse.js) | CSR |
| `/station/{crs}` | Station page with 3 tabs | CSR |

### 7.2 Search Flow

1. User types station name → Fuse.js autocomplete on `station-index.json`
2. Select station → load `services/{crs}.json` via fetch
3. Display services in timetable with times, operator colors, destination
4. Filter by: operator (dropdown), destination (dropdown), time of day (morning/afternoon/evening/night)

### 7.3 Station Page Tabs

- **Timetable**: Table of all services with departure time, headcode, operator (color-coded), destination, intermediate calls. The departure time shown is the departure from this station (services terminating here show "---"). Times matched by station name, not CRS code, due to TIPLOC→CRS discrepancies in the data pipeline.
- **Marey Chart**: D3.js SVG showing time–distance diagram with station ordering and services as diagonal lines.
- **Patterns**: Branch diagram showing how services diverge from the station, grouped by destination. Each branch expandable for service details.

### 7.4 Key Files

| File | Purpose |
|---|---|
| `src/routes/+page.svelte` | Landing page with search |
| `src/routes/station/[crs]/+page.svelte` | Station page (timetable + tabs) |
| `src/lib/components/MareyChart.svelte` | D3.js Marey diagram |
| `src/lib/components/PatternDiagram.svelte` | Pattern diagram component |
| `src/lib/data.ts` | Data loading helpers |
| `src/lib/types.ts` | TypeScript interfaces |
| `src/app.css` | Global styles + Tailwind |

---

## 8. Known Issues

### 8.1 Data Quality
- **TIPLOC stored in `crs` field**: The Darwin feed doesn't include CRS codes in its reference data reliably. The pipeline stores TIPLOC codes in the field named `crs`. The frontend works around this by matching on station name instead of CRS code.
- **Duplicate station entries**: Stations may appear under both their CRS-style code and their TIPLOC code (e.g., "EUS" and "EUSTON" both represent London Euston). Search results show a 3-letter code in parentheses to disambiguate.
- **Station names**: ~2,637 stations have names from NaPTAN; the rest use title-cased TIPLOC fallback, resulting in some stations with TIPLOC-as-name strings (e.g., "001", "002").

### 8.2 Marey Data Gaps
- Many stations (~3,700 out of 5,804) lack Marey data — only generated for stations with sufficient service density.
- 51-byte Marey files indicate fallback/empty data for some smaller stations.

### 8.3 Pattern Data Gaps
- Only ~1,500 stations have pattern files — generated for larger stations with branching services.
- Pattern file naming uses TIPLOC codes inconsistently.

---

## 9. Implementation Status (July 2026)

### Complete
- Darwin S3 download + XML parsing (Rust `darwin2data`)
- Station-centric data model (5,804 stations)
- Station names from NaPTAN (2,637) + title-cased fallback
- Station coordinates from NaPTAN
- Station type classification (terminal/major/interchange/airport/minor)
- Marey chart data generation + D3.js rendering
- Service pattern diagram data generation + branch view
- Frontend: station search (Fuse.js), timetable view with filters
- 3-tab station page (Timetable / Marey / Patterns)
- Time-of-day filtering (morning/afternoon/evening/night)
- Operator and destination filtering
- Render Static Site deployment

### Remaining (non-blocking)
- Station name→CRS disambiguation in search results
- Portion handling (split/join trains)
- Historical timetable versioning
- TIPLOC→CRS normalization in the pipeline (unblocks CRS-based matching)
- Custom domain DNS on Render

### Open Questions
1. **Service filtering**: Pre-filter by day (MF/SAT/SUN) or include all? → Currently all days included, filter client-side
2. **Historical data**: Keep previous timetable versions or always show current? → Current only
3. **Portion handling**: How to display trains that split/join? → Not yet implemented
