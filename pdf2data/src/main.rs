mod extract;
mod marey;
mod osm;
mod parse;
mod pattern;
mod route_index;
mod route_maps;
mod stations;
mod table_index;
mod types;

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let base = PathBuf::from("..");
    let raw_tt = base.join("raw-text/timetable");
    let raw_rm = base.join("raw-text/route-maps");
    let data_dir = base.join("static/data");
    std::fs::create_dir_all(&data_dir.join("services"))?;

    // Phase 1: Route maps
    println!("Phase 1: Route maps...");
    let route_maps = if raw_rm.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&raw_rm)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|e| e == "txt").unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        let texts: Vec<(String, String)> = entries
            .iter()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let text = std::fs::read_to_string(e.path()).ok()?;
                Some((name, text))
            })
            .collect();
        let rms = route_maps::parse_route_map_texts(&texts)?;
        println!("  {} route maps", rms.len());
        rms
    } else {
        Vec::new()
    };

    let route_map_tables: std::collections::HashSet<String> =
        route_maps.iter().map(|r| r.table.clone()).collect();

    // Phase 2: Timetables — stream, write JSON immediately, keep only lightweight metadata
    println!("Phase 2: Timetables...");
    let mut station_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut table_count = 0usize;
    let mut service_count = 0usize;
    let mut gap_count = 0usize;

    parse::parse_timetables(&raw_tt, |table| {
        // Update station -> table mappings (lightweight)
        for stn in &table.stations {
            station_map
                .entry(stn.clone())
                .or_default()
                .push(table.table.clone());
        }

        if table.gap {
            gap_count += 1;
        } else {
            // Write per-table service JSON immediately
            if !table.services.is_empty() {
                let path = data_dir.join(format!("services/{}.json", table.table));
                std::fs::write(&path, serde_json::to_string_pretty(&table)?)?;
                service_count += table.services.len();
            }
            table_count += 1;
        }

        Ok(())
    })?;

    println!(
        "  {} tables parsed, {} services, {} gaps",
        table_count, service_count, gap_count
    );
    println!("  {} unique stations", station_map.len());

    // Phase 3: Station index
    println!("Phase 3: Station index...");
    let mut station_entries: Vec<types::StationEntry> = station_map
        .iter()
        .map(|(crs, tables)| {
            let stype = if tables.len() > 5 {
                "major"
            } else if tables.len() > 2 {
                "interchange"
            } else {
                "minor"
            };
            types::StationEntry {
                id: crs.clone(),
                name: crs.clone(),
                aliases: Vec::new(),
                tables: tables.clone(),
                routes: Vec::new(),
                lat: None,
                lng: None,
                station_type: stype.to_string(),
            }
        })
        .collect();
    station_entries.sort_by(|a, b| a.id.cmp(&b.id));
    std::fs::write(
        data_dir.join("stations.json"),
        serde_json::to_string_pretty(&station_entries)?,
    )?;
    println!("  {} stations written", station_entries.len());

    // Phase 4: OSM coordinates
    println!("Phase 4: OSM coordinates...");
    osm::fetch_coordinates(&mut station_entries)?;
    std::fs::write(
        data_dir.join("stations.json"),
        serde_json::to_string_pretty(&station_entries)?,
    )?;

    // Phase 5: Table index (scan service JSON files)
    println!("Phase 5: Table index...");
    let mut table_entries: Vec<types::TableEntry> = Vec::new();
    for entry in std::fs::read_dir(&data_dir.join("services"))? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        let table_num = fname.trim_end_matches(".json");
        let json: types::TableData = serde_json::from_str(&std::fs::read_to_string(entry.path())?)?;
        table_entries.push(types::TableEntry {
            table: table_num.to_string(),
            name: if json.name.is_empty() { None } else { Some(json.name) },
            region: None,
            operators: json.operators.iter().map(|o| o.code.clone()).collect(),
            stations: json.stations,
            n_services: json.services.len(),
            days: json.days,
            file: Some(format!("services/{}.json", table_num)),
            routes: Vec::new(),
            has_route_map: route_map_tables.contains(table_num),
            gap: false,
        });
    }
    // Add gap tables
    for rm in &route_maps {
        if !table_entries.iter().any(|e| e.table == rm.table) {
            table_entries.push(types::TableEntry {
                table: rm.table.clone(),
                name: None,
                region: None,
                operators: Vec::new(),
                stations: Vec::new(),
                n_services: 0,
                days: Vec::new(),
                file: None,
                routes: Vec::new(),
                has_route_map: true,
                gap: true,
            });
        }
    }
    table_entries.sort_by(|a, b| a.table.cmp(&b.table));
    std::fs::write(
        data_dir.join("table-index.json"),
        serde_json::to_string_pretty(&table_entries)?,
    )?;
    println!("  {} table entries", table_entries.len());

    // Phase 6: Route index
    println!("Phase 6: Route index...");
    let tables_for_routes: Vec<types::TableData> = table_entries
        .iter()
        .filter(|e| !e.gap)
        .map(|e| types::TableData {
            table: e.table.clone(),
            name: e.name.clone().unwrap_or_default(),
            period: String::new(),
            operators: Vec::new(),
            days: e.days.clone(),
            stations: e.stations.clone(),
            services: Vec::new(),
            gap: e.gap,
        })
        .collect();
    let route_entries =
        route_index::build_route_index(&route_maps, &tables_for_routes, &station_entries)?;
    std::fs::write(
        data_dir.join("route-index.json"),
        serde_json::to_string_pretty(&route_entries)?,
    )?;
    println!("  {} routes", route_entries.len());

    println!("Done! Data in {}", data_dir.display());
    Ok(())
}
