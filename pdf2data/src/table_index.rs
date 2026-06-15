use crate::types::{TableData, TableEntry};
use anyhow::Result;

/// Build the table index from parsed timetable data.
///
/// For each table: number, name, region, operators, station CRS list,
/// service count, day periods, route map availability, gap flag.
/// Includes the 29 missing tables as gap: true entries.
pub fn build_table_index(
    tables: &[TableData],
    route_maps: &[crate::types::RouteMap],
) -> Result<Vec<TableEntry>> {
    let route_map_tables: std::collections::HashSet<String> =
        route_maps.iter().map(|rm| rm.table.clone()).collect();

    let mut entries = Vec::new();

    for table in tables {
        let has_route_map = route_map_tables.contains(&table.table);

        entries.push(TableEntry {
            table: table.table.clone(),
            name: if table.name.is_empty() {
                None
            } else {
                Some(table.name.clone())
            },
            region: None, // TODO: derive from route map directory
            operators: table.operators.iter().map(|o| o.code.clone()).collect(),
            stations: table.stations.clone(),
            n_services: table.services.len(),
            days: table.days.clone(),
            file: if table.gap {
                None
            } else {
                Some(format!("services/{}.json", table.table))
            },
            routes: Vec::new(), // TODO: link to route index
            has_route_map,
            gap: table.gap,
        });
    }

    // Sort by table number
    entries.sort_by(|a, b| a.table.cmp(&b.table));

    Ok(entries)
}
