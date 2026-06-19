use exiftool::ExifTool;
use serde_json::Value;
use std::{
    env,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

pub trait RawMetadataExtractor {
    fn extract_raw_metadata(&self, path: &Path) -> Result<Value, String>;
}

pub struct ExifToolMetadataExtractor {
    executable_path: Option<PathBuf>,
}

impl ExifToolMetadataExtractor {
    pub fn for_app(app: &AppHandle) -> Self {
        let executable_path = app
            .path()
            .resource_dir()
            .ok()
            .map(|resource_dir| {
                resource_dir
                    .join("exiftool")
                    .join(exiftool_executable_name())
            })
            .filter(|path| path.exists())
            .or_else(repo_exiftool_path);

        Self { executable_path }
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self {
            executable_path: repo_exiftool_path(),
        }
    }

    fn open_exiftool(&self) -> Result<ExifTool, String> {
        if let Some(path) = &self.executable_path {
            return ExifTool::with_executable(path).map_err(|error| {
                format!(
                    "Could not start bundled ExifTool at '{}': {error}",
                    path.display()
                )
            });
        }

        ExifTool::new().map_err(|error| {
            format!(
                "Could not start ExifTool. Expected bundled resource 'exiftool/{}', \
                 repo fallback '.agent/exiftool/{}', or an 'exiftool' binary on PATH: {error}",
                exiftool_executable_name(),
                exiftool_executable_name()
            )
        })
    }
}

impl RawMetadataExtractor for ExifToolMetadataExtractor {
    fn extract_raw_metadata(&self, path: &Path) -> Result<Value, String> {
        let exiftool = self.open_exiftool()?;

        exiftool
            .json(path, &["-g1"])
            .map_err(|error| format!("ExifTool could not extract raw metadata: {error}"))
    }
}

fn repo_exiftool_path() -> Option<PathBuf> {
    repo_root_candidates()
        .into_iter()
        .map(|root| {
            root.join(".agent")
                .join("exiftool")
                .join(exiftool_executable_name())
        })
        .find(|path| path.exists())
}

fn repo_root_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir.clone());
        if let Some(parent) = current_dir.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
    candidates
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
        let extractor = ExifToolMetadataExtractor::for_tests();
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
}
