mod commands;
mod importer;
mod models;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::create_case,
            commands::get_case,
            commands::get_case_files,
            commands::get_case_findings,
            commands::get_case_report,
            commands::get_file,
            commands::get_file_findings,
            commands::get_file_metadata,
            commands::get_finding,
            commands::get_import_config,
            commands::import_files,
            commands::list_cases,
            commands::update_case
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
