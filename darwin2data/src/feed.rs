use anyhow::Result;
use std::path::PathBuf;

pub struct S3Config<'a> {
    pub bucket: &'a str,
    pub prefix: &'a str,
    pub access_key: &'a str,
    pub secret_key: &'a str,
    pub region: &'a str,
}

/// Download the Darwin timetable ZIP from S3
pub fn download(config: &S3Config) -> Result<PathBuf> {
    // Construct S3 URL
    // Format: https://{bucket}.s3.{region}.amazonaws.com/{prefix}timetable.zip
    // Or: https://s3.{region}.amazonrail.co.uk/{bucket}/{prefix}timetable.zip

    // For National Rail's Darwin feed, the URL format is typically:
    // https://s3-eu-west-1.amazonaws.com/darwin.xmltimetable/PPTimetable/timetable.zip

    let url = format!(
        "https://s3.{}.amazonaws.com/{}/{}timetable.zip",
        config.region, config.bucket, config.prefix
    );

    log::info!("Downloading from: {}", url);

    // Create temp directory for download
    let temp_dir = std::env::temp_dir().join("darwin2data");
    std::fs::create_dir_all(&temp_dir)?;
    let zip_path = temp_dir.join("timetable.zip");

    // Download using reqwest with basic auth
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .basic_auth(config.access_key, Some(config.secret_key))
        .send()?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download: HTTP {} - {}",
            response.status(),
            response.text()?
        );
    }

    let bytes = response.bytes()?;
    std::fs::write(&zip_path, &bytes)?;

    log::info!("Downloaded {} bytes", bytes.len());

    Ok(zip_path)
}
