use serde::{Deserialize, Serialize};

/// A single extracted PDF's contents
pub struct ExtractedPdf {
    pub filename: String,
    pub table_number: String,
    pub pages: Vec<String>,
    pub path: String,
}

/// Station entry in the master index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationEntry {
    pub id: String,          // CRS code (e.g. "EUS")
    pub name: String,        // Display name
    pub aliases: Vec<String>,
    pub tables: Vec<String>, // Table numbers this station appears in
    pub routes: Vec<String>, // Route IDs
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    #[serde(rename = "type")]
    pub station_type: String, // "terminal", "major", "interchange", "minor", "airport"
}

/// A single service record (one train run)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStop {
    pub station: String,         // CRS code
    pub arr: Option<u16>,        // Minutes past midnight, or null for origin
    pub dep: Option<u16>,        // Minutes past midnight, or null for destination
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,              // Train ID (e.g. "1A01")
    pub headcode: String,        // Operating headcode
    pub operator: String,        // Operator code (e.g. "VT")
    pub days: Vec<String>,       // ["MF"], ["SAT"], ["SUN"], or combinations
    pub direction: String,       // "northbound", "southbound", etc.
    pub stops: Vec<ServiceStop>,
}

/// Parsed data for one timetable table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub table: String,           // "001", "002", etc.
    pub name: String,
    pub period: String,
    pub operators: Vec<OperatorInfo>,
    pub days: Vec<String>,       // Day sets available
    pub stations: Vec<String>,   // CRS codes in order
    pub services: Vec<Service>,
    pub gap: bool,               // true = data unavailable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorInfo {
    pub code: String,
    pub name: String,
    pub color: String,
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

/// Route map data
#[derive(Debug, Clone)]
pub struct RouteMap {
    pub table: String,           // "001", "066", etc.
    pub region: String,          // "Anglia route", "London North West", etc.
    pub stations: Vec<String>,   // Station names in route order
    pub filename: String,
}

/// Route index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub id: String,
    pub name: String,
    pub region: String,
    pub tables: Vec<String>,
    pub stations: Vec<String>,
    pub station_order_source: String, // "route_map" or "inferred"
}

/// Marey chart data entry (one route)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MareyStationPoint {
    pub name: String,
    pub crs: String,
    pub mileage: f64,
    #[serde(rename = "type")]
    pub station_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MareyStop {
    pub station: String,  // CRS
    pub arr: Option<u16>,
    pub dep: Option<u16>,
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
pub struct MareyData {
    pub route: String,
    pub route_id: String,
    pub stations: Vec<MareyStationPoint>,
    pub services: Vec<MareyService>,
}

/// Service pattern data entry (one station)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternBranch {
    pub name: String,
    pub split_station: String,
    pub stations: Vec<PatternStation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStation {
    pub name: String,
    pub crs: String,
    pub mileage: f64,
    #[serde(rename = "type")]
    pub station_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRoute {
    pub name: String,
    pub legs: Vec<PatternLeg>,
    pub operators: Vec<OperatorInfo>,
    pub services: Vec<PatternService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLeg {
    pub direction: String,
    pub stations: Vec<PatternStation>,
    pub branches: Vec<PatternBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternService {
    pub id: String,
    pub operator: String,
    pub direction: String,
    pub calling_pattern: std::collections::HashMap<String, PatternCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCall {
    pub stop: bool,
    pub dep: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternData {
    pub station: String,
    pub crs: String,
    pub mileage: f64,
    pub routes: Vec<PatternRoute>,
}
