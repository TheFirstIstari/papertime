use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::types::*;

/// Extract and parse all schedule XML files from the Darwin ZIP
pub fn extract_schedules(zip_path: &Path) -> Result<Vec<DarwinSchedule>> {
    let file = File::open(zip_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    let mut schedules = Vec::new();
    let mut tiploc_to_crs: HashMap<String, String> = HashMap::new();
    let mut tiploc_to_name: HashMap<String, String> = HashMap::new();

    // First pass: extract reference data
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.contains("ref/") && name.ends_with(".xml") {
            let mut xml = String::new();
            std::io::Read::read_to_string(&mut entry, &mut xml)?;
            parse_reference_data(&xml, &mut tiploc_to_crs, &mut tiploc_to_name)?;
        }
    }

    // Second pass: extract schedule data
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.contains("schedule/") && name.ends_with(".xml") {
            let mut xml = String::new();
            std::io::Read::read_to_string(&mut entry, &mut xml)?;
            let batch = parse_schedule_xml(&xml, &tiploc_to_crs, &tiploc_to_name)?;
            schedules.extend(batch);
        }
    }

    Ok(schedules)
}

/// Parse reference data XML for TIPLOC→CRS mapping
fn parse_reference_data(
    xml: &str,
    tiploc_to_crs: &mut HashMap<String, String>,
    tiploc_to_name: &mut HashMap<String, String>,
) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                if tag_str == "Location" || tag_str == "location" {
                    let mut tiploc = String::new();
                    let mut crs = String::new();
                    let mut name = String::new();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = std::str::from_utf8(attr.key.as_ref())?;
                        let value = attr.unescape_value()?;
                        match key {
                            "tpl" | "tiploc" => tiploc = value.to_string(),
                            "crs" => crs = value.to_string(),
                            "name" | "locationName" => name = value.to_string(),
                            _ => {}
                        }
                    }
                    if !tiploc.is_empty() {
                        if !crs.is_empty() {
                            tiploc_to_crs.insert(tiploc.clone(), crs);
                        }
                        if !name.is_empty() {
                            tiploc_to_name.insert(tiploc, name);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    log::info!("Reference data: {} TIPLOC→CRS mappings", tiploc_to_crs.len());
    Ok(())
}

/// Parse a schedule XML file containing multiple <schedule> elements
fn parse_schedule_xml(
    xml: &str,
    tiploc_to_crs: &HashMap<String, String>,
    tiploc_to_name: &HashMap<String, String>,
) -> Result<Vec<DarwinSchedule>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut schedules = Vec::new();
    let mut buf = Vec::new();
    let mut current: Option<DarwinSchedule> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                match tag_str {
                    "schedule" => {
                        current = Some(parse_schedule_start(e));
                    }
                    "OR" => {
                        if let Some(ref mut s) = current {
                            s.locations.push(parse_location(e, LocationType::Origin, tiploc_to_crs, tiploc_to_name));
                        }
                    }
                    "OPOR" => {
                        if let Some(ref mut s) = current {
                            s.locations.push(parse_location(e, LocationType::OperationalOrigin, tiploc_to_crs, tiploc_to_name));
                        }
                    }
                    "IP" => {
                        if let Some(ref mut s) = current {
                            s.locations.push(parse_location(e, LocationType::Intermediate, tiploc_to_crs, tiploc_to_name));
                        }
                    }
                    "OPIP" => {
                        if let Some(ref mut s) = current {
                            s.locations.push(parse_location(e, LocationType::OperationalIntermediate, tiploc_to_crs, tiploc_to_name));
                        }
                    }
                    "PP" => {
                        if let Some(ref mut s) = current {
                            s.locations.push(parse_location(e, LocationType::Passing, tiploc_to_crs, tiploc_to_name));
                        }
                    }
                    "DT" => {
                        if let Some(ref mut s) = current {
                            s.locations.push(parse_location(e, LocationType::Destination, tiploc_to_crs, tiploc_to_name));
                        }
                    }
                    "OPDT" => {
                        if let Some(ref mut s) = current {
                            s.locations.push(parse_location(e, LocationType::OperationalDestination, tiploc_to_crs, tiploc_to_name));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                // Self-closing location elements
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                let loc_type = match tag_str {
                    "OR" => LocationType::Origin,
                    "OPOR" => LocationType::OperationalOrigin,
                    "IP" => LocationType::Intermediate,
                    "OPIP" => LocationType::OperationalIntermediate,
                    "PP" => LocationType::Passing,
                    "DT" => LocationType::Destination,
                    "OPDT" => LocationType::OperationalDestination,
                    _ => continue,
                };
                if let Some(ref mut s) = current {
                    s.locations.push(parse_location(e, loc_type, tiploc_to_crs, tiploc_to_name));
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                if tag_str == "schedule" {
                    if let Some(schedule) = current.take() {
                        if schedule.is_active && !schedule.is_deleted && schedule.is_passenger {
                            schedules.push(schedule);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(schedules)
}

/// Parse schedule element attributes
fn parse_schedule_start(e: &quick_xml::events::BytesStart) -> DarwinSchedule {
    let mut rid = String::new();
    let mut uid = String::new();
    let mut train_id = String::new();
    let mut rsid = None;
    let mut ssd = String::new();
    let mut toc = String::new();
    let mut status = "P".to_string();
    let mut train_cat = "OO".to_string();
    let mut is_passenger = true;
    let mut is_active = true;
    let mut is_deleted = false;
    let mut is_charter = false;

    for attr in e.attributes() {
        if let Ok(attr) = attr {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let value = attr.unescape_value().unwrap_or_default();
            match key {
                "rid" => rid = value.to_string(),
                "uid" => uid = value.to_string(),
                "trainId" => train_id = value.to_string(),
                "rsid" => rsid = Some(value.to_string()),
                "ssd" => ssd = value.to_string(),
                "toc" => toc = value.to_string(),
                "status" => status = value.to_string(),
                "trainCat" => train_cat = value.to_string(),
                "isPassengerSvc" => is_passenger = value == "true",
                "isActive" => is_active = value == "true",
                "deleted" => is_deleted = value == "true",
                "isCharter" => is_charter = value == "true",
                _ => {}
            }
        }
    }

    DarwinSchedule {
        rid,
        uid,
        train_id,
        rsid,
        ssd,
        toc,
        status,
        train_cat,
        is_passenger,
        is_active,
        is_deleted,
        is_charter,
        locations: Vec::new(),
    }
}

/// Parse a location element (OR, IP, DT, PP, etc.)
fn parse_location(
    e: &quick_xml::events::BytesStart,
    loc_type: LocationType,
    tiploc_to_crs: &HashMap<String, String>,
    tiploc_to_name: &HashMap<String, String>,
) -> DarwinLocation {
    let mut tiploc = String::new();
    let mut pta = None;
    let mut ptd = None;
    let mut wta = None;
    let mut wtd = None;
    let mut wtp = None;
    let mut act = String::new();
    let mut cancelled = false;

    for attr in e.attributes() {
        if let Ok(attr) = attr {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let value = attr.unescape_value().unwrap_or_default();
            match key {
                "tpl" => tiploc = value.to_string(),
                "pta" => pta = Some(value.to_string()),
                "ptd" => ptd = Some(value.to_string()),
                "wta" => wta = Some(value.to_string()),
                "wtd" => wtd = Some(value.to_string()),
                "wtp" => wtp = Some(value.to_string()),
                "act" => act = value.to_string(),
                "can" => cancelled = value == "true",
                _ => {}
            }
        }
    }

    let crs = tiploc_to_crs.get(&tiploc).cloned();
    let name = tiploc_to_name.get(&tiploc).cloned();

    DarwinLocation {
        tiploc,
        crs,
        name,
        loc_type,
        pta,
        ptd,
        wta,
        wtd,
        wtp,
        act,
        cancelled,
    }
}

/// Parse time string (HH:MM or HH:MM:SS) to minutes past midnight
pub fn parse_time(time_str: &str) -> Option<u16> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() >= 2 {
        let hours: u16 = parts[0].parse().ok()?;
        let minutes: u16 = parts[1].parse().ok()?;
        Some(hours * 60 + minutes)
    } else {
        None
    }
}
