use crate::types::{RouteEntry, RouteMap, StationEntry, TableData};
use anyhow::Result;
use std::collections::HashSet;

/// Build the route index by grouping tables that share stations.
///
/// Primary: route-map directory structure gives groupings.
/// Secondary: Jaccard similarity clustering on timetable station sets.
pub fn build_route_index(
    route_maps: &[RouteMap],
    tables: &[TableData],
    stations: &[StationEntry],
) -> Result<Vec<RouteEntry>> {
    let mut routes: Vec<RouteEntry> = Vec::new();
    let mut route_id_counter = 0u32;

    // Group tables by route map region
    let mut region_tables: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for rm in route_maps {
        region_tables
            .entry(rm.region.clone())
            .or_default()
            .push(rm.table.clone());
    }

    // Create a route per region
    let mut used_tables: HashSet<String> = HashSet::new();

    for (region, table_nums) in &region_tables {
        let region_tables_sorted = {
            let mut v = table_nums.clone();
            v.sort();
            v
        };

        // Collect all stations across tables in this region
        let mut all_region_stations: Vec<String> = Vec::new();
        let mut station_set: HashSet<String> = HashSet::new();
        let mut operators: Vec<String> = Vec::new();
        let mut op_set: HashSet<String> = HashSet::new();

        for table_num in &region_tables_sorted {
            if used_tables.contains(table_num) {
                continue;
            }
            if let Some(table) = tables.iter().find(|t| &t.table == table_num) {
                for crs in &table.stations {
                    if station_set.insert(crs.clone()) {
                        all_region_stations.push(crs.clone());
                    }
                }
                for op in &table.operators {
                    if op_set.insert(op.code.clone()) {
                        operators.push(op.code.clone());
                    }
                }
                used_tables.insert(table_num.clone());
            }
        }

        if !all_region_stations.is_empty() {
            // Try to find a route name from the first table with a name
            let route_name = tables
                .iter()
                .find(|t| &t.table == region_tables_sorted.first().unwrap_or(&String::new()))
                .map(|t| t.name.clone())
                .unwrap_or_else(|| region.clone());

            // Determine station order source
            let station_order_source = if !route_maps.is_empty() {
                "route_map".to_string()
            } else {
                "inferred".to_string()
            };

            let route_id = format!("r{}", route_id_counter);
            let station_order_source_clone = station_order_source.clone();
            let region_clone = region.clone();
            let route_name_clone = route_name.clone();

            routes.push(RouteEntry {
                id: format!("r{}", route_id_counter),
                name: route_name,
                region: region.clone(),
                tables: region_tables_sorted.clone(),
                stations: all_region_stations,
                station_order_source,
            });
            route_id_counter += 1;
        }
    }

    // Handle tables not in any route map: group by shared stations using Jaccard
    let unmapped_tables: Vec<&TableData> = tables
        .iter()
        .filter(|t| !used_tables.contains(&t.table) && !t.gap)
        .collect();

    if !unmapped_tables.is_empty() {
        // Simple approach: group tables that share >50% of stations
        let mut grouped = vec![false; unmapped_tables.len()];

        for i in 0..unmapped_tables.len() {
            if grouped[i] {
                continue;
            }

            let mut group_tables = vec![unmapped_tables[i].table.clone()];
            let station_set_i: HashSet<String> =
                unmapped_tables[i].stations.iter().cloned().collect();
            grouped[i] = true;

            for j in (i + 1)..unmapped_tables.len() {
                if grouped[j] {
                    continue;
                }
                let station_set_j: HashSet<String> =
                    unmapped_tables[j].stations.iter().cloned().collect();
                let intersection: HashSet<_> = station_set_i.intersection(&station_set_j).collect();
                let union: HashSet<_> = station_set_i.union(&station_set_j).collect();
                let jaccard = intersection.len() as f64 / union.len() as f64;

                if jaccard > 0.5 && station_set_i.len() > 2 {
                    group_tables.push(unmapped_tables[j].table.clone());
                    grouped[j] = true;
                }
            }

            // Create route for this group
            let mut all_stations: Vec<String> = Vec::new();
            let mut station_set: HashSet<String> = HashSet::new();
            let mut all_operators: Vec<String> = Vec::new();
            let mut op_set: HashSet<String> = HashSet::new();

            for table_num in &group_tables {
                if let Some(table) = tables.iter().find(|t| &t.table == table_num) {
                    for crs in &table.stations {
                        if station_set.insert(crs.clone()) {
                            all_stations.push(crs.clone());
                        }
                    }
                    for op in &table.operators {
                        if op_set.insert(op.code.clone()) {
                            all_operators.push(op.code.clone());
                        }
                    }
                }
            }

            // Find best name
            let name = unmapped_tables[i].name.clone();
            let tables_sorted = {
                let mut v = group_tables;
                v.sort();
                v
            };

            routes.push(RouteEntry {
                id: format!("r{}", route_id_counter),
                name: if name.is_empty() {
                    format!("Route {}", route_id_counter)
                } else {
                    name
                },
                region: "Derived".to_string(),
                tables: tables_sorted,
                stations: all_stations,
                station_order_source: "inferred".to_string(),
            });
            route_id_counter += 1;
        }
    }

    Ok(routes)
}
