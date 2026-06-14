use anyhow::Result;

/// Fetch station coordinates from OpenStreetMap Overpass API.
///
/// For M0 this is a stub. Implementation begins in M1 after station index is built.
pub fn fetch_coordinates(_stations: &mut [crate::types::StationEntry]) -> Result<()> {
    println!("   ⏳ OSM stub — no coordinates fetched (M1 implements)");
    Ok(())
}
