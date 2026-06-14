use lopdf::Document;
use std::path::Path;

fn main() {
    let root = Path::new("../Route table maps - separate PDFs");
    let samples = [
        "Anglia route/Table 001 Map.pdf",
        "Anglia route/Table 002 Map.pdf",
        "London North West Route/Table 066 Map.pdf",
        "London North West Route/Table 051 Map.pdf",
    ];
    for rel in &samples {
        let path = root.join(rel);
        match Document::load(&path) {
            Ok(doc) => {
                let pages = doc.get_pages();
                println!("✅ {} ({}p, {}kB)", rel, pages.len(), 
                    std::fs::metadata(&path).map(|m| m.len()/1024).unwrap_or(0));
                for (page_num, _) in pages.iter().take(1) {
                    match doc.extract_text(&[*page_num]) {
                        Ok(text) => {
                            let preview: String = text.chars().take(300).collect();
                            let oneline = preview.replace('\n', " ");
                            println!("  {} chars: {}", text.len(), oneline);
                        }
                        Err(e) => println!("  Page {}: error: {:?}", page_num, e),
                    }
                }
            }
            Err(e) => println!("❌ {}: {:?}", rel, e),
        }
    }
}
