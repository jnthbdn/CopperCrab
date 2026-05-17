use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::tools::{drill::DrillBit, endmill::EndMill, vbit::VBit};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolLibrary {
    pub vbits: Vec<VBit>,
    pub end_mills: Vec<EndMill>,
    pub drill_bits: Vec<DrillBit>,

    tool_file: PathBuf,
}

impl ToolLibrary {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Save path: {}", self.tool_file.to_string_lossy());
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&self.tool_file, content)?;
        log::info!("Tool library saved");
        Ok(())
    }

    pub fn new(config_folder: &Path) -> Self {
        let tool_file = config_folder.join("tools.toml");
        let mut s = Self::load(&tool_file).unwrap_or_else(|_| {
            log::warn!("Tool library not found or empty.");
            Self::default()
        });

        s.tool_file = tool_file;
        s
    }

    fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let lib = toml::from_str(&content)?;
        log::info!("Tool library loaded from {}", path.display());
        Ok(lib)
    }
}
