use crate::models::MetadataField;
use serde_json::Value;
use uuid::Uuid;

const MAX_DISPLAY_VALUE_LEN: usize = 600;
const MAX_COMPLEX_VALUE_NODES: usize = 64;
const MAX_SCALAR_ARRAY_ITEMS: usize = 50;

struct FieldMapping {
    group: Option<&'static str>,
    tag: &'static str,
    label: &'static str,
    category: &'static str,
}

const FIELD_MAPPINGS: &[FieldMapping] = &[
    FieldMapping {
        group: None,
        tag: "author",
        label: "Author",
        category: "identity",
    },
    FieldMapping {
        group: Some("pdf"),
        tag: "creator",
        label: "Creator",
        category: "identity",
    },
    FieldMapping {
        group: Some("xmp"),
        tag: "creator",
        label: "Creator",
        category: "identity",
    },
    FieldMapping {
        group: None,
        tag: "lastmodifiedby",
        label: "Last modified by",
        category: "identity",
    },
    FieldMapping {
        group: None,
        tag: "owner",
        label: "Owner",
        category: "identity",
    },
    FieldMapping {
        group: None,
        tag: "company",
        label: "Company",
        category: "identity",
    },
    FieldMapping {
        group: None,
        tag: "username",
        label: "User name",
        category: "identity",
    },
    FieldMapping {
        group: None,
        tag: "artist",
        label: "Artist",
        category: "identity",
    },
    FieldMapping {
        group: Some("gps"),
        tag: "gpslatitude",
        label: "Latitude",
        category: "location",
    },
    FieldMapping {
        group: Some("gps"),
        tag: "gpslongitude",
        label: "Longitude",
        category: "location",
    },
    FieldMapping {
        group: Some("gps"),
        tag: "gpsaltitude",
        label: "Altitude",
        category: "location",
    },
    FieldMapping {
        group: Some("composite"),
        tag: "gpsposition",
        label: "GPS position",
        category: "location",
    },
    FieldMapping {
        group: Some("xmp"),
        tag: "location",
        label: "Location",
        category: "location",
    },
    FieldMapping {
        group: Some("iptc"),
        tag: "location",
        label: "Location",
        category: "location",
    },
    FieldMapping {
        group: Some("xmp"),
        tag: "city",
        label: "City",
        category: "location",
    },
    FieldMapping {
        group: Some("iptc"),
        tag: "city",
        label: "City",
        category: "location",
    },
    FieldMapping {
        group: Some("xmp"),
        tag: "sublocation",
        label: "Sublocation",
        category: "location",
    },
    FieldMapping {
        group: Some("iptc"),
        tag: "sublocation",
        label: "Sublocation",
        category: "location",
    },
    FieldMapping {
        group: Some("xmp"),
        tag: "country",
        label: "Country",
        category: "location",
    },
    FieldMapping {
        group: Some("iptc"),
        tag: "country",
        label: "Country",
        category: "location",
    },
    FieldMapping {
        group: None,
        tag: "createdate",
        label: "Created",
        category: "timeline",
    },
    FieldMapping {
        group: None,
        tag: "modifydate",
        label: "Modified",
        category: "timeline",
    },
    FieldMapping {
        group: None,
        tag: "datetimeoriginal",
        label: "Original capture time",
        category: "timeline",
    },
    FieldMapping {
        group: Some("file"),
        tag: "filemodifydate",
        label: "File modified",
        category: "timeline",
    },
    FieldMapping {
        group: Some("file"),
        tag: "fileaccessdate",
        label: "File accessed",
        category: "timeline",
    },
    FieldMapping {
        group: None,
        tag: "metadatadate",
        label: "Metadata changed",
        category: "timeline",
    },
    FieldMapping {
        group: Some("quicktime"),
        tag: "trackcreatedate",
        label: "Track created",
        category: "timeline",
    },
    FieldMapping {
        group: Some("quicktime"),
        tag: "mediacreatedate",
        label: "Media created",
        category: "timeline",
    },
    FieldMapping {
        group: None,
        tag: "software",
        label: "Software",
        category: "software",
    },
    FieldMapping {
        group: None,
        tag: "creatortool",
        label: "Creator tool",
        category: "software",
    },
    FieldMapping {
        group: Some("pdf"),
        tag: "producer",
        label: "Producer",
        category: "software",
    },
    FieldMapping {
        group: None,
        tag: "encoder",
        label: "Encoder",
        category: "software",
    },
    FieldMapping {
        group: Some("exif"),
        tag: "make",
        label: "Device manufacturer",
        category: "software",
    },
    FieldMapping {
        group: Some("exif"),
        tag: "model",
        label: "Device model",
        category: "software",
    },
    FieldMapping {
        group: Some("quicktime"),
        tag: "make",
        label: "Device manufacturer",
        category: "software",
    },
    FieldMapping {
        group: Some("quicktime"),
        tag: "model",
        label: "Device model",
        category: "software",
    },
    FieldMapping {
        group: None,
        tag: "devicemanufacturer",
        label: "Device manufacturer",
        category: "software",
    },
    FieldMapping {
        group: None,
        tag: "devicemodelname",
        label: "Device model",
        category: "software",
    },
    FieldMapping {
        group: Some("file"),
        tag: "filetype",
        label: "File type",
        category: "technical",
    },
    FieldMapping {
        group: Some("file"),
        tag: "mimetype",
        label: "MIME type",
        category: "technical",
    },
    FieldMapping {
        group: None,
        tag: "imagewidth",
        label: "Image width",
        category: "technical",
    },
    FieldMapping {
        group: None,
        tag: "imageheight",
        label: "Image height",
        category: "technical",
    },
    FieldMapping {
        group: None,
        tag: "duration",
        label: "Duration",
        category: "technical",
    },
    FieldMapping {
        group: None,
        tag: "bitrate",
        label: "Bitrate",
        category: "technical",
    },
    FieldMapping {
        group: None,
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
    if is_exiftool_file_identity_field(source, group, key) {
        return;
    }

    let Some(mapping) = mapping_for(group, key) else {
        return;
    };
    let Some(display_value) = display_value(value) else {
        return;
    };

    fields.push(MetadataField {
        id: format!("field-{}", Uuid::new_v4()),
        file_id: file_id.to_string(),
        group: group.to_string(),
        key: key.to_string(),
        display_label: Some(mapping.label.to_string()),
        value: display_value,
        source: source.to_string(),
        normalized_category: Some(mapping.category.to_string()),
    });
}

fn is_exiftool_file_identity_field(source: &str, group: &str, key: &str) -> bool {
    if !source.eq_ignore_ascii_case("exiftool") || !group.eq_ignore_ascii_case("file") {
        return false;
    }

    matches!(
        normalize_tag(key).as_str(),
        "filetype" | "filetypeextension" | "mimetype"
    )
}

fn mapping_for(group: &str, key: &str) -> Option<&'static FieldMapping> {
    let normalized_group = normalize_tag(group);
    let normalized = normalize_tag(key);

    FIELD_MAPPINGS
        .iter()
        .find(|mapping| {
            mapping
                .group
                .is_some_and(|group| group.eq_ignore_ascii_case(&normalized_group))
                && mapping.tag.eq_ignore_ascii_case(&normalized)
        })
        .or_else(|| {
            FIELD_MAPPINGS.iter().find(|mapping| {
                mapping.group.is_none() && mapping.tag.eq_ignore_ascii_case(&normalized)
            })
        })
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
        Value::Array(values) if values.iter().all(is_scalar) => display_scalar_array(values),
        Value::Array(values) if complex_value_exceeds_limit(value) => {
            format!("Array metadata value ({} items)", values.len())
        }
        Value::Object(values) if complex_value_exceeds_limit(value) => {
            format!("Object metadata value ({} keys)", values.len())
        }
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

fn display_scalar_array(values: &[Value]) -> String {
    if values.len() > MAX_SCALAR_ARRAY_ITEMS {
        return format!("Array metadata value ({} items)", values.len());
    }

    values
        .iter()
        .filter_map(display_value)
        .collect::<Vec<_>>()
        .join(", ")
}

fn complex_value_exceeds_limit(value: &Value) -> bool {
    fn count_nodes(value: &Value, remaining: &mut usize) -> bool {
        if *remaining == 0 {
            return true;
        }
        *remaining -= 1;

        match value {
            Value::Array(values) => values.iter().any(|value| count_nodes(value, remaining)),
            Value::Object(values) => values.values().any(|value| count_nodes(value, remaining)),
            _ => false,
        }
    }

    let mut remaining = MAX_COMPLEX_VALUE_NODES;
    count_nodes(value, &mut remaining)
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
        assert!(fields.iter().any(|field| field.key == "GPSLatitude"
            && field.display_label.as_deref() == Some("Latitude")
            && field.normalized_category.as_deref() == Some("location")));
        assert!(fields.iter().any(|field| field.key == "GPSLongitude"
            && field.display_label.as_deref() == Some("Longitude")));
    }

    #[test]
    fn preserves_raw_keys_and_maps_identity_timeline_software_and_technical_fields() {
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
            && field.display_label.as_deref() == Some("Author")
            && field.normalized_category.as_deref() == Some("identity")));
        assert!(fields.iter().any(|field| field.key == "CreateDate"
            && field.display_label.as_deref() == Some("Created")
            && field.normalized_category.as_deref() == Some("timeline")));
        assert!(fields.iter().any(|field| field.key == "Producer"
            && field.normalized_category.as_deref() == Some("software")));
        assert!(fields
            .iter()
            .any(|field| field.key == "PageCount"
                && field.display_label.as_deref() == Some("Page count")
                && field.normalized_category.as_deref() == Some("technical")));
        assert!(fields.iter().any(|field| field.key == "Model"
            && field.display_label.as_deref() == Some("Device model")
            && field.value == "Model X"));
    }

    #[test]
    fn skips_exiftool_file_identity_fields() {
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "File": {
                    "FileType": "MP3",
                    "FileTypeExtension": "mp3",
                    "MIMEType": "audio/mpeg"
                },
                "PNG": {
                    "ImageWidth": 16,
                    "ImageHeight": 16
                }
            }),
        );

        assert!(!fields.iter().any(|field| field.group == "File"
            && matches!(
                field.key.as_str(),
                "FileType" | "FileTypeExtension" | "MIMEType"
            )));
        assert!(fields.iter().any(|field| field.key == "ImageWidth"
            && field.normalized_category.as_deref() == Some("technical")));
        assert!(fields.iter().any(|field| field.key == "ImageHeight"
            && field.normalized_category.as_deref() == Some("technical")));
    }

    #[test]
    fn uses_group_aware_mappings_for_ambiguous_tags() {
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "File": {
                    "Model": "not a device",
                    "Location": "not a place"
                },
                "EXIF": {
                    "Model": "Device X"
                },
                "XMP": {
                    "Location": "Studio"
                }
            }),
        );

        assert_eq!(fields.len(), 2);
        assert!(fields.iter().any(|field| field.group == "EXIF"
            && field.key == "Model"
            && field.display_label.as_deref() == Some("Device model")
            && field.normalized_category.as_deref() == Some("software")));
        assert!(fields.iter().any(|field| field.group == "XMP"
            && field.key == "Location"
            && field.display_label.as_deref() == Some("Location")
            && field.normalized_category.as_deref() == Some("location")));
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
        assert!(fields.iter().any(|field| field.key == "CreatorTool"
            && field.display_label.as_deref() == Some("Creator tool")
            && field.value.contains("Nested Tool")));
        assert!(fields
            .iter()
            .any(|field| field.key == "Encoder" && field.value == "one, two"));
    }

    #[test]
    fn skips_empty_values_and_bounds_long_values() {
        let long_value = "x".repeat(700);
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "PDF": {
                    "Author": "   ",
                    "Creator": null,
                    "Producer": long_value
                }
            }),
        );

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "Producer");
        assert_eq!(fields[0].value.chars().count(), 603);
        assert!(fields[0].value.ends_with("..."));
    }

    #[test]
    fn summarizes_large_complex_values_without_full_serialization() {
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "XMP": {
                    "CreatorTool": (0..80).map(|index| json!({"index": index})).collect::<Vec<_>>()
                }
            }),
        );

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "CreatorTool");
        assert_eq!(fields[0].value, "Array metadata value (80 items)");
    }

    #[test]
    fn summarizes_large_scalar_arrays_without_joining_every_item() {
        let fields = normalize_metadata(
            "file-1",
            "exiftool",
            &json!({
                "XMP": {
                    "Encoder": (0..80).map(|index| json!(format!("encoder-{index}"))).collect::<Vec<_>>()
                }
            }),
        );

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "Encoder");
        assert_eq!(fields[0].value, "Array metadata value (80 items)");
    }
}
