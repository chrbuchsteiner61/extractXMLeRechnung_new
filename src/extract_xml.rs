use crate::errors::PDFError;

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

    /// Extract complete embedded XML file from PDF
    pub fn extract_xml_content(pdf_bytes: &[u8]) -> Option<String> {
        let pdf_string = String::from_utf8_lossy(pdf_bytes);
        
        // Look for the embedded file specification first
        if let Some(filespec_pos) = pdf_string.find("factur-x.xml") {
            // Search backwards and forwards from the filename to find the file object
            let search_start = filespec_pos.saturating_sub(2000);
            let search_end = (filespec_pos + 2000).min(pdf_string.len());
            
            // Ensure we don't have invalid bounds
            if search_start >= search_end {
                return None;
            }
            
            let search_section = &pdf_string[search_start..search_end];
            
            // Look for uncompressed stream content (try different patterns)
            if let Some(stream_start) = search_section.find("stream\n") {
                let abs_stream_start = search_start + stream_start + 7;
                
                // Make sure we don't go out of bounds
                if abs_stream_start >= pdf_string.len() {
                    return None;
                }
                
                if let Some(stream_end_offset) = pdf_string[abs_stream_start..].find("endstream") {
                    let abs_stream_end = abs_stream_start + stream_end_offset;
                    
                    // Ensure valid bounds
                    if abs_stream_end <= abs_stream_start || abs_stream_end > pdf_string.len() {
                        return None;
                    }
                    
                    let content = &pdf_string[abs_stream_start..abs_stream_end];
                    
                    // Check if content looks like XML (starts with <?xml or <)
                    let trimmed = content.trim();
                    if trimmed.starts_with("<?xml") || 
                       trimmed.starts_with("<rsm:") || 
                       trimmed.starts_with("<ubl:") ||
                       trimmed.starts_with("<Invoice") {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
        
        // If we can't find valid XML, return None to indicate extraction failed
        None
    }
}