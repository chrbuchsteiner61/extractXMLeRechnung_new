use crate::errors::PDFError;
use anyhow::Result;
use lopdf::Document;

use serde_xml_rs::from_str;

/// Low-level PDF parsing utilities
pub struct PDFParser;

impl PDFParser {
    /// Find the position of a pattern in bytes
    pub fn find_pattern(bytes: &[u8], pattern: &[u8]) -> Option<usize> {
        bytes.windows(pattern.len()).position(|window| window == pattern)
    }

    /// Extract a PDF object (dictionary) starting at a given position
    pub fn extract_object(bytes: &[u8], start_pos: usize) -> Option<String> {
        let mut depth = 0;
        let mut in_dict = false;
        let mut start = 0;

        for i in start_pos..bytes.len().saturating_sub(1) {
            if bytes[i] == b'<' && bytes[i + 1] == b'<' {
                if !in_dict {
                    start = i;
                }
                in_dict = true;
                depth += 1;
            } else if bytes[i] == b'>' && bytes[i + 1] == b'>' {
                depth -= 1;
                if depth == 0 && in_dict {
                    let end = i + 2;
                    return String::from_utf8(bytes[start..end].to_vec()).ok();
                }
            }
        }
        None
    }
}

/// Validates PDF/A-3 format
pub struct PDFA3Validator;

impl PDFA3Validator {
    pub fn validate(pdf_bytes: &[u8]) -> Result<(), PDFError> {
        if pdf_bytes.len() < 5 || &pdf_bytes[0..5] != b"%PDF-" {
            return Err(PDFError::InvalidPDF);
        }

        let pdf_string = String::from_utf8_lossy(pdf_bytes);
        let is_pdfa3 = pdf_string.contains("PDF/A-3")
            || pdf_string.contains("pdfa:part>3")
            || pdf_string.contains("pdfaid:part>3")
            || pdf_string.contains("pdfaid:conformance");

        if !is_pdfa3 {
            return Err(PDFError::NotPDFA3);
        }

        Ok(())
    }
}

/// Extracts embedded files from PDF documents
pub struct EmbeddedFilesExtractor;

impl EmbeddedFilesExtractor {
    /// Find the PDF catalog object
    pub fn find_catalog(pdf_bytes: &[u8]) -> Option<String> {
        let catalog_pattern = b"/Type /Catalog";
        let catalog_pos = PDFParser::find_pattern(pdf_bytes, catalog_pattern)?;

        let mut obj_start = catalog_pos;
        while obj_start > 2 {
            if &pdf_bytes[obj_start..obj_start + 3] == b"obj" {
                break;
            }
            obj_start = obj_start.saturating_sub(1);
        }

        PDFParser::extract_object(pdf_bytes, obj_start)
    }

    /// Find all embedded file names in the PDF
    pub fn find_embedded_files(pdf_bytes: &[u8]) -> Vec<String> {
        let pdf_string = String::from_utf8_lossy(pdf_bytes);
        let mut embedded_files = Vec::new();

        if !pdf_string.contains("/EmbeddedFiles") {
            return embedded_files;
        }

        let names_start = pdf_string.find("/Names").unwrap_or(0);
        let names_section = &pdf_string[names_start..];

        if let Some(array_start) = names_section.find('[') {
            if let Some(array_end) = names_section[array_start..].find(']') {
                let names_content = &names_section[array_start + 1..array_start + array_end];

                let mut in_string = false;
                let mut current_string = String::new();

                for ch in names_content.chars() {
                    match ch {
                        '(' => {
                            in_string = true;
                            current_string.clear();
                        }
                        ')' => {
                            if in_string && !current_string.is_empty() {
                                embedded_files.push(current_string.clone());
                            }
                            in_string = false;
                        }
                        _ if in_string => current_string.push(ch),
                        _ => {}
                    }
                }
            }
        }

        embedded_files
    }
}

/// Extracts XML content from PDF file path using lopdf
pub fn extract_xml_from_pdf(path: &str) -> Result<Vec<String>> {
    let doc = Document::load(path)?;
    extract_xml_from_document(&doc)
}

/// Extract XML content from PDF bytes using lopdf
pub fn extract_xml_from_pdf_bytes(pdf_bytes: &[u8]) -> Result<Vec<String>> {
    use std::io::Cursor;
    let cursor = Cursor::new(pdf_bytes);
    let doc = Document::load_from(cursor)?;
    extract_xml_from_document(&doc)
}

/// Helper function to extract XML from a loaded PDF document
fn extract_xml_from_document(doc: &Document) -> Result<Vec<String>> {
    let mut xml_contents = Vec::new();

    // Iterate through all objects in the PDF
    for (_, object) in doc.objects.iter() {
        if let Ok(stream) = object.as_stream() {
            // Check if it's XML metadata or embedded files
            if let Ok(decoded) = stream.decompressed_content() {
                let text = String::from_utf8_lossy(&decoded);
                if is_xml_content(&text) {
                    xml_contents.push(text.to_string());
                }
            }
        }
    }

    Ok(xml_contents)
}

/// Check if the content appears to be XML
fn is_xml_content(text: &str) -> bool {
    text.contains("<?xml") 
        || text.contains("<rdf:RDF") 
        || text.contains("<rsm:") 
        || text.contains("<ubl:") 
        || text.contains("<Invoice")
}