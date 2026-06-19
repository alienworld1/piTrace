use exiftool::ExifTool;
use serde_json::Value;
use std::path::Path;
use tauri::{AppHandle, Manager};

pub trait RawMetadataExtractor {
    fn extract_raw_metadata(&self, path: &Path) -> Result<Value, String>;
}

pub struct ExifToolMetadataExtractor {
    exiftool: ExifTool,
}

impl ExifToolMetadataExtractor {
    pub fn for_app_bundle(app: &AppHandle) -> Result<Self, String> {
        let resource_dir = app
            .path()
            .resource_dir()
            .map_err(|error| format!("Could not locate bundled resources: {error}"))?;
        Self::for_bundle_resource_dir(&resource_dir)
    }

    fn for_bundle_resource_dir(resource_dir: &Path) -> Result<Self, String> {
        if cfg!(windows) {
            return Err("Bundled ExifTool is unavailable for Windows in this build.".to_string());
        }

        let executable_path = resource_dir
            .join("exiftool")
            .join(exiftool_executable_name());
        Self::from_trusted_executable(&executable_path, "Bundled ExifTool")
    }

    #[cfg(test)]
    pub fn for_test_fixture() -> Result<Self, String> {
        let executable_path = repo_exiftool_path().ok_or_else(|| {
            "Test ExifTool fixture '.agent/exiftool/exiftool' was not found.".to_string()
        })?;
        Self::from_trusted_executable(&executable_path, "test ExifTool fixture")
    }

    fn from_trusted_executable(path: &Path, label: &str) -> Result<Self, String> {
        validate_exiftool_executable(path, label)?;
        let exiftool = ExifTool::with_executable(path)
            .map_err(|error| format!("Could not start {label} at '{}': {error}", path.display()))?;

        Ok(Self { exiftool })
    }

    fn reject_line_delimited_path(path: &Path) -> Result<(), String> {
        let path = path.to_string_lossy();
        if path.contains('\n') || path.contains('\r') {
            return Err(
                "ExifTool extraction rejected this path because it contains a line break."
                    .to_string(),
            );
        }

        Ok(())
    }
}

impl RawMetadataExtractor for ExifToolMetadataExtractor {
    fn extract_raw_metadata(&self, path: &Path) -> Result<Value, String> {
        Self::reject_line_delimited_path(path)?;

        self.exiftool
            .json(path, &["-g1"])
            .map_err(|error| format!("ExifTool could not extract raw metadata: {error}"))
    }
}

fn validate_exiftool_executable(path: &Path, label: &str) -> Result<(), String> {
    if label.is_empty() {
        return Err("ExifTool executable label is required.".to_string());
    }

    let metadata = path
        .metadata()
        .map_err(|error| format!("{label} unavailable at '{}': {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "{label} unavailable at '{}': expected a file.",
            path.display()
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!(
                "{label} unavailable at '{}': file is not executable.",
                path.display()
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
fn repo_exiftool_path() -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".agent")
        .join("exiftool")
        .join(exiftool_executable_name());
    path.exists().then_some(path)
}

fn exiftool_executable_name() -> &'static str {
    if cfg!(windows) {
        "exiftool.exe"
    } else {
        "exiftool"
    }
}

#[cfg(test)]
mod tests {
    use super::{ExifToolMetadataExtractor, RawMetadataExtractor};
    use std::path::PathBuf;

    #[test]
    fn extracts_grouped_raw_metadata_from_exiftool_fixture() {
        let extractor =
            ExifToolMetadataExtractor::for_test_fixture().expect("fixture ExifTool should start");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".agent")
            .join("exiftool")
            .join("t")
            .join("images")
            .join("GPS.jpg");

        let data = extractor
            .extract_raw_metadata(&path)
            .expect("fixture metadata should extract");

        assert!(data["SourceFile"].is_string());
        assert_eq!(data["File"]["FileType"], "JPEG");
        assert!(data["GPS"].is_object());
        assert!(data["GPS"]["GPSLatitude"].is_string());
    }

    #[test]
    fn test_fixture_resolver_finds_repo_exiftool_without_path_lookup() {
        ExifToolMetadataExtractor::for_test_fixture()
            .expect("test fixture should resolve from the repo");
    }

    #[test]
    fn bundle_resolver_rejects_missing_resource_without_path_lookup() {
        let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("missing-resource-dir");

        let error = match ExifToolMetadataExtractor::for_bundle_resource_dir(&missing) {
            Ok(_) => panic!("missing bundle should fail closed"),
            Err(error) => error,
        };

        assert!(error.contains("Bundled ExifTool unavailable"));
    }

    #[test]
    fn rejects_paths_with_line_breaks_before_calling_exiftool() {
        let extractor =
            ExifToolMetadataExtractor::for_test_fixture().expect("fixture ExifTool should start");
        let path = PathBuf::from("/tmp/pi-trace\n-unsafe.jpg");

        let error = extractor
            .extract_raw_metadata(&path)
            .expect_err("line-delimited path should be rejected");

        assert!(error.contains("line break"));
    }
}
