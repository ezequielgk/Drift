use std::fs;
use std::path::PathBuf;

use crate::DriftError;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DriftConfig {
    pub max_windows: u32,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self { max_windows: 2 }
    }
}

impl DriftConfig {
    pub fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let mut p = PathBuf::from(home);
        p.push(".config");
        p.push("drift");
        p.push("config.toml");
        p
    }

    pub fn load() -> Result<Self, DriftError> {
        let path = Self::path();
        if !path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&path).map_err(DriftError::ConfigIo)?;
        toml::from_str(&content).map_err(|e| DriftError::ConfigParse(e.to_string()))
    }

    pub fn save(&self) -> Result<(), DriftError> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(DriftError::ConfigIo)?;
        }

        let content = toml::to_string(self).map_err(|e| DriftError::ConfigParse(e.to_string()))?;
        fs::write(&path, content).map_err(DriftError::ConfigIo)?;
        Ok(())
    }
}
