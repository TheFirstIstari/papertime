use crate::types::*;
use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

lazy_static::lazy_static! {
    static ref CRS_RE: Regex = Regex::new(r"\(([A-Z]{3})\)").unwrap();
    static ref TABLE_NUM_RE: Regex = Regex::new(r"Table (\d{3})").unwrap();
}

/// Parse timetable files one at a time, calling a callback for each.
/// This avoids accumulating all tables in memory.
pub fn parse_timetables<F>(raw_dir: &Path, mut on_table: F) -> Result<()>
where
    F: FnMut(TableData) -> Result<()>,
{
    let mut entries: Vec<_> = fs::read_dir(raw_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|e| e == "txt").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let total = entries.len();

    for (idx, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let fname = path.file_stem().unwrap().to_string_lossy().to_string();
        let table_num = extract_table_number(&fname);

        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("  ⚠️  Table {}: read error: {} — gap", table_num, e);
                on_table(make_gap(&table_num))?;
                continue;
            }
        };

        match parse_one_table(&text, &table_num) {
            Ok(table) => {
                if idx % 20 == 0 {
                    println!(
                        "  [{}/{}] Table {}: {} services, {} stations",
                        idx + 1,
                        total,
                        table.table,
                        table.services.len(),
                        table.stations.len()
                    );
                }
                on_table(table)?;
            }
            Err(e) => {
                eprintln!("  ⚠️  Table {}: {} — gap", table_num, e);
                on_table(make_gap(&table_num))?;
            }
        }
    }

    Ok(())
}

pub fn parse_one_table(text: &str, table_num: &str) -> Result<TableData> {
    let mut table = TableData {
        table: table_num.to_string(),
        name: String::new(),
        period: String::new(),
        operators: Vec::new(),
        days: Vec::new(),
        stations: Vec::new(),
        services: Vec::new(),
        gap: false,
    };

    let mut station_set: HashSet<String> = HashSet::new();
    let mut op_set: HashSet<String> = HashSet::new();
    let mut day_set: HashSet<String> = HashSet::new();

    let mut line_start = 0;
    let bytes = text.as_bytes();

    while line_start < bytes.len() {
        let mut page_end = line_start;
        while page_end < bytes.len() {
            if page_end + 8 <= bytes.len()
                && bytes[page_end] == b'='
                && bytes[page_end + 1] == b'='
                && bytes[page_end + 2] == b'='
                && bytes[page_end + 3] == b' '
                && bytes[page_end + 4] == b'P'
                && bytes[page_end + 5] == b'A'
                && bytes[page_end + 6] == b'G'
                && bytes[page_end + 7] == b'E'
            {
                break;
            }
            page_end += 1;
        }

        let page_text = &text[line_start..page_end];
        if let Ok(sections) = parse_page(page_text) {
            for section in sections {
                if table.name.is_empty() && !section.route_name.is_empty() {
                    table.name = section.route_name;
                }
                if table.period.is_empty() && !section.period.is_empty() {
                    table.period = section.period;
                }
                day_set.insert(section.day_period.clone());

                for op in &section.operators {
                    if op_set.insert(op.code.clone()) {
                        table.operators.push(op.clone());
                    }
                }

                for stn in &section.stations {
                    if station_set.insert(stn.crs.clone()) {
                        table.stations.push(stn.crs.clone());
                    }
                }

                table.services.extend(section.services);
            }
        }

        line_start = page_end;
        while line_start < bytes.len() && bytes[line_start] != b'\n' {
            line_start += 1;
        }
        line_start += 1;
    }

    let mut days: Vec<String> = day_set.into_iter().collect();
    days.sort();
    table.days = days;
    Ok(table)
}

fn make_gap(table_num: &str) -> TableData {
    TableData {
        table: table_num.to_string(),
        name: String::new(),
        period: String::new(),
        operators: Vec::new(),
        days: Vec::new(),
        stations: Vec::new(),
        services: Vec::new(),
        gap: true,
    }
}

fn extract_table_number(filename: &str) -> String {
    TABLE_NUM_RE
        .captures(filename)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| filename.to_string())
}

struct Section {
    day_period: String,
    route_name: String,
    period: String,
    operators: Vec<OperatorInfo>,
    stations: Vec<StationParse>,
    services: Vec<Service>,
}

struct StationParse {
    name: String,
    crs: String,
    is_departure: bool,
    is_arrival: bool,
}

fn parse_page(text: &str) -> Result<Vec<Section>> {
    let lines: Vec<&str> = text.lines().collect();
    let mut sections = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let dp = detect_day_period(lines[i]);
        if !dp.is_empty() {
            match parse_section(&lines[i..], &dp) {
                Ok((section, consumed)) => {
                    i += consumed;
                    sections.push(section);
                }
                Err(e) => {
                    eprintln!("    Section @{}: {}", i, e);
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(sections)
}

fn detect_day_period(line: &str) -> String {
    match line.trim() {
        "Mondays to Fridays" => "MF".to_string(),
        "Saturdays" => "SAT".to_string(),
        "Sundays" => "SUN".to_string(),
        _ => String::new(),
    }
}

fn parse_section(lines: &[&str], day_period: &str) -> Result<(Section, usize)> {
    let mut section = Section {
        day_period: day_period.to_string(),
        route_name: String::new(),
        period: String::new(),
        operators: Vec::new(),
        stations: Vec::new(),
        services: Vec::new(),
    };

    let mut i = 0;
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    if i < lines.len() && lines[i].trim_start().starts_with("Operator") {
        for code in parse_op_line(lines[i]) {
            section.operators.push(OperatorInfo {
                code: code.clone(),
                name: op_name(&code),
                color: op_color(&code),
            });
        }
        i += 1;
    }

    while i < lines.len() {
        let t = lines[i].trim_start();
        if t.starts_with("Days of operation")
            || t.starts_with("1st Class")
            || t.starts_with("Catering")
        {
            i += 1;
        } else {
            break;
        }
    }

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        if detect_day_period(trimmed) != "" || trimmed.starts_with("Operator") {
            break;
        }

        if let Some(caps) = CRS_RE.captures(trimmed) {
            let crs = caps[1].to_string();
            let name_end = trimmed.find('(').unwrap_or(trimmed.len());
            let name = trimmed[..name_end].trim();

            if !name.is_empty() {
                let after = &trimmed[trimmed.find(')').map(|p| p + 1).unwrap_or(0)..];
                let (dir, times) = parse_times(after);
                let is_dep = dir.contains('d');
                let is_arr = dir.contains('a');

                section.stations.push(StationParse {
                    name: name.to_string(),
                    crs: crs.clone(),
                    is_departure: is_dep,
                    is_arrival: is_arr,
                });

                for (col, time_opt) in times.iter().enumerate() {
                    if col >= section.services.len() {
                        let op_code = section
                            .operators
                            .get(col)
                            .map(|o| o.code.clone())
                            .unwrap_or_default();
                        section.services.push(Service {
                            id: format!("{}_{}", day_period, col),
                            headcode: String::new(),
                            operator: op_code,
                            days: vec![day_period.to_string()],
                            direction: String::new(),
                            stops: Vec::new(),
                        });
                    }
                    if let Some(mins) = time_opt {
                        if col < section.services.len() {
                            section.services[col].stops.push(ServiceStop {
                                station: crs.clone(),
                                arr: if is_arr { Some(*mins) } else { None },
                                dep: if is_dep { Some(*mins) } else { None },
                            });
                        }
                    }
                }

                i += 1;
                while i < lines.len() {
                    let next = lines[i].trim();
                    if next.is_empty()
                        || detect_day_period(next) != ""
                        || next.starts_with("Operator")
                        || CRS_RE.is_match(next)
                    {
                        break;
                    }
                    let (_, cont) = parse_times(next);
                    if cont.is_empty() {
                        break;
                    }
                    for (col, time_opt) in cont.iter().enumerate() {
                        if let Some(mins) = time_opt {
                            if col < section.services.len() {
                                section.services[col].stops.push(ServiceStop {
                                    station: crs.clone(),
                                    arr: if is_arr { Some(*mins) } else { None },
                                    dep: if is_dep { Some(*mins) } else { None },
                                });
                            }
                        }
                    }
                    i += 1;
                }
                continue;
            }
        }
        i += 1;
    }

    Ok((section, i))
}

fn parse_op_line(line: &str) -> Vec<String> {
    line.split_whitespace()
        .skip(1)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_times(line: &str) -> (String, Vec<Option<u16>>) {
    let t = line.trim();
    if t.is_empty() {
        return (String::new(), Vec::new());
    }
    let s = t.trim_start_matches(|c: char| !c.is_ascii_alphanumeric() && c != ' ');
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    let dir = if parts[0] == "d" || parts[0] == "a" {
        parts[0].to_string()
    } else {
        String::new()
    };
    let start = if !dir.is_empty() { 1 } else { 0 };
    let mut times = Vec::new();
    for part in parts.iter().skip(start) {
        let p = *part;
        if let Ok(m) = parse_hhmm(p) {
            times.push(Some(m));
        } else if p == "\u{2014}" || p == "-" || p == ".." {
            times.push(None);
        }
    }
    (dir, times)
}

fn parse_hhmm(s: &str) -> Result<u16, std::num::ParseIntError> {
    match s.len() {
        4 => {
            let h: u16 = s[..2].parse()?;
            let m: u16 = s[2..4].parse()?;
            Ok(h * 60 + m)
        }
        3 => {
            let h: u16 = s[..1].parse()?;
            let m: u16 = s[1..3].parse()?;
            Ok(h * 60 + m)
        }
        _ => s.parse::<u16>(),
    }
}

fn op_name(code: &str) -> String {
    match code {
        "CC" | "XC" => "CrossCountry",
        "EM" => "East Midlands Railway",
        "LO" => "London Overground",
        "VT" => "Avanti West Coast",
        "GW" => "Great Western Railway",
        "TP" => "TransPennine Express",
        "NT" => "Northern Trains",
        "SN" => "Southern",
        "SE" => "Southeastern",
        "SW" => "South Western Railway",
        "TL" => "Thameslink",
        "GR" => "Grand Central",
        "CS" => "Caledonian Sleeper",
        "HE" => "Hull Trains",
        "LD" => "Lumo",
        "ME" => "Merseyrail",
        "AW" => "Transport for Wales",
        "SR" => "ScotRail",
        "LE" => "Greater Anglia",
        "HX" => "Heathrow Express",
        "IL" => "Island Line",
        "CH" => "Chiltern Railways",
        "LM" => "West Midlands Trains",
        _ => "Unknown",
    }
    .to_string()
}

fn op_color(code: &str) -> String {
    match code {
        "CC" | "XC" | "SE" | "LE" => "#009E73",
        "EM" | "GR" | "AW" => "#CC79A7",
        "LO" | "ME" | "IL" => "#E86A10",
        "VT" | "HE" | "HX" => "#E32636",
        "GW" | "SR" | "CS" => "#56B4E9",
        "TP" | "TL" | "LM" => "#D55E00",
        "NT" | "SW" | "CH" | "LD" => "#0072B2",
        "SN" => "#F0E442",
        _ => "#999999",
    }
    .to_string()
}
