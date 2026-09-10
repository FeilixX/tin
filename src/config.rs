use crate::{Result, project};
use serde::Deserialize;
use std::path::Path;

pub const DEFAULT_CONFIG: &str = include_str!("../resources/default_config.toml");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub save: Save,
    pub restore: Restore,
    pub documents: Vec<Document>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Save {
    pub threshold_ratio: f64,
    pub transcript_pointer: bool,
    pub instructions: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Restore {
    pub instructions: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub path: String,
    pub save: bool,
}

impl Config {
    pub fn load(root: &Path) -> Result<Self> {
        let path = project::local_path(root, ".tin/config.toml")?;
        let text = std::fs::read_to_string(&path)?;
        let config: Self =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", project::shown(&path)))?;
        if !config.save.threshold_ratio.is_finite()
            || config.save.threshold_ratio <= 0.0
            || config.save.threshold_ratio >= 1.0
        {
            return Err("save.threshold_ratio must be between 0 and 1 (exclusive)".into());
        }
        if config.documents.is_empty() {
            return Err("Configure at least one document".into());
        }
        for doc in &config.documents {
            let path = project::local_path(root, &doc.path)?;
            if doc.path.replace('\\', "/").starts_with(".tin/runtime/") {
                return Err(
                    "Documents must not use .tin/runtime/ (reserved for hook evidence)".into(),
                );
            }
            // Not existing yet is normal; the agent writes the file. Existing as
            // something other than a file is not: nothing can read or update a
            // directory, and recovery would report its age as if it were a note.
            if std::fs::metadata(&path).is_ok_and(|entry| !entry.is_file()) {
                return Err(format!("Not a file: {}", doc.path).into());
            }
        }
        Ok(config)
    }
}
