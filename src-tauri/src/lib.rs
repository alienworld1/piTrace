mod commands;
mod hashing;
mod importer;
mod metadata_extractor;
mod metadata_normalizer;
mod models;
mod storage;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let repository =
                storage::Repository::new(app.handle()).map_err(std::io::Error::other)?;
            app.manage(repository);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_case,
            commands::delete_case,
            commands::delete_file,
            commands::get_case,
            commands::get_case_files,
            commands::get_case_findings,
            commands::get_case_report,
            commands::get_file,
            commands::get_file_findings,
            commands::get_file_metadata,
            commands::get_file_raw_metadata,
            commands::get_finding,
            commands::get_import_config,
            commands::import_files,
            commands::list_case_dashboard,
            commands::update_case
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
