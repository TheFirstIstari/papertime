mod extract;
mod marey;
mod osm;
mod parse;
mod pattern;
mod route_index;
mod route_maps;
mod stations;
mod table_index;
mod types;

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        anyhow::bail!("Usage: pdf2data <timetable_pdfs_dir> <route_maps_dir>");
    }

    let timetable_dir = PathBuf::from(&args[1]);
    let route_maps_dir = PathBuf::from(&args[2]);

    // Verify directories exist
    if !timetable_dir.exists() {
        anyhow::bail!("Timetable directory not found: {}", timetable_dir.display());
    }
    if !route_maps_dir.exists() {
        anyhow::bail!("Route maps directory not found: {}", route_maps_dir.display());
    }

    println!("📄 Extracting timetable PDFs from: {}", timetable_dir.display());
    let timetable_texts = extract::extract_pdfs(&timetable_dir)?;
    println!("   ✅ {} timetable PDFs extracted", timetable_texts.len());

    println!("\n🗺️  Extracting route map PDFs from: {}", route_maps_dir.display());
    let route_map_texts = extract::extract_pdfs_recursive(&route_maps_dir)?;
    println!("   ✅ {} route map PDFs extracted", route_map_texts.len());

    println!("\n🗺️  Parsing route maps...");
    let route_maps = route_maps::parse_route_maps(&route_map_texts)?;
    println!("   ✅ {} route maps parsed", route_maps.len());

    println!("\n📊 Parsing timetables...");
    let parsed_tables = parse::parse_timetables(&timetable_texts)?;
    println!("   ✅ {} tables parsed", parsed_tables.len());

    println!("\n🏢 Building station index...");
    let mut stations = stations::build_station_index(&parsed_tables)?;
    println!("   ✅ {} stations indexed", stations.len());

    println!("\n📋 Building table index...");
    let table_index = table_index::build_table_index(&parsed_tables)?;
    println!("   ✅ {} table entries", table_index.len());

    println!("\n🛤️  Building route index...");
    let route_index = route_index::build_route_index(&route_maps, &parsed_tables)?;
    println!("   ✅ {} routes", route_index.len());

    println!("\n🌍 Fetching OSM coordinates...");
    osm::fetch_coordinates(&mut stations)?;
    let with_coords = stations.iter().filter(|s| s.lat.is_some()).count();
    println!("   ✅ {} stations with coordinates", with_coords);

    println!("\n📈 Building Marey chart data...");
    marey::build_marey_data(&parsed_tables, &route_index, &stations)?;

    println!("\n🔌 Building service pattern data...");
    pattern::build_pattern_data(&parsed_tables, &route_index, &stations)?;

    println!("\n✅ Data pipeline complete!");
    println!("   Tables:     {}", parsed_tables.len());
    println!("   Routes:     {}", route_index.len());
    println!("   Stations:   {}", stations.len());
    println!("   Route maps: {}", route_maps.len());

    Ok(())
}
