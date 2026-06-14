use crate::types::RouteMap;
use anyhow::Result;

/// Parse route map PDF texts to extract station names in order.
pub fn parse_route_maps(extracted: &[crate::types::ExtractedPdf]) -> Result<Vec<RouteMap>> {
    let mut route_maps = Vec::new();

    for pdf in extracted {
        // Determine region from parent directory path
        let region = pdf
            .path
            .split('/')
            .nth(1) // After "Route table maps - separate PDFs/"
            .unwrap_or("Unknown")
            .to_string();

        // Extract station names from all pages (route maps are 1 page usually)
        let mut stations: Vec<String> = Vec::new();
        let skip_words = [
            "legend", "version", "scale", "produced", "produced by",
            "qa by", "job no", "official", "use type", "date",
            "miles", "system operator", "national rail timetable",
            "timetable route", "corporate gis", "network rail",
            "disclaims", "warranty", "no warranty",
        ];

        for text in &pdf.pages {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.len() < 3 {
                    continue;
                }
                // Skip lines with non-alpha content (symbols, numbers, etc.)
                if !trimmed
                    .chars()
                    .all(|c| c.is_alphabetic() || c.is_whitespace() || c == '&' || c == '\'' || c == '-')
                {
                    continue;
                }
                // Skip metadata lines
                let lower = trimmed.to_lowercase();
                if skip_words.iter().any(|w| lower.starts_with(w)) {
                    continue;
                }
                // Skip lines that are too long (likely metadata paragraphs)
                if trimmed.len() > 60 {
                    continue;
                }
                stations.push(trimmed.to_string());
            }
        }

        if !stations.is_empty() {
            println!(
                "   🗺️  Table {} [{}]: {} stations extracted",
                pdf.table_number, region, stations.len()
            );
            route_maps.push(RouteMap {
                table: pdf.table_number.clone(),
                region,
                stations,
                filename: pdf.filename.clone(),
            });
        }
    }

    Ok(route_maps)
}
