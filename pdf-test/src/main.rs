use lopdf::Document;

fn main() {
    let pdf_paths = [
        "../Timetable PDFs/Table 002.pdf",
        "../Timetable PDFs/Table 051.pdf",
        "../Timetable PDFs/Table 161.pdf"
    ];

    for path in &pdf_paths {
        match Document::load(path) {
            Ok(doc) => {
                let pages = doc.get_pages();
                println!("✅ {}: {} pages loaded", path, pages.len());
                
                for (page_num, _obj_id) in pages.iter().take(3) {
                    match doc.extract_text(&[*page_num]) {
                        Ok(text) => {
                            let preview: String = text.chars().take(200).collect();
                            let oneline = preview.replace('\n', " | ");
                            println!("  Page {}: {} chars: {}...", 
                                page_num, text.len(), oneline);
                        }
                        Err(e) => println!("  Page {}: extract error: {:?}", page_num, e),
                    }
                }
            }
            Err(e) => println!("❌ {}: load error: {:?}", path, e),
        }
    }
}
