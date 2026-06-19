use crate::models::{
    CaseInput, CaseRecord, CaseReport, EvidenceFile, EvidenceStatus, Finding, MetadataField,
    RawMetadataRecord,
};
use chrono::Utc;
use rusqlite::{params, types::Type, Connection, OptionalExtension, Row, Transaction};
use std::{collections::HashSet, fs, path::PathBuf, time::Duration};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

const SCHEMA_VERSION: i64 = 1;

pub struct Repository {
    path: PathBuf,
}

impl Repository {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("Could not locate app data directory: {error}"))?;
        fs::create_dir_all(&dir)
            .map_err(|error| format!("Could not create app data directory: {error}"))?;

        Self::for_path(dir.join("pi-trace.sqlite3"))
    }

    #[cfg(test)]
    pub fn for_test_path(path: PathBuf) -> Result<Self, String> {
        Self::for_path(path)
    }

    fn for_path(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create database directory: {error}"))?;
        }

        let repository = Self { path };
        let connection = repository.connect()?;
        run_migrations(&connection)?;
        Ok(repository)
    }

    fn connect(&self) -> Result<Connection, String> {
        let connection = Connection::open(&self.path)
            .map_err(|error| format!("Could not open SQLite database: {error}"))?;
        configure_connection(&connection)?;
        Ok(connection)
    }

    pub fn list_cases(&self) -> Result<Vec<CaseRecord>, String> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, examiner_name, notes, created_at, updated_at
                 FROM cases
                 ORDER BY updated_at DESC",
            )
            .map_err(storage_error)?;

        let rows = statement
            .query_map([], case_from_row)
            .map_err(storage_error)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)
    }

    pub fn create_case(&self, input: CaseInput) -> Result<CaseRecord, String> {
        let connection = self.connect()?;
        let now = now_iso();
        let case = CaseRecord {
            id: format!("case-{}", Uuid::new_v4()),
            name: clean_required(input.name, "Case name")?,
            examiner_name: clean_optional(input.examiner_name),
            notes: clean_optional(input.notes),
            created_at: now.clone(),
            updated_at: now,
        };

        connection
            .execute(
                "INSERT INTO cases (id, name, examiner_name, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    case.id,
                    case.name,
                    case.examiner_name,
                    case.notes,
                    case.created_at,
                    case.updated_at
                ],
            )
            .map_err(storage_error)?;

        Ok(case)
    }

    pub fn update_case(&self, case_id: String, input: CaseInput) -> Result<CaseRecord, String> {
        let connection = self.connect()?;
        let now = now_iso();
        let changed = connection
            .execute(
                "UPDATE cases
                 SET name = ?1, examiner_name = ?2, notes = ?3, updated_at = ?4
                 WHERE id = ?5",
                params![
                    clean_required(input.name, "Case name")?,
                    clean_optional(input.examiner_name),
                    clean_optional(input.notes),
                    now,
                    case_id
                ],
            )
            .map_err(storage_error)?;

        if changed == 0 {
            return Err("Case not found".to_string());
        }

        self.get_case(&case_id)
    }

    pub fn case_exists(&self, case_id: &str) -> Result<bool, String> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM cases WHERE id = ?1)",
                params![case_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(storage_error)
    }

    pub fn get_case(&self, case_id: &str) -> Result<CaseRecord, String> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, name, examiner_name, notes, created_at, updated_at
                 FROM cases
                 WHERE id = ?1",
                params![case_id],
                case_from_row,
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| "Case not found".to_string())
    }

    pub fn get_case_files(&self, case_id: &str) -> Result<Vec<EvidenceFile>, String> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, case_id, original_path, file_name, extension, detected_mime_type,
                        detected_file_type, size_bytes, sha256, imported_at, analyzed_at,
                        status, error_message
                 FROM evidence_files
                 WHERE case_id = ?1
                 ORDER BY imported_at DESC",
            )
            .map_err(storage_error)?;

        let rows = statement
            .query_map(params![case_id], evidence_file_from_row)
            .map_err(storage_error)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)
    }

    pub fn get_existing_file_for_path(
        &self,
        case_id: &str,
        original_path: &str,
    ) -> Result<Option<EvidenceFile>, String> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, case_id, original_path, file_name, extension, detected_mime_type,
                        detected_file_type, size_bytes, sha256, imported_at, analyzed_at,
                        status, error_message
                 FROM evidence_files
                 WHERE case_id = ?1 AND original_path = ?2",
                params![case_id, original_path],
                evidence_file_from_row,
            )
            .optional()
            .map_err(storage_error)
    }

    pub fn get_file(&self, file_id: &str) -> Result<EvidenceFile, String> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, case_id, original_path, file_name, extension, detected_mime_type,
                        detected_file_type, size_bytes, sha256, imported_at, analyzed_at,
                        status, error_message
                 FROM evidence_files
                 WHERE id = ?1",
                params![file_id],
                evidence_file_from_row,
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| "Evidence file not found".to_string())
    }

    pub fn get_file_raw_metadata(
        &self,
        file_id: &str,
    ) -> Result<Option<RawMetadataRecord>, String> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT file_id, source, extracted_at, data_json
                 FROM raw_metadata
                 WHERE file_id = ?1",
                params![file_id],
                raw_metadata_from_row,
            )
            .optional()
            .map_err(storage_error)
    }

    pub fn delete_case(&self, case_id: &str) -> Result<CaseRecord, String> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        let deleted = transaction
            .query_row(
                "SELECT id, name, examiner_name, notes, created_at, updated_at
                 FROM cases
                 WHERE id = ?1",
                params![case_id],
                case_from_row,
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| "Case not found".to_string())?;

        transaction
            .execute("DELETE FROM cases WHERE id = ?1", params![case_id])
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;

        Ok(deleted)
    }

    pub fn delete_file(&self, file_id: &str) -> Result<EvidenceFile, String> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        let deleted = transaction
            .query_row(
                "SELECT id, case_id, original_path, file_name, extension, detected_mime_type,
                        detected_file_type, size_bytes, sha256, imported_at, analyzed_at,
                        status, error_message
                 FROM evidence_files
                 WHERE id = ?1",
                params![file_id],
                evidence_file_from_row,
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| "Evidence file not found".to_string())?;

        transaction
            .execute("DELETE FROM evidence_files WHERE id = ?1", params![file_id])
            .map_err(storage_error)?;
        transaction
            .execute(
                "DELETE FROM reports WHERE case_id = ?1",
                params![deleted.case_id],
            )
            .map_err(storage_error)?;
        transaction
            .execute(
                "UPDATE cases SET updated_at = ?1 WHERE id = ?2",
                params![now_iso(), deleted.case_id],
            )
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;

        Ok(deleted)
    }

    pub fn get_case_findings(&self, case_id: &str) -> Result<Vec<Finding>, String> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT findings.id, findings.file_id, findings.category, findings.title,
                        findings.description, findings.severity, findings.confidence,
                        findings.related_field_ids_json, findings.created_at
                 FROM findings
                 INNER JOIN evidence_files ON evidence_files.id = findings.file_id
                 WHERE evidence_files.case_id = ?1
                 ORDER BY findings.created_at DESC",
            )
            .map_err(storage_error)?;

        let rows = statement
            .query_map(params![case_id], finding_from_row)
            .map_err(storage_error)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)
    }

    pub fn get_file_findings(&self, file_id: &str) -> Result<Vec<Finding>, String> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, file_id, category, title, description, severity, confidence,
                        related_field_ids_json, created_at
                 FROM findings
                 WHERE file_id = ?1
                 ORDER BY created_at DESC",
            )
            .map_err(storage_error)?;

        let rows = statement
            .query_map(params![file_id], finding_from_row)
            .map_err(storage_error)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)
    }

    pub fn get_file_metadata(&self, file_id: &str) -> Result<Vec<MetadataField>, String> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, file_id, field_group, field_key, display_label, value, source,
                        normalized_category
                 FROM metadata_fields
                 WHERE file_id = ?1
                 ORDER BY rowid ASC",
            )
            .map_err(storage_error)?;

        let rows = statement
            .query_map(params![file_id], metadata_field_from_row)
            .map_err(storage_error)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)
    }

    pub fn get_finding(&self, finding_id: &str) -> Result<Finding, String> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, file_id, category, title, description, severity, confidence,
                        related_field_ids_json, created_at
                 FROM findings
                 WHERE id = ?1",
                params![finding_id],
                finding_from_row,
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| "Finding not found".to_string())
    }

    pub fn get_case_report(&self, case_id: &str) -> Result<Option<CaseReport>, String> {
        let connection = self.connect()?;
        connection
            .query_row(
                "SELECT id, case_id, generated_at, format, include_raw_metadata, output_path
                 FROM reports
                 WHERE case_id = ?1
                 ORDER BY generated_at DESC
                 LIMIT 1",
                params![case_id],
                case_report_from_row,
            )
            .optional()
            .map_err(storage_error)
    }

    pub fn replace_imported_files_with_metadata(
        &self,
        case_id: &str,
        imported_files: Vec<EvidenceFile>,
        raw_metadata: Vec<RawMetadataRecord>,
        metadata_fields: Vec<MetadataField>,
    ) -> Result<Vec<EvidenceFile>, String> {
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

        let mut connection = self.connect()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        if !case_exists_in_transaction(&transaction, case_id)? {
            return Err("Case not found".to_string());
        }

        let now = now_iso();
        for imported in &imported_files {
            insert_or_replace_file(&transaction, imported)?;
        }
        for file_id in imported_ids {
            transaction
                .execute(
                    "DELETE FROM raw_metadata WHERE file_id = ?1",
                    params![file_id],
                )
                .map_err(storage_error)?;
            transaction
                .execute(
                    "DELETE FROM metadata_fields WHERE file_id = ?1",
                    params![file_id],
                )
                .map_err(storage_error)?;
        }
        for record in &raw_metadata {
            insert_raw_metadata(&transaction, record)?;
        }
        for field in &metadata_fields {
            insert_metadata_field(&transaction, field)?;
        }
        transaction
            .execute(
                "UPDATE cases SET updated_at = ?1 WHERE id = ?2",
                params![now, case_id],
            )
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;

        Ok(imported_files)
    }

    #[cfg(test)]
    pub(crate) fn insert_case(&self, case: &CaseRecord) -> Result<(), String> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT INTO cases (id, name, examiner_name, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    case.id,
                    case.name,
                    case.examiner_name,
                    case.notes,
                    case.created_at,
                    case.updated_at
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    #[cfg(test)]
    fn insert_finding(&self, finding: &Finding) -> Result<(), String> {
        let connection = self.connect()?;
        let related_field_ids_json = serde_json::to_string(&finding.related_field_ids)
            .map_err(|error| format!("Could not serialize finding field references: {error}"))?;
        connection
            .execute(
                "INSERT INTO findings (id, file_id, category, title, description, severity,
                                       confidence, related_field_ids_json, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    finding.id,
                    finding.file_id,
                    finding.category,
                    finding.title,
                    finding.description,
                    finding.severity,
                    finding.confidence,
                    related_field_ids_json,
                    finding.created_at
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    #[cfg(test)]
    fn insert_report(&self, report: &CaseReport) -> Result<(), String> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT INTO reports (id, case_id, generated_at, format, include_raw_metadata,
                                      output_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    report.id,
                    report.case_id,
                    report.generated_at,
                    report.format,
                    report.include_raw_metadata,
                    report.output_path
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}

fn configure_connection(connection: &Connection) -> Result<(), String> {
    connection
        .busy_timeout(Duration::from_millis(2_500))
        .map_err(storage_error)?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(storage_error)?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(storage_error)?;
    Ok(())
}

fn run_migrations(connection: &Connection) -> Result<(), String> {
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(storage_error)?;
    if version > SCHEMA_VERSION {
        return Err(format!(
            "SQLite database schema version {version} is newer than this piTrace build supports"
        ));
    }
    if version == SCHEMA_VERSION {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            BEGIN;

            CREATE TABLE cases (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                examiner_name TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE evidence_files (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
                original_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                detected_mime_type TEXT,
                detected_file_type TEXT,
                size_bytes INTEGER NOT NULL,
                sha256 TEXT,
                imported_at TEXT NOT NULL,
                analyzed_at TEXT,
                status TEXT NOT NULL CHECK (status IN ('pending', 'analyzing', 'complete', 'error')),
                error_message TEXT,
                UNIQUE(case_id, original_path)
            );

            CREATE TABLE metadata_fields (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL REFERENCES evidence_files(id) ON DELETE CASCADE,
                field_group TEXT NOT NULL,
                field_key TEXT NOT NULL,
                display_label TEXT,
                value TEXT NOT NULL,
                source TEXT NOT NULL,
                normalized_category TEXT
            );

            CREATE TABLE raw_metadata (
                file_id TEXT PRIMARY KEY REFERENCES evidence_files(id) ON DELETE CASCADE,
                source TEXT NOT NULL,
                extracted_at TEXT NOT NULL,
                data_json TEXT NOT NULL
            );

            CREATE TABLE findings (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL REFERENCES evidence_files(id) ON DELETE CASCADE,
                category TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                severity TEXT NOT NULL,
                confidence TEXT NOT NULL,
                related_field_ids_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE reports (
                id TEXT PRIMARY KEY,
                case_id TEXT NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
                generated_at TEXT NOT NULL,
                format TEXT NOT NULL,
                include_raw_metadata INTEGER NOT NULL,
                output_path TEXT
            );

            CREATE INDEX idx_evidence_files_case_imported
                ON evidence_files(case_id, imported_at DESC);
            CREATE INDEX idx_metadata_fields_file ON metadata_fields(file_id);
            CREATE INDEX idx_findings_file ON findings(file_id);
            CREATE INDEX idx_reports_case ON reports(case_id);

            PRAGMA user_version = 1;
            COMMIT;
            ",
        )
        .map_err(storage_error)?;

    Ok(())
}

fn case_exists_in_transaction(
    transaction: &Transaction<'_>,
    case_id: &str,
) -> Result<bool, String> {
    transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM cases WHERE id = ?1)",
            params![case_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(storage_error)
}

fn insert_or_replace_file(
    transaction: &Transaction<'_>,
    file: &EvidenceFile,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO evidence_files (
                id, case_id, original_path, file_name, extension, detected_mime_type,
                detected_file_type, size_bytes, sha256, imported_at, analyzed_at, status,
                error_message
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(case_id, original_path) DO UPDATE SET
                id = excluded.id,
                file_name = excluded.file_name,
                extension = excluded.extension,
                detected_mime_type = excluded.detected_mime_type,
                detected_file_type = excluded.detected_file_type,
                size_bytes = excluded.size_bytes,
                sha256 = excluded.sha256,
                imported_at = excluded.imported_at,
                analyzed_at = excluded.analyzed_at,
                status = excluded.status,
                error_message = excluded.error_message",
            params![
                file.id,
                file.case_id,
                file.original_path,
                file.file_name,
                file.extension,
                file.detected_mime_type,
                file.detected_file_type,
                u64_to_i64(file.size_bytes)?,
                file.sha256,
                file.imported_at,
                file.analyzed_at,
                status_to_db(&file.status),
                file.error_message
            ],
        )
        .map_err(storage_error)?;
    Ok(())
}

fn insert_metadata_field(
    transaction: &Transaction<'_>,
    field: &MetadataField,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO metadata_fields (
                id, file_id, field_group, field_key, display_label, value, source,
                normalized_category
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                field.id,
                field.file_id,
                field.group,
                field.key,
                field.display_label,
                field.value,
                field.source,
                field.normalized_category
            ],
        )
        .map_err(storage_error)?;
    Ok(())
}

fn insert_raw_metadata(
    transaction: &Transaction<'_>,
    record: &RawMetadataRecord,
) -> Result<(), String> {
    let data_json = serde_json::to_string(&record.data)
        .map_err(|error| format!("Could not serialize raw metadata: {error}"))?;
    transaction
        .execute(
            "INSERT INTO raw_metadata (file_id, source, extracted_at, data_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                record.file_id,
                record.source,
                record.extracted_at,
                data_json
            ],
        )
        .map_err(storage_error)?;
    Ok(())
}

fn case_from_row(row: &Row<'_>) -> rusqlite::Result<CaseRecord> {
    Ok(CaseRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        examiner_name: row.get(2)?,
        notes: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn evidence_file_from_row(row: &Row<'_>) -> rusqlite::Result<EvidenceFile> {
    let size_bytes: i64 = row.get(7)?;
    let status: String = row.get(11)?;
    Ok(EvidenceFile {
        id: row.get(0)?,
        case_id: row.get(1)?,
        original_path: row.get(2)?,
        file_name: row.get(3)?,
        extension: row.get(4)?,
        detected_mime_type: row.get(5)?,
        detected_file_type: row.get(6)?,
        size_bytes: i64_to_u64(size_bytes, 7)?,
        sha256: row.get(8)?,
        imported_at: row.get(9)?,
        analyzed_at: row.get(10)?,
        status: status_from_db(&status, 11)?,
        error_message: row.get(12)?,
    })
}

fn metadata_field_from_row(row: &Row<'_>) -> rusqlite::Result<MetadataField> {
    Ok(MetadataField {
        id: row.get(0)?,
        file_id: row.get(1)?,
        group: row.get(2)?,
        key: row.get(3)?,
        display_label: row.get(4)?,
        value: row.get(5)?,
        source: row.get(6)?,
        normalized_category: row.get(7)?,
    })
}

fn raw_metadata_from_row(row: &Row<'_>) -> rusqlite::Result<RawMetadataRecord> {
    let data_json: String = row.get(3)?;
    let data = serde_json::from_str(&data_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(error))
    })?;
    Ok(RawMetadataRecord {
        file_id: row.get(0)?,
        source: row.get(1)?,
        extracted_at: row.get(2)?,
        data,
    })
}

fn finding_from_row(row: &Row<'_>) -> rusqlite::Result<Finding> {
    let related_field_ids_json: String = row.get(7)?;
    let related_field_ids = serde_json::from_str(&related_field_ids_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(7, Type::Text, Box::new(error))
    })?;
    Ok(Finding {
        id: row.get(0)?,
        file_id: row.get(1)?,
        category: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        severity: row.get(5)?,
        confidence: row.get(6)?,
        related_field_ids,
        created_at: row.get(8)?,
    })
}

fn case_report_from_row(row: &Row<'_>) -> rusqlite::Result<CaseReport> {
    Ok(CaseReport {
        id: row.get(0)?,
        case_id: row.get(1)?,
        generated_at: row.get(2)?,
        format: row.get(3)?,
        include_raw_metadata: row.get(4)?,
        output_path: row.get(5)?,
    })
}

fn storage_error(error: rusqlite::Error) -> String {
    format!("SQLite storage error: {error}")
}

fn u64_to_i64(value: u64) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| "File size is too large to store".to_string())
}

fn i64_to_u64(value: i64, column: usize) -> rusqlite::Result<u64> {
    u64::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Integer, Box::new(error))
    })
}

fn status_to_db(status: &EvidenceStatus) -> &'static str {
    match status {
        EvidenceStatus::Pending => "pending",
        EvidenceStatus::Analyzing => "analyzing",
        EvidenceStatus::Complete => "complete",
        EvidenceStatus::Error => "error",
    }
}

fn status_from_db(value: &str, column: usize) -> rusqlite::Result<EvidenceStatus> {
    match value {
        "pending" => Ok(EvidenceStatus::Pending),
        "analyzing" => Ok(EvidenceStatus::Analyzing),
        "complete" => Ok(EvidenceStatus::Complete),
        "error" => Ok(EvidenceStatus::Error),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            column,
            Type::Text,
            format!("Unknown evidence status: {value}").into(),
        )),
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

#[cfg(test)]
mod tests {
    use super::Repository;
    use crate::models::{
        CaseInput, CaseRecord, CaseReport, EvidenceFile, EvidenceStatus, Finding, MetadataField,
        RawMetadataRecord,
    };
    use serde_json::json;
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use uuid::Uuid;

    #[test]
    fn new_database_starts_empty() {
        let fixture = StoreFixture::new();

        assert!(fixture
            .repository
            .list_cases()
            .expect("cases should load")
            .is_empty());
        assert!(fixture
            .repository
            .get_case_files("case-1")
            .expect("files should load")
            .is_empty());
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
        let mut older = case_record("case-older", "Older");
        older.updated_at = "2026-01-01T00:00:00Z".to_string();
        let mut newer = case_record("case-newer", "Newer");
        newer.updated_at = "2026-02-01T00:00:00Z".to_string();
        fixture.repository.insert_case(&older).expect("older case");
        fixture.repository.insert_case(&newer).expect("newer case");

        let cases = fixture
            .repository
            .list_cases()
            .expect("list should succeed");

        assert_eq!(cases[0].id, "case-newer");
        assert_eq!(cases[1].id, "case-older");
    }

    #[test]
    fn replace_imported_files_with_metadata_updates_atomically() {
        let fixture = StoreFixture::new();
        fixture
            .repository
            .insert_case(&case_record("case-1", "Case"))
            .expect("case should save");

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

        let files = fixture.repository.get_case_files("case-1").expect("files");
        let raw = fixture
            .repository
            .get_file_raw_metadata("file-1")
            .expect("raw metadata")
            .expect("raw metadata should exist");
        let fields = fixture
            .repository
            .get_file_metadata("file-1")
            .expect("metadata fields");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].id, file.id);
        assert_eq!(raw.data["File"]["FileType"], "PDF");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].display_label.as_deref(), Some("Display name"));
    }

    #[test]
    fn replace_imported_files_with_metadata_clears_missing_metadata_for_imported_file() {
        let fixture = StoreFixture::new();
        fixture
            .repository
            .insert_case(&case_record("case-1", "Case"))
            .expect("case should save");
        fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![raw_metadata("file-1", json!({"File": {"FileType": "PDF"}}))],
                vec![metadata_field("field-1", "file-1")],
            )
            .expect("initial import should save");

        fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![],
                vec![],
            )
            .expect("atomic import should save");

        assert!(fixture
            .repository
            .get_file_raw_metadata("file-1")
            .expect("raw metadata should load")
            .is_none());
        assert!(fixture
            .repository
            .get_file_metadata("file-1")
            .expect("metadata fields should load")
            .is_empty());
    }

    #[test]
    fn replace_imported_files_with_metadata_rejects_unrelated_records() {
        let fixture = StoreFixture::new();
        fixture
            .repository
            .insert_case(&case_record("case-1", "Case"))
            .expect("case should save");

        let raw_error = fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![raw_metadata("file-2", json!({}))],
                vec![],
            )
            .expect_err("unrelated raw metadata should fail");
        let field_error = fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![],
                vec![metadata_field("field-1", "file-2")],
            )
            .expect_err("unrelated metadata field should fail");

        assert_eq!(raw_error, "Raw metadata must belong to an imported file");
        assert_eq!(
            field_error,
            "Metadata fields must belong to an imported file"
        );
    }

    #[test]
    fn replace_imported_files_with_metadata_requires_case() {
        let fixture = StoreFixture::new();

        let error = fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-missing",
                vec![evidence_file("file-1", "case-missing", "/tmp/a.pdf", 10)],
                vec![],
                vec![],
            )
            .expect_err("missing case should fail");

        assert_eq!(error, "Case not found");
    }

    #[test]
    fn delete_case_removes_case_and_associated_records_only() {
        let fixture = StoreFixture::new();
        fixture
            .repository
            .insert_case(&case_record("case-1", "Delete me"))
            .expect("case 1 should save");
        fixture
            .repository
            .insert_case(&case_record("case-2", "Keep me"))
            .expect("case 2 should save");
        fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![evidence_file("file-1", "case-1", "/tmp/a.pdf", 10)],
                vec![raw_metadata("file-1", json!({"File": {"FileType": "PDF"}}))],
                vec![metadata_field("field-1", "file-1")],
            )
            .expect("case 1 data should save");
        fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-2",
                vec![evidence_file("file-2", "case-2", "/tmp/b.pdf", 20)],
                vec![raw_metadata(
                    "file-2",
                    json!({"File": {"FileType": "JPEG"}}),
                )],
                vec![metadata_field("field-2", "file-2")],
            )
            .expect("case 2 data should save");
        fixture
            .repository
            .insert_finding(&finding("finding-1", "file-1"))
            .expect("finding 1 should save");
        fixture
            .repository
            .insert_finding(&finding("finding-2", "file-2"))
            .expect("finding 2 should save");
        fixture
            .repository
            .insert_report(&report("report-1", "case-1"))
            .expect("report 1 should save");
        fixture
            .repository
            .insert_report(&report("report-2", "case-2"))
            .expect("report 2 should save");

        let deleted = fixture
            .repository
            .delete_case("case-1")
            .expect("case should delete");

        assert_eq!(deleted.id, "case-1");
        assert!(fixture.repository.get_case("case-1").is_err());
        assert_eq!(
            fixture
                .repository
                .get_case_files("case-2")
                .expect("case 2 files")[0]
                .id,
            "file-2"
        );
        assert!(fixture
            .repository
            .get_file_metadata("file-2")
            .expect("case 2 metadata")
            .iter()
            .any(|field| field.id == "field-2"));
        assert!(fixture
            .repository
            .get_file_raw_metadata("file-2")
            .expect("case 2 raw")
            .is_some());
        assert_eq!(
            fixture
                .repository
                .get_case_findings("case-2")
                .expect("case 2 findings")[0]
                .id,
            "finding-2"
        );
        assert_eq!(
            fixture
                .repository
                .get_case_report("case-2")
                .expect("case 2 report")
                .expect("report should exist")
                .id,
            "report-2"
        );
    }

    #[test]
    fn delete_file_removes_associated_records_and_invalidates_case_report() {
        let fixture = StoreFixture::new();
        fixture
            .repository
            .insert_case(&case_record("case-1", "Case"))
            .expect("case should save");
        fixture
            .repository
            .replace_imported_files_with_metadata(
                "case-1",
                vec![
                    evidence_file("file-1", "case-1", "/tmp/a.pdf", 10),
                    evidence_file("file-2", "case-1", "/tmp/b.pdf", 20),
                ],
                vec![
                    raw_metadata("file-1", json!({"File": {"FileType": "PDF"}})),
                    raw_metadata("file-2", json!({"File": {"FileType": "JPEG"}})),
                ],
                vec![
                    metadata_field("field-1", "file-1"),
                    metadata_field("field-2", "file-2"),
                ],
            )
            .expect("files should save");
        fixture
            .repository
            .insert_finding(&finding("finding-1", "file-1"))
            .expect("finding 1 should save");
        fixture
            .repository
            .insert_finding(&finding("finding-2", "file-2"))
            .expect("finding 2 should save");
        fixture
            .repository
            .insert_report(&report("report-1", "case-1"))
            .expect("report should save");

        let deleted = fixture
            .repository
            .delete_file("file-1")
            .expect("file should delete");

        assert_eq!(deleted.id, "file-1");
        assert!(fixture.repository.get_file("file-1").is_err());
        assert_eq!(
            fixture
                .repository
                .get_case_files("case-1")
                .expect("files")
                .len(),
            1
        );
        assert!(fixture
            .repository
            .get_file_metadata("file-1")
            .expect("metadata")
            .is_empty());
        assert!(fixture
            .repository
            .get_file_raw_metadata("file-1")
            .expect("raw")
            .is_none());
        assert!(fixture
            .repository
            .get_file_findings("file-1")
            .expect("findings")
            .is_empty());
        assert!(fixture
            .repository
            .get_case_report("case-1")
            .expect("report lookup")
            .is_none());
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
        repository: Repository,
    }

    impl StoreFixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("pi-trace-test-{}", Uuid::new_v4()));
            fs::create_dir_all(&dir).expect("test directory should be created");
            let repository = Repository::for_test_path(dir.join("store.sqlite3"))
                .expect("repository should initialize");

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

    fn case_record(id: &str, name: &str) -> CaseRecord {
        CaseRecord {
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
            related_field_ids: vec!["field-1".to_string()],
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
