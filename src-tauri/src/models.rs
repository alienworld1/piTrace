use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseRecord {
    pub id: String,
    pub name: String,
    pub examiner_name: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseInput {
    pub name: String,
    pub examiner_name: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceFile {
    pub id: String,
    pub case_id: String,
    pub original_path: String,
    pub file_name: String,
    pub extension: String,
    pub detected_mime_type: Option<String>,
    pub detected_file_type: Option<String>,
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub imported_at: String,
    pub analyzed_at: Option<String>,
    pub status: EvidenceStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRejection {
    pub path: String,
    pub file_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBatchResult {
    pub imported_files: Vec<EvidenceFile>,
    pub rejected_files: Vec<ImportRejection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataField {
    pub id: String,
    pub file_id: String,
    pub group: String,
    pub key: String,
    pub display_label: Option<String>,
    pub value: String,
    pub source: String,
    pub normalized_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: String,
    pub file_id: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub confidence: String,
    pub related_field_ids: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseReport {
    pub id: String,
    pub case_id: String,
    pub generated_at: String,
    pub format: String,
    pub include_raw_metadata: bool,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMetadataRecord {
    pub file_id: String,
    pub source: String,
    pub extracted_at: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceStatus {
    Pending,
    Analyzing,
    Complete,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportConfig {
    pub supported_extensions: Vec<String>,
    pub dialog_filters: Vec<DialogFilter>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStore {
    #[serde(default)]
    pub cases: Vec<CaseRecord>,
    #[serde(default)]
    pub evidence_files: Vec<EvidenceFile>,
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
    #[serde(default)]
    pub raw_metadata: Vec<RawMetadataRecord>,
    #[serde(default)]
    pub findings: Vec<Finding>,
    #[serde(default)]
    pub reports: Vec<CaseReport>,
}
