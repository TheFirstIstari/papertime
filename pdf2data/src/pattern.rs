use anyhow::Result;

/// Build service pattern diagram data per station.
///
/// For M0 this is a stub. Implementation begins in M4.
pub fn build_pattern_data(
    _tables: &[crate::types::TableData],
    _routes: &[crate::types::RouteEntry],
    _stations: &[crate::types::StationEntry],
) -> Result<()> {
    println!("   ⏳ Pattern stub — no data generated (M4 implements)");
    Ok(())
}
