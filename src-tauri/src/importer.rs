use crate::{
    hashing::compute_sha256,
    metadata_extractor::{ExifToolMetadataExtractor, RawMetadataExtractor},
    metadata_normalizer::normalize_metadata,
    models::{
        EvidenceFile, EvidenceStatus, ImportBatchResult, ImportConfig, ImportRejection,
        MetadataField, RawMetadataRecord,
    },
    storage::{now_iso, Repository},
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tauri::AppHandle;
use uuid::Uuid;

pub fn import_files(
    app: &AppHandle,
    repository: &Repository,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<ImportBatchResult, String> {
    let extractor = ExifToolMetadataExtractor::for_app_bundle(app)?;
    import_files_with_repository_and_extractor(repository, &extractor, case_id, file_paths)
}

#[cfg(test)]
pub fn import_files_with_repository(
    repository: &Repository,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<ImportBatchResult, String> {
    let extractor = ExifToolMetadataExtractor::for_test_fixture()?;
    import_files_with_repository_and_extractor(repository, &extractor, case_id, file_paths)
}

fn import_files_with_repository_and_extractor(
    repository: &Repository,
    extractor: &dyn RawMetadataExtractor,
    case_id: String,
    file_paths: Vec<String>,
) -> Result<ImportBatchResult, String> {
    let config = load_import_config()?;

    if !repository.case_exists(&case_id)? {
        return Err("Case not found".to_string());
    }

    let mut rejected_files = Vec::new();
    let mut seen_paths = HashSet::new();
    let import_records = file_paths
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .filter_map(|path| {
            if !seen_paths.insert(path.clone()) {
                rejected_files.push(ImportRejection {
                    file_name: display_path_name(&path),
                    path,
                    reason: "Duplicate path in this import batch".to_string(),
                });
                return None;
            }
            let existing = match repository.get_existing_file_for_path(&case_id, &path) {
                Ok(existing) => existing,
                Err(reason) => {
                    rejected_files.push(ImportRejection {
                        file_name: display_path_name(&path),
                        path,
                        reason,
                    });
                    return None;
                }
            };
            match import_one(&case_id, &path, existing, &config, extractor) {
                Ok(record) => Some(record),
                Err(reason) => {
                    rejected_files.push(ImportRejection {
                        file_name: display_path_name(&path),
                        path,
                        reason,
                    });
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    let imported_files = import_records
        .iter()
        .map(|record| record.file.clone())
        .collect::<Vec<_>>();
    let raw_metadata = import_records
        .iter()
        .filter_map(|record| record.raw_metadata.clone())
        .collect::<Vec<_>>();
    let metadata_fields = import_records
        .iter()
        .flat_map(|record| record.metadata_fields.clone())
        .collect::<Vec<_>>();
    let imported = if imported_files.is_empty() {
        Vec::new()
    } else {
        repository.replace_imported_files_with_metadata(
            &case_id,
            imported_files,
            raw_metadata,
            metadata_fields,
        )?
    };

    Ok(ImportBatchResult {
        imported_files: imported,
        rejected_files,
    })
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

fn import_one(
    case_id: &str,
    original_path: &str,
    existing: Option<EvidenceFile>,
    config: &ImportConfig,
    extractor: &dyn RawMetadataExtractor,
) -> Result<ImportRecord, String> {
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

    let mut file = EvidenceFile {
        id: existing
            .map(|file| file.id)
            .unwrap_or_else(|| format!("file-{}", Uuid::new_v4())),
        case_id: case_id.to_string(),
        original_path: original_path.to_string(),
        file_name: fallback_name,
        extension,
        detected_mime_type: None,
        detected_file_type: None,
        size_bytes: 0,
        sha256: None,
        imported_at,
        analyzed_at: None,
        status: EvidenceStatus::Error,
        error_message: None,
    };

    let analysis_snapshot = build_import_record(&mut file, &path, config)?;
    let (raw_metadata, metadata_fields) =
        analyze_imported_file(&mut file, &path, extractor, &analysis_snapshot);
    Ok(ImportRecord {
        file,
        raw_metadata,
        metadata_fields,
    })
}

fn build_import_record(
    file: &mut EvidenceFile,
    path: &Path,
    config: &ImportConfig,
) -> Result<FileSnapshot, String> {
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

    let pre_hash_snapshot = FileSnapshot::from_metadata(metadata);
    file.size_bytes = pre_hash_snapshot.len;
    let identity = infer::get_from_path(path)
        .map_err(|error| format!("Could not inspect file type: {error}"))?;
    if let Some(identity) = identity {
        file.detected_mime_type = Some(identity.mime_type().to_string());
        file.detected_file_type = Some(identity.extension().to_uppercase());
    }

    file.sha256 = Some(compute_sha256(path)?);
    let post_hash_snapshot = file_snapshot(path)?;
    if post_hash_snapshot != pre_hash_snapshot {
        return Err(
            "File changed while piTrace was hashing it. Re-import the file when it is stable."
                .to_string(),
        );
    }
    file.status = EvidenceStatus::Pending;
    file.error_message = None;

    Ok(post_hash_snapshot)
}

fn analyze_imported_file(
    file: &mut EvidenceFile,
    path: &Path,
    extractor: &dyn RawMetadataExtractor,
    analysis_snapshot: &FileSnapshot,
) -> (Option<RawMetadataRecord>, Vec<MetadataField>) {
    file.status = EvidenceStatus::Analyzing;

    match extractor.extract_raw_metadata(path) {
        Ok(data) => {
            match file_snapshot(path) {
                Ok(current_snapshot) if &current_snapshot == analysis_snapshot => {}
                Ok(_) => {
                    file.status = EvidenceStatus::Error;
                    file.analyzed_at = Some(now_iso());
                    file.sha256 = None;
                    file.error_message = Some(
                        "File changed during ExifTool analysis. Re-import the file when it is stable."
                            .to_string(),
                    );
                    return (None, Vec::new());
                }
                Err(error) => {
                    file.status = EvidenceStatus::Error;
                    file.analyzed_at = Some(now_iso());
                    file.sha256 = None;
                    file.error_message = Some(error);
                    return (None, Vec::new());
                }
            }

            let extracted_at = now_iso();
            file.status = EvidenceStatus::Complete;
            file.analyzed_at = Some(extracted_at.clone());
            file.error_message = None;
            let metadata_fields = normalize_metadata(&file.id, "exiftool", &data);

            (
                Some(RawMetadataRecord {
                    file_id: file.id.clone(),
                    source: "exiftool".to_string(),
                    extracted_at,
                    data,
                }),
                metadata_fields,
            )
        }
        Err(error) => {
            file.status = EvidenceStatus::Error;
            file.analyzed_at = Some(now_iso());
            file.error_message = Some(error);
            (None, Vec::new())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSnapshot {
    len: u64,
    modified: Option<SystemTime>,
}

impl FileSnapshot {
    fn from_metadata(metadata: fs::Metadata) -> Self {
        Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        }
    }
}

fn file_snapshot(path: &Path) -> Result<FileSnapshot, String> {
    fs::metadata(path)
        .map(FileSnapshot::from_metadata)
        .map_err(|error| format!("File is unavailable: {error}"))
}

struct ImportRecord {
    file: EvidenceFile,
    raw_metadata: Option<RawMetadataRecord>,
    metadata_fields: Vec<MetadataField>,
}

fn display_path_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{
        import_files_with_repository, import_files_with_repository_and_extractor,
        load_import_config,
    };
    use crate::{
        metadata_extractor::RawMetadataExtractor,
        models::{CaseRecord, EvidenceStatus},
        storage::Repository,
    };
    use serde_json::{json, Value};
    use std::{
        cell::Cell,
        fs,
        path::{Path, PathBuf},
    };
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
    fn import_supported_file_records_complete_evidence_with_sha256_and_raw_metadata() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.pdf", b"%PDF-1.7\n");
        let expected_hash = "0716f9264c9fe19f5d7455276107f3ddcc1d3497f63d60689a73558ae8a1bf5e";

        let result = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("import should succeed");

        assert_eq!(result.imported_files.len(), 1);
        assert!(result.rejected_files.is_empty());
        let imported = result.imported_files;
        assert_eq!(imported[0].status, EvidenceStatus::Complete);
        assert_eq!(imported[0].file_name, "sample.pdf");
        assert_eq!(imported[0].extension, "pdf");
        assert_eq!(imported[0].size_bytes, 9);
        assert_eq!(imported[0].sha256.as_deref(), Some(expected_hash));
        assert!(imported[0].analyzed_at.is_some());
        assert_eq!(imported[0].error_message, None);

        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should persist");
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].original_path, file_path.to_string_lossy());
        assert_eq!(persisted[0].sha256.as_deref(), Some(expected_hash));

        let raw_metadata = fixture
            .repository
            .get_file_raw_metadata(&imported[0].id)
            .expect("raw metadata should load")
            .expect("raw metadata should persist");
        assert_eq!(raw_metadata.source, "exiftool");
        assert_eq!(raw_metadata.data["File"]["FileType"], "PDF");
        let metadata_fields = fixture
            .repository
            .get_file_metadata(&imported[0].id)
            .expect("metadata should load");
        assert!(metadata_fields.iter().any(|field| {
            field.file_id == imported[0].id
                && field.key == "FileType"
                && field.display_label.as_deref() == Some("File type")
                && field.value == "PDF"
                && field.normalized_category.as_deref() == Some("technical")
        }));
    }

    #[test]
    fn import_supported_file_with_unknown_content_does_not_claim_detected_type() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.pdf", b"not actually a pdf");

        let imported = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("supported extension should import")
        .imported_files;

        assert_eq!(imported[0].extension, "pdf");
        assert_eq!(imported[0].detected_mime_type, None);
        assert_eq!(imported[0].detected_file_type, None);
    }

    #[test]
    fn import_unknown_extension_rejects_without_persisting_evidence() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.xyznotallowed", b"data");

        let result = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("unsupported extension should be reported as rejection");

        assert!(result.imported_files.is_empty());
        assert_eq!(result.rejected_files.len(), 1);
        assert_eq!(result.rejected_files[0].file_name, "sample.xyznotallowed");
        assert!(result.rejected_files[0]
            .reason
            .contains("Unsupported file extension"));

        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should load");
        assert!(persisted.is_empty());
    }

    #[test]
    fn import_directory_rejects_without_persisting_evidence() {
        let fixture = ImportFixture::new();
        let directory = fixture.dir.join("directory.pdf");
        fs::create_dir_all(&directory).expect("directory should be created");

        let result = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![directory.to_string_lossy().to_string()],
        )
        .expect("directory should be reported as rejection");

        assert!(result.imported_files.is_empty());
        assert_eq!(result.rejected_files.len(), 1);
        assert!(result.rejected_files[0]
            .reason
            .contains("Directories are not supported"));
        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should load");
        assert!(persisted.is_empty());
    }

    #[test]
    fn import_missing_file_rejects_without_persisting_evidence() {
        let fixture = ImportFixture::new();
        let missing = fixture.dir.join("missing.pdf");

        let result = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![missing.to_string_lossy().to_string()],
        )
        .expect("missing file should be reported as rejection");

        assert!(result.imported_files.is_empty());
        assert_eq!(result.rejected_files.len(), 1);
        assert!(result.rejected_files[0]
            .reason
            .contains("File is unavailable"));
        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should load");
        assert!(persisted.is_empty());
    }

    #[test]
    fn mixed_import_persists_valid_files_and_reports_rejections() {
        let fixture = ImportFixture::new();
        let valid = fixture.write_file("valid.pdf", b"%PDF-1.7\n");
        let invalid = fixture.write_file("invalid.xyznotallowed", b"data");

        let result = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![
                valid.to_string_lossy().to_string(),
                invalid.to_string_lossy().to_string(),
            ],
        )
        .expect("mixed import should report structured rejections");

        assert_eq!(result.imported_files.len(), 1);
        assert_eq!(result.rejected_files.len(), 1);
        assert_eq!(result.rejected_files[0].file_name, "invalid.xyznotallowed");

        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("valid file should persist");
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].file_name, "valid.pdf");
        assert_eq!(persisted[0].status, EvidenceStatus::Complete);
    }

    #[test]
    fn duplicate_batch_paths_are_rejected_without_rolling_back_other_files() {
        let fixture = ImportFixture::new();
        let first = fixture.write_file("first.pdf", b"%PDF-1.7\nfirst");
        let second = fixture.write_file("second.pdf", b"%PDF-1.7\nsecond");
        let first_path = first.to_string_lossy().to_string();

        let result = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![
                first_path.clone(),
                first_path.clone(),
                second.to_string_lossy().to_string(),
            ],
        )
        .expect("duplicate should be a structured rejection");

        assert_eq!(result.imported_files.len(), 2);
        assert_eq!(result.rejected_files.len(), 1);
        assert_eq!(result.rejected_files[0].path, first_path);
        assert_eq!(
            result.rejected_files[0].reason,
            "Duplicate path in this import batch"
        );
        assert_eq!(
            fixture
                .repository
                .get_case_files("case-1")
                .expect("files")
                .len(),
            2
        );
    }

    #[test]
    fn import_replaces_existing_record_for_same_case_and_path() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("duplicate.pdf", b"one");
        let path = file_path.to_string_lossy().to_string();

        let first = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![path.clone()],
        )
        .expect("first import should succeed")
        .imported_files;
        fs::write(&file_path, b"larger content").expect("file should be rewritten");

        let second = import_with_success(&fixture.repository, "case-1".to_string(), vec![path])
            .expect("second import should succeed")
            .imported_files;

        assert_eq!(first[0].id, second[0].id);
        assert_eq!(second[0].size_bytes, 14);
        assert_eq!(
            first[0].sha256.as_deref(),
            Some("7692c3ad3540bb803c020b3aee66cd8887123234ea0c6e7143c0add73ff431ed")
        );
        assert_eq!(
            second[0].sha256.as_deref(),
            Some("5a728fd5846abf87ef9c9246a2dd48f2769b5fb73dff4384a5f80db258576476")
        );

        let persisted = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should load");
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].size_bytes, 14);
        assert_eq!(persisted[0].sha256, second[0].sha256);
    }

    #[test]
    fn import_records_error_status_when_metadata_extraction_fails() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.pdf", b"%PDF-1.7\n");

        let imported = import_with_failure(
            &fixture.repository,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("import should keep file identity when analysis fails")
        .imported_files;

        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].status, EvidenceStatus::Error);
        assert_eq!(
            imported[0].error_message.as_deref(),
            Some("fixture extraction failed")
        );
        assert!(imported[0].sha256.is_some());
        assert!(imported[0].analyzed_at.is_some());
        assert!(fixture
            .repository
            .get_file_raw_metadata(&imported[0].id)
            .expect("raw metadata lookup should succeed")
            .is_none());
        assert!(fixture
            .repository
            .get_file_metadata(&imported[0].id)
            .expect("metadata lookup should succeed")
            .is_empty());
    }

    #[test]
    fn import_failure_clears_previous_raw_metadata_for_same_file() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.pdf", b"%PDF-1.7\n");
        let path = file_path.to_string_lossy().to_string();

        let first = import_with_success(
            &fixture.repository,
            "case-1".to_string(),
            vec![path.clone()],
        )
        .expect("first import should succeed")
        .imported_files;
        assert!(fixture
            .repository
            .get_file_raw_metadata(&first[0].id)
            .expect("raw metadata lookup should succeed")
            .is_some());
        assert!(fixture
            .repository
            .get_file_metadata(&first[0].id)
            .expect("metadata lookup should succeed")
            .iter()
            .any(|field| field.file_id == first[0].id));

        let second = import_with_failure(&fixture.repository, "case-1".to_string(), vec![path])
            .expect("second import should keep file record")
            .imported_files;

        assert_eq!(first[0].id, second[0].id);
        assert_eq!(second[0].status, EvidenceStatus::Error);
        assert!(fixture
            .repository
            .get_file_raw_metadata(&second[0].id)
            .expect("raw metadata lookup should succeed")
            .is_none());
        assert!(!fixture
            .repository
            .get_file_metadata(&second[0].id)
            .expect("metadata lookup should succeed")
            .iter()
            .any(|field| field.file_id == second[0].id));
    }

    #[test]
    fn public_repository_import_uses_real_exiftool_extraction() {
        let fixture = ImportFixture::new();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".agent")
            .join("exiftool")
            .join("t")
            .join("images")
            .join("GPS.jpg");
        let file_path = fixture.dir.join("GPS.jpg");
        fs::copy(source, &file_path).expect("fixture image should copy");

        let imported = import_files_with_repository(
            &fixture.repository,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("import should run real exiftool")
        .imported_files;

        let raw_metadata = fixture
            .repository
            .get_file_raw_metadata(&imported[0].id)
            .expect("raw metadata should load")
            .expect("raw metadata should exist");

        assert_eq!(imported[0].status, EvidenceStatus::Complete);
        assert_eq!(raw_metadata.data["File"]["FileType"], "JPEG");
        assert!(raw_metadata.data["GPS"].is_object());
        let metadata_fields = fixture
            .repository
            .get_file_metadata(&imported[0].id)
            .expect("metadata should load");
        assert!(metadata_fields.iter().any(|field| {
            field.file_id == imported[0].id
                && field.normalized_category.as_deref() == Some("location")
        }));
        assert!(metadata_fields.iter().any(|field| {
            field.file_id == imported[0].id
                && field.normalized_category.as_deref() == Some("technical")
        }));
    }

    #[test]
    fn import_marks_error_and_clears_raw_metadata_when_file_changes_during_analysis() {
        let fixture = ImportFixture::new();
        let file_path = fixture.write_file("sample.pdf", b"%PDF-1.7\n");

        let extractor = MutatingExtractor {
            replacement: b"%PDF-1.7\nchanged".to_vec(),
        };
        let imported = import_files_with_repository_and_extractor(
            &fixture.repository,
            &extractor,
            "case-1".to_string(),
            vec![file_path.to_string_lossy().to_string()],
        )
        .expect("import should keep file record")
        .imported_files;

        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].status, EvidenceStatus::Error);
        assert_eq!(imported[0].sha256, None);
        assert!(imported[0]
            .error_message
            .as_deref()
            .is_some_and(|message| message.contains("changed during ExifTool analysis")));
        assert!(fixture
            .repository
            .get_file_raw_metadata(&imported[0].id)
            .expect("raw metadata lookup should succeed")
            .is_none());
        assert!(!fixture
            .repository
            .get_file_metadata(&imported[0].id)
            .expect("metadata lookup should succeed")
            .iter()
            .any(|field| field.file_id == imported[0].id));
    }

    #[test]
    fn import_reuses_one_extractor_instance_across_files() {
        let fixture = ImportFixture::new();
        let first_path = fixture.write_file("first.pdf", b"%PDF-1.7\n");
        let second_path = fixture.write_file("second.pdf", b"%PDF-1.7\n");
        let extractor = CountingExtractor {
            calls: Cell::new(0),
        };

        let imported = import_files_with_repository_and_extractor(
            &fixture.repository,
            &extractor,
            "case-1".to_string(),
            vec![
                first_path.to_string_lossy().to_string(),
                second_path.to_string_lossy().to_string(),
            ],
        )
        .expect("import should succeed")
        .imported_files;

        assert_eq!(imported.len(), 2);
        assert_eq!(extractor.calls.get(), 2);
        assert!(imported
            .iter()
            .all(|file| file.status == EvidenceStatus::Complete));
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
        repository: Repository,
    }

    impl ImportFixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("pi-trace-import-test-{}", Uuid::new_v4()));
            fs::create_dir_all(&dir).expect("test directory should be created");
            let repository = Repository::for_test_path(dir.join("store.sqlite3"))
                .expect("repository should initialize");
            repository
                .insert_case(&case_record("case-1"))
                .expect("fixture case should save");

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

    fn import_with_success(
        repository: &Repository,
        case_id: String,
        file_paths: Vec<String>,
    ) -> Result<crate::models::ImportBatchResult, String> {
        let extractor = StubExtractor {
            result: Ok(json!({
                "SourceFile": "fixture",
                "File": {
                    "FileType": "PDF"
                }
            })),
        };

        import_files_with_repository_and_extractor(repository, &extractor, case_id, file_paths)
    }

    fn import_with_failure(
        repository: &Repository,
        case_id: String,
        file_paths: Vec<String>,
    ) -> Result<crate::models::ImportBatchResult, String> {
        let extractor = StubExtractor {
            result: Err("fixture extraction failed".to_string()),
        };

        import_files_with_repository_and_extractor(repository, &extractor, case_id, file_paths)
    }

    struct StubExtractor {
        result: Result<Value, String>,
    }

    impl RawMetadataExtractor for StubExtractor {
        fn extract_raw_metadata(&self, _path: &Path) -> Result<Value, String> {
            self.result.clone()
        }
    }

    struct MutatingExtractor {
        replacement: Vec<u8>,
    }

    impl RawMetadataExtractor for MutatingExtractor {
        fn extract_raw_metadata(&self, path: &Path) -> Result<Value, String> {
            fs::write(path, &self.replacement)
                .map_err(|error| format!("fixture mutation failed: {error}"))?;
            Ok(json!({
                "SourceFile": "fixture",
                "File": {
                    "FileType": "PDF"
                }
            }))
        }
    }

    struct CountingExtractor {
        calls: Cell<u32>,
    }

    impl RawMetadataExtractor for CountingExtractor {
        fn extract_raw_metadata(&self, _path: &Path) -> Result<Value, String> {
            self.calls.set(self.calls.get() + 1);
            Ok(json!({
                "SourceFile": "fixture",
                "File": {
                    "FileType": "PDF"
                }
            }))
        }
    }
}
