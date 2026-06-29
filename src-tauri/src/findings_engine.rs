use crate::models::{EvidenceFile, EvidenceStatus, Finding, MetadataField};
use chrono::{DateTime, NaiveDateTime};
use uuid::Uuid;

const CATEGORY_IDENTITY: &str = "identity";
const CATEGORY_INTEGRITY: &str = "integrity";
const CATEGORY_LOCATION: &str = "location";
const CATEGORY_PRIVACY: &str = "privacy";
const CATEGORY_SOFTWARE: &str = "software";
const CATEGORY_TIMELINE: &str = "timeline";

const SEVERITY_HIGH: &str = "high";
const SEVERITY_MEDIUM: &str = "medium";
const SEVERITY_LOW: &str = "low";

const CONFIDENCE_HIGH: &str = "high";
const CONFIDENCE_MEDIUM: &str = "medium";

const LARGE_TIMESTAMP_SEPARATION_SECONDS: i64 = 30 * 24 * 60 * 60;

pub fn generate_findings(
    file: &EvidenceFile,
    fields: &[MetadataField],
    created_at: &str,
) -> Vec<Finding> {
    if file.status != EvidenceStatus::Complete {
        return Vec::new();
    }

    let analysis = FieldAnalysis::from_fields(fields);
    let mut findings = Vec::new();

    push_gps_finding(&mut findings, file, &analysis, created_at);
    push_identity_finding(&mut findings, file, &analysis, created_at);
    push_software_finding(&mut findings, file, &analysis, created_at);
    push_timestamp_findings(&mut findings, file, &analysis, created_at);
    push_extension_mismatch_finding(&mut findings, file, &analysis, created_at);
    push_privacy_finding(&mut findings, file, &analysis, created_at);

    findings
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TimestampRole {
    Creation,
    Modification,
    Other,
}

struct TimestampField {
    id: String,
    timestamp: Option<i64>,
    role: TimestampRole,
}

struct FindingInput {
    category: &'static str,
    title: &'static str,
    description: String,
    severity: &'static str,
    confidence: &'static str,
    related_field_ids: Vec<String>,
}

#[derive(Default)]
struct FieldAnalysis {
    gps_latitude_ids: Vec<String>,
    gps_longitude_ids: Vec<String>,
    gps_position_ids: Vec<String>,
    identity_direct_ids: Vec<String>,
    identity_organization_ids: Vec<String>,
    software_tool_ids: Vec<String>,
    device_ids: Vec<String>,
    serial_ids: Vec<String>,
    timeline_fields: Vec<TimestampField>,
    detected_type_ids: Vec<String>,
}

impl FieldAnalysis {
    fn from_fields(fields: &[MetadataField]) -> Self {
        let mut analysis = Self::default();

        for field in fields {
            let normalized_key = normalize_tag(&field.key);
            let normalized_group = normalize_tag(&field.group);
            let category = field.normalized_category.as_deref();

            match category {
                Some(CATEGORY_LOCATION) => {
                    if normalized_key == "gpslatitude" {
                        analysis.gps_latitude_ids.push(field.id.clone());
                    } else if normalized_key == "gpslongitude" {
                        analysis.gps_longitude_ids.push(field.id.clone());
                    } else if normalized_key == "gpsposition" {
                        analysis.gps_position_ids.push(field.id.clone());
                    }
                }
                Some(CATEGORY_IDENTITY) => {
                    if is_organization_identity_field(&normalized_key) {
                        analysis.identity_organization_ids.push(field.id.clone());
                    } else {
                        analysis.identity_direct_ids.push(field.id.clone());
                    }
                }
                Some(CATEGORY_SOFTWARE) => {
                    if is_device_field(&normalized_key) {
                        analysis.device_ids.push(field.id.clone());
                    } else {
                        analysis.software_tool_ids.push(field.id.clone());
                    }
                }
                Some(CATEGORY_TIMELINE) => analysis.timeline_fields.push(TimestampField {
                    id: field.id.clone(),
                    timestamp: parse_metadata_timestamp(&field.value),
                    role: timestamp_role(&normalized_key),
                }),
                Some("technical") if is_serial_field(&normalized_key) => {
                    analysis.serial_ids.push(field.id.clone());
                }
                _ => {}
            }

            if field.source == "internal"
                && normalized_group == "pitrace"
                && matches!(
                    normalized_key.as_str(),
                    "detectedfiletype" | "detectedmimetype"
                )
            {
                analysis.detected_type_ids.push(field.id.clone());
            }
        }

        analysis.deduplicate();
        analysis
    }

    fn deduplicate(&mut self) {
        dedup_ids(&mut self.gps_latitude_ids);
        dedup_ids(&mut self.gps_longitude_ids);
        dedup_ids(&mut self.gps_position_ids);
        dedup_ids(&mut self.identity_direct_ids);
        dedup_ids(&mut self.identity_organization_ids);
        dedup_ids(&mut self.software_tool_ids);
        dedup_ids(&mut self.device_ids);
        dedup_ids(&mut self.serial_ids);
        dedup_ids(&mut self.detected_type_ids);
    }

    fn gps_coordinate_ids(&self) -> Vec<String> {
        if !self.gps_position_ids.is_empty() {
            return self.gps_position_ids.clone();
        }

        if self.gps_latitude_ids.is_empty() || self.gps_longitude_ids.is_empty() {
            return Vec::new();
        }

        let mut ids = Vec::new();
        ids.extend(self.gps_latitude_ids.clone());
        ids.extend(self.gps_longitude_ids.clone());
        dedup_ids(&mut ids);
        ids
    }

    fn identity_ids(&self) -> Vec<String> {
        merge_ids(&[&self.identity_direct_ids, &self.identity_organization_ids])
    }

    fn privacy_field_ids(&self) -> Vec<String> {
        merge_ids(&[
            &self.gps_coordinate_ids(),
            &self.identity_direct_ids,
            &self.identity_organization_ids,
            &self.device_ids,
            &self.serial_ids,
            &self.software_tool_ids,
        ])
    }
}

fn push_gps_finding(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    analysis: &FieldAnalysis,
    created_at: &str,
) {
    let related_field_ids = analysis.gps_coordinate_ids();
    if related_field_ids.is_empty() {
        return;
    }

    findings.push(finding(
        file,
        created_at,
        FindingInput {
            category: CATEGORY_LOCATION,
            title: "GPS coordinates found",
            description: "This file contains embedded GPS coordinates. This may reveal where the file was captured or created.".to_string(),
            severity: SEVERITY_HIGH,
            confidence: CONFIDENCE_HIGH,
            related_field_ids,
        },
    ));
}

fn push_identity_finding(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    analysis: &FieldAnalysis,
    created_at: &str,
) {
    let related_field_ids = analysis.identity_ids();
    if related_field_ids.is_empty() {
        return;
    }

    let confidence = if analysis.identity_direct_ids.is_empty() {
        CONFIDENCE_MEDIUM
    } else {
        CONFIDENCE_HIGH
    };

    findings.push(finding(
        file,
        created_at,
        FindingInput {
            category: CATEGORY_IDENTITY,
            title: "Possible user identity metadata found",
            description: "This file contains author, user, owner, company, or creator metadata that may identify a person, device user, or organization.".to_string(),
            severity: SEVERITY_MEDIUM,
            confidence,
            related_field_ids,
        },
    ));
}

fn push_software_finding(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    analysis: &FieldAnalysis,
    created_at: &str,
) {
    let related_field_ids = merge_ids(&[&analysis.software_tool_ids, &analysis.device_ids]);
    if related_field_ids.is_empty() {
        return;
    }

    findings.push(finding(
        file,
        created_at,
        FindingInput {
            category: CATEGORY_SOFTWARE,
            title: "Software or editing tool detected",
            description: "Metadata indicates that this file was created, processed, encoded, or edited using the listed software or device information.".to_string(),
            severity: SEVERITY_LOW,
            confidence: CONFIDENCE_HIGH,
            related_field_ids,
        },
    ));
}

fn push_timestamp_findings(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    analysis: &FieldAnalysis,
    created_at: &str,
) {
    if analysis.timeline_fields.is_empty() {
        findings.push(finding(
            file,
            created_at,
            FindingInput {
                category: CATEGORY_TIMELINE,
                title: "No embedded timestamp metadata found",
                description: "No normalized timestamp metadata was found in this file. This may limit timeline analysis for the evidence item.".to_string(),
                severity: SEVERITY_LOW,
                confidence: CONFIDENCE_MEDIUM,
                related_field_ids: Vec::new(),
            },
        ));
        return;
    }

    let unparsable_field_ids = analysis
        .timeline_fields
        .iter()
        .filter(|field| field.timestamp.is_none())
        .map(|field| field.id.clone())
        .collect::<Vec<_>>();
    if !unparsable_field_ids.is_empty() {
        findings.push(finding(
            file,
            created_at,
            FindingInput {
                category: CATEGORY_TIMELINE,
                title: "Timestamp metadata could not be parsed",
                description: "One or more timestamp fields could not be parsed into a standard date. Review the raw metadata before relying on this file for timeline analysis.".to_string(),
                severity: SEVERITY_LOW,
                confidence: CONFIDENCE_MEDIUM,
                related_field_ids: unparsable_field_ids,
            },
        ));
    }

    if let Some(conflicting_ids) = timestamp_order_conflict(&analysis.timeline_fields) {
        findings.push(finding(
            file,
            created_at,
            FindingInput {
                category: CATEGORY_TIMELINE,
                title: "Potential timestamp inconsistency",
                description: "Some metadata timestamps appear inconsistent or unusually separated. This may be normal, but review is recommended.".to_string(),
                severity: SEVERITY_MEDIUM,
                confidence: CONFIDENCE_MEDIUM,
                related_field_ids: conflicting_ids,
            },
        ));
    }
}

fn push_extension_mismatch_finding(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    analysis: &FieldAnalysis,
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

    findings.push(finding(
        file,
        created_at,
        FindingInput {
            category: CATEGORY_INTEGRITY,
            title: "File extension does not match detected type",
            description: "The declared file extension differs from the detected file type. This may indicate renaming, conversion, or an intentionally misleading extension.".to_string(),
            severity: SEVERITY_HIGH,
            confidence: CONFIDENCE_HIGH,
            related_field_ids: analysis.detected_type_ids.clone(),
        },
    ));
}

fn push_privacy_finding(
    findings: &mut Vec<Finding>,
    file: &EvidenceFile,
    analysis: &FieldAnalysis,
    created_at: &str,
) {
    let score = privacy_score(findings, analysis);
    if score == 0 {
        return;
    }

    let severity = match score {
        0..=20 => SEVERITY_LOW,
        21..=50 => SEVERITY_MEDIUM,
        _ => SEVERITY_HIGH,
    };
    let mut related_field_ids = analysis.privacy_field_ids();
    for finding in findings.iter() {
        if matches!(
            finding.category.as_str(),
            CATEGORY_LOCATION | CATEGORY_TIMELINE | CATEGORY_INTEGRITY
        ) {
            related_field_ids.extend(finding.related_field_ids.clone());
        }
    }
    dedup_ids(&mut related_field_ids);

    findings.push(finding(
        file,
        created_at,
        FindingInput {
            category: CATEGORY_PRIVACY,
            title: "Metadata privacy exposure detected",
            description: format!(
                "This file contains metadata that may reveal personal, location, device, organization, or software information. Privacy exposure score: {score}. This score is a triage aid only and is not a legal or definitive risk score."
            ),
            severity,
            confidence: CONFIDENCE_MEDIUM,
            related_field_ids,
        },
    ));
}

fn privacy_score(findings: &[Finding], analysis: &FieldAnalysis) -> u32 {
    let mut score = 0;

    if findings
        .iter()
        .any(|finding| finding.category == CATEGORY_LOCATION)
    {
        score += 40;
    }
    if !analysis.identity_direct_ids.is_empty() {
        score += 20;
    }
    if !analysis.identity_organization_ids.is_empty() {
        score += 15;
    }
    if !analysis.device_ids.is_empty() || !analysis.serial_ids.is_empty() {
        score += 15;
    }
    if !analysis.software_tool_ids.is_empty() {
        score += 10;
    }
    if findings
        .iter()
        .any(|finding| finding.category == CATEGORY_TIMELINE && finding.severity == SEVERITY_MEDIUM)
    {
        score += 10;
    }
    if findings
        .iter()
        .any(|finding| finding.category == CATEGORY_INTEGRITY)
    {
        score += 10;
    }

    score
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

fn timestamp_order_conflict(fields: &[TimestampField]) -> Option<Vec<String>> {
    let creation_times = fields
        .iter()
        .filter(|field| field.role == TimestampRole::Creation)
        .filter_map(|field| field.timestamp.map(|timestamp| (&field.id, timestamp)))
        .collect::<Vec<_>>();
    let modification_times = fields
        .iter()
        .filter(|field| field.role == TimestampRole::Modification)
        .filter_map(|field| field.timestamp.map(|timestamp| (&field.id, timestamp)))
        .collect::<Vec<_>>();

    for (creation_id, creation_time) in &creation_times {
        for (modification_id, modification_time) in &modification_times {
            if creation_time > modification_time {
                return Some(vec![(*creation_id).clone(), (*modification_id).clone()]);
            }
        }
    }

    for (creation_id, creation_time) in creation_times {
        for (modification_id, modification_time) in &modification_times {
            if modification_time - creation_time > LARGE_TIMESTAMP_SEPARATION_SECONDS {
                return Some(vec![creation_id.clone(), (*modification_id).clone()]);
            }
        }
    }

    None
}

fn timestamp_role(key: &str) -> TimestampRole {
    if is_creation_timestamp(key) {
        TimestampRole::Creation
    } else if is_modification_timestamp(key) {
        TimestampRole::Modification
    } else {
        TimestampRole::Other
    }
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

fn is_organization_identity_field(key: &str) -> bool {
    matches!(key, "company" | "organization")
}

fn is_device_field(key: &str) -> bool {
    matches!(
        key,
        "make" | "model" | "devicemanufacturer" | "devicemodelname"
    )
}

fn is_serial_field(key: &str) -> bool {
    matches!(
        key,
        "serialnumber" | "bodyserialnumber" | "cameraserialnumber" | "lensserialnumber"
    )
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

fn finding(file: &EvidenceFile, created_at: &str, input: FindingInput) -> Finding {
    Finding {
        id: format!("finding-{}", Uuid::new_v4()),
        file_id: file.id.clone(),
        category: input.category.to_string(),
        title: input.title.to_string(),
        description: input.description,
        severity: input.severity.to_string(),
        confidence: input.confidence.to_string(),
        related_field_ids: input.related_field_ids,
        created_at: created_at.to_string(),
    }
}

fn merge_ids(groups: &[&Vec<String>]) -> Vec<String> {
    let mut ids = groups
        .iter()
        .flat_map(|group| group.iter().cloned())
        .collect::<Vec<_>>();
    dedup_ids(&mut ids);
    ids
}

fn dedup_ids(ids: &mut Vec<String>) {
    let mut unique = Vec::new();
    for id in ids.drain(..) {
        if !unique.contains(&id) {
            unique.push(id);
        }
    }
    *ids = unique;
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
    fn gps_finding_requires_latitude_and_longitude() {
        let file = complete_file("jpg", Some("JPEG"), Some("image/jpeg"));
        let only_latitude = vec![field("field-lat", "GPS", "GPSLatitude", "location", "1.23")];

        let findings = generate_findings(&file, &only_latitude, "2026-01-01T00:00:00Z");
        assert!(!findings.iter().any(|finding| {
            finding.category == "location" && finding.title == "GPS coordinates found"
        }));

        let coordinates = vec![
            field("field-lat", "GPS", "GPSLatitude", "location", "1.23"),
            field("field-lon", "GPS", "GPSLongitude", "location", "4.56"),
        ];
        let findings = generate_findings(&file, &coordinates, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "location"
                && finding.title == "GPS coordinates found"
                && finding.severity == "high"
                && finding.confidence == "high"
                && finding.related_field_ids == vec!["field-lat", "field-lon"]
        }));
    }

    #[test]
    fn composite_gps_position_generates_gps_finding() {
        let file = complete_file("jpg", Some("JPEG"), Some("image/jpeg"));
        let fields = vec![field(
            "field-position",
            "Composite",
            "GPSPosition",
            "location",
            "1.23 4.56",
        )];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "location"
                && finding.title == "GPS coordinates found"
                && finding.related_field_ids == vec!["field-position"]
        }));
    }

    #[test]
    fn city_location_does_not_generate_gps_finding() {
        let file = complete_file("jpg", Some("JPEG"), Some("image/jpeg"));
        let fields = vec![field("field-city", "XMP", "City", "location", "Pune")];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(!findings.iter().any(|finding| {
            finding.category == "location" && finding.title == "GPS coordinates found"
        }));
    }

    #[test]
    fn identity_confidence_tracks_direct_and_organization_fields() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));
        let organization = vec![field(
            "field-company",
            "PDF",
            "Company",
            "identity",
            "Example Org",
        )];
        let findings = generate_findings(&file, &organization, "2026-01-01T00:00:00Z");
        assert!(findings.iter().any(|finding| {
            finding.category == "identity"
                && finding.title == "Possible user identity metadata found"
                && finding.confidence == "medium"
                && finding.related_field_ids == vec!["field-company"]
        }));

        let direct = vec![field(
            "field-author",
            "PDF",
            "Author",
            "identity",
            "Analyst",
        )];
        let findings = generate_findings(&file, &direct, "2026-01-01T00:00:00Z");
        assert!(findings.iter().any(|finding| {
            finding.category == "identity"
                && finding.confidence == "high"
                && finding.related_field_ids == vec!["field-author"]
        }));
    }

    #[test]
    fn software_fields_generate_low_severity_finding() {
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
                && finding.title == "Software or editing tool detected"
                && finding.severity == "low"
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
                && finding.title == "Potential timestamp inconsistency"
                && finding.severity == "medium"
                && finding.confidence == "medium"
                && finding.related_field_ids == vec!["field-created", "field-modified"]
        }));
    }

    #[test]
    fn large_timestamp_separation_generates_conflict_finding() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));
        let fields = vec![
            field(
                "field-original",
                "EXIF",
                "DateTimeOriginal",
                "timeline",
                "2026:01:01 12:00:00",
            ),
            field(
                "field-modified",
                "PDF",
                "ModifyDate",
                "timeline",
                "2026:02:15 12:00:00",
            ),
        ];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "timeline"
                && finding.title == "Potential timestamp inconsistency"
                && finding.related_field_ids == vec!["field-original", "field-modified"]
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
        assert!(findings.iter().any(|finding| {
            finding.category == "privacy"
                && finding.severity == "low"
                && finding.description.contains("Privacy exposure score: 10")
        }));
    }

    #[test]
    fn privacy_score_generates_low_medium_and_high_bands() {
        let file = complete_file("pdf", Some("PDF"), Some("application/pdf"));
        let low_fields = vec![field(
            "field-producer",
            "PDF",
            "Producer",
            "software",
            "PDF Engine",
        )];
        let low = generate_findings(&file, &low_fields, "2026-01-01T00:00:00Z");
        assert!(low.iter().any(|finding| {
            finding.category == "privacy"
                && finding.severity == "low"
                && finding.description.contains("Privacy exposure score: 10")
        }));

        let medium_fields = vec![
            field("field-lat", "GPS", "GPSLatitude", "location", "1.23"),
            field("field-lon", "GPS", "GPSLongitude", "location", "4.56"),
        ];
        let medium = generate_findings(&file, &medium_fields, "2026-01-01T00:00:00Z");
        assert!(medium.iter().any(|finding| {
            finding.category == "privacy"
                && finding.severity == "medium"
                && finding.description.contains("Privacy exposure score: 40")
        }));

        let high_fields = vec![
            field("field-lat", "GPS", "GPSLatitude", "location", "1.23"),
            field("field-lon", "GPS", "GPSLongitude", "location", "4.56"),
            field("field-author", "PDF", "Author", "identity", "Analyst"),
        ];
        let high = generate_findings(&file, &high_fields, "2026-01-01T00:00:00Z");
        assert!(high.iter().any(|finding| {
            finding.category == "privacy"
                && finding.severity == "high"
                && finding.description.contains("Privacy exposure score: 60")
        }));
    }

    #[test]
    fn serial_fields_contribute_to_privacy_score() {
        let file = complete_file("jpg", Some("JPEG"), Some("image/jpeg"));
        let fields = vec![field(
            "field-serial",
            "EXIF",
            "SerialNumber",
            "technical",
            "12345",
        )];

        let findings = generate_findings(&file, &fields, "2026-01-01T00:00:00Z");

        assert!(findings.iter().any(|finding| {
            finding.category == "privacy"
                && finding.severity == "low"
                && finding.related_field_ids == vec!["field-serial"]
                && finding.description.contains("Privacy exposure score: 15")
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
