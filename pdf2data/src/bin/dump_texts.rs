/// Extract text from all PDFs using pymupdf (Python) as a subprocess.
///
/// Outputs raw text files to raw-text/timetable/ and raw-text/route-maps/.
/// These are the input for the Rust parse.rs module.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let base = Path::new(".."); // pdf2data/ -> PaperTime/
    let timetable_dir = base.join("Timetable PDFs");
    let route_map_dir = base.join("Route table maps - separate PDFs");
    let out_dir = base.join("raw-text");

    fs::create_dir_all(out_dir.join("timetable")).unwrap();
    fs::create_dir_all(out_dir.join("route-maps")).unwrap();

    // Write the Python extraction script
    let py_script = base.join("pdf2data/extract_text.py");
    fs::write(&py_script, PY_EXTRACT_SCRIPT).unwrap();

    println!("=== Timetable PDFs ===");
    let tt_count = dump_pdfs(&timetable_dir, out_dir.join("timetable").as_ref(), &py_script, false);

    println!("\n=== Route Map PDFs ===");
    let rm_count = dump_pdfs(&route_map_dir, out_dir.join("route-maps").as_ref(), &py_script, true);

    // Clean up temp script
    fs::remove_file(&py_script).ok();

    println!(
        "\n✅ Done. {} timetable PDFs, {} route map PDFs extracted.",
        tt_count, rm_count
    );
}

fn dump_pdfs(dir: &Path, out_base: &Path, py_script: &Path, recursive: bool) -> usize {
    let mut entries: Vec<PathBuf> = Vec::new();
    if recursive {
        collect_pdfs(dir, &mut entries);
    } else {
        entries = fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "pdf").unwrap_or(false))
            .collect();
    }
    entries.sort();

    let mut count = 0;
    let mut errors = 0;

    for path in &entries {
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let out_path = out_base.join(format!("{}.txt", stem));

        let output = Command::new("python3")
            .arg(py_script)
            .arg(path.to_str().unwrap())
            .arg(out_path.to_str().unwrap())
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let size = fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
                if size > 0 {
                    count += 1;
                    println!("  ✅ {}: {} B", stem, size);
                } else {
                    println!("  ⚠️  {}: empty output", stem);
                    errors += 1;
                }
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("  ❌ {}: {}", stem, stderr.lines().next().unwrap_or(""));
                errors += 1;
            }
            Err(e) => {
                eprintln!("  ❌ {}: python3 failed: {}", stem, e);
                errors += 1;
            }
        }
    }

    println!("  {} OK, {} errors", count, errors);
    count
}

fn collect_pdfs(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_pdfs(&path, out);
            } else if path.extension().map(|e| e == "pdf").unwrap_or(false) {
                out.push(path);
            }
        }
    }
}

const PY_EXTRACT_SCRIPT: &str = r#"#!/usr/bin/env python3
"""Extract text from a PDF using pymupdf, output page-separated text."""
import sys
import fitz

def main():
    if len(sys.argv) != 3:
        print("Usage: extract_text.py <input.pdf> <output.txt>", file=sys.stderr)
        sys.exit(1)

    pdf_path = sys.argv[1]
    out_path = sys.argv[2]

    doc = fitz.open(pdf_path)
    parts = []
    for i, page in enumerate(doc):
        parts.append(f"=== PAGE {i + 1} ===\n")
        parts.append(page.get_text())
        parts.append("\n")

    with open(out_path, "w") as f:
        f.write("\n".join(parts))

    print(f"  {len(doc)} pages, {sum(len(p) for p in parts)} chars")

if __name__ == "__main__":
    main()
"#;
