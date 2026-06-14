use lopdf::Document;
use std::path::Path;

fn main() {
    let root = Path::new("../Route table maps - separate PDFs");
    let path = root.join("London North West Route/Table 066 Map.pdf");
    let doc = Document::load(&path).unwrap();
    let text = doc.extract_text(&[1]).unwrap();
    // Print raw chars to see the format
    for (i, c) in text.char_indices() {
        if c.is_alphabetic() || c.is_whitespace() {
            print!("{}", c);
        }
    }
    println!();
}
