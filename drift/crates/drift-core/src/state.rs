use std::fs;
use std::path::{Path, PathBuf};

use crate::DriftError;

pub struct LockfileState {
    path: PathBuf,
}

impl LockfileState {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.path.exists()
    }

    pub fn set_active(&self) -> Result<(), DriftError> {
        fs::write(&self.path, "").map_err(DriftError::StateIo)?;
        Ok(())
    }

    pub fn set_inactive(&self) -> Result<(), DriftError> {
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(DriftError::StateIo)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct LayoutState {
    pub active: bool,
}
