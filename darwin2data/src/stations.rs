use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::*;

/// Build station index from parsed schedules
pub fn build_index(schedules: &[DarwinSchedule]) -> Vec<Station> {
    let mut station_map: HashMap<String, Station> = HashMap::new();

    for schedule in schedules {
        for loc in &schedule.locations {
            let crs = loc.crs.clone().unwrap_or_else(|| loc.tiploc.clone());
            let name = loc.name.clone().unwrap_or_else(|| loc.tiploc.clone());

            station_map
                .entry(crs.clone())
                .and_modify(|s| {
                    if s.name.len() < name.len() {
                        s.name = name.clone();
                    }
                })
                .or_insert(Station {
                    id: crs.clone(),
                    name,
                    tiploc: loc.tiploc.clone(),
                    aliases: Vec::new(),
                    tables: Vec::new(),
                    routes: Vec::new(),
                    lat: None,
                    lng: None,
                    station_type: "minor".to_string(),
                });
        }
    }

    let mut stations: Vec<Station> = station_map.into_values().collect();
    stations.sort_by(|a, b| a.id.cmp(&b.id));
    stations
}

/// Write stations.json
pub fn write(stations: &[Station], output_dir: &Path) -> anyhow::Result<()> {
    let path = output_dir.join("stations.json");
    let json = serde_json::to_string_pretty(stations)?;
    fs::write(&path, json)?;
    log::info!("Wrote {} stations to {}", stations.len(), path.display());
    Ok(())
}
