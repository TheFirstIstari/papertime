/// Darwin Timetable Feed → PaperTime JSON (Station-Centric)
///
/// Connects to National Rail's Darwin S3 static feed, downloads the timetable ZIP,
/// parses the XML schedule files, and outputs structured JSON indexed by station.
///
/// Usage:
///   cd darwin2data
///   cp .env.example .env
///   # Fill in your Darwin S3 credentials
///   cargo run --release
///
/// Output:
///   ../static/station-index.json
///   ../static/services/{crs}.json   (one per station)
///   ../static/marey/{crs}.json      (one per station)

mod types;
mod feed;
mod parse;
mod stations;

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Load .env from the darwin2data directory
    let env_path = std::path::Path::new("darwin2data/.env");
    dotenvy::from_path(env_path).ok();
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

    let output_dir = PathBuf::from("static");
    std::fs::create_dir_all(&output_dir)?;
    std::fs::create_dir_all(output_dir.join("services"))?;
    std::fs::create_dir_all(output_dir.join("marey"))?;

    log::info!("Darwin2data starting...");
    log::info!("Bucket: {}/{}", bucket, prefix);
    log::info!("Region: {}", region);

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

    // Phase 2: Extract and parse XML → services
    log::info!("Phase 2: Parsing XML schedules...");
    let (services, _tiploc_to_crs, tiploc_to_name) = parse::extract_schedules(&zip_path)?;
    log::info!("  Parsed {} services", services.len());

    // Phase 3: Index services by station
    log::info!("Phase 3: Indexing services by station...");
    let station_index = stations::index_services(&services);
    log::info!("  {} stations", station_index.len());

    // Phase 4: Write output
    log::info!("Phase 4: Writing JSON output...");
    stations::write_index(&station_index, &output_dir)?;
    stations::write_station_services(&station_index, &output_dir)?;
    stations::write_marey_data(&station_index, &output_dir, &tiploc_to_name)?;

    log::info!("Done! Output in {}", output_dir.display());
    Ok(())
}
