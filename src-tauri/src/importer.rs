use crate::{
    models::{EvidenceFile, EvidenceStatus, ImportConfig},
    storage::{now_iso, JsonRepository},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::AppHandle;
use uuid::Uuid;

pub fn import_files(
    app: &AppHandle,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<Vec<EvidenceFile>, String> {
    let repository = JsonRepository::new(app)?;
    import_files_with_repository(&repository, case_id, file_paths)
}

pub fn import_files_with_repository(
    repository: &JsonRepository,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<Vec<EvidenceFile>, String> {
    let store = repository.load()?;

    if !store.cases.iter().any(|case| case.id == case_id) {
        return Err("Case not found".to_string());
    }

    let imported = file_paths
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .map(|path| {
            let existing = store
                .evidence_files
                .iter()
                .find(|file| file.case_id == case_id && file.original_path == path);
            import_one(&case_id, &path, existing)
        })
        .collect::<Vec<_>>();

    repository.replace_imported_files(&case_id, imported)
}

pub fn load_import_config() -> Result<ImportConfig, String> {
    let data = include_str!("../import_config.json");
    let mut config: ImportConfig =
        serde_json::from_str(data).map_err(|error| format!("Import config is invalid: {error}"))?;

    config.supported_extensions = config
        .supported_extensions
        .into_iter()
        .map(|extension| normalize_extension(&extension))
        .filter(|extension| !extension.is_empty())
        .collect();

    for filter in &mut config.dialog_filters {
        filter.extensions = filter
            .extensions
            .iter()
            .map(|extension| normalize_extension(extension))
            .filter(|extension| !extension.is_empty())
            .collect();
    }

    Ok(config)
}

fn import_one(case_id: &str, original_path: &str, existing: Option<&EvidenceFile>) -> EvidenceFile {
    let path = PathBuf::from(original_path);
    let imported_at = now_iso();
    let fallback_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unavailable file")
        .to_string();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(normalize_extension)
        .unwrap_or_default();

    let base = EvidenceFile {
        id: existing
            .map(|file| file.id.clone())
            .unwrap_or_else(|| format!("file-{}", Uuid::new_v4())),
        case_id: case_id.to_string(),
        original_path: original_path.to_string(),
        file_name: fallback_name,
        extension,
        detected_mime_type: None,
        detected_file_type: None,
        size_bytes: 0,
        imported_at,
        analyzed_at: None,
        status: EvidenceStatus::Error,
        error_message: None,
    };

    match build_import_record(base.clone(), &path) {
        Ok(file) => file,
        Err(message) => EvidenceFile {
            error_message: Some(message),
            ..base
        },
    }
}

fn build_import_record(mut file: EvidenceFile, path: &Path) -> Result<EvidenceFile, String> {
    let config = load_import_config()?;
    if file.extension.is_empty() || !config.supported_extensions.contains(&file.extension) {
        return Err(format!(
            "Unsupported file extension. Add '{}' to import_config.json to allow this file type.",
            if file.extension.is_empty() {
                "extension"
            } else {
                file.extension.as_str()
            }
        ));
    }

    let metadata = fs::metadata(path).map_err(|error| format!("File is unavailable: {error}"))?;

    if metadata.is_dir() {
        return Err("Directories are not supported for import yet.".to_string());
    }

    if !metadata.is_file() {
        return Err("Only regular files can be imported.".to_string());
    }

    file.size_bytes = metadata.len();
    let identity = infer::get_from_path(path)
        .map_err(|error| format!("Could not inspect file type: {error}"))?;
    if let Some(identity) = identity {
        file.detected_mime_type = Some(identity.mime_type().to_string());
        file.detected_file_type = Some(identity.extension().to_uppercase());
    } else {
        file.detected_file_type = if file.extension.is_empty() {
            None
        } else {
            Some(file.extension.to_uppercase())
        };
    }

    file.status = EvidenceStatus::Pending;
    file.error_message = None;

    Ok(file)
}

fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{import_files_with_repository, load_import_config};
    use crate::{
        models::{AppStore, CaseRecord, EvidenceStatus},
        storage::JsonRepository,
    };
    use std::{fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn load_import_config_normalizes_extensions_and_filters() {
        let config = load_import_config().expect("config should load");

        assert!(config.supported_extensions.contains(&"jpg".to_string()));
        assert!(config.supported_extensions.contains(&"pdf".to_string()));
        assert!(config
            .dialog_filters
            .iter()
            .any(|filter| filter.name == "Supported forensic files"));
        assert!(config
            .dialog_filters
            .iter()
            .flat_map(|filter| filter.extensions.iter())
            .all(|extension| extension == &extension.to_ascii_lowercase()));
    }

    #[test]
    fn import_supported_file_records_pending_evidence_without_hashing() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.pdf", b"%PDF-1.7\n");

        let imported = import_files_with_repository(
            &fixture.repository,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("import should succeed");

        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].status, EvidenceStatus::Pending);
        assert_eq!(imported[0].file_name, "sample.pdf");
        assert_eq!(imported[0].extension, "pdf");
        assert_eq!(imported[0].size_bytes, 9);
        assert_eq!(imported[0].error_message, None);

        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should persist");
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].original_path, file_path.to_string_lossy());
    }

    #[test]
    fn import_unknown_extension_persists_error_record() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.xyznotallowed", b"data");

        let imported = import_files_with_repository(
            &fixture.repository,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("import command should persist per-file errors");

        assert_eq!(imported[0].status, EvidenceStatus::Error);
        assert_eq!(imported[0].extension, "xyznotallowed");
        assert!(imported[0]
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("Unsupported file extension"));

        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("error record should persist");
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].status, EvidenceStatus::Error);
    }

    #[test]
    fn import_directory_persists_error_record() {
        let fixture = ImportFixture::new();
        let directory = fixture.dir.join("directory.pdf");
        fs::create_dir_all(&directory).expect("directory should be created");

        let imported = import_files_with_repository(
            &fixture.repository,
            "case-1".to_string(),
            vec![directory.to_string_lossy().to_string()],
        )
        .expect("import command should persist per-path errors");

        assert_eq!(imported[0].status, EvidenceStatus::Error);
        assert!(imported[0]
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("Directories are not supported"));
    }

    #[test]
    fn import_missing_file_persists_error_record() {
        let fixture = ImportFixture::new();
        let missing = fixture.dir.join("missing.pdf");

        let imported = import_files_with_repository(
            &fixture.repository,
            "case-1".to_string(),
            vec![missing.to_string_lossy().to_string()],
        )
        .expect("import command should persist missing file error");

        assert_eq!(imported[0].status, EvidenceStatus::Error);
        assert!(imported[0]
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("File is unavailable"));
    }

    #[test]
    fn import_replaces_existing_record_for_same_case_and_path() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("duplicate.pdf", b"one");
        let path = file_path.to_string_lossy().to_string();

        let first = import_files_with_repository(
            &fixture.repository,
            "case-1".to_string(),
            vec![path.clone()],
        )
        .expect("first import should succeed");
        fs::write(&file_path, b"larger content").expect("file should be rewritten");

        let second =
            import_files_with_repository(&fixture.repository, "case-1".to_string(), vec![path])
                .expect("second import should succeed");

        assert_eq!(first[0].id, second[0].id);
        assert_eq!(second[0].size_bytes, 14);

        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should load");
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].size_bytes, 14);
    }

    #[test]
    fn import_requires_existing_case() {
        let fixture = ImportFixture::new();

        let error =
            import_files_with_repository(&fixture.repository, "case-missing".to_string(), vec![])
                .expect_err("missing case should fail");

        assert_eq!(error, "Case not found");
    }

    struct ImportFixture {
        dir: PathBuf,
        repository: JsonRepository,
    }

    impl ImportFixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("pi-trace-import-test-{}", Uuid::new_v4()));
            fs::create_dir_all(&dir).expect("test directory should be created");
            let repository = JsonRepository::for_path(dir.join("store.json"));
            let mut store = AppStore::default();
            store.cases.push(case_record("case-1"));
            repository.save(&store).expect("fixture store should save");

            Self { dir, repository }
        }

        fn write_file(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.dir.join(name);
            fs::write(&path, bytes).expect("test file should be written");
            path
        }
    }

    impl Drop for ImportFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn case_record(id: &str) -> CaseRecord {
        CaseRecord {
            id: id.to_string(),
            name: "Case".to_string(),
            examiner_name: None,
            notes: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }
}
