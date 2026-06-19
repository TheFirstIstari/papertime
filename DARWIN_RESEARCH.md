# Darwin Timetable Feed — Research Notes

## 1. Connection Architecture

### STOMP Protocol
- Darwin uses STOMP 1.2 (Simple Text Oriented Messaging Protocol)
- Default port: 61613
- Credentials: username + password from Rail Data Marketplace
- Host: provided by National Rail after signup (typically `stomp.nationalrail.co.uk` or similar)

### Connection Flow
1. TCP connection to STOMP server
2. STOMP CONNECT frame with credentials
3. SUBSCRIBE to topic (e.g., `/topic/timetable`)
4. Receive MESSAGE frames containing XML payloads
5. Each message is a complete XML document wrapped in STOMP frame

### Message Types
The Darwin Push Port sends multiple message types:
- **Schedule** — Full train schedule (what we need for timetables)
- **DeactivatedSchedule** — Schedule cancellation
- **Association** — Train associations (joining/splitting)
- **TS (TrainStatus)** — Real-time train status
- **OW (StationMessage)** — Station alerts
- **TrainAlert** — Train alerts
- **TrainOrder** — Platform ordering
- **Formation/Loading** — Rolling stock data

For PaperTime, we primarily need **Schedule** messages.

## 2. XML Data Format

### Root Element
```xml
<Pport xmlns="http://www.thalesgroup.com/rtti/PushPort/v16"
       ts="2026-06-19T12:00:00" version="16.0">
  <uR updateOrigin="Darwin">
    <schedule>...</schedule>
    <schedule>...</schedule>
    ...
  </uR>
</Pport>
```

### Schedule Element (Train Timetable)
```xml
<schedule rid="202606190123456789" uid="C12345" trainId="1A01"
          rsid="VT01" ssd="2026-06-19" toc="VT" status="P"
          trainCat="OO" isPassengerSvc="true" isActive="true">
  <OR tpl="EUS" wta="" wtd="06:13" pta="06:13" ptd="06:13"/>
  <IP tpl="WFJ" wta="06:30" wtd="06:31" pta="06:30" ptd="06:31"/>
  <IP tpl="MKC" wta="06:52" wtd="06:53" pta="06:52" ptd="06:53"/>
  <DT tpl="GLC" wta="09:42" wtd="" pta="09:42" ptd=""/>
</schedule>
```

### Location Types
| Type | Meaning | Has pta/ptd | Has wta/wtd |
|------|---------|-------------|-------------|
| **OR** | Origin (passenger) | Yes | wtd required |
| **OPOR** | Origin (operational) | No | wtd required |
| **IP** | Intermediate (passenger) | Yes | Both required |
| **OPIP** | Intermediate (operational) | No | Both required |
| **PP** | Passing point | No | wtp only |
| **DT** | Destination (passenger) | Yes | wta required |
| **OPDT** | Destination (operational) | No | wta required |

### Key Attributes

**Schedule-level:**
- `rid` — RTTI unique Train ID (required)
- `uid` — Train UID, e.g. "C12345" (required)
- `trainId` — Headcode, e.g. "1A01" (required)
- `rsid` — Retail Service Identifier (optional)
- `ssd` — Scheduled Start Date, e.g. "2026-06-19" (required)
- `toc` — ATOC operator code, e.g. "VT" (required)
- `status` — Service type: P=Train, B=Bus, S=Ship (default: P)
- `trainCat` — Category, e.g. "OO" (default: OO)
- `isPassengerSvc` — Boolean (default: true)
- `isActive` — Boolean (default: true)
- `deleted` — Boolean (default: false)
- `isCharter` — Boolean (default: false)

**Location-level (all types):**
- `tpl` — TIPLOC code (required)
- `act` — Activity codes, e.g. "TB" (Train begins), "TF" (Train finishes), "T" (Stops), "P" (Passes)
- `planAct` — Planned activity (if different from current)
- `can` — Cancelled (boolean)
- `fid` — Formation ID

**Passenger locations (OR, IP, DT):**
- `pta` — Public Time of Arrival (HH:MM format)
- `ptd` — Public Time of Departure (HH:MM format)

**Working times (all types):**
- `wta` — Working Time of Arrival (HH:MM:SS format)
- `wtd` — Working Time of Departure (HH:MM:SS format)
- `wtp` — Working Time of Passing (for PP type)

**Delay tracking:**
- `rdelay` — Route delay value

### Time Formats
- **RTTITimeType** (pta/ptd): `HH:MM` — e.g., "06:13", "23:59"
- **WTimeType** (wta/wtd/wtp): `HH:MM:SS` — e.g., "06:13:00", "23:59:00"
- **RTTIDateType** (ssd): `YYYY-MM-DD` — e.g., "2026-06-19"
- **RTTIDateTimeType** (ts): ISO 8601 — e.g., "2026-06-19T12:00:00"

### Activity Codes (act attribute)
Common values:
- `TB` — Train begins
- `TF` — Train finishes
- `T` — Stops to take/set down passengers
- `P` — Passes (doesn't stop)
- `OP` — Operational stop (not for passengers)
- `RM` — Reverses
- `SH` — Shunts

### Train Categories (trainCat)
Passenger services include: OL, OO, OW, XC, XD, XI, XR, XX, XZ
Freight: FF, FS, etc.

## 3. Snapshot vs Live Updates

Darwin provides two modes:

### Snapshot (Initial Load)
- Request full database snapshot via `GetFullSnapshotReq` or `GetSnapshotReq`
- Response comes as `sR` (snapshot Response) element containing all current schedules
- Large snapshots may be delivered via FTP (set `viaftp="true"`)
- Snapshot ID returned in `SnapshotId` element

### Live Updates (Incremental)
- After snapshot, subscribe to updates via `StartUpdateReq`
- Updates delivered as `uR` (update Response) elements
- Each update contains new/modified/deleted schedules
- For PaperTime, we only need periodic full snapshots (timetable changes ~daily)

## 4. S3 Static Feed Alternative

National Rail also provides a static timetable feed via S3:
- URL: `https://opendata.nationalrail.co.uk/api/staticfeeds/3.0/timetable`
- Requires Access Key + Secret Key (from RDM)
- Returns ZIP file containing XML timetable data
- Updated daily
- **Much simpler than STOMP** — single HTTP request, no persistent connection

### S3 Feed Format
The S3 feed returns a ZIP containing:
- `schedule/` — Individual schedule XML files
- `ref/` — Reference data (stations, operators, etc.)
- Full timetable in a single download

## 5. Data Volume Estimates

- ~69,000 services in the May 2026 timetable
- Each schedule XML: ~200-500 bytes
- Full snapshot: ~20-30 MB of XML
- After parsing/processing: ~6-8 MB JSON (matches our current output)

## 6. Mapping Darwin → PaperTime Data Model

### Darwin Schedule → PaperTime Service
```
Darwin schedule.rid    → Service.id (unique train ID)
Darwin schedule.trainId → Service.headcode (e.g. "1A01")
Darwin schedule.uid    → Service.uid (train UID)
Darwin schedule.toc    → Service.operator (TOC code)
Darwin schedule.ssd    → Service.startDate
Darwin OR/IP/DT        → Service.stops[]
  .tpl                 → ServiceStop.station (TIPLOC → CRS mapping needed)
  .pta/.ptd            → ServiceStop.arr/dep (minutes past midnight)
  .act                 → activity type (T=stop, P=pass, etc.)
Darwin schedule.isActive → filter: skip inactive
Darwin schedule.deleted  → filter: skip deleted
```

### TIPLOC → CRS Mapping
Darwin uses TIPLOC codes (e.g., "EUSTON" for London Euston), but our frontend uses CRS codes (e.g., "EUS"). Need a TIPLOC→CRS mapping from Darwin reference data or the static feed's ref/ directory.

### Station Reference Data
Darwin provides station reference data including:
- TIPLOC codes
- CRS codes
- Station names
- Coordinates

This can replace our OSM coordinate lookup.

## 7. Rust Implementation Plan

### Recommended Approach: S3 Static Feed
Rather than implementing STOMP (persistent connection, complex protocol), use the S3 static feed:
1. Single HTTP GET request to S3
2. Download ZIP file
3. Extract and parse XML
4. Output JSON

**Advantages:**
- No persistent connection needed
- No STOMP library dependency
- Simpler error handling
- Can be run on-demand (e.g., daily cron)
- Same data as STOMP push feed

### Alternative: STOMP Push Feed
If real-time updates are needed later:
1. Use `stomp` crate for STOMP protocol
2. Connect, subscribe, receive messages
3. Accumulate schedules in memory
4. Periodically write snapshot to disk

### Rust Crates Needed
```toml
[dependencies]
# HTTP client (for S3 feed)
reqwest = { version = "0.12", features = ["blocking"] }

# ZIP extraction
zip = "2"

# XML parsing
quick-xml = "0.37"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# STOMP (if using push feed)
stomp = "0.6"

# Async runtime (for STOMP)
tokio = { version = "1", features = ["full"] }

# Error handling
anyhow = "1"

# Date/time parsing
chrono = "0.4"

# AWS S3 (if using SDK)
aws-sdk-s3 = "1"
aws-config = "1"
```

### Project Structure
```
darwin2data/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point, CLI args, orchestration
│   ├── feed.rs          # Feed download (S3 or STOMP)
│   ├── parse.rs         # XML → internal types
│   ├── schedule.rs      # Schedule processing
│   ├── stations.rs      # Station index + TIPLOC→CRS mapping
│   ├── table_index.rs   # Table index generation
│   ├── route_index.rs   # Route grouping
│   ├── osm.rs           # OSM coordinates (fallback)
│   ├── marey.rs         # Marey data generation
│   ├── pattern.rs       # Pattern diagram layout
│   └── types.rs         # Shared data structures
```

### CLI Interface
```bash
# Using S3 feed (recommended)
darwin2data s3 \
  --access-key "$DARWIN_ACCESS_KEY" \
  --secret-key "$DARWIN_SECRET_KEY" \
  --region "eu-west-1" \
  --output-dir ../static/

# Using STOMP feed (future)
darwin2data stomp \
  --host "$DARWIN_HOST" \
  --port 61613 \
  --username "$DARWIN_USERNAME" \
  --password "$DARWIN_PASSWORD" \
  --output-dir ../static/
```

## 8. Open Questions to Resolve

1. **S3 bucket name and prefix** — Need from your Darwin credentials
2. **TIPLOC→CRS mapping source** — Is it in the S3 ref/ data or do we need a separate feed?
3. **Snapshot frequency** — How often does the S3 feed update? (likely daily)
4. **Authentication method** — AWS credentials? API key? (need to confirm from your RDM account)
5. **STOMP topic name** — Exact topic path for timetable subscription (if going STOMP route)

## 9. Gotchas

1. **Time format differences**: Darwin uses `HH:MM` for public times and `HH:MM:SS` for working times. Our model uses minutes past midnight (integer). Need consistent conversion.

2. **TIPLOC vs CRS**: Darwin uses TIPLOC codes internally. We need CRS codes for the frontend. The mapping is available in Darwin reference data.

3. **Schedule date ranges**: Darwin schedules have a start date (ssd) and may have end dates. Need to filter for the current timetable period.

4. **Active/inactive schedules**: Darwin marks schedules as active/inactive. Only active schedules should be included.

5. **Cancelled schedules**: The `can` attribute on locations and the `deleted` attribute on schedules need to be handled.

6. **Portion identifiers**: Some trains run in portions (split/join). The `rsid` attribute handles this. May need special handling for display.

7. **Multiple TOCs**: A service may have multiple operators (e.g., through services). Darwin provides the primary TOC in `toc`.
