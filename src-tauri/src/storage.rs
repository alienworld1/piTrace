use crate::models::{
    AppStore, CaseInput, CaseRecord, EvidenceFile, MetadataField, RawMetadataRecord,
};
use chrono::Utc;
use std::{collections::HashSet, fs, path::PathBuf};
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

    pub fn get_file_raw_metadata(
        &self,
        file_id: &str,
    ) -> Result<Option<RawMetadataRecord>, String> {
        Ok(self
            .load()?
            .raw_metadata
            .into_iter()
            .find(|record| record.file_id == file_id))
    }

    pub fn delete_case(&self, case_id: &str) -> Result<CaseRecord, String> {
        let mut store = self.load()?;
        let case_index = store
            .cases
            .iter()
            .position(|case| case.id == case_id)
            .ok_or_else(|| "Case not found".to_string())?;
        let deleted = store.cases.remove(case_index);
        let file_ids = store
            .evidence_files
            .iter()
            .filter(|file| file.case_id == case_id)
            .map(|file| file.id.clone())
            .collect::<Vec<_>>();

        store.evidence_files.retain(|file| file.case_id != case_id);
        store
            .metadata_fields
            .retain(|field| !file_ids.contains(&field.file_id));
        store
            .raw_metadata
            .retain(|record| !file_ids.contains(&record.file_id));
        store
            .findings
            .retain(|finding| !file_ids.contains(&finding.file_id));
        store.reports.retain(|report| report.case_id != case_id);

        self.save(&store)?;
        Ok(deleted)
    }

    pub fn delete_file(&self, file_id: &str) -> Result<EvidenceFile, String> {
        let mut store = self.load()?;
        let file_index = store
            .evidence_files
            .iter()
            .position(|file| file.id == file_id)
            .ok_or_else(|| "Evidence file not found".to_string())?;
        let deleted = store.evidence_files.remove(file_index);

        store
            .metadata_fields
            .retain(|field| field.file_id != deleted.id);
        store
            .raw_metadata
            .retain(|record| record.file_id != deleted.id);
        store
            .findings
            .retain(|finding| finding.file_id != deleted.id);
        store
            .reports
            .retain(|report| report.case_id != deleted.case_id);

        if let Some(case) = store
            .cases
            .iter_mut()
            .find(|case| case.id == deleted.case_id)
        {
            case.updated_at = now_iso();
        }

        self.save(&store)?;
        Ok(deleted)
    }

    #[cfg(test)]
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

    pub fn replace_imported_files_with_metadata(
        &self,
        case_id: &str,
        imported_files: Vec<EvidenceFile>,
        raw_metadata: Vec<RawMetadataRecord>,
        metadata_fields: Vec<MetadataField>,
    ) -> Result<Vec<EvidenceFile>, String> {
        let mut store = self.load()?;
        if !store.cases.iter().any(|case| case.id == case_id) {
            return Err("Case not found".to_string());
        }

        let imported_ids = imported_files
            .iter()
            .map(|file| file.id.as_str())
            .collect::<HashSet<_>>();
        if raw_metadata
            .iter()
            .any(|record| !imported_ids.contains(record.file_id.as_str()))
        {
            return Err("Raw metadata must belong to an imported file".to_string());
        }
        if metadata_fields
            .iter()
            .any(|field| !imported_ids.contains(field.file_id.as_str()))
        {
            return Err("Metadata fields must belong to an imported file".to_string());
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

        store
            .raw_metadata
            .retain(|record| !imported_ids.contains(record.file_id.as_str()));
        store.raw_metadata.extend(raw_metadata);
        store
            .metadata_fields
            .retain(|field| !imported_ids.contains(field.file_id.as_str()));
        store.metadata_fields.extend(metadata_fields);

        self.save(&store)?;
        Ok(imported_files)
    }

    #[cfg(test)]
    pub fn replace_raw_metadata(&self, record: RawMetadataRecord) -> Result<(), String> {
        let mut store = self.load()?;
        if !store
            .evidence_files
            .iter()
            .any(|file| file.id == record.file_id)
        {
            return Err("Evidence file not found".to_string());
        }

        if let Some(existing) = store
            .raw_metadata
            .iter_mut()
            .find(|existing| existing.file_id == record.file_id)
        {
            *existing = record;
        } else {
            store.raw_metadata.push(record);
        }

        self.save(&store)
    }
}

#[cfg(test)]
mod tests {
    use super::JsonRepository;
    use crate::models::{
        AppStore, CaseInput, CaseReport, EvidenceFile, EvidenceStatus, Finding, MetadataField,
        RawMetadataRecord,
    };
    use serde_json::json;
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
        assert!(store.raw_metadata.is_empty());
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

    #[test]
    fn replace_imported_files_with_metadata_updates_atomically() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Case"));
        fixture
            .repository
            .save(&store)
            .expect("fixture store should save");

        let file = evidence_file("file-1", "case-1", "/tmp/a.pdf", 10);
        fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![file.clone()],
                vec![raw_metadata("file-1", json!({"File": {"FileType": "PDF"}}))],
                vec![metadata_field("field-1", "file-1")],
            )
            .expect("atomic import should save");

        let loaded = fixture.repository.load().expect("store should load");
        assert_eq!(loaded.evidence_files.len(), 1);
        assert_eq!(loaded.raw_metadata.len(), 1);
        assert_eq!(loaded.metadata_fields.len(), 1);
        assert_eq!(loaded.raw_metadata[0].file_id, file.id);
        assert_eq!(loaded.metadata_fields[0].file_id, file.id);
        assert_eq!(
            loaded.metadata_fields[0].display_label.as_deref(),
            Some("Display name")
        );
    }

    #[test]
    fn load_accepts_metadata_fields_without_display_label() {
        let fixture = StoreFixture::new();
        fs::write(
            &fixture.repository.path,
            serde_json::to_string_pretty(&json!({
                "cases": [],
                "evidenceFiles": [],
                "metadataFields": [{
                    "id": "field-1",
                    "fileId": "file-1",
                    "group": "File",
                    "key": "FileType",
                    "value": "PDF",
                    "source": "exiftool",
                    "normalizedCategory": "technical"
                }],
                "rawMetadata": [],
                "findings": [],
                "reports": []
            }))
            .expect("fixture JSON should serialize"),
        )
        .expect("fixture store should write");

        let loaded = fixture.repository.load().expect("old store should load");

        assert_eq!(loaded.metadata_fields.len(), 1);
        assert_eq!(loaded.metadata_fields[0].key, "FileType");
        assert_eq!(loaded.metadata_fields[0].display_label, None);
    }

    #[test]
    fn replace_imported_files_with_metadata_clears_missing_metadata_for_imported_file() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Case"));
        store
            .evidence_files
            .push(evidence_file("file-1", "case-1", "/tmp/a.pdf", 10));
        store
            .raw_metadata
            .push(raw_metadata("file-1", json!({"File": {"FileType": "PDF"}})));
        store
            .metadata_fields
            .push(metadata_field("field-1", "file-1"));
        fixture
            .repository
            .save(&store)
            .expect("fixture store should save");

        fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![],
                vec![],
            )
            .expect("atomic import should save");

        let loaded = fixture.repository.load().expect("store should load");
        assert_eq!(loaded.evidence_files.len(), 1);
        assert!(loaded.raw_metadata.is_empty());
        assert!(loaded.metadata_fields.is_empty());
    }

    #[test]
    fn replace_imported_files_with_metadata_rejects_unrelated_raw_record() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Case"));
        fixture
            .repository
            .save(&store)
            .expect("fixture store should save");

        let error = fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![raw_metadata("file-2", json!({}))],
                vec![],
            )
            .expect_err("unrelated raw metadata should fail");

        assert_eq!(error, "Raw metadata must belong to an imported file");
    }

    #[test]
    fn replace_imported_files_with_metadata_rejects_unrelated_metadata_field() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Case"));
        fixture
            .repository
            .save(&store)
            .expect("fixture store should save");

        let error = fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![],
                vec![metadata_field("field-1", "file-2")],
            )
            .expect_err("unrelated metadata field should fail");

        assert_eq!(error, "Metadata fields must belong to an imported file");
    }

    #[test]
    fn raw_metadata_round_trips_and_replaces_by_file_id() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Case"));
        store
            .evidence_files
            .push(evidence_file("file-1", "case-1", "/tmp/a.jpg", 10));
        fixture
            .repository
            .save(&store)
            .expect("fixture store should save");

        fixture
            .repository
            .replace_raw_metadata(raw_metadata(
                "file-1",
                json!({"File": {"FileType": "JPEG"}}),
            ))
            .expect("raw metadata should save");
        fixture
            .repository
            .replace_raw_metadata(raw_metadata(
                "file-1",
                json!({"GPS": {"GPSLatitude": "1 deg"}}),
            ))
            .expect("raw metadata should replace");

        let loaded = fixture
            .repository
            .get_file_raw_metadata("file-1")
            .expect("raw metadata should load")
            .expect("raw metadata should exist");
        let store = fixture.repository.load().expect("store should load");

        assert_eq!(store.raw_metadata.len(), 1);
        assert_eq!(loaded.data["GPS"]["GPSLatitude"], "1 deg");
    }

    #[test]
    fn replace_raw_metadata_requires_existing_file() {
        let fixture = StoreFixture::new();

        let error = fixture
            .repository
            .replace_raw_metadata(raw_metadata("file-missing", json!({})))
            .expect_err("missing file should fail");

        assert_eq!(error, "Evidence file not found");
    }

    #[test]
    fn delete_case_removes_case_and_associated_records_only() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases = vec![
            case_record("case-1", "Delete me"),
            case_record("case-2", "Keep me"),
        ];
        store.evidence_files = vec![
            evidence_file("file-1", "case-1", "/tmp/a.pdf", 10),
            evidence_file("file-2", "case-2", "/tmp/b.pdf", 20),
        ];
        store.metadata_fields = vec![
            metadata_field("field-1", "file-1"),
            metadata_field("field-2", "file-2"),
        ];
        store.raw_metadata = vec![
            raw_metadata("file-1", json!({"File": {"FileType": "PDF"}})),
            raw_metadata("file-2", json!({"File": {"FileType": "JPEG"}})),
        ];
        store.findings = vec![
            finding("finding-1", "file-1"),
            finding("finding-2", "file-2"),
        ];
        store.reports = vec![report("report-1", "case-1"), report("report-2", "case-2")];
        fixture
            .repository
            .save(&store)
            .expect("fixture store should save");

        let deleted = fixture
            .repository
            .delete_case("case-1")
            .expect("case should delete");

        assert_eq!(deleted.id, "case-1");
        let remaining = fixture.repository.load().expect("store should load");
        assert_eq!(remaining.cases.len(), 1);
        assert_eq!(remaining.cases[0].id, "case-2");
        assert_eq!(remaining.evidence_files.len(), 1);
        assert_eq!(remaining.evidence_files[0].id, "file-2");
        assert_eq!(remaining.metadata_fields.len(), 1);
        assert_eq!(remaining.metadata_fields[0].id, "field-2");
        assert_eq!(remaining.raw_metadata.len(), 1);
        assert_eq!(remaining.raw_metadata[0].file_id, "file-2");
        assert_eq!(remaining.findings.len(), 1);
        assert_eq!(remaining.findings[0].id, "finding-2");
        assert_eq!(remaining.reports.len(), 1);
        assert_eq!(remaining.reports[0].id, "report-2");
    }

    #[test]
    fn delete_file_removes_associated_records_and_invalidates_case_report() {
        let fixture = StoreFixture::new();
        let mut store = AppStore::default();
        store.cases.push(case_record("case-1", "Case"));
        store.evidence_files = vec![
            evidence_file("file-1", "case-1", "/tmp/a.pdf", 10),
            evidence_file("file-2", "case-1", "/tmp/b.pdf", 20),
        ];
        store.metadata_fields = vec![
            metadata_field("field-1", "file-1"),
            metadata_field("field-2", "file-2"),
        ];
        store.raw_metadata = vec![
            raw_metadata("file-1", json!({"File": {"FileType": "PDF"}})),
            raw_metadata("file-2", json!({"File": {"FileType": "JPEG"}})),
        ];
        store.findings = vec![
            finding("finding-1", "file-1"),
            finding("finding-2", "file-2"),
        ];
        store.reports.push(report("report-1", "case-1"));
        fixture
            .repository
            .save(&store)
            .expect("fixture store should save");

        let deleted = fixture
            .repository
            .delete_file("file-1")
            .expect("file should delete");

        assert_eq!(deleted.id, "file-1");
        let remaining = fixture.repository.load().expect("store should load");
        assert_eq!(remaining.cases.len(), 1);
        assert_eq!(remaining.evidence_files.len(), 1);
        assert_eq!(remaining.evidence_files[0].id, "file-2");
        assert_eq!(remaining.metadata_fields.len(), 1);
        assert_eq!(remaining.metadata_fields[0].id, "field-2");
        assert_eq!(remaining.raw_metadata.len(), 1);
        assert_eq!(remaining.raw_metadata[0].file_id, "file-2");
        assert_eq!(remaining.findings.len(), 1);
        assert_eq!(remaining.findings[0].id, "finding-2");
        assert!(remaining.reports.is_empty());
        assert_ne!(remaining.cases[0].updated_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn delete_missing_case_and_file_return_not_found_errors() {
        let fixture = StoreFixture::new();

        let case_error = fixture
            .repository
            .delete_case("case-missing")
            .expect_err("missing case should fail");
        let file_error = fixture
            .repository
            .delete_file("file-missing")
            .expect_err("missing file should fail");

        assert_eq!(case_error, "Case not found");
        assert_eq!(file_error, "Evidence file not found");
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
            sha256: Some("fixture-sha256".to_string()),
            imported_at: "2026-01-01T00:00:00Z".to_string(),
            analyzed_at: None,
            status: EvidenceStatus::Pending,
            error_message: None,
        }
    }

    fn metadata_field(id: &str, file_id: &str) -> MetadataField {
        MetadataField {
            id: id.to_string(),
            file_id: file_id.to_string(),
            group: "File".to_string(),
            key: "Name".to_string(),
            display_label: Some("Display name".to_string()),
            value: "value".to_string(),
            source: "internal".to_string(),
            normalized_category: None,
        }
    }

    fn raw_metadata(file_id: &str, data: serde_json::Value) -> RawMetadataRecord {
        RawMetadataRecord {
            file_id: file_id.to_string(),
            source: "exiftool".to_string(),
            extracted_at: "2026-01-01T00:00:00Z".to_string(),
            data,
        }
    }

    fn finding(id: &str, file_id: &str) -> Finding {
        Finding {
            id: id.to_string(),
            file_id: file_id.to_string(),
            category: "identity".to_string(),
            title: "Finding".to_string(),
            description: "Description".to_string(),
            severity: "low".to_string(),
            confidence: "medium".to_string(),
            related_field_ids: Vec::new(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn report(id: &str, case_id: &str) -> CaseReport {
        CaseReport {
            id: id.to_string(),
            case_id: case_id.to_string(),
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            format: "html".to_string(),
            include_raw_metadata: true,
            output_path: None,
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
