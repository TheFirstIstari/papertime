use lopdf::Document;
use std::path::Path;

fn main() {
    let root = Path::new("../Route table maps - separate PDFs");
    let samples = [
        "London North West Route/Table 051 Map.pdf",
        "London North West Route/Table 001 Map.pdf", // won't exist, fine
        "Anglia route/Table 001 Map.pdf",
        "Scotland Route/Table 216 Map.pdf",
        "Western Route/Table 116 Map.pdf",
        "London North East Route/Table 026 Map.pdf",
    ];
    for rel in &samples {
        let path = root.join(rel);
        match Document::load(&path) {
            Ok(doc) => {
                let pages = doc.get_pages();
                let size = std::fs::metadata(&path).map(|m| m.len()/1024).unwrap_or(0);
                if let Ok(text) = doc.extract_text(&[1]) {
                    let stations: Vec<&str> = text.lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty() 
                            && l.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '&' || c == '\'' || c == '-')
                            && l.len() > 2)
                        .collect();
                    println!("✅ {} ({}kB, {}p) — {} stations" , rel, size, pages.len(), stations.len());
                    if stations.len() > 3 {
                        println!("   {} … {}", stations[0], stations[stations.len()-1]);
                    }
                }
            }
            Err(e) => println!("❌ {}: {:?}", rel, e),
        }
    }
}
