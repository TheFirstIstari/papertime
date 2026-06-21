use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::types::*;

pub fn extract_schedules(data_dir: &Path) -> Result<(Vec<DarwinSchedule>, HashMap<String, String>, HashMap<String, String>)> {
    let mut schedules = Vec::new();
    let mut tiploc_to_crs: HashMap<String, String> = HashMap::new();
    let mut tiploc_to_name: HashMap<String, String> = HashMap::new();

    let entries = fs::read_dir(data_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if name.ends_with(".gz") && !name.ends_with(".gz.gz") {
            log::info!("Processing: {}", name);

            let file = fs::File::open(&path)?;
            let mut decoder = flate2::read::GzDecoder::new(file);
            let mut xml = String::new();
            Read::read_to_string(&mut decoder, &mut xml)?;

            if name.contains("ref") {
                parse_reference_data(&xml, &mut tiploc_to_crs, &mut tiploc_to_name)?;
            } else {
                let batch = parse_schedule_xml(&xml, &tiploc_to_crs, &mut tiploc_to_name)?;
                log::info!("  Parsed {} schedules from {}", batch.len(), name);
                schedules.extend(batch);
            }
        }
    }

    log::info!(
        "Total: {} schedules, {} stations",
        schedules.len(),
        tiploc_to_crs.len()
    );
    Ok((schedules, tiploc_to_crs, tiploc_to_name))
}

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
            Ok(Event::Empty(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                if tag_str == "LocationRef" {
                    let mut tiploc = String::new();
                    let mut crs = String::new();
                    let mut name = String::new();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = std::str::from_utf8(attr.key.as_ref())?;
                        let value = attr.unescape_value()?;
                        match key {
                            "tpl" => tiploc = value.to_string(),
                            "crs" => crs = value.to_string(),
                            "locname" => name = value.to_string(),
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
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                match tag_str {
                    "Journey" => {
                        current = Some(parse_journey_start(e));
                    }
                    "OR" | "OPOR" | "IP" | "OPIP" | "PP" | "DT" | "OPDT" => {
                        if let Some(ref mut s) = current {
                            let loc_type = match tag_str {
                                "OR" => LocationType::Origin,
                                "OPOR" => LocationType::OperationalOrigin,
                                "IP" => LocationType::Intermediate,
                                "OPIP" => LocationType::OperationalIntermediate,
                                "PP" => LocationType::Passing,
                                "DT" => LocationType::Destination,
                                "OPDT" => LocationType::OperationalDestination,
                                _ => unreachable!(),
                            };
                            let (tiploc, pta, ptd) = parse_location_attrs(e);
                            let crs = tiploc_to_crs.get(&tiploc).cloned();
                            let name = tiploc_to_name.get(&tiploc).cloned();
                            s.locations.push(DarwinLocation {
                                tiploc, crs, name, loc_type, pta, ptd,
                                wta: None, wtd: None, wtp: None,
                                act: String::new(), cancelled: false,
                            });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref())?;
                if tag_str == "Journey" {
                    if let Some(journey) = current.take() {
                        // Only include active, non-deleted passenger services
                        if journey.is_active && !journey.is_deleted {
                            schedules.push(journey);
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

fn parse_journey_start(e: &quick_xml::events::BytesStart) -> DarwinSchedule {
    let mut rid = String::new();
    let mut uid = String::new();
    let mut train_id = String::new();
    let mut ssd = String::new();
    let mut toc = String::new();
    let mut status = "P".to_string();
    let mut train_cat = "OO".to_string();
    let mut is_passenger = true;
    let mut is_active = true;
    let mut is_deleted = false;

    for attr in e.attributes() {
        if let Ok(attr) = attr {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let value = attr.unescape_value().unwrap_or_default();
            match key {
                "rid" => rid = value.to_string(),
                "uid" => uid = value.to_string(),
                "trainId" => train_id = value.to_string(),
                "ssd" => ssd = value.to_string(),
                "toc" => toc = value.to_string(),
                "status" => status = value.to_string(),
                "trainCat" => train_cat = value.to_string(),
                "isPassengerSvc" => is_passenger = value == "true",
                "isActive" => is_active = value == "true",
                "deleted" => is_deleted = value == "true",
                _ => {}
            }
        }
    }

    DarwinSchedule {
        rid, uid, train_id, rsid: None, ssd, toc, status, train_cat,
        is_passenger, is_active, is_deleted, is_charter: false,
        locations: Vec::new(),
    }
}

fn parse_location_attrs(e: &quick_xml::events::BytesStart) -> (String, Option<String>, Option<String>) {
    let mut tiploc = String::new();
    let mut pta = None;
    let mut ptd = None;

    for attr in e.attributes() {
        if let Ok(attr) = attr {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let value = attr.unescape_value().unwrap_or_default();
            match key {
                "ftl" => tiploc = value.trim().to_string(),  // TIPLOC code (with trailing spaces)
                "pta" => pta = Some(value.to_string()),
                "ptd" => ptd = Some(value.to_string()),
                "wta" => pta = Some(value.to_string()),  // Use wta as fallback
                "wtd" => ptd = Some(value.to_string()),  // Use wtd as fallback
                _ => {}
            }
        }
    }

    (tiploc, pta, ptd)
}

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
