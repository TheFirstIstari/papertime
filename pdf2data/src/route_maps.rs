use crate::types::RouteMap;
use anyhow::Result;

/// Parse route map station names from raw text.
///
/// Input: Vec<(filename, text)> where text is from pymupdf page extraction
/// Output: Vec<RouteMap> with station ordering per table
pub fn parse_route_map_texts(extracted: &[(String, String)]) -> Result<Vec<RouteMap>> {
    let mut route_maps = Vec::new();

    for (filename, text) in extracted {
        // Extract table number from filename
        let table_number = extract_table_number(filename);

        // Determine region from filename (e.g. "Table 001 Map" -> no parent dir info, but filename contains "Map")
        let region = if filename.contains("Map") {
            "Route Map".to_string()
        } else {
            "Unknown".to_string()
        };

        // Parse station names from text pages
        let mut stations: Vec<String> = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();

            // Skip page markers and metadata
            if trimmed.starts_with("=== PAGE") || trimmed.is_empty() {
                continue;
            }

            // Skip lines with non-alpha content (symbols, numbers, etc.)
            if trimmed.chars().any(|c| {
                !c.is_alphabetic() && !c.is_whitespace() && c != '&' && c != '\'' && c != '-'
            }) {
                // But allow "!(" which is a map marker followed by text
                if !trimmed.starts_with("!(") {
                    continue;
                }
            }

            // Skip short lines and metadata
            if trimmed.len() < 3 || trimmed.len() > 60 {
                continue;
            }

            // Skip known metadata keywords
            let lower = trimmed.to_lowercase();
            let skip_words = [
                "legend",
                "version",
                "scale",
                "produced",
                "job no",
                "official",
                "use type",
                "date",
                "miles",
                "system operator",
                "national rail",
                "timetable route",
                "corporate gis",
                "network rail",
                "disclaim",
                "warranty",
            ];
            if skip_words.iter().any(|w| lower.starts_with(w)) {
                continue;
            }

            // Clean up: remove leading "!(" markers
            let cleaned = trimmed
                .trim_start_matches("!(")
                .trim_start_matches('!')
                .trim();

            if cleaned.len() >= 3 {
                stations.push(cleaned.to_string());
            }
        }

        if !stations.is_empty() {
            println!(
                "   🗺️  Table {} [{}]: {} stations",
                table_number,
                region,
                stations.len()
            );
            route_maps.push(RouteMap {
                table: table_number,
                region,
                stations,
                filename: filename.clone(),
            });
        }
    }

    Ok(route_maps)
}

/// Legacy: parse RouteMap from ExtractedPdf (kept for API compat)
pub fn parse_route_maps(extracted: &[crate::types::ExtractedPdf]) -> Result<Vec<RouteMap>> {
    let mut route_maps = Vec::new();

    for pdf in extracted {
        let table_number = pdf.table_number.clone();
        let region = pdf
            .path
            .split('/')
            .nth(8) // After "Route table maps - separate PDFs/"
            .unwrap_or("Unknown")
            .to_string();

        let mut stations: Vec<String> = Vec::new();
        let skip_words = [
            "legend",
            "version",
            "scale",
            "produced",
            "produced by",
            "qa by",
            "job no",
            "official",
            "use type",
            "date",
            "miles",
            "system operator",
            "national rail timetable",
            "timetable route",
            "corporate gis",
            "network rail",
            "disclaims",
            "warranty",
            "no warranty",
        ];

        for text in &pdf.pages {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.len() < 3 {
                    continue;
                }
                // Skip lines with non-alpha content
                if !trimmed.chars().all(|c| {
                    c.is_alphabetic() || c.is_whitespace() || c == '&' || c == '\'' || c == '-'
                }) {
                    continue;
                }
                let lower = trimmed.to_lowercase();
                if skip_words.iter().any(|w| lower.starts_with(w)) {
                    continue;
                }
                if trimmed.len() > 60 {
                    continue;
                }
                stations.push(trimmed.to_string());
            }
        }

        if !stations.is_empty() {
            route_maps.push(RouteMap {
                table: table_number,
                region,
                stations,
                filename: pdf.filename.clone(),
            });
        }
    }

    Ok(route_maps)
}

fn extract_table_number(filename: &str) -> String {
    let re = regex::Regex::new(r"Table (\d{3})").unwrap();
    re.captures(filename)
        .map(|c| c[1].to_string())
        .unwrap_or_default()
}
