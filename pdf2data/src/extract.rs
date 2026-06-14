use crate::types::ExtractedPdf;
use anyhow::Result;
use lopdf::Document;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

/// Extract text from all PDFs in a flat directory
pub fn extract_pdfs(dir: &Path) -> Result<Vec<ExtractedPdf>> {
    let mut results = Vec::new();
    let re = Regex::new(r"Table (\d{3})")?;

    for entry in WalkDir::new(dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "pdf").unwrap_or(false) {
            let filename = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let table_number = re
                .captures(&filename)
                .map(|c| c[1].to_string())
                .unwrap_or_default();

            match Document::load(path) {
                Ok(doc) => {
                    let pages = doc.get_pages();
                    let mut page_texts = Vec::new();
                    for (page_num, _) in pages.iter() {
                        match doc.extract_text(&[*page_num]) {
                            Ok(text) => page_texts.push(text),
                            Err(e) => {
                                eprintln!("  ⚠️  {} page {}: {:?}", filename, page_num, e);
                                page_texts.push(String::new());
                            }
                        }
                    }
                    let total_chars: usize = page_texts.iter().map(|t| t.len()).sum();
                    println!(
                        "   📄 {}: {} pages, {} chars",
                        filename, pages.len(), total_chars
                    );
                    results.push(ExtractedPdf {
                        filename,
                        table_number,
                        pages: page_texts,
                        path: path.to_string_lossy().to_string(),
                    });
                }
                Err(e) => {
                    eprintln!("  ❌ {}: load error: {:?}", filename, e);
                }
            }
        }
    }

    Ok(results)
}

/// Extract text from all PDFs in a directory tree (recursive — for route maps)
pub fn extract_pdfs_recursive(dir: &Path) -> Result<Vec<ExtractedPdf>> {
    let mut results = Vec::new();
    let re = Regex::new(r"Table (\d{3})")?;

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "pdf").unwrap_or(false) {
            let filename = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let table_number = re
                .captures(&filename)
                .map(|c| c[1].to_string())
                .unwrap_or_default();

            match Document::load(path) {
                Ok(doc) => {
                    let pages = doc.get_pages();
                    let mut page_texts = Vec::new();
                    for (page_num, _) in pages.iter() {
                        match doc.extract_text(&[*page_num]) {
                            Ok(text) => page_texts.push(text),
                            Err(e) => {
                                eprintln!("  ⚠️  {} page {}: {:?}", filename, page_num, e);
                                page_texts.push(String::new());
                            }
                        }
                    }
                    let parent_dir = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default();
                    println!(
                        "   🗺️  {} [{}]: {} pages",
                        filename, parent_dir, pages.len()
                    );
                    results.push(ExtractedPdf {
                        filename,
                        table_number,
                        pages: page_texts,
                        path: path.to_string_lossy().to_string(),
                    });
                }
                Err(e) => {
                    eprintln!("  ❌ {}: load error: {:?}", filename, e);
                }
            }
        }
    }

    Ok(results)
}
