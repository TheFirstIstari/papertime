use lopdf::Document;
use std::path::Path;

fn main() {
    let root = Path::new("../Route table maps - separate PDFs");
    let mut multi_page = 0;
    let mut count = 0;
    
    for region in std::fs::read_dir(root).unwrap() {
        let region = region.unwrap();
        if !region.path().is_dir() { continue; }
        for entry in std::fs::read_dir(region.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("pdf") { continue; }
            if let Ok(doc) = Document::load(&path) {
                let pages = doc.get_pages();
                if pages.len() > 1 {
                    multi_page += 1;
                    if multi_page <= 3 {
                        println!("   Multi-page: {} ({}p)", path.display(), pages.len());
                    }
                }
            }
            count += 1;
        }
    }
    println!("Total: {}, multi-page: {}", count, multi_page);
}
