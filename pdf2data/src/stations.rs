use crate::types::{StationEntry, TableData};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Build the master station index from all parsed tables.
///
/// For each station: CRS code, display name, aliases,
/// list of tables + routes it appears in, station type.
pub fn build_station_index(
    tables: &[TableData],
    route_maps: &[crate::types::RouteMap],
) -> Result<Vec<StationEntry>> {
    let mut station_map: HashMap<String, StationEntry> = HashMap::new();
    let mut station_names: HashMap<String, String> = HashMap::new(); // CRS -> best name

    // Collect station names and table memberships from parsed tables
    for table in tables {
        if table.gap {
            continue;
        }
        for crs in &table.stations {
            station_map
                .entry(crs.clone())
                .or_insert_with(|| StationEntry {
                    id: crs.clone(),
                    name: String::new(),
                    aliases: Vec::new(),
                    tables: Vec::new(),
                    routes: Vec::new(),
                    lat: None,
                    lng: None,
                    station_type: "minor".to_string(),
                });
        }
    }

    // Extract station names from route maps (these have full names)
    for route_map in route_maps {
        for station_name in &route_map.stations {
            // Try to find CRS for this station name
            // For now, just store the name
            station_names.insert(station_name.clone(), station_name.clone());
        }
    }

    // Second pass: assign best names from table data
    for table in tables {
        if table.gap {
            continue;
        }
        for service in &table.services {
            for stop in &service.stops {
                if let Some(entry) = station_map.get_mut(&stop.station) {
                    if entry.name.is_empty() {
                        entry.name = stop.station.clone();
                    }
                }
            }
        }
    }

    // Build table membership lists
    for table in tables {
        if table.gap {
            continue;
        }
        for crs in &table.stations {
            if let Some(entry) = station_map.get_mut(crs) {
                if !entry.tables.contains(&table.table) {
                    entry.tables.push(table.table.clone());
                }
            }
        }
    }

    // Determine station types based on connectivity
    for (_, entry) in station_map.iter_mut() {
        if entry.tables.len() > 5 {
            entry.station_type = "major".to_string();
        } else if entry.tables.len() > 2 {
            entry.station_type = "interchange".to_string();
        }
    }

    let mut stations: Vec<StationEntry> = station_map.into_values().collect();
    stations.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(stations)
}
