use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use crate::types::*;

/// Pattern diagram data for a station
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationPattern {
    pub station: String,
    pub station_name: String,
    pub n_services: usize,
    pub branches: Vec<PatternBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternBranch {
    pub next_stop: Option<String>,
    pub next_stop_name: String,
    pub destination: String,
    pub destination_tiploc: String,
    pub frequency: usize,
    pub operators: Vec<String>,
    pub operator_color: String,
    pub services: Vec<PatternService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternService {
    pub id: String,
    pub operator: String,
    pub headcode: String,
    pub dep: Option<u16>,
    pub arr: Option<u16>,
    pub days: Vec<String>,
}

/// Build service pattern data for each station.
/// Groups services by their next stop after this station, then by ultimate destination.
pub fn build_pattern_data(
    stations: &[StationIndex],
    tiploc_to_name: &HashMap<String, String>,
) -> Vec<(String, StationPattern)> {
    let mut results = Vec::new();

    for station in stations {
        if station.services.is_empty() {
            continue;
        }

        // For each service, find the next stop after this station
        // Group by (next_stop, destination)
        let mut branch_map: HashMap<(Option<String>, String), Vec<&ServiceRef>> = HashMap::new();

        for svc in &station.services {
            let calls = &svc.calls;
            // Find this station's position in the calls
            let station_pos = calls.iter().position(|c| c.crs == station.id);

            if let Some(pos) = station_pos {
                let next_call = calls.get(pos + 1);
                let next_stop = next_call.map(|c| c.crs.clone());
                let dest = svc.destination.clone();

                branch_map.entry((next_stop, dest)).or_default().push(svc);
            }
        }

        let mut branches: Vec<PatternBranch> = branch_map
            .into_iter()
            .map(|((next_stop, destination), services)| {
                let next_stop_name = next_stop
                    .as_ref()
                    .and_then(|crs| {
                        // Try to find name from service calls
                        services
                            .iter()
                            .flat_map(|s| s.calls.iter())
                            .find(|c| c.crs == *crs)
                            .map(|c| c.name.clone())
                    })
                    .or_else(|| {
                        next_stop
                            .as_ref()
                            .and_then(|crs| tiploc_to_name.get(crs).cloned())
                    })
                    .unwrap_or_else(|| next_stop.clone().unwrap_or_default());

                let mut operators: Vec<String> = services
                    .iter()
                    .map(|s| s.operator.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                operators.sort();

                let operator_color =
                    operator_color(operators.first().map(|s| s.as_str()).unwrap_or(""));

                let pattern_services: Vec<PatternService> = services
                    .iter()
                    .map(|s| {
                        let call = s.calls.iter().find(|c| c.crs == station.id);
                        PatternService {
                            id: s.id.clone(),
                            operator: s.operator.clone(),
                            headcode: s.headcode.clone(),
                            dep: call.and_then(|c| c.dep),
                            arr: call.and_then(|c| c.arr),
                            days: s.days.clone(),
                        }
                    })
                    .collect();

                PatternBranch {
                    next_stop,
                    next_stop_name,
                    destination: services
                        .first()
                        .map(|s| s.destination_name.clone())
                        .unwrap_or_default(),
                    destination_tiploc: destination,
                    frequency: pattern_services.len(),
                    operators,
                    operator_color,
                    services: pattern_services,
                }
            })
            .collect();

        // Sort branches by frequency descending
        branches.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        let station_name = tiploc_to_name
            .get(&station.id)
            .cloned()
            .unwrap_or_else(|| station.name.clone());

        results.push((
            station.id.clone(),
            StationPattern {
                station: station.id.clone(),
                station_name,
                n_services: station.services.len(),
                branches,
            },
        ));
    }

    results
}

/// Write pattern data files
pub fn write_pattern_data(
    patterns: &[(String, StationPattern)],
    output_dir: &Path,
) -> anyhow::Result<()> {
    let patterns_dir = output_dir.join("patterns");
    fs::create_dir_all(&patterns_dir)?;

    for (id, data) in patterns {
        let path = patterns_dir.join(format!("{}.json", id));
        // Use compact JSON to save space
        let json = serde_json::to_string(data)?;
        fs::write(&path, json)?;
    }

    log::info!("Wrote {} pattern files", patterns.len());
    Ok(())
}

fn operator_color(op: &str) -> String {
    match op {
        "CC" | "XC" | "SE" | "LE" => "#009E73",
        "EM" | "GR" | "AW" => "#CC79A7",
        "LO" | "ME" => "#E86A10",
        "VT" | "HX" | "HT" => "#E32636",
        "GW" | "SR" => "#56B4E9",
        "TP" | "TL" | "LM" => "#D55E00",
        "NT" | "SW" | "CH" => "#0072B2",
        "SN" | "GN" => "#F0E442",
        "GC" => "#882255",
        "GX" => "#56B4E9",
        "LF" => "#E86A10",
        "XR" => "#D55E00",
        "CS" => "#E32636",
        _ => "#64748b",
    }
    .to_string()
}
