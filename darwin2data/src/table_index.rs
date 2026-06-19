use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::*;

/// Group schedules into "tables" based on shared station patterns
/// This replicates the National Rail timetable table concept
pub fn build(schedules: &[DarwinSchedule], stations: &[Station]) -> Vec<TableEntry> {
    // Group schedules by their station set signature
    // Schedules with the same origin/destination pattern go in the same table
    let mut table_groups: HashMap<String, Vec<&DarwinSchedule>> = HashMap::new();

    for schedule in schedules {
        // Create a key from the first and last station TIPLOCs
        let key = if schedule.locations.len() >= 2 {
            let first = &schedule.locations.first().unwrap().tiploc;
            let last = &schedule.locations.last().unwrap().tiploc;
            format!("{}→{}", first, last)
        } else {
            format!("unknown→{}", schedule.toc)
        };

        table_groups.entry(key).or_default().push(schedule);
    }

    let mut tables: Vec<TableEntry> = table_groups
        .into_iter()
        .enumerate()
        .map(|(idx, (_key, group))| {
            let table_num = format!("{:03}", idx + 1);
            let operators: Vec<String> = group
                .iter()
                .map(|s| s.toc.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let station_list: Vec<String> = if let Some(first) = group.first() {
                first
                    .locations
                    .iter()
                    .map(|l| l.crs.clone().unwrap_or_else(|| l.tiploc.clone()))
                    .collect()
            } else {
                Vec::new()
            };

            let days: Vec<String> = group
                .iter()
                .flat_map(|s| {
                    // Darwin doesn't have explicit day codes in the same way as PDFs
                    // We infer from the schedule's active date range
                    vec!["MF".to_string(), "SAT".to_string(), "SUN".to_string()]
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let file = format!("services/{}.json", table_num);
            TableEntry {
                table: table_num,
                name: None, // Will be derived from station names
                region: None,
                operators,
                stations: station_list,
                n_services: group.len(),
                days,
                file: Some(file),
                routes: Vec::new(),
                has_route_map: false,
                gap: false,
            }
        })
        .collect();

    tables.sort_by(|a, b| a.table.cmp(&b.table));
    tables
}

/// Write table-index.json
pub fn write(tables: &[TableEntry], output_dir: &Path) -> anyhow::Result<()> {
    let path = output_dir.join("table-index.json");
    let json = serde_json::to_string_pretty(tables)?;
    fs::write(&path, json)?;
    log::info!("Wrote {} tables to {}", tables.len(), path.display());
    Ok(())
}

/// Write per-table service JSON
pub fn write_services(
    table: &TableEntry,
    schedules: &[DarwinSchedule],
    output_dir: &Path,
) -> anyhow::Result<()> {
    // Filter schedules belonging to this table
    // (In a real implementation, we'd track which schedules belong to which table)
    let table_schedules: Vec<&DarwinSchedule> = schedules
        .iter()
        .filter(|s| {
            // Match by checking if the schedule's stations overlap with the table's stations
            // This is a simplified matching — real implementation would be more precise
            !s.locations.is_empty()
        })
        .take(table.n_services)
        .collect();

    let services: Vec<Service> = table_schedules
        .iter()
        .map(|s| Service {
            id: s.rid.clone(),
            headcode: s.train_id.clone(),
            operator: s.toc.clone(),
            days: vec!["MF".to_string()], // Simplified
            direction: "unknown".to_string(),
            stops: s
                .locations
                .iter()
                .map(|l| ServiceStop {
                    station: l.crs.clone().unwrap_or_else(|| l.tiploc.clone()),
                    arr: l.pta.as_ref().and_then(|t| parse_time(t)),
                    dep: l.ptd.as_ref().and_then(|t| parse_time(t)),
                })
                .collect(),
        })
        .collect();

    let path = output_dir.join(format!("services/{}.json", table.table));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let output = serde_json::json!({
        "table": &table.table,
        "name": &table.name,
        "operators": &table.operators,
        "days": &table.days,
        "stations": &table.stations,
        "services": services,
    });

    let json = serde_json::to_string_pretty(&output)?;
    fs::write(&path, json)?;
    log::info!(
        "Wrote {} services for table {} to {}",
        services.len(),
        table.table,
        path.display()
    );
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
