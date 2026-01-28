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

    /// Extract XML content from PDF streams
    pub fn extract_xml_content(pdf_bytes: &[u8]) -> Option<String> {
        let pdf_string = String::from_utf8_lossy(pdf_bytes);
        let mut stream_start = 0;

        // Try to find XML in uncompressed streams first
        while let Some(start) = pdf_string[stream_start..].find("stream") {
            let absolute_start = stream_start + start + 6;

            if let Some(end_offset) = pdf_string[absolute_start..].find("endstream") {
                let stream_content = &pdf_string[absolute_start..absolute_start + end_offset];
                let trimmed = stream_content.trim();

                // Check for XML markers in uncompressed content
                if trimmed.contains("<?xml")
                    || trimmed.contains("<Invoice")
                    || trimmed.contains("<rsm:CrossIndustryInvoice")
                    || trimmed.contains("<ubl:Invoice")
                {
                    return Some(trimmed.to_string());
                }

                stream_start = absolute_start + end_offset;
            } else {
                break;
            }
        }

        // Try alternative approach: Look for EmbeddedFile objects with /Subtype /text#2Fxml
        // or search for the actual embedded file data between stream/endstream markers
        // that follow a filespec object
        if let Some(xml_pos) = pdf_string.find("/Subtype /text#2Fxml")
            .or_else(|| pdf_string.find("/Subtype/text#2Fxml"))
            .or_else(|| pdf_string.find("text/xml"))
        {
            // Found an XML file reference, now find the associated stream
            if let Some(stream_start) = pdf_string[xml_pos..].find("stream\n").or_else(|| pdf_string[xml_pos..].find("stream\r")) {
                let absolute_start = xml_pos + stream_start + 7;
                if let Some(end_offset) = pdf_string[absolute_start..].find("endstream") {
                    let stream_content = &pdf_string[absolute_start..absolute_start + end_offset];
                    let trimmed = stream_content.trim();
                    
                    // Check if it looks like XML
                    if trimmed.starts_with("<?xml") || trimmed.starts_with("<") {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_parser_find_pattern() {
        let data = b"Hello World";
        assert_eq!(PDFParser::find_pattern(data, b"World"), Some(6));
        assert_eq!(PDFParser::find_pattern(data, b"Rust"), None);
    }

    #[test]
    fn test_validator_invalid_pdf() {
        let invalid = b"Not a PDF";
        assert!(PDFA3Validator::validate(invalid).is_err());
    }

    #[test]
    fn test_embedded_files_extractor_empty() {
        let data = b"%PDF-1.7\nNo embedded files here";
        let files = EmbeddedFilesExtractor::find_embedded_files(data);
        assert!(files.is_empty());
    }
}
