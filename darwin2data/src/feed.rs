use anyhow::Result;
use std::path::PathBuf;

pub struct S3Config<'a> {
    pub bucket: &'a str,
    pub prefix: &'a str,
    pub access_key: &'a str,
    pub secret_key: &'a str,
    pub region: &'a str,
}

pub fn download(config: &S3Config) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join("darwin2data");
    std::fs::create_dir_all(&temp_dir)?;

    // Check if we already have files cached
    let existing: Vec<_> = std::fs::read_dir(&temp_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".gz") && !name.ends_with(".gz.gz")
        })
        .collect();

    if existing.len() >= 2 {
        log::info!("Using {} cached files in {}", existing.len(), temp_dir.display());
        return Ok(temp_dir);
    }

    // Otherwise download from S3
    download_from_s3(config, &temp_dir)
}

fn download_from_s3(config: &S3Config, temp_dir: &std::path::Path) -> Result<PathBuf> {
    use aws_credential_types::Credentials;
    use aws_sdk_s3::Client;

    let credentials = Credentials::new(
        config.access_key,
        config.secret_key,
        None,
        None,
        "darwin",
    );

    let s3_config = aws_sdk_s3::config::Builder::new()
        .credentials_provider(credentials)
        .region(aws_sdk_s3::config::Region::new(config.region.to_string()))
        .build();

    let client = Client::from_conf(s3_config);
    let rt = tokio::runtime::Runtime::new()?;

    log::info!("Listing objects...");
    let objects = rt.block_on(async {
        let result = client
            .list_objects_v2()
            .bucket(config.bucket)
            .prefix(config.prefix)
            .send()
            .await?;
        Ok::<_, anyhow::Error>(result.contents().to_vec())
    })?;

    log::info!("Found {} objects", objects.len());

    let mut ref_key: Option<&str> = None;
    let mut sched_key: Option<&str> = None;

    for obj in &objects {
        if let Some(key) = obj.key() {
            if key.ends_with(".gz") {
                let filename = key.rsplit('/').next().unwrap_or(key);
                if filename.contains("_ref_") && ref_key.is_none() {
                    ref_key = Some(key);
                } else if !filename.contains("_ref_") && sched_key.is_none() {
                    sched_key = Some(key);
                }
            }
        }
    }

    for key in [ref_key, sched_key].iter().flatten() {
        let filename = key.rsplit('/').next().unwrap_or(key);
        let path = temp_dir.join(filename);
        log::info!("Downloading: {}", filename);
        let bytes = rt.block_on(download_object(&client, config.bucket, key))?;
        std::fs::write(&path, &bytes)?;
        log::info!("  Downloaded {} bytes", bytes.len());
    }

    Ok(temp_dir.to_path_buf())
}


async fn download_object(client: &aws_sdk_s3::Client, bucket: &str, key: &str) -> Result<bytes::Bytes> {
    let output = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;
    let data = output.body.collect().await?;
    Ok(data.into_bytes())
}
