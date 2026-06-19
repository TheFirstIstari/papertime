use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::*;

/// Generate Marey chart data for each route
pub fn generate(schedules: &[DarwinSchedule], routes: &[RouteEntry]) -> Vec<MareyData> {
    let mut marey_data = Vec::new();

    for route in routes {
        // Collect all schedules that belong to this route's tables
        // (Simplified — real implementation would track table→schedule mapping)
        let route_schedules: Vec<&DarwinSchedule> = schedules
            .iter()
            .filter(|s| {
                // Check if any of the schedule's stations are in this route
                s.locations.iter().any(|l| {
                    let crs = l.crs.as_ref().unwrap_or(&l.tiploc);
                    route.stations.contains(crs)
                })
            })
            .collect();

        // Build station list with mileages (simplified — would use OSM coordinates)
        let mut marey_stations: Vec<MareyStation> = route
            .stations
            .iter()
            .enumerate()
            .map(|(idx, crs)| MareyStation {
                name: crs.clone(), // Would lookup actual name
                crs: crs.clone(),
                mileage: idx as f64 * 10.0, // Placeholder — would compute from coordinates
                station_type: "minor".to_string(),
            })
            .collect();

        // Build service list
        let marey_services: Vec<MareyService> = route_schedules
            .iter()
            .map(|s| MareyService {
                id: s.rid.clone(),
                operator: s.toc.clone(),
                direction: "unknown".to_string(),
                days: vec!["MF".to_string()],
                stops: s
                    .locations
                    .iter()
                    .map(|l| MareyStop {
                        station: l.crs.clone().unwrap_or_else(|| l.tiploc.clone()),
                        arr: l.pta.as_ref().and_then(|t| parse_time(t)),
                        dep: l.ptd.as_ref().and_then(|t| parse_time(t)),
                    })
                    .collect(),
            })
            .collect();

        marey_data.push(MareyData {
            route: route.name.clone(),
            route_id: route.id.clone(),
            stations: marey_stations,
            services: marey_services,
        });
    }

    marey_data
}

/// Write Marey data files
pub fn write(marey_data: &[MareyData], output_dir: &Path) -> anyhow::Result<()> {
    let marey_dir = output_dir.join("marey");
    fs::create_dir_all(&marey_dir)?;

    for data in marey_data {
        let path = marey_dir.join(format!("{}.json", data.route_id));
        let json = serde_json::to_string_pretty(data)?;
        fs::write(&path, json)?;
    }

    log::info!("Wrote {} Marey charts to {}", marey_data.len(), marey_dir.display());
    Ok(())
}

fn parse_time(time_str: &str) -> Option<u16> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() >= 2 {
        let hours: u16 = parts[0].parse().ok()?;
        let minutes: u16 = parts[1].parse().ok()?;
        Some(hours * 60 + minutes)
    } else {
        None
    }
}
