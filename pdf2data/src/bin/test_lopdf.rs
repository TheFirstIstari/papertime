use lopdf::Document;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 {
        args[1].clone()
    } else {
        "../Timetable PDFs/Table 002.pdf".to_string()
    };

    match Document::load(&path) {
        Ok(doc) => {
            let pages = doc.get_pages();
            println!("Pages: {}", pages.len());
            for (page_num, _) in pages.iter() {
                match doc.extract_text(&[*page_num]) {
                    Ok(text) => println!("Page {}: {} chars", page_num, text.len()),
                    Err(e) => println!("Page {}: ERROR: {:?}", page_num, e),
                }
            }
        }
        Err(e) => eprintln!("Load error: {:?}", e),
    }
}
