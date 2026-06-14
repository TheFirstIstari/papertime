use crate::types::StationEntry;
use anyhow::Result;
use std::collections::HashSet;

/// Build the master station index from all parsed tables.
///
/// For each station: CRS code, display name, aliases,
/// list of tables + routes it appears in, station type.
///
/// For M0 this is a stub. Implementation begins in M1.
pub fn build_station_index(_tables: &[crate::types::TableData]) -> Result<Vec<StationEntry>> {
    let stations: Vec<StationEntry> = Vec::new();
    println!("   ⏳ Stations stub — 0 stations (M1 implements)");
    Ok(stations)
}
