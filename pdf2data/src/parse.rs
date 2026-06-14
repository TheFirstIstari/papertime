use crate::types::TableData;
use anyhow::Result;

/// Parse timetable service records from extracted PDF text.
///
/// This is the most complex module in the pipeline. Handles:
/// - Header parsing (table number, route name, period)
/// - Day-period block splitting (MF / SAT / SUN)
/// - Column-to-operator mapping
/// - Station name + CRS code + arrival/departure time extraction
/// - Page-break merging (trains spanning pages)
/// - Direction detection via lat/lng comparison
/// - 29 missing tables → gap markers
///
/// For M0 this is a stub. Implementation begins in M1.
pub fn parse_timetables(extracted: &[crate::types::ExtractedPdf]) -> Result<Vec<TableData>> {
    let tables: Vec<TableData> = extracted
        .iter()
        .map(|pdf| TableData {
            table: pdf.table_number.clone(),
            name: String::new(),
            period: String::new(),
            operators: Vec::new(),
            days: vec!["MF".to_string()],
            stations: Vec::new(),
            services: Vec::new(),
            gap: false,
        })
        .collect();

    println!("   ⏳ Parse.rs stub — {} tables stubbed (M1 implements full parser)", tables.len());
    Ok(tables)
}
