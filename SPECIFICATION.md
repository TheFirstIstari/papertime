# PaperTime — Specification

> May 2026 National Rail timetable explorer. Rust data pipeline consuming Darwin Timetable Feed. SvelteKit SSG frontend. Render Static Site.
>
> Domain: `papertime.tweak.wiki`

---

## 1. Overview

**PaperTime** serves the May 2026 National Rail timetable data in three distinct views, each addressing a different way railway enthusiasts want to interact with timetable information:

1. **Paper Timetable View** — given a journey (origin + destination), render the relevant timetable as an interactive HTML table in the style of the classic paper timetable
2. **iBRY (Marey) Traffic Flow Graphs** — classic time–distance diagrams showing train services as slanted lines (stations on Y-axis, time on X-axis)
3. **Service Pattern Diagrams** — interactive schematic diagrams (like the West Coast Main Line example) for any station

**Key architectural decisions:**
- **Data source: Darwin Timetable Feed** — official GB rail industry timetable data via National Rail's Darwin push feed. No PDF parsing. No OCR. Authoritative source.
- **Build pipeline: Pure Rust** — single binary (`darwin2data/`) pulls from Darwin, processes, outputs structured JSON.
- **Hosting: Render Static Site** — free, CDN-served, no cold starts, always on, $0/mo.
- **Data strategy: Pre-computed JSON committed to git.** Render only runs `npm run build`. No Rust or data processing runs on Render.
- **No backend, no database, no API routes.** All logic runs client-side in the browser.
- **One data version at a time.** No version manifest. When the Dec 2026 timetable arrives, re-run the Rust pipeline and redeploy.
- **Station mileage: Great-circle from OSM coordinates.** Start simple, upgrade to track routing later if visually necessary.
- **PDF source deprecated.** The `Timetable PDFs/` and `pdf2data/` (lopdf-based) pipeline is replaced by Darwin. PDFs kept for reference only.

---

## 2. Data Source: Darwin Timetable Feed

### 2.1 What is Darwin?

Darwin is the GB rail industry's official train running information engine, operated by National Rail. It takes feeds directly from every Train Operating Company (TOC) customer information system (CIS), combining it with train location data from Network Rail.

**Key facts:**
- Powers all National Rail products, retailing websites (Trainline, etc.), TOC real-time digital channels
- Powers departure/arrival board screens at almost every station in the UK
- Data is openly available via the Rail Data Marketplace under NRE OGL license
- Free for all users regardless of usage volumes

### 2.2 Darwin Timetable Feed

The **Darwin Timetable Feed** is a push feed that provides the full GB rail timetable. This replaces the PDF parsing pipeline entirely.

**Access:**
- Register at [Rail Data Marketplace](https://www.raildatamarketplace.org.uk/)
- Subscribe to "Darwin Timetable Feed"
- Credentials: username + password for STOMP push port connection
- Terms: NRE OGL (Open Government Licence)

**Feed characteristics:**
- **Format**: XML push feed over STOMP protocol
- **Content**: Full timetable schedules, including all services, stations, operators, days of operation
- **Update frequency**: Periodic pushes (typically daily) + schedule changes
- **Coverage**: All GB rail services — matches the National Rail timetable PDFs exactly (same source data)

### 2.3 Why Darwin over PDFs?

| Aspect | PDF Parsing (old) | Darwin Feed (new) |
|--------|-------------------|-------------------|
| **Data quality** | OCR errors, parsing bugs, multi-part table overwrites | Authoritative, structured XML |
| **Station names** | Extracted from OCR'd text (error-prone) | Official CRS codes + names |
| **Operator codes** | OCR-corrupted (e.g. `/2`, `*1`, `$W`) | Official TOC codes |
| **Service times** | Minutes past midnight from parsed text | Direct from source |
| **Days of operation** | Inferred from section headers | Explicit in data |
| **Build time** | 10-15 min (Python parser) | Seconds (Rust + HTTP) |
| **Maintenance** | Breaks when PDF format changes | Stable API |
| **Updates** | Manual PDF download | Automatic push feed |
| **Completeness** | 29 missing tables (PDF gaps) | Full coverage |

---

## 3. What Are iBRY Graphs? (Research Summary)

An **iBRY graph** (also called a **Marey chart** or **time–distance diagram**) was invented by French railway engineer **Charles Ibry** in the 1840s for the Paris–Le Havre line, and later popularised by physicist **Étienne-Jules Marey**. It's the classic train timetable visualisation used by railway operators worldwide.

### Anatomy of a Marey chart

```
TIME →    06:00    07:00    08:00    09:00
         ┌─────────────────────────────────
London   │   ╲         ╲
Euston   │    ╲        ╲
         │     ╲       ╲   ╱
Watford  │──────╲──────╲─╱──────╲
Junction │      ╲  ╱   ╲       ╲
         │       ╲╱     ╲      ╲
Rugby    │───────╱───────╲──────╲──────
         │    ╱╱         ╲     ╲
Milton   │   ╱ ╱          ╲    ╲
Keynes   │──╱──╱───────────╲────╲───────
         │ ╱ ╱             ╲   ╲
Crewe    │╱ ╱               ╲  ╲
```

**Key properties:**
- **Y-axis**: stations, spaced proportionally to their actual physical distance on the track
- **X-axis**: time of day (left → right)
- **Each train** = a continuous diagonal line from departure station to arrival station
- **Slope** = speed (steeper = faster, shallower = slower)
- **Horizontal segments** = train stopped at a station
- **Lines must never intersect** — an intersection means two trains at the same place at the same time (conflict)
- **Line crossings** = trains passing each other in opposite directions on different tracks

---

## 4. Architecture (Decision Summary)

| Decision | Choice | Rationale |
|---|---|---|
| **Data source** | Darwin Timetable Feed (official) | Authoritative, structured, no OCR errors, free |
| **Build pipeline** | Pure Rust — `darwin2data` binary | Single binary, fast, reliable. Consumes Darwin XML → outputs JSON. |
| **Frontend framework** | SvelteKit + `adapter-static` | SSG-capable, Vite-based, tiny bundles, client routing. User knows Svelte. |
| **CSS** | Tailwind v4 | Already in the user's stack |
| **Charts** | D3.js (lazy-loaded, code-split) | Best-in-class for custom SVG visualisations |
| **Station search** | Fuse.js (client-side fuzzy search) | 2,500 stations, instant typo-tolerant match |
| **Hosting** | Render Static Site | Free, CDN, no cold starts, no sleep |
| **Domain** | `papertime.tweak.wiki` | User handles DNS |
| **Data strategy** | Pre-computed JSON committed to git, no version manifest | Render only runs `npm run build`. Single data version at a time. |
| **Station mileage** | Great-circle from OSM coords (start simple) | For Marey charts and pattern diagrams. Upgrade to track routing if visually insufficient. |
| **State management** | URL search params as source of truth | Free shareability, back/forward support, no global state. |
| **PDF source** | **DEPRECATED** — kept for reference only | Replaced by Darwin Timetable Feed. `pdf2data/` (lopdf) and `Timetable PDFs/` retained but not used in pipeline. |
| **Build orchestrator** | `mise.toml` tasks | User's standard tool. |

---

## 5. Data Inventory

### 5.1 Source Material

| Dataset | Source | Size | Purpose |
|---|---|---|---|
| **Darwin Timetable Feed** | STOMP push feed (XML) | Streaming | **Primary data** — official service records with times, stations, operators |
| Timetable PDFs (deprecated) | 195 files, ~99MB | `Timetable PDFs/` | Reference only — no longer parsed |
| Route map PDFs (deprecated) | 188 files, ~80MB | `Route table maps - separate PDFs/` | Reference only — no longer parsed |

### 5.2 Darwin Feed Data Model

The Darwin Timetable Feed provides XML messages containing:

```xml
<!-- Simplified example of Darwin timetable data structure -->
<schedule>
  <scheduleId>...</scheduleId>
  <trainId>1A01</trainId>
  <operator>VT</operator>
  <daysRun>MF</daysRun>
  <origin>
    <crs>EUS</crs>
    <departure>06:13</departure>
  </origin>
  <callingPoint>
    <crs>WFJ</crs>
    <arrival>06:30</arrival>
    <departure>06:31</departure>
  </callingPoint>
  <callingPoint>
    <crs>MKC</crs>
    <arrival>06:52</arrival>
    <departure>06:53</departure>
  </callingPoint>
  <destination>
    <crs>GLC</crs>
    <arrival>09:42</arrival>
  </destination>
</schedule>
```

Key fields mapped to our data model:
- `trainId` → `Service.id` / `Service.headcode`
- `operator` → `Service.operator` (TOC code)
- `daysRun` → `Service.days` (MF/SAT/SUN)
- `crs` → `ServiceStop.station`
- `arrival`/`departure` → minutes past midnight

### 5.3 Estimated Structured Output

| Output | Estimated size | Format |
|---|---|---|
| Stations index | ~300 KB | JSON (compact) |
| Table index | ~50 KB | JSON |
| Route index | ~30 KB | JSON |
| Service records (195 files) | ~6–8 MB | JSON |
| Marey chart data (per route) | ~2–3 MB | JSON |
| Pattern diagram data (per station) | ~3–5 MB | JSON |
| **Total** | **~12–16 MB** → **~3–4 MB brotli** | |

---

## 6. Feature Specifications

### 6.1 Paper Timetable View

**Goal:** Given a user's origin and destination stations, render the relevant timetable as an interactive HTML table in the familiar style of the National Rail paper timetable.

**Inputs:**
- Origin station (text autocomplete — Fuse.js on `stations.json`)
- Destination station (text autocomplete)
- Optional: day of week / time filter

**Landing page (empty state):** The `/` route when no `?from=&to=` params are present.

**Data model (pre-computed at build time by Rust pipeline):**
- `stations.json` — index of ~2,500 station names → CRS codes → table numbers
- `services/{table_number}.json` — one file per table, containing operator, direction, days of operation, calling pattern as (station, arr, dep) tuples
- `table-index.json` — for each table: route name, region, stations served, gap flag
- `route-index.json` — route ↔ tables ↔ stations mapping

**Logic (all client-side from pre-computed JSON):**
1. User searches origin + destination
2. Intersect their table sets to find tables where both appear
3. Results prioritised: direct service > fewest changes > route importance
4. User selects a result → renders the full timetable in "paper" style

**Paper-style rendering:** HTML table mirroring the classic paper layout — column-per-service, row-per-station, arrival/departure times, operator colour-coding, responsive layout, CSS print stylesheet.

**Interactive enhancements:** Click service column to highlight, click station row to highlight, day-of-week filtering (MF | SAT | SUN) tabs, time range filter, search within timetable, sort by departure time, toggle between paper view and list view.

### 6.2 iBRY (Marey) Traffic Flow Graphs

**Goal:** Visualise the timetable as classic Marey time–distance diagrams.

**Technical implementation:**
- **Build step** (Rust `marey.rs`): Parse each timetable into structured service records with station mileages (from OSM great-circle). Output per-route Marey JSON.
- **Client-side** (D3.js): Transform raw service records into Marey chart coordinates at render time. D3.js maps station mileages to Y positions and times (minutes past midnight) to X positions, draws polylines per service.
- **Runtime** (browser): D3.js reads the JSON, renders interactive SVG — zoom/pan, hover tooltips, click to highlight, time range filter, operator filter, direction filter.

**Data structure:** Per-route JSON with station list (name, CRS, mileage, type) and services array (id, operator, direction, days, stops with arr/dep times in minutes past midnight).

**Station ordering:** Derived from Darwin feed calling patterns. Stations ordered by their position in the service calling patterns, validated against route groupings.

**Performance:** Split data by route region. Lazy-load D3.js on first chart navigation (code-split via SvelteKit dynamic import). Only load Marey data for the route the user is viewing.

### 6.3 Service Pattern Diagrams

**Goal:** Generate interactive service pattern diagrams (in the style of the West Coast Main Line SVG example) for any station the user queries.

**What these diagrams show:**
- A vertical schematic of a railway line with stations positioned at intervals
- Coloured service lines running vertically between stations, each representing a specific operator/route
- Calling pattern indicators: filled circles = stop, empty = pass, dashes = limited service
- Departure minute labels at major stations
- Interactive: click a service line to highlight its full route

**Layout algorithm (Rust `pattern.rs`):**
1. For each station, find all routes passing through it (from route + table index)
2. For each route leg, build a directed graph: stations along the leg, connected by segments
3. **Branch detection (the core algorithm):** Compare service patterns across overlapping routes. A branch point is a station where the set of downstream stations diverges between services.
4. Compute branch offsets (horizontal displacement for diverging lines) using a topological sort of the route graph
5. Output per-station pattern JSON

**Rendering:** Pre-compute geometry (station positions, branch offsets, line routes) as JSON at build time. Render in browser with D3.js/SVG.

**Station page routing:** `/station/WFJ` shows all patterns for Watford Junction. Multi-route stations get tab navigation.

### 6.4 Error States & Fallback UX

The site is a static SPA — all error states are handled client-side.

| Scenario | UX Behaviour |
|---|---|
| **No matching tables** | Clear message explaining no direct service found. Suggest nearby stations. |
| **Station not in index** | Fuse.js autocomplete prevents this, but show a fallback message if it somehow occurs. |
| **Station has no pattern diagram** | "Service pattern diagram not yet available for this station." Target ~300 major stations for v1. |
| **JavaScript disabled** | SSG prerenders the landing page as static HTML. `<noscript>` tag explaining JS is needed for full functionality. |
| **JSON data fails to load** | Catch network errors, show retry button with link back to search. |
| **404 page** | SvelteKit error page with navigation back to search. |

---

## 7. Performance Budget

| Metric | Target | How |
|---|---|---|
| **Initial page load** | < 1s | Static Site CDN, minimal HTML shell, preload `stations.json` |
| **Time to interactive** | < 2s | Lazy-load D3.js on chart navigation, Fuse.js for search |
| **Paper timetable render** | < 500ms for 100 services | Pre-computed JSON, efficient HTML table rendering |
| **Marey chart render** | < 500ms for 100 trains | Pre-computed JSON, efficient SVG. Canvas fallback if > 500 trains. |
| **Station autocomplete** | < 50ms | Client-side Fuse.js search on pre-loaded stations index |
| **Bundle size (initial)** | < 100 KB JS | Code-split D3.js (unused on landing page). SvelteKit lazy routes. |
| **Data transfer per visit** | < 500 KB | Data-on-demand by route/table. Brotli compression at CDN layer. |
| **Lighthouse score** | > 90 | Static host, semantic HTML, progressive enhancement. |

**Render build warning:** Free tier includes 500 build minutes/month. Each `npm run build` with 300+ prerendered pages takes ~1–2 min. That's ~250–500 builds/month. Only the `data` task (Rust pipeline) is expensive — and that runs locally, not on Render.

### 7.1 Accessibility & Visual Design

**Target: WCAG 2.1 Level AA** for all three visualisation types.

- **Colour-blind safe palette** — Wong 8-colour palette. Lines use both colour and line style (dashed, dotted, solid) for redundancy.
- **Keyboard navigation** — All interactive elements keyboard-accessible. Marey chart: Tab to focus a train line, Enter to select, arrow keys to navigate.
- **Screen reader support** — Charts get descriptive alt text. Interactive elements have ARIA labels. Timetable is a proper `<table>` with `<thead>`, `<th scope>`.
- **Reduced motion** — Respect `prefers-reduced-motion`. Animations are purely decorative.
- **Print stylesheet** — Paper timetable view prints cleanly on A4/Letter.
- **Semantic HTML** — Search as `<form>` with `<label>`. Results as `<nav>` or `<section>`.
- **Focus management** — Search → results timetable → chart: Tab order matches visual flow.

---

## 8. Tech Stack

| Layer | Technology |
|---|---|
| **Data source** | Darwin Timetable Feed (National Rail, STOMP/XML) |
| **Data pipeline** | Rust — `darwin2data` binary (reqwest + quick-xml/stomp) |
| **Hosting** | Render Static Site (free, CDN, 500 build min/mo) |
| **Frontend framework** | SvelteKit + `adapter-static` |
| **CSS** | Tailwind v4 |
| **Marey chart rendering** | D3.js (lazy-loaded, code-split at route level) |
| **Service pattern diagrams** | D3.js / raw SVG (same lazy-loaded instance) |
| **Station search** | Fuse.js (client-side fuzzy search) |
| **Paper timetable render** | Pure Svelte components (HTML `<table>`) |
| **Data format** | Pre-compressed JSON (brotli at CDN layer) |
| **Build orchestration** | `mise.toml` tasks (`mise run data → build → deploy`) |
| **Version control** | Git → GitHub (Render auto-deploys from main) |

---

## 9. Architecture & Data Flow

```
┌──────────────────────────────────────────────────────────────┐
│                    Darwin Timetable Feed                      │
│                  (National Rail, STOMP/XML)                   │
│                  Credentials from RDM signup                 
└──────────────────────────┬───────────────────────────────────┘
                           │ (STOMP push connection)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                   Rust: darwin2data                           │
│                                                               │
│  1. darwin.rs          ← Connect to Darwin STOMP feed         │
│     └→ Subscribe to timetable push messages                   │
│     └→ Parse XML schedule messages                            │
│                                                               │
│  2. parse.rs           ← Transform Darwin XML → internal      │
│     └→ Map CRS codes, operator codes, times                   │
│     └→ Group services by table/route                          │
│     └→ Handle days of operation (MF/SAT/SUN)                  │
│                                                               │
│  3. stations.rs        ← Build master station index           │
│     └→ Station names from Darwin reference data               │
│     └→ CRS code → table mappings                              │
│                                                               │
│  4. table_index.rs     ← Per-table metadata                   │
│     └→ n_services, days, operators, station list              │
│                                                               │
│  5. route_index.rs     ← Derive route groupings               │
│     └→ Group tables by shared stations                        │
│     └→ Station ordering from calling patterns                 │
│                                                               │
│  6. osm.rs             ← Overpass API → coordinates           │
│     └→ great-circle mileage between stations                  │
│     └→ cached locally, re-fetch on demand                     │
│                                                               │
│  7. marey.rs           ← Marey coordinate compiler            │
│     └→ station mileages × service times                       │
│                                                               │
│  8. pattern.rs         ← Pattern diagram layout               │
│     └→ branch detection from service analysis                 │
│     └→ topological sort → branch offsets                      │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼  (committed to git)
┌──────────────────────────────────────────────────────────────┐
│  static/                                                      │
│  ├── stations.json                                            │
│  ├── table-index.json                                         │
│  ├── route-index.json                                         │
│  ├── services/{nnn}.json                                      │
│  ├── marey/{route-id}.json                                    │
│  └── patterns/{crs}.json                                      │
└──────────────────────────┬───────────────────────────────────┘
                           │  (npm run build)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  SvelteKit SSG + adapter-static                              │
│  └→ prerenders all known routes                              │
│     (/table/[id], /marey/[route],                            │
│      /station/[crs])                                         │
└──────────────────────────┬───────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  Render Static Site (CDN)                                    │
│  Always on, $0/mo                                            │
│  papertime.tweak.wiki                                        │
└──────────────────────────────────────────────────────────────┘
```

**Client-side flow (no backend, no database, no cold starts):**

1. User loads `index.html`
2. SvelteKit hydration boots, renders search UI
3. User types station → Fuse.js fuzzy matches from `stations.json`
4. User selects origin + destination
5. `table-index.json` → find matching tables
6. Render paper-style HTML timetable from `services/{table}.json`
7. User clicks "Marey graph" tab → lazy-load D3.js + `marey/{route}.json`
8. D3.js renders interactive Marey chart
9. User clicks a station name → load `patterns/{station}.json` → render pattern diagram

### 9.1 Build Orchestration

The build pipeline has strict ordering. All orchestrated via `mise.toml`:

```toml
[tasks.data]
description = "Pull Darwin timetable data and generate JSON"
run = "cargo run --release --manifest-path darwin2data/Cargo.toml"

[tasks.build]
description = "Build the SvelteKit static site"
depends = ["data"]
run = "npm run build"

[tasks.deploy]
description = "Commit data + push to GitHub → Render auto-deploys"
depends = ["build"]
run = """
git add static/
git commit -m 'update timetable data'
git push origin main
"""
```

**SSG prerender entries:** `adapter-static` needs explicit prerender entries for dynamic routes. Generated during build as a script that reads the JSON indexes:
- `table/[id]` → entries from `table-index.json` (~191 tables)
- `station/[crs]` → entries from `stations.json` (cover all stations)
- `marey/[route]` → entries from `route-index.json`

Expect ~500+ prerendered pages. adapter-static handles this efficiently.

### 9.2 Data Strategy: Precomputed + Committed

**Key decision:** The structured JSON datastore (~12–16 MB) is generated once by the Rust pipeline and **committed to git**. Render only runs `npm run build`.

| Location | Runs | Reads | Writes |
|---|---|---|---|
| Local machine | Rust `darwin2data` | Darwin Timetime Feed (STOMP) | `static/*.json` |
| Git repo | — | `static/*.json` | — |
| Render | `npm run build` | `static/*.json` | `build/` (HTML/JS/CSS) |

**Workflow for timetable updates:**
1. Run `mise run data` locally → fresh JSON from Darwin
2. Commit the updated `static/` directory
3. Push to GitHub → Render auto-deploys

**Versioning:** One data version at a time. No `manifest.json`. When the December 2026 timetable arrives, re-run the pipeline. Archive previous data as a git tag.

### 9.3 State Management: URL-Driven Search Flow

**Principle:** The search flow uses **URL search params** as the source of truth. This gives free shareability and back/forward support.

| Route | Params | Example |
|---|---|---|
| `/?from=CRS&to=CRS` | origin + destination | `/?from=EUS&to=MKC` |
| `/table/001?from=EUS&to=MKC` | table + original search context | `/table/001?from=EUS&to=MKC` |
| `/marey/wcml?from=EUS&to=MKC` | route + context | `/marey/wcml?from=EUS&to=MKC` |
| `/station/WFJ` | CRS code only | `/station/WFJ` |

**SvelteKit integration:** `$page.url.searchParams` is the single source of truth. No global stores, no localStorage, no session state.

### 9.4 Day-of-Week Model

Each service record carries a `days` field:

```json
{
  "id": "1A01",
  "days": ["MF", "SAT"],
  "operator": "VT",
  ...
}
```

**Day filter UI:** Three tabs (MF | SAT | SUN). Default: MF. Client-side show/hide.

**Edge cases:**
- Some services run different days in different directions
- Some tables have only one day block
- Bank holidays follow National Rail convention — no special handling
- **Empty filter state:** Message: *"No services on [day] for this table."*

---

## 10. Data Format Reference

### 10.1 Paper Timetable — Data Format

Stored as `static/services/{table_number}.json`:

```json
{
  "table": "002",
  "name": "Romford to Upminster",
  "period": "17 May to 12 December 2026",
  "operators": [
    {"code": "LO", "name": "London Overground", "color": "#E86A10"}
  ],
  "days": ["MF"],
  "stations": ["RMF", "EMP", "UPM"],
  "services": [
    {
      "id": "LO 2J01",
      "headcode": "2J01",
      "operator": "LO",
      "days": ["MF"],
      "direction": "westbound",
      "stops": [
        {"station": "RMF", "arr": null, "dep": 613},
        {"station": "EMP", "arr": 617, "dep": 617},
        {"station": "UPM", "arr": 623, "dep": null}
      ]
    }
  ]
}
```

**Format conventions:**
- Times: **minutes past midnight** (613 = 06:13, 1430 = 14:30) — integer arithmetic
- `arr`/`dep`: integer or `null`. `null` = terminus start/end
- `arr === dep`: train passes through (shown as `|` in paper style)
- `days`: array of day-set codes — `["MF"]`, `["SAT"]`, `["SUN"]`, or combinations
- One file per table, all day-period blocks merged
- **Reverse direction** encoded as separate services with `direction: "eastbound"` etc.

### 10.2 Table Index — Data Format

```json
{
  "table": "002",
  "name": "Romford to Upminster",
  "region": "Anglia",
  "operators": ["LO"],
  "stations": ["RMF", "EMP", "UPM"],
  "n_services": 42,
  "days": ["MF"],
  "file": "services/002.json",
  "routes": ["romford-upminster"],
  "has_route_map": true,
  "gap": false
}
```

### 10.3 Route Index — Data Format

```json
{
  "routes": [
    {
      "id": "romford-upminster",
      "name": "Romford to Upminster",
      "region": "Anglia",
      "tables": ["002"],
      "stations": ["RMF", "EMP", "UPM"],
      "station_order_source": "darwin"
    }
  ]
}
```

### 10.4 Stations Index — Data Format

```json
[
  {
    "id": "EUS",
    "name": "London Euston",
    "aliases": ["Euston", "London Euston"],
    "tables": ["001", "002", "010", "011", "066", "070", "080"],
    "routes": ["wcml"],
    "lat": 51.5286,
    "lng": -0.1337,
    "type": "terminal"
  }
]
```

Designed for direct consumption by Fuse.js. Station coordinates from Overpass API.

**Station types:** `terminal`, `major`, `interchange`, `minor`, `airport`. Derived from Darwin reference data.

### 10.5 Marey Chart — Data Format

```json
{
  "route": "West Coast Main Line",
  "route_id": "wcml",
  "stations": [
    {"name": "London Euston", "crs": "EUS", "mileage": 0, "type": "terminal"},
    {"name": "Watford Junction", "crs": "WFJ", "mileage": 17.5, "type": "interchange"},
    {"name": "Milton Keynes Central", "crs": "MKC", "mileage": 49.7, "type": "major"}
  ],
  "services": [
    {
      "id": "1A01",
      "operator": "VT",
      "direction": "northbound",
      "days": ["MF"],
      "stops": [
        {"station": "EUS", "dep": 373},
        {"station": "WFJ", "arr": 390, "dep": 391},
        {"station": "MKC", "arr": 412, "dep": 413}
      ]
    }
  ]
}
```

---

## 11. Rust Pipeline: darwin2data

### 11.1 Project Structure

```
darwin2data/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point, orchestrates all phases
│   ├── darwin.rs        # STOMP connection to Darwin, XML parsing
│   ├── parse.rs         # Transform Darwin XML → internal types
│   ├── stations.rs      # Build master station index
│   ├── table_index.rs   # Per-table metadata
│   ├── route_index.rs   # Route grouping derivation
│   ├── osm.rs           # Overpass API → coordinates
│   ├── marey.rs         # Marey coordinate compiler
│   ├── pattern.rs       # Pattern diagram layout
│   └── types.rs         # Shared data structures
```

### 11.2 Dependencies

```toml
[dependencies]
# HTTP client for Overpass API
reqwest = { version = "0.12", features = ["blocking", "json"] }

# STOMP protocol for Darwin push feed
stomp = "0.6"

# XML parsing
quick-xml = "0.37"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Async runtime (for STOMP connection)
tokio = { version = "1", features = ["full"] }

# Error handling
anyhow = "1"

# Utilities
walkdir = "2"
rayon = "1"
regex = "1"
chrono = "0.4"
```

### 11.3 Darwin Connection

The Darwin Timetable Feed uses the STOMP protocol (Simple Text Oriented Messaging Protocol). The Rust `stomp` crate handles this.

**Connection parameters:**
- Host: Provided by National Rail after RDM signup
- Port: 61613 (default STOMP)
- Username/Password: From RDM credentials
- Destination: `/topic/timetable` (or similar, confirmed on signup)

**Message flow:**
1. Connect to Darwin STOMP server
2. Subscribe to timetable push topic
3. Receive XML schedule messages
4. Parse each message into internal `Service` structures
5. Accumulate until all messages received (or timeout)
6. Write output JSON

### 11.4 Credentials Management

Credentials stored in environment variables (never committed):

```bash
# .env (gitignored)
DARWIN_HOST=stomp.nationalrail.co.uk
DARWIN_PORT=61613
DARWIN_USERNAME=your_rdm_username
DARWIN_PASSWORD=your_rdm_password
```

Loaded via `dotenv` crate or shell environment.

### 11.5 Output

All output written to `static/`:
- `static/stations.json`
- `static/table-index.json`
- `static/route-index.json`
- `static/services/{nnn}.json`
- `static/marey/{route-id}.json`
- `static/patterns/{crs}.json`

---

## 12. Migration Plan

### Phase 1: Set up Darwin access
1. Register on Rail Data Marketplace (user has account)
2. Subscribe to Darwin Timetable Feed
3. Receive STOMP credentials

### Phase 2: Build darwin2data
1. Create `darwin2data/` Rust project
2. Implement STOMP connection + XML parsing
3. Map Darwin data to existing JSON format
4. Test against live Darwin feed

### Phase 3: Validate output
1. Compare Darwin output with existing PDF-parsed data
2. Verify station counts, service counts match
3. Fix any data mapping issues

### Phase 4: Switch pipeline
1. Update `mise.toml` to use `darwin2data` instead of PDF pipeline
2. Remove Python parser dependencies (or keep as deprecated)
3. Update build scripts
4. Deploy

### Phase 5: Deprecate PDF pipeline
1. Move `pdf2data/` to `deprecated/pdf2data/`
2. Move `Timetable PDFs/` to `deprecated/Timetable PDFs/`
3. Update README
4. Keep for historical reference

---

## 13. Open Questions

1. **Darwin STOMP topic name** — Need to confirm exact topic/path for timetable feed after signup
2. **Message volume** — How many messages per full timetable dump? Need to size timeout appropriately
3. **Incremental vs full** — Does Darwin support incremental updates, or do we need to re-pull everything?
4. **Station reference data** — Does Darwin include station names/CRS codes, or do we need a separate reference feed?
5. **Table grouping** — Darwin provides raw services; we still need to group them into "tables" (the National Rail timetable table concept). This is a derived grouping based on shared routes/stations.
