use crate::models::{EvidenceFile, EvidenceStatus, Finding, MetadataField};
use chrono::{DateTime, NaiveDateTime};
use uuid::Uuid;

pub fn generate_findings(
    file: &EvidenceFile,
    fields: &[MetadataField],
    created_at: &str,
) -> Vec<Finding> {
    if file.status != EvidenceStatus::Complete {
        return Vec::new();
    }

    let mut findings = Vec::new();
    push_category_finding(
        &mut findings,
        file,
        fields,
        created_at,
        CategoryRule {
            category: "location",
            title: "Location metadata found",
            description: "GPS or location metadata was found in this file. These fields may reveal where the file was captured, edited, or described.",
            severity: "high",
            confidence: "high",
        },
    );
    push_category_finding(
        &mut findings,
        file,
        fields,
        created_at,
        CategoryRule {
            category: "identity",
            title: "Identity metadata found",
            description: "Author, owner, company, or creator metadata was found in this file. These fields may reveal personal or organizational context.",
            severity: "medium",
            confidence: "high",
        },
    );
    push_category_finding(
        &mut findings,
        file,
        fields,
        created_at,
        CategoryRule {
            category: "software",
            title: "Software or device metadata found",
            description: "Software, encoder, producer, or device metadata was found in this file. These fields may reveal workflow, tooling, or device context.",
            severity: "medium",
            confidence: "high",
        },
    );
    push_timestamp_findings(&mut findings, file, fields, created_at);
    push_extension_mismatch_finding(&mut findings, file, fields, created_at);

    findings
}

struct CategoryRule {
    category: &'static str,
    title: &'static str,
    description: &'static str,
    severity: &'static str,
    confidence: &'static str,
}

fn push_category_finding(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    fields: &[MetadataField],
    created_at: &str,
    rule: CategoryRule,
) {
    let related_field_ids = fields
        .iter()
        .filter(|field| field.normalized_category.as_deref() == Some(rule.category))
        .map(|field| field.id.clone())
        .collect::<Vec<_>>();

    if related_field_ids.is_empty() {
        return;
    }

    findings.push(finding(
        file,
        rule.category,
        rule.title,
        rule.description,
        rule.severity,
        rule.confidence,
        related_field_ids,
        created_at,
    ));
}

fn push_timestamp_findings(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    fields: &[MetadataField],
    created_at: &str,
) {
    let timeline_fields = fields
        .iter()
        .filter(|field| field.normalized_category.as_deref() == Some("timeline"))
        .collect::<Vec<_>>();

    if timeline_fields.is_empty() {
        findings.push(finding(
            file,
            "timeline",
            "No embedded timestamp metadata found",
            "No normalized timestamp metadata was found in this file. This may limit timeline analysis for the evidence item.",
            "low",
            "medium",
            Vec::new(),
            created_at,
        ));
        return;
    }

    let mut parsed_creation_times = Vec::new();
    let mut parsed_modification_times = Vec::new();
    let mut unparsable_field_ids = Vec::new();

    for field in timeline_fields {
        match parse_metadata_timestamp(&field.value) {
            Some(timestamp) => {
                let normalized_key = normalize_tag(&field.key);
                if is_creation_timestamp(&normalized_key) {
                    parsed_creation_times.push((field.id.clone(), timestamp));
                }
                if is_modification_timestamp(&normalized_key) {
                    parsed_modification_times.push((field.id.clone(), timestamp));
                }
            }
            None => unparsable_field_ids.push(field.id.clone()),
        }
    }

    if !unparsable_field_ids.is_empty() {
        findings.push(finding(
            file,
            "timeline",
            "Timestamp metadata could not be parsed",
            "One or more timestamp fields could not be parsed into a standard date. Review the raw metadata before relying on this file for timeline analysis.",
            "low",
            "medium",
            unparsable_field_ids,
            created_at,
        ));
    }

    if let Some(conflicting_ids) =
        timestamp_order_conflict(&parsed_creation_times, &parsed_modification_times)
    {
        findings.push(finding(
            file,
            "timeline",
            "Timestamp ordering conflict found",
            "A creation or capture timestamp appears later than a modification timestamp. This may indicate a timeline inconsistency that should be reviewed.",
            "medium",
            "high",
            conflicting_ids,
            created_at,
        ));
    }
}

fn parse_metadata_timestamp(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(date_time) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(date_time.timestamp());
    }

    for candidate in exiftool_timestamp_candidates(trimmed) {
        if let Ok(date_time) = DateTime::parse_from_str(&candidate, "%Y:%m:%d %H:%M:%S%:z") {
            return Some(date_time.timestamp());
        }
        if let Ok(date_time) = DateTime::parse_from_str(&candidate, "%Y:%m:%d %H:%M:%S%.f%:z") {
            return Some(date_time.timestamp());
        }
    }

    for format in [
        "%Y:%m:%d %H:%M:%S",
        "%Y:%m:%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
    ] {
        if let Ok(date_time) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(date_time.and_utc().timestamp());
        }
    }

    None
}

fn exiftool_timestamp_candidates(value: &str) -> Vec<String> {
    let compacted = value.replace(" UTC", "+00:00");
    let without_space_before_offset = compacted
        .rfind(' ')
        .filter(|index| compacted[*index + 1..].starts_with(['+', '-']))
        .map(|index| {
            let mut candidate = compacted.clone();
            candidate.remove(index);
            candidate
        });

    let mut candidates = vec![compacted];
    if let Some(candidate) = without_space_before_offset {
        candidates.push(candidate);
    }
    candidates
}

fn timestamp_order_conflict(
    creation_times: &[(String, i64)],
    modification_times: &[(String, i64)],
) -> Option<Vec<String>> {
    for (creation_id, creation_time) in creation_times {
        for (modification_id, modification_time) in modification_times {
            if creation_time > modification_time {
                return Some(vec![creation_id.clone(), modification_id.clone()]);
            }
        }
    }

    None
}

fn is_creation_timestamp(key: &str) -> bool {
    matches!(
        key,
        "createdate" | "datetimeoriginal" | "trackcreatedate" | "mediacreatedate"
    )
}

fn is_modification_timestamp(key: &str) -> bool {
    matches!(key, "modifydate" | "filemodifydate" | "metadatadate")
}

fn push_extension_mismatch_finding(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    fields: &[MetadataField],
    created_at: &str,
) {
    let Some(expected_extensions) = expected_extensions(file) else {
        return;
    };

    if expected_extensions
        .iter()
        .any(|extension| extension.eq_ignore_ascii_case(&file.extension))
    {
        return;
    }

    let related_field_ids = fields
        .iter()
        .filter(|field| {
            field.source == "internal"
                && field.group == "piTrace"
                && matches!(field.key.as_str(), "DetectedFileType" | "DetectedMIMEType")
        })
        .map(|field| field.id.clone())
        .collect();

    findings.push(finding(
        file,
        "integrity",
        "File extension does not match detected type",
        "The file extension does not match the detected content type. This may indicate a renamed file or a misleading extension.",
        "high",
        "high",
        related_field_ids,
        created_at,
    ));
}

fn expected_extensions(file: &EvidenceFile) -> Option<&'static [&'static str]> {
    if let Some(file_type) = file.detected_file_type.as_deref() {
        if let Some(extensions) = expected_extensions_for_type(file_type) {
            return Some(extensions);
        }
    }

    file.detected_mime_type
        .as_deref()
        .and_then(expected_extensions_for_mime_type)
}

fn expected_extensions_for_type(file_type: &str) -> Option<&'static [&'static str]> {
    match normalize_tag(file_type).as_str() {
        "jpeg" | "jpg" => Some(&["jpg", "jpeg"]),
        "png" => Some(&["png"]),
        "tiff" | "tif" => Some(&["tif", "tiff"]),
        "heic" => Some(&["heic"]),
        "pdf" => Some(&["pdf"]),
        "docx" => Some(&["docx"]),
        "pptx" => Some(&["pptx"]),
        "xlsx" => Some(&["xlsx"]),
        "mp3" => Some(&["mp3"]),
        "wav" => Some(&["wav"]),
        "m4a" => Some(&["m4a"]),
        "mp4" => Some(&["mp4"]),
        "mov" => Some(&["mov"]),
        _ => None,
    }
}

fn expected_extensions_for_mime_type(mime_type: &str) -> Option<&'static [&'static str]> {
    match mime_type.trim().to_ascii_lowercase().as_str() {
        "image/jpeg" => Some(&["jpg", "jpeg"]),
        "image/png" => Some(&["png"]),
        "image/tiff" => Some(&["tif", "tiff"]),
        "image/heic" | "image/heif" => Some(&["heic"]),
        "application/pdf" => Some(&["pdf"]),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            Some(&["docx"])
        }
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            Some(&["pptx"])
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some(&["xlsx"]),
        "audio/mpeg" => Some(&["mp3"]),
        "audio/wav" | "audio/x-wav" => Some(&["wav"]),
        "audio/mp4" | "audio/x-m4a" => Some(&["m4a"]),
        "video/mp4" => Some(&["mp4"]),
        "video/quicktime" => Some(&["mov"]),
        _ => None,
    }
}

fn finding(
    file: &EvidenceFile,
    category: &str,
    title: &str,
    description: &str,
    severity: &str,
    confidence: &str,
    related_field_ids: Vec<String>,
    created_at: &str,
) -> Finding {
    Finding {
        id: format!("finding-{}", Uuid::new_v4()),
        file_id: file.id.clone(),
        category: category.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        severity: severity.to_string(),
        confidence: confidence.to_string(),
        related_field_ids,
        created_at: created_at.to_string(),
    }
}

fn normalize_tag(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::generate_findings;
    use crate::models::{EvidenceFile, EvidenceStatus, MetadataField};

    #[test]
    fn location_fields_generate_high_confidence_gps_finding() {
        let file = complete_file("jpg", Some("JPEG"), Some("image/jpeg"));
        let fields = vec![field("field-gps", "GPS", "GPSLatitude", "location", "1.23")];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "location"
                && finding.severity == "high"
                && finding.confidence == "high"
                && finding.related_field_ids == vec!["field-gps"]
        }));
    }

    #[test]
    fn identity_fields_generate_author_finding() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));
        let fields = vec![field(
            "field-author",
            "PDF",
            "Author",
            "identity",
            "Analyst",
        )];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "identity"
                && finding.severity == "medium"
                && finding.confidence == "high"
                && finding.related_field_ids == vec!["field-author"]
        }));
    }

    #[test]
    fn software_fields_generate_workflow_finding() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));
        let fields = vec![field(
            "field-producer",
            "PDF",
            "Producer",
            "software",
            "PDF Engine",
        )];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "software"
                && finding.severity == "medium"
                && finding.confidence == "high"
                && finding.related_field_ids == vec!["field-producer"]
        }));
    }

    #[test]
    fn missing_timeline_fields_generate_low_confidence_timeline_finding() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));

        let findings = generate_findings(&file, &[], "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "timeline"
                && finding.title == "No embedded timestamp metadata found"
                && finding.severity == "low"
                && finding.confidence == "medium"
                && finding.related_field_ids.is_empty()
        }));
    }

    #[test]
    fn unparsable_timeline_fields_generate_timeline_finding() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));
        let fields = vec![field(
            "field-date",
            "PDF",
            "CreateDate",
            "timeline",
            "not a date",
        )];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "timeline"
                && finding.title == "Timestamp metadata could not be parsed"
                && finding.related_field_ids == vec!["field-date"]
        }));
    }

    #[test]
    fn creation_after_modification_generates_conflict_finding() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));
        let fields = vec![
            field(
                "field-created",
                "PDF",
                "CreateDate",
                "timeline",
                "2026:01:02 12:00:00+05:30",
            ),
            field(
                "field-modified",
                "PDF",
                "ModifyDate",
                "timeline",
                "2026:01:01 12:00:00 +05:30",
            ),
        ];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "timeline"
                && finding.title == "Timestamp ordering conflict found"
                && finding.severity == "medium"
                && finding.confidence == "high"
                && finding.related_field_ids == vec!["field-created", "field-modified"]
        }));
    }

    #[test]
    fn matching_extension_and_detected_type_does_not_flag_mismatch() {
        let file = complete_file("jpeg", Some("JPEG"), Some("image/jpeg"));

        let findings = generate_findings(&file, &[], "2026-01-01T00:00:00Z");

        assert!(!findings
            .iter()
            .any(|finding| finding.category == "integrity"));
    }

    #[test]
    fn renamed_content_generates_extension_mismatch() {
        let file = complete_file("mp3", Some("PNG"), Some("image/png"));
        let fields = vec![
            internal_field("field-type", "DetectedFileType", "PNG"),
            internal_field("field-mime", "DetectedMIMEType", "image/png"),
        ];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "integrity"
                && finding.severity == "high"
                && finding.confidence == "high"
                && finding.related_field_ids == vec!["field-type", "field-mime"]
        }));
    }

    fn complete_file(
        extension: &str,
        detected_file_type: Option<&str>,
        detected_mime_type: Option<&str>,
    ) -> EvidenceFile {
        EvidenceFile {
            id: "file-1".to_string(),
            case_id: "case-1".to_string(),
            original_path: "/tmp/sample".to_string(),
            file_name: "sample".to_string(),
            extension: extension.to_string(),
            detected_mime_type: detected_mime_type.map(str::to_string),
            detected_file_type: detected_file_type.map(str::to_string),
            size_bytes: 1,
            sha256: Some("hash".to_string()),
            imported_at: "2026-01-01T00:00:00Z".to_string(),
            analyzed_at: Some("2026-01-01T00:00:00Z".to_string()),
            status: EvidenceStatus::Complete,
            error_message: None,
        }
    }

    fn field(id: &str, group: &str, key: &str, category: &str, value: &str) -> MetadataField {
        MetadataField {
            id: id.to_string(),
            file_id: "file-1".to_string(),
            group: group.to_string(),
            key: key.to_string(),
            display_label: Some(key.to_string()),
            value: value.to_string(),
            source: "exiftool".to_string(),
            normalized_category: Some(category.to_string()),
        }
    }

    fn internal_field(id: &str, key: &str, value: &str) -> MetadataField {
        MetadataField {
            id: id.to_string(),
            file_id: "file-1".to_string(),
            group: "piTrace".to_string(),
            key: key.to_string(),
            display_label: Some(key.to_string()),
            value: value.to_string(),
            source: "internal".to_string(),
            normalized_category: Some("technical".to_string()),
        }
    }
}
