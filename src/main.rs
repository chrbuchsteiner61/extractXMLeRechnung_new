// Module declarations
mod errors;
mod handlers;
mod models;
mod pdf;
mod service;

use actix_web::{middleware, web, App, HttpServer};
use handlers::{extract_xml, health_check};

// Main Application
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting eRechnung PDF/A-3 XML Extractor API");
    println!("📡 Server running at http://127.0.0.1:8080");
    println!("📋 Endpoints:");
    println!("   POST /extract_xml - Extract XML from PDF/A-3");
    println!("   GET  /health - Health check");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/extract_xml", web::post().to(extract_xml))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}