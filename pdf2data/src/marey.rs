use anyhow::Result;

/// Build Marey chart coordinate data per route.
///
/// For M0 this is a stub. Implementation begins in M3.
pub fn build_marey_data(
    _tables: &[crate::types::TableData],
    _routes: &[crate::types::RouteEntry],
    _stations: &[crate::types::StationEntry],
) -> Result<()> {
    println!("   ⏳ Marey stub — no data generated (M3 implements)");
    Ok(())
}
