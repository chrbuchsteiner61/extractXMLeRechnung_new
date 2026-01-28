use actix_web::{test, App, web, http::StatusCode};
use serde_json::Value;
use bytes::Bytes;
use extract_xml_rechnung::{health_check, extract_xml, ErrorResponse, SuccessResponse, PDFError, ERechnungService};

/// Create test application without middleware to avoid type complexity
fn create_test_app() -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .route("/health", web::get().to(health_check))
        .route("/extract_xml", web::post().to(extract_xml))
}

#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(create_test_app()).await;
    
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "eRechnung PDF/A-3 XML Extractor");
}

#[actix_web::test]
async fn test_extract_xml_no_file() {
    let app = test::init_service(create_test_app()).await;
    
    // Test with empty payload
    let req = test::TestRequest::post()
        .uri("/extract_xml")
        .set_payload(Bytes::new())
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    
    // Try to parse as JSON, but handle cases where it might not be JSON
    let body_bytes = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap_or("");
    
    if body_str.trim().starts_with('{') {
        let body: ErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body.file_status, "No file uploaded");
        assert!(body.embedded_files.is_none());
    } else {
        // If it's not JSON, it's likely a multipart parsing error, which is also valid
        assert!(!body_str.is_empty());
    }
}

#[actix_web::test]
async fn test_extract_xml_invalid_multipart() {
    let app = test::init_service(create_test_app()).await;
    
    // Test with invalid multipart data
    let req = test::TestRequest::post()
        .uri("/extract_xml")
        .insert_header(("content-type", "multipart/form-data; boundary=invalid"))
        .set_payload("invalid multipart data")
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    // Should result in a bad request due to invalid multipart format
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_extract_xml_with_fake_pdf() {
    let app = test::init_service(create_test_app()).await;
    
    // Create a fake PDF that will fail validation
    let fake_pdf = create_fake_pdf_multipart();
    
    let req = test::TestRequest::post()
        .uri("/extract_xml")
        .insert_header(("content-type", "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW"))
        .set_payload(fake_pdf)
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    
    let body: ErrorResponse = test::read_body_json(resp).await;
    assert!(body.file_status.contains("Not a valid PDF file") || 
           body.file_status.contains("PDF is not in PDF/A-3 format"));
}

#[actix_web::test]
async fn test_routes_exist() {
    let app = test::init_service(create_test_app()).await;
    
    // Test that the routes exist (even if they return errors)
    let health_req = test::TestRequest::get().uri("/health").to_request();
    let health_resp = test::call_service(&app, health_req).await;
    assert_ne!(health_resp.status(), StatusCode::NOT_FOUND);
    
    let extract_req = test::TestRequest::post().uri("/extract_xml").to_request();
    let extract_resp = test::call_service(&app, extract_req).await;
    assert_ne!(extract_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_wrong_http_methods() {
    let app = test::init_service(create_test_app()).await;
    
    // Test wrong method for health endpoint
    let req = test::TestRequest::post().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    // Actix returns 404 for POST to GET-only endpoint
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::METHOD_NOT_ALLOWED);
    
    // Test wrong method for extract endpoint - Actix may return 404 for unmatched routes
    let req = test::TestRequest::get().uri("/extract_xml").to_request();
    let resp = test::call_service(&app, req).await;
    // Accept either 404 (no matching route) or 405 (method not allowed)
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::METHOD_NOT_ALLOWED);
}

/// Create a fake multipart form data that mimics a PDF upload
fn create_fake_pdf_multipart() -> Bytes {
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let fake_pdf_content = b"Not a real PDF file";
    
    let multipart_body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.pdf\"\r\n\
        Content-Type: application/pdf\r\n\r\n\
        {content}\r\n\
        --{boundary}--\r\n",
        boundary = boundary,
        content = std::str::from_utf8(fake_pdf_content).unwrap()
    );
    
    Bytes::from(multipart_body)
}

// Unit tests for core functionality
#[tokio::test]
async fn test_pdf_error_display() {
    let error = PDFError::InvalidPDF;
    assert_eq!(error.to_string(), "Not a valid PDF file");
    
    let error = PDFError::NotPDFA3;
    assert_eq!(error.to_string(), "PDF is not in PDF/A-3 format");
    
    let error = PDFError::NoXMLFile;
    assert_eq!(error.to_string(), "No embedded XML-file");
}

#[tokio::test]
async fn test_erechnung_service_with_invalid_data() {
    let invalid_data = vec![0x00, 0x01, 0x02, 0x03]; // Not a PDF
    let result = ERechnungService::process_pdf(invalid_data);
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.file_status, "Not a valid PDF file");
}

#[tokio::test]
async fn test_error_response_serialization() {
    let error = ErrorResponse {
        file_status: "Test error".to_string(),
        embedded_files: Some("file1.xml".to_string()),
    };
    
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("Test error"));
    assert!(json.contains("file1.xml"));
}

#[tokio::test]
async fn test_success_response_serialization() {
    let success = SuccessResponse {
        file_status: "Success".to_string(),
        embedded_files: "factur-x.xml".to_string(),
        xml_content: "<xml>test</xml>".to_string(),
        xml_filename: "factur-x.xml".to_string(),
    };
    
    let json = serde_json::to_string(&success).unwrap();
    assert!(json.contains("Success"));
    assert!(json.contains("factur-x.xml"));
    assert!(json.contains("<xml>test</xml>"));
}