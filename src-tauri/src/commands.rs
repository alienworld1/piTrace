use crate::{
    importer,
    models::{
        CaseInput, CaseRecord, CaseReport, EvidenceFile, Finding, ImportConfig, MetadataField,
    },
    storage::JsonRepository,
};
use tauri::AppHandle;

#[tauri::command]
pub fn get_import_config() -> Result<ImportConfig, String> {
    importer::load_import_config()
}

#[tauri::command]
pub fn list_cases(app: AppHandle) -> Result<Vec<CaseRecord>, String> {
    JsonRepository::new(&app)?.list_cases()
}

#[tauri::command]
pub fn create_case(app: AppHandle, input: CaseInput) -> Result<CaseRecord, String> {
    JsonRepository::new(&app)?.create_case(input)
}

#[tauri::command]
pub fn update_case(
    app: AppHandle,
    case_id: String,
    input: CaseInput,
) -> Result<CaseRecord, String> {
    JsonRepository::new(&app)?.update_case(case_id, input)
}

#[tauri::command]
pub fn get_case(app: AppHandle, case_id: String) -> Result<CaseRecord, String> {
    JsonRepository::new(&app)?.get_case(&case_id)
}

#[tauri::command]
pub fn get_case_files(app: AppHandle, case_id: String) -> Result<Vec<EvidenceFile>, String> {
    JsonRepository::new(&app)?.get_case_files(&case_id)
}

#[tauri::command]
pub fn get_file(app: AppHandle, file_id: String) -> Result<EvidenceFile, String> {
    JsonRepository::new(&app)?.get_file(&file_id)
}

#[tauri::command]
pub fn import_files(
    app: AppHandle,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<Vec<EvidenceFile>, String> {
    importer::import_files(&app, case_id, file_paths)
}

#[tauri::command]
pub fn get_case_findings(app: AppHandle, case_id: String) -> Result<Vec<Finding>, String> {
    let repository = JsonRepository::new(&app)?;
    let store = repository.load()?;
    let case_file_ids = store
        .evidence_files
        .iter()
        .filter(|file| file.case_id == case_id)
        .map(|file| file.id.as_str())
        .collect::<Vec<_>>();

    Ok(store
        .findings
        .into_iter()
        .filter(|finding| case_file_ids.contains(&finding.file_id.as_str()))
        .collect())
}

#[tauri::command]
pub fn get_file_findings(app: AppHandle, file_id: String) -> Result<Vec<Finding>, String> {
    Ok(JsonRepository::new(&app)?
        .load()?
        .findings
        .into_iter()
        .filter(|finding| finding.file_id == file_id)
        .collect())
}

#[tauri::command]
pub fn get_file_metadata(app: AppHandle, file_id: String) -> Result<Vec<MetadataField>, String> {
    Ok(JsonRepository::new(&app)?
        .load()?
        .metadata_fields
        .into_iter()
        .filter(|field| field.file_id == file_id)
        .collect())
}

#[tauri::command]
pub fn get_finding(app: AppHandle, finding_id: String) -> Result<Finding, String> {
    JsonRepository::new(&app)?
        .load()?
        .findings
        .into_iter()
        .find(|finding| finding.id == finding_id)
        .ok_or_else(|| "Finding not found".to_string())
}

#[tauri::command]
pub fn get_case_report(app: AppHandle, case_id: String) -> Result<Option<CaseReport>, String> {
    Ok(JsonRepository::new(&app)?
        .load()?
        .reports
        .into_iter()
        .find(|report| report.case_id == case_id))
}
