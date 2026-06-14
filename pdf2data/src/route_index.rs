use crate::types::RouteEntry;
use anyhow::Result;

/// Build the route index by grouping tables that share stations.
///
/// Primary: route-map directory structure gives groupings.
/// Secondary: Jaccard similarity clustering on timetable station sets.
///
/// For M0 this is a stub. Implementation begins in M1.
pub fn build_route_index(
    _route_maps: &[crate::types::RouteMap],
    _tables: &[crate::types::TableData],
) -> Result<Vec<RouteEntry>> {
    let entries: Vec<RouteEntry> = Vec::new();
    println!("   ⏳ Route-index stub — 0 entries (M1 implements)");
    Ok(entries)
}
