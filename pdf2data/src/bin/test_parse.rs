fn main() {
    let text = std::fs::read_to_string("../raw-text/timetable/Table 002.txt").expect("read failed");
    let page_count = text.matches("=== PAGE").count();
    println!("Table 002: {} pages", page_count);
    let pages: Vec<&str> = text.split("=== PAGE").collect();
    if pages.len() > 1 {
        let p1 = pages[1].trim();
        println!("Page 1 (first 800 chars):");
        println!("{}", &p1[..800.min(p1.len())]);
    }
}
