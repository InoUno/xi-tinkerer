use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PersistenceData {
    pub ffxi_path: Option<PathBuf>,
    pub recent_projects: Vec<PathBuf>,
}

impl PersistenceData {
    const FILENAME: &'static str = "persistence.yml";

    fn load_existing_data(local_data_dir: &PathBuf) -> Option<Self> {
        let data: PersistenceData =
            serde_yaml::from_str(&fs::read_to_string(local_data_dir.join(Self::FILENAME)).ok()?)
                .ok()?;

        Some(data)
    }

    pub fn load(local_data_dir: &PathBuf) -> Self {
        Self::load_existing_data(local_data_dir).unwrap_or_default()
    }

    pub fn save(&self, local_data_dir: &PathBuf) -> Option<()> {
        let str = serde_yaml::to_string(self).ok()?;
        fs::write(local_data_dir.join(Self::FILENAME), str).ok()?;

        Some(())
    }
}

impl Default for PersistenceData {
    fn default() -> Self {
        Self {
            ffxi_path: None,
            recent_projects: vec![],
        }
    }
}
