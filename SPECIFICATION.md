# PaperTime — Specification

> May 2026 National Rail timetable explorer. Pure Rust build pipeline. SvelteKit SSG frontend. Render Static Site.
>
> Domain: `papertime.tweak.wiki`

---

## 1. Overview

**PaperTime** serves the new May 2026 National Rail timetable data in three distinct views, each addressing a different way railway enthusiasts want to interact with timetable information:

1. **Paper Timetable View** — given a journey (origin + destination), render the relevant timetable as an interactive HTML table in the style of the classic paper timetable
2. **iBRY (Marey) Traffic Flow Graphs** — classic time–distance diagrams showing train services as slanted lines (stations on Y-axis, time on X-axis)
3. **Service Pattern Diagrams** — interactive schematic diagrams (like the West Coast Main Line example) for any station

**Key architectural decisions:**
- **Build pipeline: Pure Rust** — no Python, no pymupdf. PDF extraction uses `lopdf`, all data processing is a single Rust binary (`pdf2data/`).
- **Hosting: Render Static Site** — free, CDN-served, no cold starts, always on, $0/mo.
- **Data strategy: Pre-computed JSON committed to git.** Render only runs `npm run build`. No Rust or data processing runs on Render.
- **No backend, no database, no API routes.** All logic runs client-side in the browser.
- **One data version at a time.** No version manifest. When the Dec 2026 timetable arrives, overwrite `static/data/` and redeploy.
- **Route maps ARE processed in v1.** The 188 route-map PDFs are parsed for station ordering and route grouping — but NOT served or displayed. They feed the data pipeline.
- **Station mileage: Great-circle from OSM coordinates.** Start simple, upgrade to track routing later if visually necessary.

---

## 2. What Are iBRY Graphs? (Research Summary)

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

## 3. Architecture (Decision Summary)

| Decision | Choice | Rationale |
|---|---|---|
| **Build pipeline** | Pure Rust — no Python, no pymupdf | `lopdf` validated on 3 diverse PDFs + 188 route maps (all single-page). |
| **Frontend framework** | SvelteKit + `adapter-static` | SSG-capable, Vite-based, tiny bundles, client routing. User knows Svelte. |
| **CSS** | Tailwind v4 | Already in the user's stack |
| **Charts** | D3.js (lazy-loaded, code-split) | Best-in-class for custom SVG visualisations |
| **Station search** | Fuse.js (client-side fuzzy search) | 2,500 stations, instant typo-tolerant match |
| **Hosting** | Render Static Site | Free, CDN, no cold starts, no sleep |
| **Domain** | `papertime.tweak.wiki` | User handles DNS |
| **Data strategy** | Pre-computed JSON committed to git, no version manifest | Render only runs `npm run build`. Single data version at a time. |
| **Route maps** | **Processed in v1** — extract station ordering, validate against timetables, feed route index | 188 single-page PDFs, simple station list extraction. Key input for service pattern diagrams. |
| **Station mileage** | Great-circle from OSM coords (start simple) | For Marey charts and pattern diagrams. Upgrade to track routing if visually insufficient. |
| **State management** | URL search params as source of truth | Free shareability, back/forward support, no global state. |
| **PDF serving** | PDFs are source-only, never served to users | Extract data into structured datastore (~10–15MB vs 232MB PDFs). |
| **Build orchestrator** | `mise.toml` tasks | User's standard tool. |

---

## 4. Data Inventory

### 4.1 Source Material

| Dataset | Files | Size | Purpose |
|---|---|---|---|
| Timetable PDFs | 195 files, ~99MB | `Timetable PDFs/` | Primary data — service records with times, stations, operators |
| Route map PDFs | 188 files, ~80MB | `Route table maps - separate PDFs/` (9 regions) | Station ordering & route grouping — feeds the pipeline |
| Service pattern examples | 3 files | Root directory | `example3.svg`, `exampleservicepatterndiagram.pdf`, `example2.pdf` — visual reference |

**Timetable coverage:** 191 standard tables (001–220, minus 29 missing) plus 4 supplementary sections (029a, 073a, 152a, 153a).

**Missing tables (29):** 018, 019, 045–049, 060, 079, 088, 089, 108–111, 129, 147–149, 159, 169, 179, 193–199. These also lack route maps (except a few that exist — e.g., Table 045, 047, 048, 049, 109 maps exist in the route map directory but the timetable is missing).

**Route map regions (9):** Anglia, East Midlands, Kent, London North East, London North West, Scotland, Sussex, Wessex, Western.

**Note:** Route maps and timetables are linked by **table number**, not by directory — Table 001 timetable is in `Timetable PDFs/` but its route map lives in `Anglia route/`. This is historical filing, not a schema. The pipeline matches them by extracting `Table NNN` from the filename.

### 4.2 Estimated Structured Output

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

## 5. Feature Specifications

### 5.1 Paper Timetable View

**Goal:** Given a user's origin and destination stations, render the relevant timetable as an interactive HTML table in the familiar style of the National Rail paper timetable.

**Inputs:**
- Origin station (text autocomplete — Fuse.js on `stations.json`)
- Destination station (text autocomplete)
- Optional: day of week / time filter

**Landing page (empty state):** The `/` route when no `?from=&to=` params are present.

**Data model (pre-computed at build time):**
- `stations.json` — index of ~2,500 station names → CRS codes → table numbers
- `services/{table_number}.json` — one file per table, containing operator, direction, days of operation, calling pattern as (station, arr, dep) tuples
- `table-index.json` — for each table: route name, region, stations served, gap flag
- `route-index.json` — route ↔ tables ↔ stations mapping, derived from shared-station analysis + validated against route-map directory structure

**Logic (all client-side from pre-computed JSON):**
1. User searches origin + destination
2. Intersect their table sets to find tables where both appear
3. Results prioritised: direct service > fewest changes > route importance
4. User selects a result → renders the full timetable in "paper" style

**Paper-style rendering:** HTML table mirroring the classic paper layout — column-per-service, row-per-station, arrival/departure times, operator colour-coding, responsive layout, CSS print stylesheet.

**Interactive enhancements:** Click service column to highlight, click station row to highlight, day-of-week filtering (MF | SAT | SUN tabs), time range filter, search within timetable, sort by departure time, toggle between paper view and list view.

### 5.2 iBRY (Marey) Traffic Flow Graphs

**Goal:** Visualise the timetable as classic Marey time–distance diagrams.

**Technical implementation:**
- **Build step** (Rust `marey.rs`): Parse each timetable + route map station ordering into structured service records with station mileages (from OSM great-circle). Output per-route Marey JSON.
- **Client-side** (D3.js): Transform raw service records into Marey chart coordinates at render time. D3.js maps station mileages to Y positions and times (minutes past midnight) to X positions, draws polylines per service.
- **Runtime** (browser): D3.js reads the JSON, renders interactive SVG — zoom/pan, hover tooltips, click to highlight, time range filter, operator filter, direction filter.

**Data structure:** Per-route JSON with station list (name, CRS, mileage, type) and services array (id, operator, direction, days, stops with arr/dep times in minutes past midnight).

**Station ordering:** Route-map PDFs provide the authoritative station order for each route. Timetable service records are validated against this order. If a timetable lists a station that the route map doesn't include, the timetable wins (service pattern may extend beyond the route map's scope).

**Performance:** Split data by route region. Lazy-load D3.js on first chart navigation (code-split via SvelteKit dynamic import). Only load Marey data for the route the user is viewing.

### 5.3 Service Pattern Diagrams

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
3. **Branch detection (the core algorithm):** Compare service patterns across overlapping routes. A branch point is a station where the set of downstream stations diverges between services. For example, if Table 001 services go to both Manchester (via Crewe) and Liverpool (via Crewe), Crewe is a branch point. This is computed by analysing the **union of calling patterns** across all services on a given route — where the set of subsequent stations begins to differ, that's a split.
4. Compute branch offsets (horizontal displacement for diverging lines) using a topological sort of the route graph — earlier branches get larger offsets to leave room
5. Output per-station pattern JSON

**Rendering:** Pre-compute geometry (station positions, branch offsets, line routes) as JSON at build time. Render in browser with D3.js/SVG, following the `example3.svg` visual language:
- **Station nodes:** ● major, ○ minor, ■ interchange, ═ terminal, ✈ airport
- **Service lines:** ━━ solid (all services), ╌ dashed (limited), ┄ dotted (occasional)
- **Calling pattern:** Filled circle = stop, empty = pass-through, no symbol = doesn't call
- **Departure minutes:** Below station node, comma-separated for each service

**Station page routing:** `/station/WFJ` shows all patterns for Watford Junction. Multi-route stations get tab navigation. Minor stations (not in ~300 major set) show "Service pattern diagram not yet available" with link to paper timetable.

### 5.4 Error States & Fallback UX

The site is a static SPA — all error states are handled client-side.

| Scenario | UX Behaviour |
|---|---|
| **No matching tables** | Clear message explaining data gap (29 missing tables). Link to missing-tables list. |
| **Station not in index** | Fuse.js autocomplete prevents this, but show a fallback message if it somehow occurs. |
| **Missing table (29 gaps)** | Distinct indicator: "Table ### — not available in this dataset". Show table number even without data (from `table-index.json` gap markers). |
| **Station has no pattern diagram** | "Service pattern diagram not yet available for this station." Target ~300 major stations for v1. |
| **JavaScript disabled** | SSG prerenders the landing page as static HTML. `<noscript>` tag explaining JS is needed for full functionality. |
| **JSON data fails to load** | Catch network errors, show retry button with link back to search. |
| **404 page** | SvelteKit error page with navigation back to search. |

**29 missing tables — gap communication:** Pre-rendered from `table-index.json`. Counts `n_missing` and lists missing table numbers dynamically — not hardcoded. Each missing table shows a GitHub issue link for status tracking.

---

## 6. Performance Budget

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

### 6.1 Accessibility & Visual Design

**Target: WCAG 2.1 Level AA** for all three visualisation types.

- **Colour-blind safe palette** — Wong 8-colour palette. Lines use both colour and line style (dashed, dotted, solid) for redundancy.
- **Keyboard navigation** — All interactive elements keyboard-accessible. Marey chart: Tab to focus a train line, Enter to select, arrow keys to navigate.
- **Screen reader support** — Charts get descriptive alt text. Interactive elements have ARIA labels. Timetable is a proper `<table>` with `<thead>`, `<th scope>`.
- **Reduced motion** — Respect `prefers-reduced-motion`. Animations are purely decorative.
- **Print stylesheet** — Paper timetable view prints cleanly on A4/Letter.
- **Semantic HTML** — Search as `<form>` with `<label>`. Results as `<nav>` or `<section>`.
- **Focus management** — Search → results timetable → chart: Tab order matches visual flow.

**Operator colour palette:**

```json
[
  {"operator": "Avanti West Coast", "color": "#E32636", "pattern": "solid"},
  {"operator": "London Northwestern Railway", "color": "#0072B2", "pattern": "solid"},
  {"operator": "CrossCountry", "color": "#009E73", "pattern": "solid"},
  {"operator": "TransPennine Express", "color": "#D55E00", "pattern": "solid"},
  {"operator": "East Midlands Railway", "color": "#CC79A7", "pattern": "dashed"},
  {"operator": "Great Western Railway", "color": "#56B4E9", "pattern": "dashed"},
  {"operator": "Greater Anglia", "color": "#F0E442", "pattern": "dotted"},
  {"operator": "London Overground", "color": "#E86A10", "pattern": "dotted"}
]
```

(Operator list is illustrative — actual palette built from all operators discovered during M1 data pipeline.)

---

## 7. Tech Stack

| Layer | Technology |
|---|---|
| **Hosting** | Render Static Site (free, CDN, 500 build min/mo) |
| **Frontend framework** | SvelteKit + `adapter-static` |
| **CSS** | Tailwind v4 |
| **Marey chart rendering** | D3.js (lazy-loaded, code-split at route level) |
| **Service pattern diagrams** | D3.js / raw SVG (same lazy-loaded instance) |
| **Station search** | Fuse.js (client-side fuzzy search) |
| **Paper timetable render** | Pure Svelte components (HTML `<table>`) |
| **Build pipeline — PDF extraction** | Rust `lopdf` (only dep needed — not `pdf-extract`) |
| **Build pipeline — data processing** | Rust `serde`, `serde_json`, `anyhow`, `walkdir`, `rayon`, `regex`, `reqwest` |
| **Build pipeline — OSM fetching** | Rust `reqwest` for Overpass API, with `serde` for JSON response parsing |
| **Data format** | Pre-compressed JSON (brotli at CDN layer) |
| **Build orchestration** | `mise.toml` tasks (`mise run data → build → deploy`) |
| **Version control** | Git → GitHub (Render auto-deploys from main) |

---

## 8. Architecture & Data Flow

```
Timetable PDFs/                Route table maps - separate PDFs/
(195 PDFs, ~99MB)              (188 PDFs, ~80MB — 9 regions)
        │                              │
        ▼                              ▼
┌──────────────────────────────────────────────────┐
│              Rust: pdf2data                       │
│                                                    │
│  1. extract.rs       ← lopdf text extraction      │
│     └→ both timetable + route map PDFs            │
│                                                    │
│  2. route_maps.rs    ← parse route maps:           │
│     └→ station ordering per table                  │
│     └→ route grouping by directory                 │
│                                                    │
│  3. parse.rs         ← timetable text parser:      │
│     └→ service records (station, times, operator)  │
│     └→ cross-references route map station order    │
│                                                    │
│  4. stations.rs      ← build master station index  │
│                                                    │
│  5. table-index.rs   ← per-table metadata          │
│                                                    │
│  6. route-index.rs   ← derive route groupings:     │
│     └→ Jaccard similarity on shared stations       │
│     └→ validated against route-map directories     │
│                                                    │
│  7. osm.rs           ← Overpass API → coords       │
│     └→ great-circle mileage between stations       │
│     └→ cached locally, re-fetch on demand          │
│                                                    │
│  8. marey.rs         ← Marey coordinate compiler   │
│     └→ station mileages × service times            │
│                                                    │
│  9. pattern.rs       ← pattern diagram layout:     │
│     └→ branch detection from service analysis      │
│     └→ topological sort → branch offsets           │
└───────────────────────┬────────────────────────────┘
                        │
                        ▼  (committed to git)
┌────────────────────────────────────┐
│  static/data/                      │
│  ├── stations.json                 │
│  ├── table-index.json              │
│  ├── route-index.json              │
│  ├── services/{nnn}.json           │
│  ├── marey/{route-id}.json         │
│  └── patterns/{crs}.json           │
└───────────────────┬────────────────┘
                    │  (npm run build)
                    ▼
┌────────────────────────────────────┐
│  SvelteKit SSG + adapter-static    │
│  └→ prerenders all known routes    │
│     (/table/[id], /marey/[route],  │
│      /station/[crs])               │
└───────────────────┬────────────────┘
                    ▼
┌────────────────────────────────────┐
│  Render Static Site (CDN)          │
│  Always on, $0/mo                  │
│  papertime.tweak.wiki              │
└────────────────────────────────────┘
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

### 8.1 Build Orchestration

The build pipeline has strict ordering. All orchestrated via `mise.toml`:

```toml
[tasks.data]
description = "Run the full data pipeline (Rust pdf2data binary)"
run = "cargo run --release --manifest-path pdf2data/Cargo.toml -- Timetable PDFs/ Route\\ table\\ maps\\ -\\ separate\\ PDFs/"

[tasks.build]
description = "Build the SvelteKit static site"
depends = ["data"]
run = "npm run build"

[tasks.deploy]
description = "Commit data + push to GitHub → Render auto-deploys"
depends = ["build"]
run = """
git add static/data/
git commit -m 'update timetable data'
git push origin main
"""
```

**SSG prerender entries:** `adapter-static` needs explicit prerender entries for dynamic routes. Generated during M5 as a build script that reads the JSON indexes and emits page entries:
- `table/[id]` → entries from `table-index.json` (191 tables)
- `station/[crs]` → entries from `stations.json` (cover all stations — adapter-static handles missing data gracefully; pattern diagram data only exists for ~300 stations)
- `marey/[route]` → entries from `route-index.json`

Expect ~500+ prerendered pages. adapter-static handles this efficiently — worst case is a ~1–2 min build time, which fits Render's free tier comfortably.

### 8.2 Data Strategy: Precomputed + Committed

**Key decision:** The structured JSON datastore (~12–16 MB) is generated once by the Rust pipeline and **committed to git**. Render only runs `npm run build`.

| Location | Runs | Reads | Writes |
|---|---|---|---|
| Local machine | Rust `pdf2data` | Timetable PDFs/ + Route maps/ | `static/data/*.json` |
| Git repo | — | `static/data/*.json` | — |
| Render | `npm run build` | `static/data/*.json` | `build/` (HTML/JS/CSS) |

**Workflow for timetable updates:**
1. Receive new PDFs (Dec 2026, etc.)
2. Place in `Timetable PDFs/` and/or `Route table maps - separate PDFs/`
3. Run `mise run data` locally → fresh JSON
4. Commit the updated `static/data/` directory
5. Push to GitHub → Render auto-deploys

**Versioning:** One data version at a time. No `manifest.json`. When the December 2026 timetable arrives, overwrite `static/data/`. Archive previous data as a git tag.

**Render dashboard configuration:**

| Setting | Value |
|---|---|
| Build command | `npm run build` |
| Publish directory | `build/` |
| Node version | 22 |
| Auto-deploy | Yes (main branch) |
| Build filter | **Always build when `src/` or `static/data/` changes.** Skip when only `*.md`, `Timetable PDFs/`, `Route table maps/` change (data-only rebuilds run locally). |

### 8.3 State Management: URL-Driven Search Flow

**Principle:** The search flow uses **URL search params** as the source of truth. This gives free shareability and back/forward support.

| Route | Params | Example |
|---|---|---|
| `/?from=CRS&to=CRS` | origin + destination | `/?from=EUS&to=MKC` |
| `/table/001?from=EUS&to=MKC` | table + original search context | `/table/001?from=EUS&to=MKC` |
| `/marey/wcml?from=EUS&to=MKC` | route + context | `/marey/wcml?from=EUS&to=MKC` |
| `/station/WFJ` | CRS code only | `/station/WFJ` |

**SvelteKit integration:** `$page.url.searchParams` is the single source of truth. No global stores, no localStorage, no session state. Fuse.js index (`stations.json`) is loaded once and cached in a Svelte module singleton.

**Search flow state machine:**
```
Empty → Typing → Has suggestions (Fuse.js) → Selected origin → Same for dest →
  Has matches (table list) → Selected table → Viewing timetable →
    Tab switched to Marey → Viewing chart →
      Clicked station → Viewing pattern diagram
```
Each transition is URL-driven. The back button unrolls the sequence naturally.

### 8.4 Day-of-Week Model

Each timetable PDF splits services into three day-period blocks: **Mondays to Fridays (MF)**, **Saturdays (SAT)**, **Sundays (SUN)**.

Each service record carries a `days` field rather than duplicating data per day:

```json
{
  "id": "1A01",
  "days": ["MF", "SAT"],
  "operator": "Avanti West Coast",
  ...
}
```

**Day filter UI:** Three tabs (MF | SAT | SUN). Default: MF. Client-side show/hide.

**Edge cases:**
- Some services run different days in different directions
- Some tables have only one day block (e.g., Table 002 is MF only)
- Bank holidays follow National Rail convention — no special handling
- **Empty filter state:** Message: *"No services on [day] for this table."* Tabs remain visible.

---

## 9. Data Format Reference

### 9.1 Paper Timetable — Data Format

Stored as `static/data/services/{table_number}.json`:

```json
{
  "table": "002",
  "name": "Romford to Upminster",
  "period": "17 May to 12 December 2026",
  "operators": [
    {"code": "LO", "name": "London Overground", "color": "#E86A10"}
  ],
  "days": ["MF"],
  "stations": [
    {"name": "Romford", "crs": "RMF"},
    {"name": "Emerson Park", "crs": "EMP"},
    {"name": "Upminster", "crs": "UPM"}
  ],
  "services": [
    {
      "id": "LO 2J01",
      "headcode": "2J01",
      "operator": "LO",
      "days": ["MF"],
      "direction": "westbound",
      "stops": [
        {"station": "Romford", "arr": null, "dep": 613},
        {"station": "Emerson Park", "arr": 617, "dep": 617},
        {"station": "Upminster", "arr": 623, "dep": null}
      ]
    }
  ]
}
```

**Format conventions:**
- Times: **minutes past midnight** (613 = 06:13, 1430 = 14:30) — integer arithmetic, no string parsing
- `arr`/`dep`: integer or `null`. `null` = terminus start/end
- `arr === dep`: train passes through (shown as `|` in paper style)
- `days`: array of day-set codes — `["MF"]`, `["SAT"]`, `["SUN"]`, or combinations
- One file per table, all day-period blocks merged into a single file at build time
- **Reverse direction** is encoded as separate services with `direction: "eastbound"` or `"southbound"` in the same file

### 9.2 Table Index — Data Format

```json
{
  "tables": [
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
    },
    {
      "table": "018",
      "name": null,
      "region": null,
      "operators": [],
      "stations": [],
      "n_services": 0,
      "days": [],
      "file": null,
      "routes": [],
      "has_route_map": false,
      "gap": true
    }
  ]
}
```

The `gap` field marks the 29 missing tables. These entries still appear in the index so the UI can display "Table 018 — not available" rather than silently omitting it.

### 9.3 Route Index — Data Format

```json
{
  "routes": [
    {
      "id": "romford-upminster",
      "name": "Romford to Upminster",
      "region": "Anglia",
      "tables": ["002"],
      "stations": ["RMF", "EMP", "UPM"],
      "station_order_source": "route_map"
    },
    {
      "id": "wcml",
      "name": "West Coast Main Line",
      "region": "London North West",
      "tables": ["001", "002", "003", "010", "011"],
      "stations": ["EUS", "MKC", "CRE", "PRE", "CAR", "GLC", "LIV"],
      "station_order_source": "route_map"
    }
  ]
}
```

**Derivation:**
1. **Primary:** Route-map PDF directory structure gives the grouping (London North West route contains Tables 001, 051, 066, etc.)
2. **Secondary:** Jaccard similarity clustering on timetable station sets validates and fills gaps where route maps don't exist
3. **Station ordering:** From route-map PDF text, validated against timetable data

**`station_order_source` field:** Tracks whether station order came from a route map or was inferred from timetables (for the ~30 tables missing route maps).

### 9.4 Stations Index — Data Format

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

Designed for direct consumption by Fuse.js. Station coordinates (lat/lng) are included for great-circle mileage computation and for the client-side map fallback if added later.

**Station types:** `terminal`, `major`, `interchange`, `minor`, `airport`. Derived from station facility data and cross-referenced against route maps.

### 9.5 Marey Chart — Data Format

Pre-computed JSON for the D3.js Marey chart renderer. Generated by `marey.rs` using route-map station ordering + OSM great-circle mileages:

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
      "operator": "Avanti West Coast",
      "direction": "southbound",
      "days": ["MF"],
      "stops": [
        {"station": "Crewe", "arr": null, "dep": 390},
        {"station": "Milton Keynes Central", "arr": 435, "dep": 437},
        {"station": "London Euston", "arr": 475, "dep": null}
      ]
    }
  ]
}
```

**Key points:**
- Times are minutes past midnight (390 = 06:30) — no string parsing needed for D3.js
- Stations ordered by route direction with mileage for Y-axis spacing
- D3.js maps mileages → Y positions, time integers → X positions, draws polylines per service
- Handling branches: one Marey JSON per route leg. The WCML route has one file for the main WCML and separate files for branching legs (Liverpool branch, Manchester branch, etc.)

### 9.6 Service Pattern Diagram — Data Format

Pre-computed JSON per station. Generated by `pattern.rs`:

```json
{
  "station": "Watford Junction",
  "crs": "WFJ",
  "mileage": 17.5,
  "routes": [
    {
      "name": "West Coast Main Line",
      "legs": [
        {
          "direction": "northbound",
          "stations": [
            {"name": "London Euston", "crs": "EUS", "mileage": 0, "type": "terminal"},
            {"name": "Watford Junction", "crs": "WFJ", "mileage": 17.5, "type": "interchange"},
            {"name": "Milton Keynes Central", "crs": "MKC", "mileage": 49.7, "type": "major"}
          ],
          "branches": [
            {
              "name": "Liverpool branch",
              "split_station": "Crewe",
              "stations": [
                {"name": "Crewe", "crs": "CRE", "mileage": 158.0, "type": "interchange"},
                {"name": "Liverpool Lime Street", "crs": "LIV", "mileage": 178.0, "type": "terminal"}
              ]
            },
            {
              "name": "Manchester branch",
              "split_station": "Crewe",
              "stations": [
                {"name": "Crewe", "crs": "CRE", "mileage": 158.0, "type": "interchange"},
                {"name": "Wilmslow", "crs": "WML", "mileage": 168.0, "type": "major"},
                {"name": "Manchester Piccadilly", "crs": "MAN", "mileage": 182.0, "type": "terminal"}
              ]
            }
          ]
        }
      ],
      "operators": [
        {"name": "Avanti West Coast", "color": "#E32636"}
      ],
      "services": [
        {
          "id": "1A01",
          "operator": "Avanti West Coast",
          "direction": "northbound",
          "calling_pattern": {
            "EUS": {"stop": true, "dep": 43},
            "WFJ": {"stop": false},
            "MKC": {"stop": true, "dep": 15},
            "CRE": {"stop": true, "dep": 03}
          }
        }
      ]
    }
  ]
}
```

**Branch detection algorithm (in `pattern.rs`):**
1. Group all services serving a given station
2. For each service, trace its calling pattern forward from that station
3. Group services by their **next station after the current station** — if services diverge to different next stations, that's the first split point
4. Recursively apply this: within each group, if services diverge again at a further station, that's a secondary branch
5. Result: a tree of branches rooted at the current station, each with an associated set of services

**Layout algorithm (branch offsets):**
1. Each branch depth level gets a horizontal offset increment (e.g., +30px per level)
2. Branches at the same depth level offset in opposite directions (left/right of the main line)
3. Topological sort ensures branches don't overlap spatially
4. Output is the JSON above with explicit branch coordinates pre-computed

---

## 10. Development Roadmap

### Milestone 0: Foundation
**Goal:** Scaffolded project that compiles and deploys a blank page to Render. (~1 session)

1. **Init `pdf2data/` Rust binary**
   - `cargo new pdf2data` inside PaperTime/
   - Dependencies: `lopdf`, `serde`, `serde_json`, `anyhow`, `walkdir`, `rayon`, `regex`, `reqwest`
   - Port existing `pdf-test/main.rs` text extraction into `extract.rs`
   - Verify: `cargo run --release -- Timetable PDFs/ "Route table maps - separate PDFs/"` extracts text from all PDFs

2. **Scaffold SvelteKit + adapter-static**
   - `bun create svelte@latest .` with Skeleton project + TypeScript
   - Install `adapter-static`, `tailwindcss@4`, `@tailwindcss/vite`, `fuse.js`, `d3`
   - Configure `svelte.config.js` with `adapter-static`
   - Wire Tailwind v4
   - Verify: `npm run build` produces `build/` with static HTML

3. **Set up `mise.toml`** with `data`, `build`, `deploy` tasks

4. **Deploy blank page to Render**
   - Create GitHub repo, add to Render via GitHub Connect
   - Build command: `npm run build`, Publish dir: `build/`
   - Verify blank page loads at `papertime.onrender.com`

5. **Archive `pdf-test/`** — the spike code now lives in `pdf2data/`

**Deliverable:** Blank page on Render. Rust binary extracts text from all PDFs. Build pipeline orchestrated by mise.

---

### Milestone 1: Data Pipeline
**Goal:** All 383 PDFs (195 timetables + 188 route maps) parsed into structured JSON. Station index, table index, route index, OSM mileages. (~3–4 sessions)

1. **PDF text extraction at scale** (`extract.rs`)
   - Walk both `Timetable PDFs/` and `Route table maps - separate PDFs/`
   - Extract text via `lopdf`. Handle edge cases: empty pages, image-only pages, corrupted metadata, encoding issues
   - Route maps: single-page, simpler extraction (just station names in order)
   - Output: raw text files per PDF (for debugging)

2. **Route map parsing** (`route_maps.rs`)
   - For each route map PDF, extract the station list in order
   - Use known station name dictionary (from timetable headers) to validate and filter
   - Map table number ↔ route map (by extracting `Table NNN` from filename)
   - Output: station ordering per table, route grouping (by directory), branch point candidates

3. **Service record parser** (`parse.rs`)
   - Parse extracted text into structured records:
     - Header: table number, route name, period
     - Day-period blocks: MF / SAT / SUN
     - Operator codes per column
     - Station names + CRS codes + arrival/departure times
   - Handle the **page-break problem**: a train section may span page boundaries — merge by matching column headers across page breaks
   - Handle **direction detection**: compare first vs last station lat/lng. If first is north of last → northbound. Extract station order from route maps as cross-reference.
   - Handle the **29 missing tables**: emit a gap marker instead of crashing
   - **Cross-reference** station ordering from route maps against timetable data

4. **Station index builder** (`stations.rs`)
   - Build master station list from all 195 tables
   - For each station: CRS code, name, aliases, list of tables + routes it appears in, lat/lng (once OSM data is fetched)
   - Output: `static/data/stations.json`

5. **Table index builder** (`table-index.rs`)
   - For each table: number, name, region, operators, station CRS list, service count, day periods, route map availability, gap flag
   - Includes the 29 missing tables as `gap: true` entries
   - Output: `static/data/table-index.json`

6. **Route index builder** (`route-index.rs`)
   - Derive route groupings from the route-map directory structure (primary) + Jaccard similarity clustering on timetable station sets (secondary for cross-reference)
   - Correlate any route maps that exist for missing timetable tables (e.g., Table 045, 047, 048 maps exist but timetables are missing)
   - Output: `static/data/route-index.json`

7. **OSM coordinate fetching** (`osm.rs`)
   - Query OpenStreetMap Overpass API for UK station coordinates
   - Cache results locally (avoid re-downloading on subsequent runs)
   - Compute **great-circle distances** between consecutive stations on each route
   - Handle rate limiting with polite delays
   - Output: mileage data embedded in station records

**Deliverable:** `mise run data` produces complete JSON dataset. All 383 PDFs processed. ~2,500 stations indexed, routes derived, mileages computed.

---

### Milestone 2: Paper Timetable Lookup
**Goal:** Search stations → find matching tables → render paper-style HTML table. (~2 sessions)

1. **Landing page UI** (`src/routes/+page.svelte`)
   - Two autocomplete inputs with Fuse.js on `stations.json`
   - Quick links to popular routes
   - Data-gap notification (29 missing tables — dynamic from `table-index.json`)
   - URL-driven state: `/?from=EUS&to=MKC`

2. **Table matching logic**
   - Intersect station→table sets
   - Rank results: direct service > fewer changes > route importance
   - Display result cards with route name, operator, service frequency, gap indicator

3. **PaperTable component** (`src/lib/components/PaperTable.svelte`)
   - Renders the classic paper timetable layout
   - Day-of-week filtering (MF | SAT | SUN tabs)
   - Interactive enhancements: highlight service column, highlight station row, time range filter, search within timetable, sort by departure time
   - CSS print stylesheet for A4/Letter
   - Responsive: horizontal scroll on mobile

4. **Error states** (§5.4) — no results, missing table, station not found, JS disabled

**Deliverable:** `/?from=RMF&to=UPM` shows the Romford→Upminster timetable as an interactive HTML table.

---

### Milestone 3: iBRY (Marey) Traffic Flow Graphs
**Goal:** Classic time–distance diagrams for any route. (~2–3 sessions)

1. **Marey coordinate computation** (`marey.rs`)
   - Input: service records + route-map station ordering + OSM great-circle mileages
   - Y-axis: stations positioned by cumulative mileage
   - X-axis: time (minutes past midnight)
   - Generate per-route Marey JSON with stop segments
   - Handle branching routes: generate separate Marey files for route legs (main line + each branch)
   - Handle direction separation: northbound vs southbound in separate files or overlaid

2. **Marey chart renderer** (`src/lib/components/MareyChart.svelte`)
   - D3.js SVG renderer
   - Zoom/pan via D3 zoom behaviour
   - Hover tooltip: train ID, origin, destination, calling pattern
   - Click to highlight a train's full path
   - Time range filter (presets: peak 07–09, all day, custom)
   - Operator filter toggle, direction filter toggle
   - WCAG colour palette
   - Canvas fallback if SVG exceeds 500 elements

3. **Mobile strategy**
   - Default viewport: zoomed to 07:00–09:00 peak
   - Touch gestures for pinch-zoom and drag-pan
   - Time-range presets as buttons (easier than slider on mobile)

4. **Route selector UI**
   - From `/table/001` → link to `/marey/wcml`
   - Browse all routes with available Marey charts

**Deliverable:** `/marey/wcml` shows an interactive Marey chart of the West Coast Main Line.

---

### Milestone 4: Service Pattern Diagrams
**Goal:** Interactive schematic diagrams (like `example3.svg`) for any station. (~2 sessions)

1. **Pattern layout engine** (`pattern.rs`)
   - For each of ~300 major stations, find all routes passing through it
   - Build the station graph: stations along each route leg, with branches
   - **Branch detection:** Compare service patterns across overlapping routes using the algorithm in §9.6
   - Compute branch offsets via topological sort
   - Output: per-station pattern JSON

2. **Pattern diagram renderer** (`src/lib/components/PatternDiagram.svelte`)
   - SVG renderer consuming pattern JSON
   - Visual language from `example3.svg`:
     - Vertical station axis with varying node types
     - Coloured service lines between stations
     - Calling pattern indicators
     - Departure minute labels
     - Click-to-highlight service paths
     - Legend
   - Responsive: vertical scroll with fixed-width SVG

3. **Station page routing**
   - `/station/WFJ` shows all patterns for Watford Junction
   - Multi-route stations: tab navigation
   - Minor stations: "Service pattern diagram not yet available" with link to paper timetable

**Deliverable:** `/station/WFJ` shows an interactive WCML-style pattern diagram for Watford Junction.

---

### Milestone 5: Deploy, Polish, Ship
**Goal:** Production-ready site at `papertime.tweak.wiki`. (~1 session)

1. **SSG prerender entry generation**
   - Build script reads JSON indexes → emits prerender entries for adapter-static
   - Covers all tables, stations, routes (~500+ pages)

2. **SEO & meta tags**
   - Title, description, Open Graph for every page type
   - Sitemap.xml

3. **Performance pass**
   - Measure against §6 budget
   - Code-split D3.js (dynamic import on chart navigation)
   - Preload `stations.json` on landing page
   - Lazy-load Marey data and pattern data

4. **Accessibility pass**
   - Keyboard navigation for all three visualisation types
   - Screen reader labels for charts and diagrams
   - Colour-blind safe palette validated
   - Reduced-motion support
   - Focus management for search → results → timetable flow

5. **Custom domain setup**
   - User handles DNS at registrar → Render dashboard
   - Update with domain: `papertime.tweak.wiki`

6. **README & documentation**
   - How to run the data pipeline (`mise run data`)
   - How to build and deploy
   - Data gap documentation (29 missing tables)
   - Route map source provenance

**Deliverable:** Live site at `papertime.tweak.wiki`.

---

## 11. Design Decisions

### 11.1 Resolved

| Decision | Resolution | Rationale |
|---|---|---|
| **PDF parsing** | ✅ `lopdf` works on all tested PDFs | Validated on timetables (002, 051, 161) + 188 route maps (all single-page) |
| **Station mileage** | ✅ Great-circle from OSM coords | Good enough for v1. Upgrade to track routing if needed. |
| **Framework** | ✅ SvelteKit + `adapter-static` | SSG, routing, Vite, tiny bundles, Render-compatible |
| **Build toolchain** | ✅ Pure Rust — no Python | `lopdf` handles PDFs, Rust handles everything else |
| **Route maps** | ✅ **Included in v1 pipeline** | Station ordering, route grouping, branch topology. Not served or displayed. |
| **Data versioning** | ✅ No version manifest | One data version at a time. Archive with git tags. |
| **Day-of-week model** | ✅ `days` field on each service | MF/SAT/SUN codes. Single file per table. Client-side filter. |
| **State management** | ✅ URL search params | Free shareability, back/forward, no global state. |
| **Data storage** | ✅ JSON | Brotli-compressed at CDN layer. ~12–16 MB total. |
| **Operator colours** | ✅ Wong colour-blind safe palette | See §6.1. |
| **Missing table UX** | ✅ Distinct indicator + gap note | Dynamic from `table-index.json` gap markers. |
| **Build orchestrator** | ✅ `mise.toml` | User's standard tool. |

### 11.2 Still Open During Development

1. **Direction detection from PDFs** — PDFs don't label "northbound" explicitly. Heuristic: compare first vs last station lat/lng. If first is north of last → northbound. Fallback: `"unknown"`. Route maps provide station ordering which constrains the possibilities.

2. **Route map ↔ timetable mapping strategy** — Some route maps exist for tables that have no timetable (Table 045 map exists but 045 timetable is missing). Some timetables have no route map (Table 002). The pipeline must handle one-to-zero and zero-to-one mappings.

3. **Page-break merge heuristic** — The hardest parsing problem. Trains spanning pages must be merged by matching column structure. The current approach: detect repeated header on each page, align columns by operator code and time sequence, merge. Needs iterative refinement during M1.

4. **Service pattern scope** — Generate for all stations (~2,500) or only major/interchange (~300)? v1 target: ~300 major stations. Pattern data for minor stations = "not yet available" during M4. The pipeline computes all stations but only emits JSON for the ~300 target set.

5. **Result ranking for multi-table matches** — Default: sort by table number (roughly correlates with route importance). Refine later with user testing.

6. **OSM Overpass API rate limits** — Must be polite (1 req/s) and cache aggressively. If a run is interrupted, the cache prevents re-fetching.

---

## 12. Key Risks & Mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| **PDF parsing fails on some tables** | Medium | Test all 383 early (M0/M1). Tag failures as data gaps. Use lopdf's lenient parsing mode. |
| **Page-break merge is unreliable** | High | The hardest subproblem. Design iteratively: start with simple tables (002), add complexity per observed pattern. Fallback: break at page boundary with a visual indicator. |
| **Route map station list extraction is noisy** | Low | Single-page PDFs with clean station lists. Filter against known station dictionary. Metadata keywords (LEGEND, VERSION) easily stripped. |
| **OSM great-circle is too inaccurate for Marey charts** | Medium | Start with it. If visually unacceptable, add OSM track routing as a M3 enhancement. |
| **300 service pattern diagrams blow up build time** | Low | Pattern computation is per-station and embarrassingly parallel (rayon). |
| **SSG prerender for 500+ pages is slow** | Low | adapter-static handles this. Worst case: ~1–2 min build, which fits Render's free tier. |
| **D3.js Marey chart performance on mobile** | Medium | Limit visible trains to viewport zoom level. Canvas fallback if SVG > 500 elements. |
| **Parse.rs needs to handle 195 different PDF layouts** | High | Design parser iteratively: start with simple tables (002), add complexity per observed pattern. Reserve 2–3 sessions for M1. |
| **Overpass API blocks requests** | Low | Cache aggressively. Use public OSM mirror if primary endpoint goes down. |
| **Route map ↔ timetable table number mismatch** | Low | Link by extracting `Table NNN` from both filenames. Validate that extracted table numbers are in valid range (001–244). |

---

## 13. File Structure

```
PaperTime/
│
├── SPECIFICATION.md              ← this file (single authoritative spec)
│
├── Timetable PDFs/               ← SOURCE: 195 timetable PDFs, ~99MB (not served)
├── Route table maps - separate PDFs/  ← SOURCE: 188 route map PDFs, ~80MB, 9 regions (not served)
│
├── example3.svg                  ← reference service pattern diagram
├── exampleservicepatterndiagram.pdf
├── example2.pdf
│
├── pdf-test/                     ← original lopdf spike (DELETED after M0 — archived in git)
│
├── pdf2data/                     ← Rust binary (the build pipeline)
│   ├── Cargo.toml                ← lopdf, serde, serde_json, anyhow, walkdir, rayon, regex, reqwest
│   └── src/
│       ├── main.rs               ← orchestrator: parse all PDFs → JSON
│       ├── extract.rs            ← PDF text extraction (lopdf — both timetable + route map PDFs)
│       ├── route_maps.rs         ← route map parser: station ordering, route grouping
│       ├── parse.rs              ← timetable service record parser
│       ├── stations.rs           ← station index builder
│       ├── table_index.rs        ← table index builder
│       ├── route_index.rs        ← route index builder (Jaccard + route-map validation)
│       ├── marey.rs              ← Marey chart coordinate computation
│       ├── pattern.rs            ← service pattern layout engine (branch detection + offsets)
│       └── osm.rs                ← OSM coordinate fetching + great-circle distance
│
├── src/                          ← SvelteKit web application
│   ├── app.html
│   ├── routes/
│   │   ├── +page.svelte          ← landing page / search
│   │   ├── table/[id]/           ← paper-style timetable view
│   │   ├── marey/[route]/        ← iBRY chart view
│   │   └── station/[crs]/        ← service pattern diagram
│   └── lib/
│       ├── components/
│       │   ├── PaperTable.svelte
│       │   ├── MareyChart.svelte
│       │   └── PatternDiagram.svelte
│       ├── search.js             ← Fuse.js wrapper (singleton module)
│       ├── data.js               ← fetch helpers (JSON loader with caching)
│       └── prerender-entries.js  ← generates adapter-static prerender list from JSON indexes
│
├── static/
│   └── data/                     ← generated JSON (committed to git)
│       ├── stations.json         ← ~2,500 stations
│       ├── table-index.json      ← 195+29 entries
│       ├── route-index.json      ← derived route groupings
│       ├── services/             ← per-table service records
│       ├── marey/                ← per-route Marey chart data
│       └── patterns/             ← per-station pattern data (~300 stations)
│
├── package.json
├── svelte.config.js
├── vite.config.js
├── mise.toml                     ← build orchestration
└── README.md                     ← after M5
```

---

## 14. Estimated Timeline

| Milestone | Effort | Dependencies |
|---|---|---|
| **M0: Foundation** | ~1 session | None |
| **M1: Data Pipeline** | ~3–4 sessions | M0 (includes route map parsing, all PDFs, OSM) |
| **M2: Paper Timetable** | ~2 sessions | M1 |
| **M3: Marey Charts** | ~2–3 sessions | M1 |
| **M4: Pattern Diagrams** | ~2 sessions | M1 |
| **M5: Deploy & Polish** | ~1 session | M2–M4 |

Each session is roughly an evening's work. Total: ~11–13 sessions.

**Milestone path with route maps added:**
- M1 gets +1 module (`route_maps.rs`) and +1 session vs the original plan
- M3 and M4 benefit from better station ordering data
- No changes to M2, M5
