use crate::errors::PDFError;
use crate::models::{ErrorResponse, SuccessResponse};
use crate::extract_xml::{EmbeddedFilesExtractor, extract_xml_from_pdf_bytes};

/// Main business logic for eRechnung processing
pub struct ERechnungService;

impl ERechnungService {
    /// Process a PDF file and extract XML content
    pub fn process_pdf(pdf_bytes: Vec<u8>) -> Result<SuccessResponse, ErrorResponse> {
        // Basic PDF validation
        if pdf_bytes.len() < 5 || &pdf_bytes[0..5] != b"%PDF-" {
            return Err(ErrorResponse {
                file_status: PDFError::InvalidPDF.to_string(),
                embedded_files: None,
            });
        }

        // Find embedded files
        let embedded_files = EmbeddedFilesExtractor::find_embedded_files(&pdf_bytes);

        if embedded_files.is_empty() {
            return Err(ErrorResponse {
                file_status: PDFError::NoXMLFile.to_string(),
                embedded_files: None,
            });
        }

        // Find XML file
        let xml_file = embedded_files
            .iter()
            .find(|name| name.to_lowercase().ends_with(".xml"))
            .ok_or_else(|| ErrorResponse {
                file_status: PDFError::NoXMLFile.to_string(),
                embedded_files: Some(embedded_files.join(", ")),
            })?;

        // Extract XML content using lopdf
        let xml_contents = extract_xml_from_pdf_bytes(&pdf_bytes).map_err(|_| {
            ErrorResponse {
                file_status: PDFError::ExtractionFailed.to_string(),
                embedded_files: Some(embedded_files.join(", ")),
            }
        })?;

        // Find the best XML content (prefer one that looks like an invoice)
        let xml_content = xml_contents
            .iter()
            .find(|content| {
                content.contains("<rsm:") || 
                content.contains("<ubl:") || 
                content.contains("<Invoice")
            })
            .or_else(|| xml_contents.first())
            .ok_or_else(|| ErrorResponse {
                file_status: PDFError::ExtractionFailed.to_string(),
                embedded_files: Some(embedded_files.join(", ")),
            })?;

        // Determine status based on XML filename
        let is_facturx = xml_file.to_lowercase() == "factur-x.xml";
        let status = if is_facturx {
            "Success".to_string()
        } else {
            "XML is not Factur-x.xml".to_string()
        };

        Ok(SuccessResponse {
            file_status: status,
            embedded_files: embedded_files.join(", "),
            xml_content: xml_content.clone(),
            xml_filename: xml_file.clone(),
        })
    }
}
