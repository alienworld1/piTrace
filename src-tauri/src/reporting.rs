use crate::{
    models::{
        CaseReport, EvidenceFile, EvidenceStatus, FileMetadataGroup, Finding, MetadataField,
        ReportExportInput, ReportExportResult, ReportPayload, ReportSummary,
    },
    storage::{now_iso, Repository},
};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const PDF_PAGE_WIDTH: f32 = 210.0;
const PDF_PAGE_HEIGHT: f32 = 297.0;
const PDF_MARGIN: f32 = 16.0;
const PDF_LINE_HEIGHT: f32 = 6.0;

pub fn build_report_payload(
    repository: &Repository,
    case_id: &str,
    include_raw_metadata: bool,
) -> Result<ReportPayload, String> {
    let case_record = repository.get_case(case_id)?;
    let files = repository.get_case_files(case_id)?;
    let mut findings = repository.get_case_findings(case_id)?;
    findings.sort_by(|left, right| {
        severity_rank(&right.severity)
            .cmp(&severity_rank(&left.severity))
            .then_with(|| right.created_at.cmp(&left.created_at))
    });
    let metadata_by_file = repository.get_case_metadata(case_id)?;
    let raw_metadata_by_file = if include_raw_metadata {
        Some(repository.get_case_raw_metadata(case_id)?)
    } else {
        None
    };

    Ok(ReportPayload {
        summary: summarize(&files, &findings),
        case_record,
        files,
        findings,
        metadata_by_file,
        raw_metadata_by_file,
        generated_at: now_iso(),
    })
}

pub fn export_case_report(
    repository: &Repository,
    input: ReportExportInput,
) -> Result<ReportExportResult, String> {
    let format = normalize_format(&input.format)?;
    let output_path = PathBuf::from(input.output_path.trim());
    validate_output_path(&output_path, format)?;
    let payload = build_report_payload(repository, &input.case_id, input.include_raw_metadata)?;
    write_report_file(&payload, format, &output_path)?;

    let report = CaseReport {
        id: format!("report-{}", Uuid::new_v4()),
        case_id: input.case_id,
        generated_at: payload.generated_at,
        format: format.to_string(),
        include_raw_metadata: input.include_raw_metadata,
        output_path: Some(output_path.to_string_lossy().to_string()),
    };
    repository.insert_report(&report)?;

    Ok(ReportExportResult {
        report,
        output_path: output_path.to_string_lossy().to_string(),
    })
}

fn write_report_file(
    payload: &ReportPayload,
    format: &str,
    output_path: &Path,
) -> Result<(), String> {
    let parent = output_path
        .parent()
        .ok_or_else(|| "Export path must include a destination directory".to_string())?;
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Export path must include a valid file name".to_string())?;
    let temporary_path = parent.join(format!(".{file_name}.tmp-{}", Uuid::new_v4()));

    match format {
        "json" => fs::write(
            &temporary_path,
            serde_json::to_string_pretty(payload)
                .map_err(|error| format!("Could not serialize JSON report: {error}"))?,
        )
        .map_err(|error| format!("Could not write JSON report: {error}"))?,
        "html" => fs::write(&temporary_path, render_html(payload))
            .map_err(|error| format!("Could not write HTML report: {error}"))?,
        "pdf" => render_pdf(payload, &temporary_path)?,
        _ => return Err("Unsupported report format".to_string()),
    }

    if output_path.exists() {
        fs::remove_file(output_path)
            .map_err(|error| format!("Could not replace existing report file: {error}"))?;
    }
    fs::rename(&temporary_path, output_path)
        .map_err(|error| format!("Could not finalize report file: {error}"))
}

fn render_html(payload: &ReportPayload) -> String {
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "piTrace report - " (payload.case_record.name) }
                style { (PreEscaped(report_css())) }
            }
            body {
                main class="report" {
                    header class="hero" {
                        p class="eyebrow" { "piTrace evidence-style report" }
                        h1 { (payload.case_record.name) }
                        p class="muted" { "Generated " (payload.generated_at) }
                    }
                    section class="grid two" {
                        div class="panel" {
                            h2 { "Case information" }
                            dl {
                                (detail("Case ID", &payload.case_record.id))
                                (detail("Examiner", payload.case_record.examiner_name.as_deref().unwrap_or("Not recorded")))
                                (detail("Created", &payload.case_record.created_at))
                                (detail("Updated", &payload.case_record.updated_at))
                            }
                            @if let Some(notes) = &payload.case_record.notes {
                                h3 { "Notes" }
                                p { (notes) }
                            }
                        }
                        div class="panel" {
                            h2 { "Summary" }
                            div class="metrics" {
                                (metric("Evidence", payload.summary.evidence_count))
                                (metric("Findings", payload.summary.finding_count))
                                (metric("High", payload.summary.high_count))
                                (metric("Complete", payload.summary.complete_file_count))
                            }
                        }
                    }
                    section class="panel" {
                        h2 { "Evidence files and hashes" }
                        (evidence_table(&payload.files))
                    }
                    section class="panel" {
                        h2 { "Findings" }
                        @if payload.findings.is_empty() {
                            p class="muted" { "No rule-based findings are recorded for this case." }
                        } @else {
                            @for finding in &payload.findings {
                                article class="finding" {
                                    div class="finding-head" {
                                        h3 { (finding.title) }
                                        span class=(format!("badge {}", finding.severity)) { (finding.severity) }
                                    }
                                    p { (finding.description) }
                                    p class="muted" {
                                        "Category: " (finding.category) " · Confidence: " (finding.confidence)
                                    }
                                }
                            }
                        }
                    }
                    section class="panel" {
                        h2 { "File-by-file metadata" }
                        @for file in &payload.files {
                            article class="file-section" {
                                h3 { (file.file_name) }
                                p class="muted" { (file.detected_file_type.as_deref().unwrap_or("Unknown type")) }
                                (metadata_table(metadata_for_file(&payload.metadata_by_file, &file.id)))
                            }
                        }
                    }
                    @if let Some(raw_records) = &payload.raw_metadata_by_file {
                        section class="panel" {
                            h2 { "Raw metadata appendix" }
                            p class="muted" { "Raw metadata may include sensitive paths, usernames, GPS data, and software history." }
                            @for record in raw_records {
                                article class="raw-record" {
                                    h3 { (file_name_for_id(&payload.files, &record.file_id)) }
                                    p class="muted" { "Source: " (record.source) " · Extracted: " (record.extracted_at) }
                                    pre { (pretty_json(&record.data)) }
                                }
                            }
                        }
                    }
                    footer {
                        "Report generated locally by piTrace. Findings are metadata indicators for review, not proof of authorship, intent, or authenticity."
                    }
                }
            }
        }
    };
    markup.into_string()
}

fn render_pdf(payload: &ReportPayload, output_path: &Path) -> Result<(), String> {
    let (document, page, layer) = PdfDocument::new(
        format!("piTrace report - {}", payload.case_record.name),
        Mm(PDF_PAGE_WIDTH),
        Mm(PDF_PAGE_HEIGHT),
        "Report",
    );
    let font = document
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|error| format!("Could not load PDF font: {error}"))?;
    let mut current_layer = document.get_page(page).get_layer(layer);
    let mut y = PDF_PAGE_HEIGHT - PDF_MARGIN;

    for line in pdf_lines(payload) {
        if y < PDF_MARGIN {
            let (page, layer) =
                document.add_page(Mm(PDF_PAGE_WIDTH), Mm(PDF_PAGE_HEIGHT), "Report");
            current_layer = document.get_page(page).get_layer(layer);
            y = PDF_PAGE_HEIGHT - PDF_MARGIN;
        }
        let font_size = if line.starts_with("# ") {
            18.0
        } else if line.starts_with("## ") {
            13.0
        } else {
            9.0
        };
        let text = line.trim_start_matches("# ").trim_start_matches("## ");
        current_layer.use_text(text, font_size, Mm(PDF_MARGIN), Mm(y), &font);
        y -= PDF_LINE_HEIGHT;
    }

    let file = File::create(output_path)
        .map_err(|error| format!("Could not write PDF report: {error}"))?;
    document
        .save(&mut BufWriter::new(file))
        .map_err(|error| format!("Could not save PDF report: {error}"))
}

fn pdf_lines(payload: &ReportPayload) -> Vec<String> {
    let mut lines = Vec::new();
    push_wrapped(
        &mut lines,
        &format!("# piTrace report: {}", payload.case_record.name),
        90,
    );
    lines.push(format!("Generated: {}", payload.generated_at));
    lines.push(format!("Case ID: {}", payload.case_record.id));
    lines.push(format!(
        "Examiner: {}",
        payload
            .case_record
            .examiner_name
            .as_deref()
            .unwrap_or("Not recorded")
    ));
    lines.push(String::new());
    lines.push("## Summary".to_string());
    lines.push(format!(
        "Evidence: {}  Findings: {}  High: {}  Medium: {}  Low: {}",
        payload.summary.evidence_count,
        payload.summary.finding_count,
        payload.summary.high_count,
        payload.summary.medium_count,
        payload.summary.low_count
    ));
    lines.push(String::new());
    lines.push("## Evidence files and hashes".to_string());
    for file in &payload.files {
        push_wrapped(
            &mut lines,
            &format!(
                "{} | {} bytes | SHA-256: {}",
                file.file_name,
                file.size_bytes,
                file.sha256.as_deref().unwrap_or("Not available")
            ),
            105,
        );
    }
    lines.push(String::new());
    lines.push("## Findings".to_string());
    if payload.findings.is_empty() {
        lines.push("No rule-based findings are recorded for this case.".to_string());
    }
    for finding in &payload.findings {
        push_wrapped(
            &mut lines,
            &format!(
                "{} [{} / {} confidence]: {}",
                finding.title, finding.severity, finding.confidence, finding.description
            ),
            105,
        );
    }
    lines.push(String::new());
    lines.push("## Metadata fields".to_string());
    for file in &payload.files {
        lines.push(file.file_name.clone());
        for field in metadata_for_file(&payload.metadata_by_file, &file.id)
            .iter()
            .take(24)
        {
            push_wrapped(
                &mut lines,
                &format!(
                    "  {}:{} = {}",
                    field.group,
                    field.display_label.as_deref().unwrap_or(&field.key),
                    field.value
                ),
                105,
            );
        }
    }
    if let Some(raw_records) = &payload.raw_metadata_by_file {
        lines.push(String::new());
        lines.push("## Raw metadata appendix".to_string());
        for record in raw_records {
            lines.push(format!(
                "{} ({})",
                file_name_for_id(&payload.files, &record.file_id),
                record.source
            ));
            push_wrapped(&mut lines, &pretty_json(&record.data), 105);
        }
    }
    lines.push(String::new());
    push_wrapped(
        &mut lines,
        "Findings are metadata indicators for review, not proof of authorship, intent, or authenticity.",
        105,
    );
    lines
}

fn push_wrapped(lines: &mut Vec<String>, text: &str, max_chars: usize) {
    for raw_line in text.lines() {
        let mut line = String::new();
        for word in raw_line.split_whitespace() {
            if !line.is_empty() && line.len() + word.len() + 1 > max_chars {
                lines.push(line);
                line = String::new();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
}

fn evidence_table(files: &[EvidenceFile]) -> Markup {
    html! {
        table {
            thead {
                tr {
                    th { "File" }
                    th { "Type" }
                    th { "Status" }
                    th { "SHA-256" }
                }
            }
            tbody {
                @for file in files {
                    tr {
                        td { (file.file_name) }
                        td { (file.detected_file_type.as_deref().unwrap_or("Unknown")) }
                        td { (status_label(&file.status)) }
                        td class="mono" { (file.sha256.as_deref().unwrap_or("Not available")) }
                    }
                }
            }
        }
    }
}

fn metadata_table(fields: &[MetadataField]) -> Markup {
    html! {
        @if fields.is_empty() {
            p class="muted" { "No normalized metadata fields are recorded for this file." }
        } @else {
            table {
                thead {
                    tr {
                        th { "Group" }
                        th { "Field" }
                        th { "Value" }
                        th { "Category" }
                    }
                }
                tbody {
                    @for field in fields {
                        tr {
                            td { (field.group) }
                            td { (field.display_label.as_deref().unwrap_or(&field.key)) }
                            td { (field.value) }
                            td { (field.normalized_category.as_deref().unwrap_or("other")) }
                        }
                    }
                }
            }
        }
    }
}

fn metadata_for_file<'a>(groups: &'a [FileMetadataGroup], file_id: &str) -> &'a [MetadataField] {
    groups
        .iter()
        .find(|group| group.file_id == file_id)
        .map(|group| group.fields.as_slice())
        .unwrap_or(&[])
}

fn file_name_for_id<'a>(files: &'a [EvidenceFile], file_id: &str) -> &'a str {
    files
        .iter()
        .find(|file| file.id == file_id)
        .map(|file| file.file_name.as_str())
        .unwrap_or("Unknown evidence file")
}

fn summarize(files: &[EvidenceFile], findings: &[Finding]) -> ReportSummary {
    ReportSummary {
        evidence_count: files.len() as u64,
        finding_count: findings.len() as u64,
        high_count: findings
            .iter()
            .filter(|finding| finding.severity == "high")
            .count() as u64,
        medium_count: findings
            .iter()
            .filter(|finding| finding.severity == "medium")
            .count() as u64,
        low_count: findings
            .iter()
            .filter(|finding| finding.severity == "low")
            .count() as u64,
        complete_file_count: files
            .iter()
            .filter(|file| file.status == EvidenceStatus::Complete)
            .count() as u64,
        pending_file_count: files
            .iter()
            .filter(|file| {
                file.status == EvidenceStatus::Pending || file.status == EvidenceStatus::Analyzing
            })
            .count() as u64,
        error_file_count: files
            .iter()
            .filter(|file| file.status == EvidenceStatus::Error)
            .count() as u64,
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn normalize_format(format: &str) -> Result<&'static str, String> {
    match format.trim().to_ascii_lowercase().as_str() {
        "html" => Ok("html"),
        "json" => Ok("json"),
        "pdf" => Ok("pdf"),
        _ => Err("Unsupported report format".to_string()),
    }
}

fn validate_output_path(path: &Path, format: &str) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("Export path is required".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Export path must include a destination directory".to_string())?;
    if !parent.is_dir() {
        return Err("Export destination directory does not exist".to_string());
    }
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("Export path must not be a symbolic link".to_string());
    }
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .unwrap_or_default();
    if extension != format {
        return Err(format!("Export path must end with .{format}"));
    }
    Ok(())
}

fn status_label(status: &EvidenceStatus) -> &'static str {
    match status {
        EvidenceStatus::Pending => "pending",
        EvidenceStatus::Analyzing => "analyzing",
        EvidenceStatus::Complete => "complete",
        EvidenceStatus::Error => "error",
    }
}

fn detail(label: &str, value: &str) -> Markup {
    html! {
        div {
            dt { (label) }
            dd { (value) }
        }
    }
}

fn metric(label: &str, value: u64) -> Markup {
    html! {
        div class="metric" {
            span { (label) }
            strong { (value) }
        }
    }
}

fn pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn report_css() -> &'static str {
    r#"
        :root {
            color: #e4e2e3;
            background: #0e0e0f;
            font-family: Inter, system-ui, sans-serif;
        }
        * { box-sizing: border-box; }
        body { margin: 0; background: #0e0e0f; }
        .report { max-width: 1120px; margin: 0 auto; padding: 40px 24px; }
        .hero, .panel {
            border: 1px solid rgba(143, 144, 151, 0.28);
            background: #1f1f21;
            border-radius: 12px;
            padding: 24px;
            margin-bottom: 20px;
        }
        h1, h2, h3 { margin: 0; }
        h1 { font-size: 36px; }
        h2 { font-size: 20px; margin-bottom: 16px; }
        h3 { font-size: 16px; margin: 16px 0 8px; }
        p { line-height: 1.55; }
        .eyebrow {
            color: #9ee7ff;
            font-size: 12px;
            font-weight: 700;
            letter-spacing: 0.05em;
            margin: 0 0 8px;
            text-transform: uppercase;
        }
        .muted { color: #c6c6cd; }
        .mono, pre { font-family: "JetBrains Mono", Consolas, monospace; }
        .grid.two { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
        dl { display: grid; grid-template-columns: 1fr 2fr; gap: 10px 16px; margin: 0; }
        dt { color: #c6c6cd; }
        dd { margin: 0; }
        .metrics { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; }
        .metric { border: 1px solid #45474c; border-radius: 8px; padding: 14px; background: #131314; }
        .metric span { display: block; color: #c6c6cd; font-size: 12px; }
        .metric strong { display: block; margin-top: 8px; font-family: "JetBrains Mono", Consolas, monospace; font-size: 24px; }
        table { width: 100%; border-collapse: collapse; margin-top: 12px; table-layout: fixed; }
        th, td { border-bottom: 1px solid #45474c; padding: 10px; text-align: left; vertical-align: top; overflow-wrap: anywhere; }
        th { color: #bfc6db; font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; }
        .finding, .file-section, .raw-record { border-top: 1px solid #45474c; padding-top: 14px; margin-top: 14px; }
        .finding-head { display: flex; justify-content: space-between; gap: 16px; align-items: center; }
        .badge { border: 1px solid #8f9097; border-radius: 999px; padding: 4px 10px; font-size: 12px; text-transform: uppercase; }
        .badge.high { color: #ffb4ab; border-color: #ffb4ab; }
        .badge.medium { color: #e2c0a7; border-color: #e2c0a7; }
        .badge.low { color: #9dd8bd; border-color: #9dd8bd; }
        pre { max-height: none; overflow: auto; white-space: pre-wrap; background: #131314; border: 1px solid #45474c; border-radius: 8px; padding: 16px; }
        footer { color: #c6c6cd; font-size: 13px; line-height: 1.5; margin-top: 24px; }
        @media print {
            body { background: #fff; color: #111; }
            .hero, .panel { background: #fff; border-color: #bbb; break-inside: avoid; }
            .muted, dt, footer { color: #444; }
        }
    "#
}

#[cfg(test)]
mod tests {
    use super::{build_report_payload, export_case_report, render_html};
    use crate::{
        models::{
            CaseRecord, EvidenceFile, EvidenceStatus, Finding, MetadataField, RawMetadataRecord,
            ReportExportInput,
        },
        storage::Repository,
    };
    use serde_json::json;
    use std::{fs, path::Path, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn payload_includes_required_report_data() {
        let fixture = ReportFixture::new();
        fixture.seed();

        let payload = build_report_payload(&fixture.repository, "case-1", true).expect("payload");

        assert_eq!(payload.case_record.name, "Report Case");
        assert_eq!(payload.files[0].sha256.as_deref(), Some("fixture-sha256"));
        assert_eq!(payload.findings.len(), 1);
        assert_eq!(payload.metadata_by_file[0].fields[0].value, "<unsafe>");
        assert!(payload.raw_metadata_by_file.is_some());
        assert_eq!(payload.summary.evidence_count, 1);
        assert_eq!(payload.summary.finding_count, 1);
        assert!(!payload.generated_at.is_empty());
    }

    #[test]
    fn payload_omits_raw_metadata_when_requested() {
        let fixture = ReportFixture::new();
        fixture.seed();

        let payload = build_report_payload(&fixture.repository, "case-1", false).expect("payload");

        assert!(payload.raw_metadata_by_file.is_none());
    }

    #[test]
    fn html_report_escapes_metadata_values() {
        let fixture = ReportFixture::new();
        fixture.seed();
        let payload = build_report_payload(&fixture.repository, "case-1", true).expect("payload");

        let html = render_html(&payload);

        assert!(html.contains("&lt;unsafe&gt;"));
        assert!(!html.contains("<script>alert"));
    }

    #[test]
    fn json_export_writes_payload_and_records_report() {
        let fixture = ReportFixture::new();
        fixture.seed();
        let output_path = fixture.dir.join("report.json");

        let result = export_case_report(
            &fixture.repository,
            ReportExportInput {
                case_id: "case-1".to_string(),
                format: "json".to_string(),
                include_raw_metadata: true,
                output_path: output_path.to_string_lossy().to_string(),
            },
        )
        .expect("export");

        let exported: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output_path).expect("report")).expect("json");
        assert_eq!(exported["caseRecord"]["name"], "Report Case");
        assert_eq!(exported["files"][0]["sha256"], "fixture-sha256");
        assert_eq!(result.report.format, "json");
        assert_eq!(
            fixture
                .repository
                .get_case_report("case-1")
                .expect("latest report")
                .expect("report row")
                .id,
            result.report.id
        );
    }

    #[test]
    fn export_rejects_mismatched_extension_and_missing_case() {
        let fixture = ReportFixture::new();
        fixture.seed();

        let extension_error = export_case_report(
            &fixture.repository,
            ReportExportInput {
                case_id: "case-1".to_string(),
                format: "html".to_string(),
                include_raw_metadata: false,
                output_path: fixture
                    .dir
                    .join("report.json")
                    .to_string_lossy()
                    .to_string(),
            },
        )
        .expect_err("extension mismatch");
        assert_eq!(extension_error, "Export path must end with .html");

        let missing_case = export_case_report(
            &fixture.repository,
            ReportExportInput {
                case_id: "case-missing".to_string(),
                format: "json".to_string(),
                include_raw_metadata: false,
                output_path: fixture
                    .dir
                    .join("missing.json")
                    .to_string_lossy()
                    .to_string(),
            },
        )
        .expect_err("missing case");
        assert_eq!(missing_case, "Case not found");
        assert!(!fixture.dir.join("missing.json").exists());
    }

    struct ReportFixture {
        dir: PathBuf,
        repository: Repository,
    }

    impl ReportFixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("pi-trace-report-test-{}", Uuid::new_v4()));
            fs::create_dir_all(&dir).expect("test directory");
            let repository =
                Repository::for_test_path(dir.join("store.sqlite3")).expect("repository");
            Self { dir, repository }
        }

        fn seed(&self) {
            self.repository.insert_case(&case_record()).expect("case");
            self.repository
                .replace_imported_files_with_metadata(
                    "case-1",
                    vec![evidence_file()],
                    vec![raw_metadata()],
                    vec![metadata_field()],
                    vec![finding()],
                )
                .expect("case data");
        }
    }

    impl Drop for ReportFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn case_record() -> CaseRecord {
        CaseRecord {
            id: "case-1".to_string(),
            name: "Report Case".to_string(),
            examiner_name: Some("Analyst".to_string()),
            notes: Some("Review recommended.".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-02T00:00:00Z".to_string(),
        }
    }

    fn evidence_file() -> EvidenceFile {
        EvidenceFile {
            id: "file-1".to_string(),
            case_id: "case-1".to_string(),
            original_path: "/tmp/evidence.pdf".to_string(),
            file_name: Path::new("/tmp/evidence.pdf")
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap()
                .to_string(),
            extension: "pdf".to_string(),
            detected_mime_type: Some("application/pdf".to_string()),
            detected_file_type: Some("PDF".to_string()),
            size_bytes: 123,
            sha256: Some("fixture-sha256".to_string()),
            imported_at: "2026-01-01T00:00:00Z".to_string(),
            analyzed_at: Some("2026-01-01T00:01:00Z".to_string()),
            status: EvidenceStatus::Complete,
            error_message: None,
        }
    }

    fn metadata_field() -> MetadataField {
        MetadataField {
            id: "field-1".to_string(),
            file_id: "file-1".to_string(),
            group: "PDF".to_string(),
            key: "Author".to_string(),
            display_label: Some("Author".to_string()),
            value: "<unsafe>".to_string(),
            source: "exiftool".to_string(),
            normalized_category: Some("identity".to_string()),
        }
    }

    fn raw_metadata() -> RawMetadataRecord {
        RawMetadataRecord {
            file_id: "file-1".to_string(),
            source: "exiftool".to_string(),
            extracted_at: "2026-01-01T00:01:00Z".to_string(),
            data: json!({"PDF": {"Author": "<script>alert(1)</script>"}}),
        }
    }

    fn finding() -> Finding {
        Finding {
            id: "finding-1".to_string(),
            file_id: "file-1".to_string(),
            category: "identity".to_string(),
            title: "Author metadata present".to_string(),
            description: "Metadata suggests an author field is present. Review recommended."
                .to_string(),
            severity: "high".to_string(),
            confidence: "medium".to_string(),
            related_field_ids: vec!["field-1".to_string()],
            created_at: "2026-01-01T00:01:00Z".to_string(),
        }
    }
}
