use crate::models::{AppStore, CaseInput, CaseRecord, EvidenceFile};
use chrono::Utc;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub struct JsonRepository {
    path: PathBuf,
}

impl JsonRepository {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("Could not locate app data directory: {error}"))?;
        fs::create_dir_all(&dir)
            .map_err(|error| format!("Could not create app data directory: {error}"))?;

        Ok(Self {
            path: dir.join("pi-trace-store.json"),
        })
    }

    #[cfg(test)]
    pub fn for_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<AppStore, String> {
        if !self.path.exists() {
            return Ok(AppStore::default());
        }

        let data = fs::read_to_string(&self.path)
            .map_err(|error| format!("Could not read local store: {error}"))?;

        serde_json::from_str(&data).map_err(|error| format!("Could not parse local store: {error}"))
    }

    pub fn save(&self, store: &AppStore) -> Result<(), String> {
        let data = serde_json::to_string_pretty(store)
            .map_err(|error| format!("Could not serialize local store: {error}"))?;
        fs::write(&self.path, data).map_err(|error| format!("Could not write local store: {error}"))
    }

    pub fn list_cases(&self) -> Result<Vec<CaseRecord>, String> {
        let mut cases = self.load()?.cases;
        cases.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(cases)
    }

    pub fn create_case(&self, input: CaseInput) -> Result<CaseRecord, String> {
        let mut store = self.load()?;
        let now = now_iso();
        let case = CaseRecord {
            id: format!("case-{}", Uuid::new_v4()),
            name: clean_required(input.name, "Case name")?,
            examiner_name: clean_optional(input.examiner_name),
            notes: clean_optional(input.notes),
            created_at: now.clone(),
            updated_at: now,
        };

        store.cases.push(case.clone());
        self.save(&store)?;
        Ok(case)
    }

    pub fn update_case(&self, case_id: String, input: CaseInput) -> Result<CaseRecord, String> {
        let mut store = self.load()?;
        let case = store
            .cases
            .iter_mut()
            .find(|case| case.id == case_id)
            .ok_or_else(|| "Case not found".to_string())?;

        case.name = clean_required(input.name, "Case name")?;
        case.examiner_name = clean_optional(input.examiner_name);
        case.notes = clean_optional(input.notes);
        case.updated_at = now_iso();

        let updated = case.clone();
        self.save(&store)?;
        Ok(updated)
    }

    pub fn get_case(&self, case_id: &str) -> Result<CaseRecord, String> {
        self.load()?
            .cases
            .into_iter()
            .find(|case| case.id == case_id)
            .ok_or_else(|| "Case not found".to_string())
    }

    pub fn get_case_files(&self, case_id: &str) -> Result<Vec<EvidenceFile>, String> {
        let mut files: Vec<EvidenceFile> = self
            .load()?
            .evidence_files
            .into_iter()
            .filter(|file| file.case_id == case_id)
            .collect();
        files.sort_by(|a, b| b.imported_at.cmp(&a.imported_at));
        Ok(files)
    }

    pub fn get_file(&self, file_id: &str) -> Result<EvidenceFile, String> {
        self.load()?
            .evidence_files
            .into_iter()
            .find(|file| file.id == file_id)
            .ok_or_else(|| "Evidence file not found".to_string())
    }

    pub fn replace_imported_files(
        &self,
        case_id: &str,
        imported_files: Vec<EvidenceFile>,
    ) -> Result<Vec<EvidenceFile>, String> {
        let mut store = self.load()?;
        if !store.cases.iter().any(|case| case.id == case_id) {
            return Err("Case not found".to_string());
        }

        let now = now_iso();
        if let Some(case) = store.cases.iter_mut().find(|case| case.id == case_id) {
            case.updated_at = now;
        }

        for imported in &imported_files {
            if let Some(existing) = store.evidence_files.iter_mut().find(|file| {
                file.case_id == case_id && file.original_path == imported.original_path
            }) {
                *existing = imported.clone();
            } else {
                store.evidence_files.push(imported.clone());
            }
        }

        self.save(&store)?;
        Ok(imported_files)
    }
}

#[cfg(test)]
mod tests {
    use super::JsonRepository;
    use crate::models::{AppStore, CaseInput, EvidenceFile, EvidenceStatus};
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use uuid::Uuid;

    #[test]
    fn load_returns_empty_store_when_file_does_not_exist() {
        let fixture = StoreFixture::new();
        let store = fixture.repository.load().expect("load should succeed");

        assert!(store.cases.is_empty());
        assert!(store.evidence_files.is_empty());
        assert!(store.metadata_fields.is_empty());
        assert!(store.findings.is_empty());
        assert!(store.reports.is_empty());
    }

    #[test]
    fn save_and_load_round_trips_store_data() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Round Trip"));

        fixture
            .repository
            .save(&store)
            .expect("save should succeed");
        let loaded = fixture.repository.load().expect("load should succeed");

        assert_eq!(loaded.cases.len(), 1);
        assert_eq!(loaded.cases[0].id, "case-1");
        assert_eq!(loaded.cases[0].name, "Round Trip");
    }

    #[test]
    fn create_case_trims_required_and_optional_fields() {
        let fixture = StoreFixture::new();

        let case = fixture
            .repository
            .create_case(CaseInput {
                name: "  Evidence Review  ".to_string(),
                examiner_name: Some("  Mira  ".to_string()),
                notes: Some("   ".to_string()),
            })
            .expect("case should be created");

        assert!(case.id.starts_with("case-"));
        assert_eq!(case.name, "Evidence Review");
        assert_eq!(case.examiner_name.as_deref(), Some("Mira"));
        assert_eq!(case.notes, None);
        assert_eq!(case.created_at, case.updated_at);
    }

    #[test]
    fn create_case_rejects_blank_name() {
        let fixture = StoreFixture::new();

        let error = fixture
            .repository
            .create_case(CaseInput {
                name: "  ".to_string(),
                examiner_name: None,
                notes: None,
            })
            .expect_err("blank case name should fail");

        assert_eq!(error, "Case name is required");
    }

    #[test]
    fn update_case_changes_fields_and_rejects_missing_case() {
        let fixture = StoreFixture::new();
        let case = fixture
            .repository
            .create_case(CaseInput {
                name: "Original".to_string(),
                examiner_name: None,
                notes: None,
            })
            .expect("case should be created");

        let updated = fixture
            .repository
            .update_case(
                case.id.clone(),
                CaseInput {
                    name: "Updated".to_string(),
                    examiner_name: Some("Analyst".to_string()),
                    notes: Some("Notes".to_string()),
                },
            )
            .expect("case should update");

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.examiner_name.as_deref(), Some("Analyst"));
        assert_eq!(updated.notes.as_deref(), Some("Notes"));

        let missing = fixture
            .repository
            .update_case(
                "case-missing".to_string(),
                CaseInput {
                    name: "Nope".to_string(),
                    examiner_name: None,
                    notes: None,
                },
            )
            .expect_err("missing case should fail");

        assert_eq!(missing, "Case not found");
    }

    #[test]
    fn list_cases_orders_by_updated_at_descending() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        let mut older = case_record("case-older", "Older");
        older.updated_at = "2026-01-01T00:00:00Z".to_string();
        let mut newer = case_record("case-newer", "Newer");
        newer.updated_at = "2026-02-01T00:00:00Z".to_string();
        store.cases = vec![older, newer];
        fixture
            .repository
            .save(&store)
            .expect("save should succeed");

        let cases = fixture
            .repository
            .list_cases()
            .expect("list should succeed");

        assert_eq!(cases[0].id, "case-newer");
        assert_eq!(cases[1].id, "case-older");
    }

    #[test]
    fn replace_imported_files_requires_case_and_replaces_matching_path() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Case"));
        fixture
            .repository
            .save(&store)
            .expect("save should succeed");

        let first = evidence_file("file-1", "case-1", "/tmp/a.pdf", 10);
        fixture
            .repository
            .replace_imported_files("case-1", vec![first])
            .expect("initial import should persist");

        let replacement = evidence_file("file-1", "case-1", "/tmp/a.pdf", 20);
        fixture
            .repository
            .replace_imported_files("case-1", vec![replacement])
            .expect("replacement should persist");

        let files = fixture
            .repository
            .get_case_files("case-1")
            .expect("files should load");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].size_bytes, 20);

        let error = fixture
            .repository
            .replace_imported_files("case-missing", vec![])
            .expect_err("missing case should fail");
        assert_eq!(error, "Case not found");
    }

    struct StoreFixture {
        _dir: PathBuf,
        repository: JsonRepository,
    }

    impl StoreFixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("pi-trace-test-{}", Uuid::new_v4()));
            fs::create_dir_all(&dir).expect("test directory should be created");
            let repository = JsonRepository::for_path(dir.join("store.json"));

            Self {
                _dir: dir,
                repository,
            }
        }
    }

    impl Drop for StoreFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self._dir);
        }
    }

    fn case_record(id: &str, name: &str) -> crate::models::CaseRecord {
        crate::models::CaseRecord {
            id: id.to_string(),
            name: name.to_string(),
            examiner_name: None,
            notes: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn evidence_file(id: &str, case_id: &str, path: &str, size_bytes: u64) -> EvidenceFile {
        EvidenceFile {
            id: id.to_string(),
            case_id: case_id.to_string(),
            original_path: path.to_string(),
            file_name: Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("file")
                .to_string(),
            extension: "pdf".to_string(),
            detected_mime_type: None,
            detected_file_type: Some("PDF".to_string()),
            size_bytes,
            imported_at: "2026-01-01T00:00:00Z".to_string(),
            analyzed_at: None,
            status: EvidenceStatus::Pending,
            error_message: None,
        }
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn clean_required(value: String, label: &str) -> Result<String, String> {
    let cleaned = value.trim().to_string();
    if cleaned.is_empty() {
        Err(format!("{label} is required"))
    } else {
        Ok(cleaned)
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let cleaned = value.trim().to_string();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    })
}
