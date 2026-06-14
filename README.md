# PaperTime

May 2026 National Rail timetable explorer. Pure Rust build pipeline. SvelteKit SSG frontend. Render Static Site.

## Features

1. **Paper Timetable View** — given a journey (origin + destination), render the relevant timetable as an interactive HTML table
2. **iBRY (Marey) Traffic Flow Graphs** — classic time–distance diagrams showing train services as slanted lines
3. **Service Pattern Diagrams** — interactive schematic diagrams for any station

## Development

### Prerequisites

- [mise](https://mise.jdx.dev) — environment manager (Rust, Node, Bun all managed via mise)
- [Bun](https://bun.sh) — for the SvelteKit frontend

### Build Pipeline

```bash
# Run the full data pipeline (Rust pdf2data binary)
mise run data

# Build the SvelteKit static site
mise run build

# Deploy (commit data + push to GitHub → Render auto-deploys)
mise run deploy

# Dev server
mise run dev
```

### Project Structure

```
PaperTime/
├── Timetable PDFs/               ← SOURCE: 195 timetable PDFs (not served)
├── Route table maps - separate PDFs/  ← SOURCE: 188 route map PDFs (not served)
├── pdf2data/                     ← Rust binary (build pipeline)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               ← orchestrator
│       ├── extract.rs            ← PDF text extraction (lopdf)
│       ├── route_maps.rs         ← route map parser
│       ├── parse.rs              ← timetable service record parser
│       ├── stations.rs           ← station index builder
│       ├── table_index.rs        ← table index builder
│       ├── route_index.rs        ← route index builder
│       ├── osm.rs                ← OSM coordinate fetching
│       ├── marey.rs              ← Marey chart coordinate computation
│       └── pattern.rs            ← service pattern layout engine
├── src/                          ← SvelteKit web application
│   ├── routes/
│   │   ├── +page.svelte          ← landing page / search
│   │   ├── table/[id]/           ← paper-style timetable view
│   │   ├── marey/[route]/        ← iBRY chart view
│   │   └── station/[crs]/        ← service pattern diagram
│   └── lib/
├── static/
│   └── data/                     ← pre-computed JSON (committed to git)
└── render.yaml                   ← Render Blueprint
```

## Data Strategy

PDFs are source material only — extract once into structured JSON, commit with codebase, render on-demand client-side. No backend, no database, no API routes.

## License

National Rail timetable data is © Network Rail. This project is for educational purposes.
