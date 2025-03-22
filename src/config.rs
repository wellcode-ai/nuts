use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
}

impl Config {

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".nuts")
            .join("config.json"))
    }

    pub fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }

    pub fn merge(mut self, other: Config) -> Self {
        if self.anthropic_api_key.is_none() {
            self.anthropic_api_key = other.anthropic_api_key;
        }
        if self.api_key.is_none() {
            self.api_key = other.api_key;
        }
        self
    }
}
