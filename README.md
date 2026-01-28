# eRechnung PDF/A-3 XML Extractor API

A Rust-based REST API service for extracting XML invoice data from PDF/A-3 documents, specifically designed for German eRechnung (electronic invoice) processing.

## Features

- 🔍 **PDF/A-3 Validation**: Validates PDF documents conform to PDF/A-3 standard
- 📄 **XML Extraction**: Extracts embedded XML files from PDF/A-3 documents  
- 🏷️ **Factur-X Detection**: Specifically identifies and processes Factur-X compliant invoices
- 🚀 **High Performance**: Built with Actix-Web for concurrent request handling
- 📊 **Health Monitoring**: Built-in health check endpoint
- 🛡️ **Error Handling**: Comprehensive error responses with detailed status information

## Quick Start

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo package manager

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd extractXMLeRechnung
```

2. Install dependencies:
```bash
cargo build
```

3. Run the service:
```bash
cargo run
```

The API will be available at `http://127.0.0.1:8080`

## API Endpoints

### POST /extract_xml

Extracts XML content from an uploaded PDF/A-3 file.

**Request:**
- Method: `POST`
- Content-Type: `multipart/form-data`
- Body: PDF file as form data

**Response (Success):**
```json
{
  "file status": "Success",
  "embedded files": "factur-x.xml",
  "xml_content": "<xml content here>",
  "xml_filename": "factur-x.xml"
}
```

**Response (Error):**
```json
{
  "file status": "Error description",
  "embedded files": "list of found files (optional)"
}
```

### GET /health

Health check endpoint for monitoring service status.

**Response:**
```json
{
  "status": "healthy",
  "service": "eRechnung PDF/A-3 XML Extractor"
}
```

## Usage Examples

### Using curl

```bash
# Extract XML from PDF
curl -X POST \
  -F "file=@invoice.pdf" \
  http://127.0.0.1:8080/extract_xml

# Health check
curl http://127.0.0.1:8080/health
```

### Using HTTPie

```bash
# Extract XML from PDF
http --form POST localhost:8080/extract_xml file@invoice.pdf

# Health check
http GET localhost:8080/health
```

## Architecture

The application is structured into several modules:

- **`main.rs`**: Application entry point and server configuration
- **`handlers.rs`**: HTTP request handlers and routing logic
- **`service.rs`**: Core business logic for PDF processing
- **`models.rs`**: Data structures for requests/responses
- **`pdf.rs`**: PDF/A-3 validation and XML extraction utilities
- **`errors.rs`**: Custom error types and handling

## Dependencies

- **actix-web**: Web framework for HTTP server
- **actix-multipart**: Multipart form data handling
- **serde**: Serialization/deserialization
- **tokio**: Async runtime
- **futures-util**: Stream utilities
- **thiserror**: Error handling macros

## Error Handling

The service provides detailed error responses for various scenarios:

- **No file uploaded**: When no PDF file is provided
- **Invalid PDF/A-3**: When the uploaded file doesn't meet PDF/A-3 standards  
- **No embedded files**: When the PDF contains no embedded files
- **No XML file**: When no XML files are found in embedded files
- **Extraction failed**: When XML content extraction fails

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Building for Production

```bash
cargo build --release
```

The optimized binary will be available at `target/release/extractXMLeRechnung`

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For issues and questions, please open a GitHub issue or contact the development team.

---

Made with ❤️ in Rust for German eRechnung processing