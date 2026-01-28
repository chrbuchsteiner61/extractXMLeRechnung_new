use thiserror::Error;

#[derive(Error, Debug)]
pub enum PDFError {
    #[error("Not a valid PDF file")]
    InvalidPDF,
    #[error("PDF is not in PDF/A-3 format")]
    NotPDFA3,
    #[error("No embedded XML-file")]
    NoXMLFile,
    #[error("XML file found but could not extract content")]
    ExtractionFailed,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
