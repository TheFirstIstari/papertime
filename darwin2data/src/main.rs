mod feed;
mod parse;
mod stations;
mod patterns;
mod types;

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let env_path = std::path::Path::new("darwin2data/.env");
    dotenvy::from_path(env_path).ok();
    env_logger::init();

    let bucket = std::env::var("DARWIN_S3_BUCKET").expect("DARWIN_S3_BUCKET not set");
    let prefix = std::env::var("DARWIN_S3_PREFIX").expect("DARWIN_S3_PREFIX not set");
    let access_key = std::env::var("DARWIN_S3_ACCESS_KEY").expect("DARWIN_S3_ACCESS_KEY not set");
    let secret_key = std::env::var("DARWIN_S3_SECRET_KEY").expect("DARWIN_S3_SECRET_KEY not set");
    let region = std::env::var("DARWIN_S3_REGION").unwrap_or_else(|_| "eu-west-1".to_string());

    let output_dir = PathBuf::from("static");
    std::fs::create_dir_all(&output_dir)?;
    std::fs::create_dir_all(output_dir.join("services"))?;
    std::fs::create_dir_all(output_dir.join("marey"))?;
    std::fs::create_dir_all(output_dir.join("patterns"))?;

    log::info!("Darwin2data starting...");
    log::info!("Bucket: {}/{}", bucket, prefix);

    // Phase 1: Download from S3
    let zip_path = feed::download(&feed::S3Config {
        bucket: &bucket,
        prefix: &prefix,
        access_key: &access_key,
        secret_key: &secret_key,
        region: &region,
    })?;

    // Phase 2: Parse XML
    let (services, _tiploc_to_crs, tiploc_to_name) = parse::extract_schedules(&zip_path)?;
    log::info!("Parsed {} services", services.len());

    // Phase 3: Index by station
    let station_index = stations::index_services(&services);
    log::info!("{} stations", station_index.len());

    // Phase 4: Write output
    stations::write_index(&station_index, &output_dir)?;
    stations::write_station_services(&station_index, &output_dir)?;
    stations::write_marey_data(&station_index, &output_dir, &tiploc_to_name)?;

    // Phase 5: Generate pattern diagrams
    log::info!("Generating pattern diagrams...");
    let pattern_data = patterns::build_pattern_data(&station_index, &tiploc_to_name);
    patterns::write_pattern_data(&pattern_data, &output_dir)?;

    log::info!("Done! Output in {}", output_dir.display());
    Ok(())
}
