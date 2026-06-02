use std::fs;
use std::path::{Path, PathBuf};

use hyperspace_core::{HyperspaceError, HyperspaceState, Result};

/// JSON-backed workspace persistence for development builds on Windows/macOS/Linux.
pub struct JsonWorkspaceStore {
    path: PathBuf,
}

impl JsonWorkspaceStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("os3-hyperspace").join("workspace.json")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_or_default(&self) -> HyperspaceState {
        match self.load() {
            Ok(state) => state,
            Err(err) => {
                tracing::info!("starting fresh workspace ({err})");
                HyperspaceState::with_demo_content()
            }
        }
    }

    pub fn load(&self) -> Result<HyperspaceState> {
        let raw = fs::read_to_string(&self.path)
            .map_err(|err| HyperspaceError::Filesystem(err.to_string()))?;
        serde_json::from_str(&raw).map_err(|err| HyperspaceError::Filesystem(err.to_string()))
    }

    pub fn save(&self, state: &HyperspaceState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| HyperspaceError::Filesystem(err.to_string()))?;
        }

        let raw = serde_json::to_string_pretty(state)
            .map_err(|err| HyperspaceError::Filesystem(err.to_string()))?;
        fs::write(&self.path, raw).map_err(|err| HyperspaceError::Filesystem(err.to_string()))
    }
}
