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
    let mut current_tiploc = String::new();
    let mut current_crs = String::new();
    let mut current_name = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                match tag_str {
                    "Location" | "location" => {
                        current_tiploc.clear();
                        current_crs.clear();
                        current_name.clear();
                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())?;
                            let value = attr.unescape_value()?;
                            match key {
                                "tpl" | "tiploc" => current_tiploc = value.to_string(),
                                "crs" => current_crs = value.to_string(),
                                "name" | "locationName" => current_name = value.to_string(),
                                _ => {}
                            }
                        }
                        if !current_tiploc.is_empty() {
                            if !current_crs.is_empty() {
                                tiploc_to_crs.insert(current_tiploc.clone(), current_crs.clone());
                            }
                            if !current_name.is_empty() {
                                tiploc_to_name.insert(current_tiploc.clone(), current_name.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                if !current_tiploc.is_empty() {
                    if !current_crs.is_empty() {
                        tiploc_to_crs.insert(current_tiploc.clone(), current_crs.clone());
                    }
                    if !current_name.is_empty() {
                        tiploc_to_name.insert(current_tiploc.clone(), current_name.clone());
                    }
                }
                current_tiploc.clear();
                current_crs.clear();
                current_name.clear();
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    log::info!(
        "Reference data: {} TIPLOC→CRS mappings",
        tiploc_to_crs.len()
    );
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
    let mut current_schedule: Option<DarwinSchedule> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                if tag_str == "schedule" {
                    current_schedule = Some(parse_schedule_element(e, tiploc_to_crs, tiploc_to_name));
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                if tag_str == "schedule" {
                    if let Some(schedule) = current_schedule.take() {
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

/// Parse a single <schedule> XML element
fn parse_schedule_element(
    e: &quick_xml::events::BytesStart,
    tiploc_to_crs: &HashMap<String, String>,
    tiploc_to_name: &HashMap<String, String>,
) -> DarwinSchedule {
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
    let mut locations = Vec::new();

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

    // Parse child location elements from attributes
    // Darwin schedule elements contain child <OR>, <IP>, <DT>, <PP> elements
    // For now, we parse the schedule attributes. Full location parsing would
    // require reading child elements in a second pass.

    // Build locations from the schedule's calling pattern
    // This is simplified — real implementation would parse child elements
    for attr in e.attributes() {
        if let Ok(attr) = attr {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let value = attr.unescape_value().unwrap_or_default();
            // Location data is in child elements, not attributes
            // This is a placeholder for the full implementation
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
        locations,
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
