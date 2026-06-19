use crate::models::MetadataField;
use serde_json::Value;
use uuid::Uuid;

const MAX_DISPLAY_VALUE_LEN: usize = 600;

struct FieldMapping {
    tag: &'static str,
    label: &'static str,
    category: &'static str,
}

const FIELD_MAPPINGS: &[FieldMapping] = &[
    FieldMapping {
        tag: "author",
        label: "Author",
        category: "identity",
    },
    FieldMapping {
        tag: "creator",
        label: "Creator",
        category: "identity",
    },
    FieldMapping {
        tag: "lastmodifiedby",
        label: "Last modified by",
        category: "identity",
    },
    FieldMapping {
        tag: "owner",
        label: "Owner",
        category: "identity",
    },
    FieldMapping {
        tag: "company",
        label: "Company",
        category: "identity",
    },
    FieldMapping {
        tag: "username",
        label: "User name",
        category: "identity",
    },
    FieldMapping {
        tag: "artist",
        label: "Artist",
        category: "identity",
    },
    FieldMapping {
        tag: "gpslatitude",
        label: "Latitude",
        category: "location",
    },
    FieldMapping {
        tag: "gpslongitude",
        label: "Longitude",
        category: "location",
    },
    FieldMapping {
        tag: "gpsaltitude",
        label: "Altitude",
        category: "location",
    },
    FieldMapping {
        tag: "gpsposition",
        label: "GPS position",
        category: "location",
    },
    FieldMapping {
        tag: "location",
        label: "Location",
        category: "location",
    },
    FieldMapping {
        tag: "city",
        label: "City",
        category: "location",
    },
    FieldMapping {
        tag: "sublocation",
        label: "Sublocation",
        category: "location",
    },
    FieldMapping {
        tag: "country",
        label: "Country",
        category: "location",
    },
    FieldMapping {
        tag: "createdate",
        label: "Created",
        category: "timeline",
    },
    FieldMapping {
        tag: "modifydate",
        label: "Modified",
        category: "timeline",
    },
    FieldMapping {
        tag: "datetimeoriginal",
        label: "Original capture time",
        category: "timeline",
    },
    FieldMapping {
        tag: "filemodifydate",
        label: "File modified",
        category: "timeline",
    },
    FieldMapping {
        tag: "fileaccessdate",
        label: "File accessed",
        category: "timeline",
    },
    FieldMapping {
        tag: "metadatadate",
        label: "Metadata changed",
        category: "timeline",
    },
    FieldMapping {
        tag: "trackcreatedate",
        label: "Track created",
        category: "timeline",
    },
    FieldMapping {
        tag: "mediacreatedate",
        label: "Media created",
        category: "timeline",
    },
    FieldMapping {
        tag: "software",
        label: "Software",
        category: "software",
    },
    FieldMapping {
        tag: "creatortool",
        label: "Creator tool",
        category: "software",
    },
    FieldMapping {
        tag: "producer",
        label: "Producer",
        category: "software",
    },
    FieldMapping {
        tag: "encoder",
        label: "Encoder",
        category: "software",
    },
    FieldMapping {
        tag: "make",
        label: "Device manufacturer",
        category: "software",
    },
    FieldMapping {
        tag: "model",
        label: "Device model",
        category: "software",
    },
    FieldMapping {
        tag: "devicemanufacturer",
        label: "Device manufacturer",
        category: "software",
    },
    FieldMapping {
        tag: "devicemodelname",
        label: "Device model",
        category: "software",
    },
    FieldMapping {
        tag: "filetype",
        label: "File type",
        category: "technical",
    },
    FieldMapping {
        tag: "mimetype",
        label: "MIME type",
        category: "technical",
    },
    FieldMapping {
        tag: "imagewidth",
        label: "Image width",
        category: "technical",
    },
    FieldMapping {
        tag: "imageheight",
        label: "Image height",
        category: "technical",
    },
    FieldMapping {
        tag: "duration",
        label: "Duration",
        category: "technical",
    },
    FieldMapping {
        tag: "bitrate",
        label: "Bitrate",
        category: "technical",
    },
    FieldMapping {
        tag: "pagecount",
        label: "Page count",
        category: "technical",
    },
];

pub fn normalize_metadata(file_id: &str, source: &str, raw_metadata: &Value) -> Vec<MetadataField> {
    let mut fields = Vec::new();

    if let Some(groups) = raw_metadata.as_object() {
        for (group, group_value) in groups {
            if let Some(tags) = group_value.as_object() {
                for (key, value) in tags {
                    push_normalized_field(&mut fields, file_id, source, group, key, value);
                }
            }
        }
    }

    fields
}

fn push_normalized_field(
    fields: &mut Vec<MetadataField>,
    file_id: &str,
    source: &str,
    group: &str,
    key: &str,
    value: &Value,
) {
    let Some(mapping) = mapping_for(key) else {
        return;
    };
    let Some(display_value) = display_value(value) else {
        return;
    };

    fields.push(MetadataField {
        id: format!("field-{}", Uuid::new_v4()),
        file_id: file_id.to_string(),
        group: group.to_string(),
        key: mapping.label.to_string(),
        value: display_value,
        source: source.to_string(),
        normalized_category: Some(mapping.category.to_string()),
    });
}

fn mapping_for(key: &str) -> Option<&'static FieldMapping> {
    let normalized = normalize_tag(key);
    FIELD_MAPPINGS
        .iter()
        .find(|mapping| mapping.tag.eq_ignore_ascii_case(&normalized))
}

fn normalize_tag(key: &str) -> String {
    key.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn display_value(value: &Value) -> Option<String> {
    let rendered = match value {
        Value::Null => return None,
        Value::String(value) => value.trim().to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(values) if values.iter().all(is_scalar) => values
            .iter()
            .filter_map(display_value)
            .collect::<Vec<_>>()
            .join(", "),
        _ => serde_json::to_string(value).ok()?,
    };

    let rendered = rendered.trim();
    if rendered.is_empty() {
        return None;
    }

    Some(truncate_display_value(rendered))
}

fn is_scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::String(_) | Value::Bool(_) | Value::Number(_)
    )
}

fn truncate_display_value(value: &str) -> String {
    let character_count = value.chars().count();
    if character_count <= MAX_DISPLAY_VALUE_LEN {
        return value.to_string();
    }

    let mut truncated = String::new();
    for character in value.chars().take(MAX_DISPLAY_VALUE_LEN) {
        truncated.push(character);
    }

    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::normalize_metadata;
    use serde_json::json;

    #[test]
    fn maps_location_fields_to_readable_labels() {
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "GPS": {
                    "GPSLatitude": "40 deg 26' 46.00\" N",
                    "GPSLongitude": "79 deg 58' 56.00\" W"
                }
            }),
        );

        assert_eq!(fields.len(), 2);
        assert!(fields.iter().any(|field| field.key == "Latitude"
            && field.normalized_category.as_deref() == Some("location")));
        assert!(fields.iter().any(|field| field.key == "Longitude"));
    }

    #[test]
    fn maps_identity_timeline_software_and_technical_fields() {
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "PDF": {
                    "Author": "Analyst",
                    "Creator": "Writer",
                    "CreateDate": "2026:01:01 12:00:00",
                    "Producer": "PDF Engine",
                    "PageCount": 4
                },
                "File": {
                    "FileType": "PDF",
                    "MIMEType": "application/pdf"
                },
                "EXIF": {
                    "Make": "Camera Co",
                    "Model": "Model X"
                }
            }),
        );

        assert!(fields.iter().any(|field| field.key == "Author"
            && field.normalized_category.as_deref() == Some("identity")));
        assert!(fields.iter().any(|field| field.key == "Created"
            && field.normalized_category.as_deref() == Some("timeline")));
        assert!(fields.iter().any(|field| field.key == "Producer"
            && field.normalized_category.as_deref() == Some("software")));
        assert!(fields.iter().any(|field| field.key == "File type"
            && field.normalized_category.as_deref() == Some("technical")));
        assert!(fields
            .iter()
            .any(|field| field.key == "Device model" && field.value == "Model X"));
    }

    #[test]
    fn excludes_unknown_fields_and_serializes_complex_values() {
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "XMP": {
                    "UnknownTag": "hidden in raw metadata",
                    "CreatorTool": {
                        "name": "Nested Tool",
                        "version": 1
                    },
                    "Encoder": ["one", "two"]
                }
            }),
        );

        assert_eq!(fields.len(), 2);
        assert!(fields.iter().all(|field| field.key != "UnknownTag"));
        assert!(fields
            .iter()
            .any(|field| field.key == "Creator tool" && field.value.contains("Nested Tool")));
        assert!(fields
            .iter()
            .any(|field| field.key == "Encoder" && field.value == "one, two"));
    }
}
