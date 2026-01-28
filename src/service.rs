use crate::errors::PDFError;
use crate::models::{ErrorResponse, SuccessResponse};
use crate::extract_xml::{EmbeddedFilesExtractor, PDFA3Validator};

/// Main business logic for eRechnung processing
pub struct ERechnungService;

impl ERechnungService {
    /// Process a PDF file and extract XML content
    pub fn process_pdf(pdf_bytes: Vec<u8>) -> Result<SuccessResponse, ErrorResponse> {
        // Validate PDF/A-3 format
        if let Err(e) = PDFA3Validator::validate(&pdf_bytes) {
            return Err(ErrorResponse {
                file_status: e.to_string(),
                embedded_files: None,
            });
        }

        // Find catalog (validate structure)
        let _catalog = EmbeddedFilesExtractor::find_catalog(&pdf_bytes).ok_or_else(|| {
            ErrorResponse {
                file_status: PDFError::NoCatalog.to_string(),
                embedded_files: None,
            }
        })?;

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

        // Extract XML content
        let xml_content =
            EmbeddedFilesExtractor::extract_xml_content(&pdf_bytes).ok_or_else(|| {
                ErrorResponse {
                    file_status: PDFError::ExtractionFailed.to_string(),
                    embedded_files: Some(embedded_files.join(", ")),
                }
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
            xml_content,
            xml_filename: xml_file.clone(),
        })
    }
}
