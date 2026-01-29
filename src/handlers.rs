use actix_multipart::Multipart;
use actix_web::{HttpResponse, Result as ActixResult};
use futures_util::stream::StreamExt;
use std::io::Write;

use crate::erechnung_pdf_service::ERechnungService;
use crate::models::ErrorResponse;

extern crate serde_json;

/// Handler for extracting XML from PDF/A-3 files and returning as downloadable file
pub async fn extract_xml_file(mut payload: Multipart) -> ActixResult<HttpResponse> {
    let mut pdf_data: Vec<u8> = Vec::new();

    // Read multipart data
    while let Some(item) = payload.next().await {
        let mut field = item?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            pdf_data.write_all(&data)?;
        }
    }

    // Validate that a file was uploaded
    if pdf_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            file_status: "No file uploaded".to_string(),
            embedded_files: None,
        }));
    }

    // Process the PDF
    match ERechnungService::process_pdf(pdf_data) {
        Ok(response) => {
            // Return the XML file as a downloadable attachment
            Ok(HttpResponse::Ok()
                .content_type("application/xml")
                .append_header(("Content-Disposition", 
                    format!("attachment; filename=\"{}\"", response.xml_filename)))
                .body(response.xml_content))
        }
        Err(error) => Ok(HttpResponse::BadRequest().json(error)),
    }
}

/// Handler for extracting XML from PDF/A-3 files
pub async fn extract_xml(mut payload: Multipart) -> ActixResult<HttpResponse> {
    let mut pdf_data: Vec<u8> = Vec::new();

    // Read multipart data
    while let Some(item) = payload.next().await {
        let mut field = item?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            pdf_data.write_all(&data)?;
        }
    }

    // Validate that a file was uploaded
    if pdf_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            file_status: "No file uploaded".to_string(),
            embedded_files: None,
        }));
    }

    // Process the PDF and return response
    match ERechnungService::process_pdf(pdf_data) {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(HttpResponse::BadRequest().json(error)),
    }
}

/// Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "eRechnung PDF/A-3 XML Extractor"
    })))
}
