use lopdf::Document;

fn main() {
    let path = std::env::args().nth(1).expect("provide pdf path");
    let doc = Document::load(&path).expect("failed to load");
    let pages = doc.get_pages();
    println!("Pages: {}", pages.len());
    for (page_num, page_id) in pages.iter() {
        // Try extract_text
        match doc.extract_text(&[*page_num]) {
            Ok(text) => println!("Page {}: {} chars", page_num, text.len()),
            Err(e) => println!("Page {}: extract_text FAILED: {:?}", page_num, e),
        }
        // Try getting page content
        match doc.get_page_content(*page_id) {
            Ok(content) => println!("  content stream: {} bytes", content.len()),
            Err(e) => println!("  content stream FAILED: {:?}", e),
        }
    }
}
