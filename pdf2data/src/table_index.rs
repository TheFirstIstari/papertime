use crate::types::TableEntry;
use anyhow::Result;

/// Build the table index from parsed timetable data.
///
/// For each table: number, name, region, operators, station CRS list,
/// service count, day periods, route map availability, gap flag.
/// Includes the 29 missing tables as gap: true entries.
///
/// For M0 this is a stub. Implementation begins in M1.
pub fn build_table_index(_tables: &[crate::types::TableData]) -> Result<Vec<TableEntry>> {
    let entries: Vec<TableEntry> = Vec::new();
    println!("   ⏳ Table-index stub — 0 entries (M1 implements)");
    Ok(entries)
}
