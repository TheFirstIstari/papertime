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
    // National Rail Darwin S3 feed URL format
    let url = format!(
        "https://s3.{}.amazonaws.com/{}/{}timetable.zip",
        config.region, config.bucket, config.prefix
    );

    log::info!("Downloading from: {}", url);

    let temp_dir = std::env::temp_dir().join("darwin2data");
    std::fs::create_dir_all(&temp_dir)?;
    let zip_path = temp_dir.join("timetable.zip");

    // Download with basic auth (National Rail uses HTTP Basic Auth, not AWS signatures)
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .basic_auth(config.access_key, Some(config.secret_key))
        .send()?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Download failed: HTTP {} - {}", status, body);
    }

    let bytes = response.bytes()?;
    std::fs::write(&zip_path, &bytes)?;

    log::info!("Downloaded {} bytes to {}", bytes.len(), zip_path.display());

    Ok(zip_path)
}
