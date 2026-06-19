use crate::{
    importer,
    models::{
        CaseInput, CaseRecord, CaseReport, EvidenceFile, Finding, ImportBatchResult, ImportConfig,
        MetadataField, RawMetadataRecord,
    },
    storage::Repository,
};
use tauri::AppHandle;

#[tauri::command]
pub fn get_import_config() -> Result<ImportConfig, String> {
    importer::load_import_config()
}

#[tauri::command]
pub fn list_cases(app: AppHandle) -> Result<Vec<CaseRecord>, String> {
    Repository::new(&app)?.list_cases()
}

#[tauri::command]
pub fn create_case(app: AppHandle, input: CaseInput) -> Result<CaseRecord, String> {
    Repository::new(&app)?.create_case(input)
}

#[tauri::command]
pub fn update_case(
    app: AppHandle,
    case_id: String,
    input: CaseInput,
) -> Result<CaseRecord, String> {
    Repository::new(&app)?.update_case(case_id, input)
}

#[tauri::command]
pub fn delete_case(app: AppHandle, case_id: String) -> Result<CaseRecord, String> {
    Repository::new(&app)?.delete_case(&case_id)
}

#[tauri::command]
pub fn get_case(app: AppHandle, case_id: String) -> Result<CaseRecord, String> {
    Repository::new(&app)?.get_case(&case_id)
}

#[tauri::command]
pub fn get_case_files(app: AppHandle, case_id: String) -> Result<Vec<EvidenceFile>, String> {
    Repository::new(&app)?.get_case_files(&case_id)
}

#[tauri::command]
pub fn get_file(app: AppHandle, file_id: String) -> Result<EvidenceFile, String> {
    Repository::new(&app)?.get_file(&file_id)
}

#[tauri::command]
pub fn delete_file(app: AppHandle, file_id: String) -> Result<EvidenceFile, String> {
    Repository::new(&app)?.delete_file(&file_id)
}

#[tauri::command]
pub fn import_files(
    app: AppHandle,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<ImportBatchResult, String> {
    importer::import_files(&app, case_id, file_paths)
}

#[tauri::command]
pub fn get_case_findings(app: AppHandle, case_id: String) -> Result<Vec<Finding>, String> {
    Repository::new(&app)?.get_case_findings(&case_id)
}

#[tauri::command]
pub fn get_file_findings(app: AppHandle, file_id: String) -> Result<Vec<Finding>, String> {
    Repository::new(&app)?.get_file_findings(&file_id)
}

#[tauri::command]
pub fn get_file_metadata(app: AppHandle, file_id: String) -> Result<Vec<MetadataField>, String> {
    Repository::new(&app)?.get_file_metadata(&file_id)
}

#[tauri::command]
pub fn get_file_raw_metadata(
    app: AppHandle,
    file_id: String,
) -> Result<Option<RawMetadataRecord>, String> {
    Repository::new(&app)?.get_file_raw_metadata(&file_id)
}

#[tauri::command]
pub fn get_finding(app: AppHandle, finding_id: String) -> Result<Finding, String> {
    Repository::new(&app)?.get_finding(&finding_id)
}

#[tauri::command]
pub fn get_case_report(app: AppHandle, case_id: String) -> Result<Option<CaseReport>, String> {
    Repository::new(&app)?.get_case_report(&case_id)
}
