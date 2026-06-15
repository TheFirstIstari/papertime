use crate::types::StationEntry;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;

const OSM_CACHE_PATH: &str = "../../.osm_cache.json";

pub fn fetch_coordinates(_stations: &mut [StationEntry]) -> Result<()> {
    println!("   OSM fetch: stub (returning Ok)");
    Ok(())
}
