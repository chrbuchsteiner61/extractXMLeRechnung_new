use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    #[serde(rename = "file status")]
    pub file_status: String,
    #[serde(rename = "embedded files", skip_serializing_if = "Option::is_none")]
    pub embedded_files: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SuccessResponse {
    #[serde(rename = "file status")]
    pub file_status: String,
    #[serde(rename = "embedded files")]
    pub embedded_files: String,
    #[serde(rename = "xml_content")]
    pub xml_content: String,
    #[serde(rename = "xml_filename")]
    pub xml_filename: String,
}
