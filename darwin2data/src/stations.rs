use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::*;

/// Index services by station and build station metadata
pub fn index_services(services: &[DarwinSchedule]) -> Vec<StationIndex> {
    let mut station_map: HashMap<String, StationIndex> = HashMap::new();

    for schedule in services {
        for loc in &schedule.locations {
            let crs = loc.crs.clone().unwrap_or_else(|| loc.tiploc.clone());
            let name = loc.name.clone().unwrap_or_else(|| loc.tiploc.clone());

            let entry = station_map.entry(crs.clone()).or_insert_with(|| StationIndex {
                id: crs.clone(),
                name: name.clone(),
                tiploc: loc.tiploc.clone(),
                lat: None,
                lng: None,
                station_type: "minor".to_string(),
                services: Vec::new(),
                operators: Vec::new(),
                destinations: Vec::new(),
            });

            if entry.name.len() < name.len() || entry.name == entry.tiploc {
                entry.name = name;
            }

            entry.services.push(ServiceRef {
                id: schedule.rid.clone(),
                headcode: schedule.train_id.clone(),
                operator: schedule.toc.clone(),
                origin: schedule
                    .locations
                    .first()
                    .map(|l| l.crs.clone().unwrap_or_else(|| l.tiploc.clone()))
                    .unwrap_or_default(),
                origin_name: schedule
                    .locations
                    .first()
                    .and_then(|l| l.name.clone())
                    .unwrap_or_default(),
                destination: schedule
                    .locations
                    .last()
                    .map(|l| l.crs.clone().unwrap_or_else(|| l.tiploc.clone()))
                    .unwrap_or_default(),
                destination_name: schedule
                    .locations
                    .last()
                    .and_then(|l| l.name.clone())
                    .unwrap_or_default(),
                calls: schedule
                    .locations
                    .iter()
                    .map(|l| CallRef {
                        crs: l.crs.clone().unwrap_or_else(|| l.tiploc.clone()),
                        arr: l.pta.clone(),
                        dep: l.ptd.clone(),
                    })
                    .collect(),
                days: vec!["MF".to_string()],
            });

            if !entry.operators.contains(&schedule.toc) {
                entry.operators.push(schedule.toc.clone());
            }

            if let Some(last) = schedule.locations.last() {
                let dest_crs = last.crs.clone().unwrap_or_else(|| last.tiploc.clone());
                if !entry.destinations.contains(&dest_crs) {
                    entry.destinations.push(dest_crs);
                }
            }
        }
    }

    let mut stations: Vec<StationIndex> = station_map.into_values().collect();
    stations.sort_by(|a, b| a.id.cmp(&b.id));
    stations
}

/// Write station-index.json
pub fn write_index(stations: &[StationIndex], output_dir: &Path) -> anyhow::Result<()> {
    let path = output_dir.join("station-index.json");
    let json = serde_json::to_string_pretty(stations)?;
    fs::write(&path, json)?;
    log::info!("Wrote {} stations to {}", stations.len(), path.display());
    Ok(())
}

/// Write per-station service files
pub fn write_station_services(stations: &[StationIndex], output_dir: &Path) -> anyhow::Result<()> {
    let services_dir = output_dir.join("services");
    fs::create_dir_all(&services_dir)?;

    for station in stations {
        let path = services_dir.join(format!("{}.json", station.id));
        let output = serde_json::json!({
            "station": &station.id,
            "name": &station.name,
            "services": &station.services,
        });
        let json = serde_json::to_string_pretty(&output)?;
        fs::write(&path, json)?;
    }

    log::info!("Wrote {} station service files", stations.len());
    Ok(())
}

/// Write per-station Marey data
pub fn write_marey_data(stations: &[StationIndex], output_dir: &Path) -> anyhow::Result<()> {
    let marey_dir = output_dir.join("marey");
    fs::create_dir_all(&marey_dir)?;

    for station in stations {
        let path = marey_dir.join(format!("{}.json", station.id));
        let output = serde_json::json!({
            "station": &station.id,
            "name": &station.name,
            "n_services": station.services.len(),
        });
        let json = serde_json::to_string_pretty(&output)?;
        fs::write(&path, json)?;
    }

    log::info!("Wrote {} Marey data files", stations.len());
    Ok(())
}
