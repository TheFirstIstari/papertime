use serde::{Deserialize, Serialize};

/// Raw Darwin schedule extracted from XML
#[derive(Debug, Clone)]
pub struct DarwinSchedule {
    pub rid: String,           // RTTI unique train ID
    pub uid: String,           // Train UID
    pub train_id: String,      // Headcode (e.g. "1A01")
    pub rsid: Option<String>,  // Retail service ID
    pub ssd: String,           // Scheduled start date
    pub toc: String,           // ATOC operator code
    pub status: String,        // Service type (P=Train, B=Bus, S=Ship)
    pub train_cat: String,     // Category
    pub is_passenger: bool,
    pub is_active: bool,
    pub is_deleted: bool,
    pub is_charter: bool,
    pub locations: Vec<DarwinLocation>,
}

/// A location within a schedule (origin, calling point, destination)
#[derive(Debug, Clone)]
pub struct DarwinLocation {
    pub tiploc: String,       // TIPLOC code
    pub crs: Option<String>,  // CRS code (if available)
    pub name: Option<String>, // Station name
    pub loc_type: LocationType,
    pub pta: Option<u16>,    // Public time of arrival (minutes past midnight)
    pub ptd: Option<u16>,    // Public time of departure (minutes past midnight)
    pub wta: Option<String>, // Working time of arrival (HH:MM:SS)
    pub wtd: Option<String>, // Working time of departure (HH:MM:SS)
    pub wtp: Option<String>, // Working time of passing (HH:MM:SS)
    pub act: String,         // Activity codes
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocationType {
    Origin,         // OR
    OperationalOrigin, // OPOR
    Intermediate,   // IP
    OperationalIntermediate, // OPIP
    Passing,        // PP
    Destination,    // DT
    OperationalDestination, // OPDT
}

/// Processed service record for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub headcode: String,
    pub operator: String,
    pub days: Vec<String>,
    pub direction: String,
    pub stops: Vec<ServiceStop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStop {
    pub station: String,    // CRS code
    pub arr: Option<u16>,   // Minutes past midnight
    pub dep: Option<u16>,   // Minutes past midnight
}

/// Station entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: String,           // CRS code
    pub name: String,
    pub tiploc: String,       // TIPLOC code
    pub aliases: Vec<String>,
    pub tables: Vec<String>,
    pub routes: Vec<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub station_type: String,
}

/// Table index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableEntry {
    pub table: String,
    pub name: Option<String>,
    pub region: Option<String>,
    pub operators: Vec<String>,
    pub stations: Vec<String>,
    pub n_services: usize,
    pub days: Vec<String>,
    pub file: Option<String>,
    pub routes: Vec<String>,
    pub has_route_map: bool,
    pub gap: bool,
}

/// Route index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub id: String,
    pub name: String,
    pub region: String,
    pub tables: Vec<String>,
    pub stations: Vec<String>,
    pub station_order_source: String,
}

/// Marey chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MareyData {
    pub route: String,
    pub route_id: String,
    pub stations: Vec<MareyStation>,
    pub services: Vec<MareyService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MareyStation {
    pub name: String,
    pub crs: String,
    pub mileage: f64,
    #[serde(rename = "type")]
    pub station_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MareyService {
    pub id: String,
    pub operator: String,
    pub direction: String,
    pub days: Vec<String>,
    pub stops: Vec<MareyStop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MareyStop {
    pub station: String,
    pub arr: Option<u16>,
    pub dep: Option<u16>,
}

// Station-centric indexing types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationIndex {
    pub id: String,
    pub name: String,
    pub tiploc: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub station_type: String,
    pub services: Vec<ServiceRef>,
    pub operators: Vec<String>,
    pub destinations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRef {
    pub id: String,
    pub headcode: String,
    pub operator: String,
    pub origin: String,
    pub origin_name: String,
    pub destination: String,
    pub destination_name: String,
    pub calls: Vec<CallRef>,
    pub days: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRef {
    pub crs: String,
    pub arr: Option<u16>,
    pub dep: Option<u16>,
}
