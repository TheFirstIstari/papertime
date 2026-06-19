use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::*;

/// Build route index by grouping tables that share stations
pub fn build(tables: &[TableEntry]) -> Vec<RouteEntry> {
    // Group tables by shared station patterns
    // Tables with significant station overlap are grouped into routes
    let mut route_groups: Vec<Vec<usize>> = Vec::new();
    let mut assigned: Vec<bool> = vec![false; tables.len()];

    for i in 0..tables.len() {
        if assigned[i] {
            continue;
        }

        let mut group = vec![i];
        assigned[i] = true;

        for j in (i + 1)..tables.len() {
            if assigned[j] {
                continue;
            }

            // Check station overlap
            let stations_a: std::collections::HashSet<&str> =
                tables[i].stations.iter().map(|s| s.as_str()).collect();
            let stations_b: std::collections::HashSet<&str> =
                tables[j].stations.iter().map(|s| s.as_str()).collect();

            let overlap: Vec<&&str> = stations_a.intersection(&stations_b).collect();
            let min_len = stations_a.len().min(stations_b.len());

            // If >50% overlap, group together
            if min_len > 0 && overlap.len() * 2 >= min_len {
                group.push(j);
                assigned[j] = true;
            }
        }

        route_groups.push(group);
    }

    let mut routes: Vec<RouteEntry> = route_groups
        .into_iter()
        .enumerate()
        .map(|(idx, group)| {
            let route_id = format!("route_{:03}", idx + 1);
            let mut all_stations: Vec<String> = Vec::new();
            let mut route_tables: Vec<String> = Vec::new();

            for &table_idx in &group {
                let table = &tables[table_idx];
                route_tables.push(table.table.clone());
                for station in &table.stations {
                    if !all_stations.contains(station) {
                        all_stations.push(station.clone());
                    }
                }
            }

            RouteEntry {
                id: route_id,
                name: format!("Route {}", idx + 1),
                region: "Unknown".to_string(),
                tables: route_tables,
                stations: all_stations,
                station_order_source: "darwin".to_string(),
            }
        })
        .collect();

    routes.sort_by(|a, b| a.id.cmp(&b.id));
    routes
}

/// Write route-index.json
pub fn write(routes: &[RouteEntry], output_dir: &Path) -> anyhow::Result<()> {
    let path = output_dir.join("route-index.json");
    let json = serde_json::to_string_pretty(routes)?;
    fs::write(&path, json)?;
    log::info!("Wrote {} routes to {}", routes.len(), path.display());
    Ok(())
}
