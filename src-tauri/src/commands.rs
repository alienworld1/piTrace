use crate::{
    importer,
    models::{
        CaseDashboardItem, CaseInput, CaseRecord, CaseReport, EvidenceFile, Finding,
        ImportBatchResult, ImportConfig, MetadataField, RawMetadataRecord, ReportExportInput,
        ReportExportResult, ReportPayload,
    },
    reporting,
    storage::Repository,
};
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn get_import_config() -> Result<ImportConfig, String> {
    importer::load_import_config()
}

#[tauri::command]
pub fn list_case_dashboard(
    repository: State<'_, Repository>,
) -> Result<Vec<CaseDashboardItem>, String> {
    repository.list_case_dashboard()
}

#[tauri::command]
pub fn create_case(
    repository: State<'_, Repository>,
    input: CaseInput,
) -> Result<CaseRecord, String> {
    repository.create_case(input)
}

#[tauri::command]
pub fn update_case(
    repository: State<'_, Repository>,
    case_id: String,
    input: CaseInput,
) -> Result<CaseRecord, String> {
    repository.update_case(case_id, input)
}

#[tauri::command]
pub fn delete_case(
    repository: State<'_, Repository>,
    case_id: String,
) -> Result<CaseRecord, String> {
    repository.delete_case(&case_id)
}

#[tauri::command]
pub fn get_case(repository: State<'_, Repository>, case_id: String) -> Result<CaseRecord, String> {
    repository.get_case(&case_id)
}

#[tauri::command]
pub fn get_case_files(
    repository: State<'_, Repository>,
    case_id: String,
) -> Result<Vec<EvidenceFile>, String> {
    repository.get_case_files(&case_id)
}

#[tauri::command]
pub fn get_file(
    repository: State<'_, Repository>,
    file_id: String,
) -> Result<EvidenceFile, String> {
    repository.get_file(&file_id)
}

#[tauri::command]
pub fn delete_file(
    repository: State<'_, Repository>,
    file_id: String,
) -> Result<EvidenceFile, String> {
    repository.delete_file(&file_id)
}

#[tauri::command]
pub async fn import_files(
    app: AppHandle,
    repository: State<'_, Repository>,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<ImportBatchResult, String> {
    let repository = repository.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        importer::import_files(&app, &repository, case_id, file_paths)
    })
    .await
    .map_err(|error| format!("Import worker failed: {error}"))?
}

#[tauri::command]
pub fn get_case_findings(
    repository: State<'_, Repository>,
    case_id: String,
) -> Result<Vec<Finding>, String> {
    repository.get_case_findings(&case_id)
}

#[tauri::command]
pub fn get_file_findings(
    repository: State<'_, Repository>,
    file_id: String,
) -> Result<Vec<Finding>, String> {
    repository.get_file_findings(&file_id)
}

#[tauri::command]
pub fn get_file_metadata(
    repository: State<'_, Repository>,
    file_id: String,
) -> Result<Vec<MetadataField>, String> {
    repository.get_file_metadata(&file_id)
}

#[tauri::command]
pub fn get_file_raw_metadata(
    repository: State<'_, Repository>,
    file_id: String,
) -> Result<Option<RawMetadataRecord>, String> {
    repository.get_file_raw_metadata(&file_id)
}

#[tauri::command]
pub fn get_finding(
    repository: State<'_, Repository>,
    finding_id: String,
) -> Result<Finding, String> {
    repository.get_finding(&finding_id)
}

#[tauri::command]
pub fn get_case_report(
    repository: State<'_, Repository>,
    case_id: String,
) -> Result<Option<CaseReport>, String> {
    repository.get_case_report(&case_id)
}

#[tauri::command]
pub async fn get_case_report_payload(
    repository: State<'_, Repository>,
    case_id: String,
    include_raw_metadata: bool,
) -> Result<ReportPayload, String> {
    let repository = repository.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        reporting::build_report_payload(&repository, &case_id, include_raw_metadata)
    })
    .await
    .map_err(|error| format!("Report preview worker failed: {error}"))?
}

#[tauri::command]
pub async fn export_case_report(
    repository: State<'_, Repository>,
    input: ReportExportInput,
) -> Result<ReportExportResult, String> {
    let repository = repository.inner().clone();
    tauri::async_runtime::spawn_blocking(move || reporting::export_case_report(&repository, input))
        .await
        .map_err(|error| format!("Report export worker failed: {error}"))?
}

#[tauri::command]
pub fn open_exported_report(
    app: AppHandle,
    repository: State<'_, Repository>,
    report_id: String,
) -> Result<(), String> {
    let report = repository.get_report(&report_id)?;
    let output_path = report
        .output_path
        .ok_or_else(|| "Report does not have an exported file path".to_string())?;

    if !matches!(report.format.as_str(), "html" | "json" | "pdf") {
        return Err("Report format is not supported for opening".to_string());
    }
    let output_path = PathBuf::from(output_path);
    reporting::validate_existing_report_path(&output_path, &report.format)?;

    app.opener()
        .open_path(output_path.to_string_lossy().to_string(), None::<&str>)
        .map_err(|error| format!("Could not open report: {error}"))
}
