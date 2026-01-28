use crate::errors::PDFError;
use anyhow::Result;
use lopdf::Document;

/// Validates PDF/A-3 format
pub struct PDFA3Validator;

impl PDFA3Validator {
    pub fn validate(pdf_bytes: &[u8]) -> Result<(), PDFError> {

        let pdf_string = String::from_utf8_lossy(pdf_bytes);
        let is_pdfa3 = pdf_string.contains("<pdfaid:part>3</pdfaid:part>");

        if !is_pdfa3 {
            return Err(PDFError::NotPDFA3);
        }

        Ok(())
    }
}

/// Extracts embedded files from PDF documents
pub struct EmbeddedFilesExtractor;

impl EmbeddedFilesExtractor {

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

/// Carve out XML content from PDF bytes using lopdf
pub fn carveout_xml_from_pdf(pdf_bytes: &[u8]) -> Result<Vec<String>> {
    use std::io::Cursor;
    let cursor = Cursor::new(pdf_bytes);
    let doc = Document::load_from(cursor)?;

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
}