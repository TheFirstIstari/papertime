/// Darwin Timetable Feed → PaperTime JSON
///
/// Connects to National Rail's Darwin S3 static feed, downloads the timetable ZIP,
/// parses the XML schedule files, and outputs structured JSON for the PaperTime frontend.
///
/// Usage:
///   cd darwin2data
///   cp .env.example .env
///   # Fill in your Darwin S3 credentials
///   cargo run --release
///
/// Output:
///   ../static/stations.json
///   ../static/table-index.json
///   ../static/route-index.json
///   ../static/services/{nnn}.json
///   ../static/marey/{route-id}.json

mod types;
mod feed;
mod parse;
mod stations;
mod table_index;
mod route_index;
mod marey;

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Load credentials from .env
    dotenvy::dotenv().ok();
    env_logger::init();

    let bucket = std::env::var("DARWIN_S3_BUCKET")
        .expect("DARWIN_S3_BUCKET not set");
    let prefix = std::env::var("DARWIN_S3_PREFIX")
        .expect("DARWIN_S3_PREFIX not set");
    let access_key = std::env::var("DARWIN_S3_ACCESS_KEY")
        .expect("DARWIN_S3_ACCESS_KEY not set");
    let secret_key = std::env::var("DARWIN_S3_SECRET_KEY")
        .expect("DARWIN_S3_SECRET_KEY not set");
    let region = std::env::var("DARWIN_S3_REGION")
        .unwrap_or_else(|_| "eu-west-1".to_string());

    let output_dir = PathBuf::from("../static");
    std::fs::create_dir_all(&output_dir)?;

    log::info!("Darwin2data starting...");
    log::info!("Bucket: {}/{}", bucket, prefix);
    log::info!("Region: {}", region);
    log::info!("Output: {}", output_dir.display());

    // Phase 1: Download ZIP from S3
    log::info!("Phase 1: Downloading Darwin timetable feed...");
    let zip_path = feed::download(&feed::S3Config {
        bucket: &bucket,
        prefix: &prefix,
        access_key: &access_key,
        secret_key: &secret_key,
        region: &region,
    })?;
    log::info!("  Downloaded: {}", zip_path.display());

    // Phase 2: Extract and parse XML
    log::info!("Phase 2: Parsing XML schedules...");
    let schedules = parse::extract_schedules(&zip_path)?;
    log::info!("  Parsed {} schedules", schedules.len());

    // Phase 3: Build station index
    log::info!("Phase 3: Building station index...");
    let station_index = stations::build_index(&schedules);
    log::info!("  {} unique stations", station_index.len());

    // Phase 4: Group into tables and generate indexes
    log::info!("Phase 4: Building table index...");
    let table_index = table_index::build(&schedules, &station_index);
    log::info!("  {} tables", table_index.len());

    // Phase 5: Build route index
    log::info!("Phase 5: Building route index...");
    let route_index = route_index::build(&table_index);
    log::info!("  {} routes", route_index.len());

    // Phase 6: Generate Marey data
    log::info!("Phase 6: Generating Marey chart data...");
    let marey_data = marey::generate(&schedules, &route_index);
    log::info!("  {} Marey charts", marey_data.len());

    // Phase 7: Write all output
    log::info!("Phase 7: Writing JSON output...");
    stations::write(&station_index, &output_dir)?;
    table_index::write(&table_index, &output_dir)?;
    route_index::write(&route_index, &output_dir)?;
    marey::write(&marey_data, &output_dir)?;

    // Write per-table service files
    for table in &table_index {
        if !table.gap {
            table_index::write_services(table, &schedules, &output_dir)?;
        }
    }

    log::info!("Done! Output in {}", output_dir.display());
    Ok(())
}
