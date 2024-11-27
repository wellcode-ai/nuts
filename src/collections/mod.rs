use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub base_url: String,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub name: String,
    pub path: String,
    pub method: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<String>,
    pub tests: Option<Vec<Test>>,
    pub mock: Option<MockConfig>,
    pub perf: Option<PerfConfig>,
    pub mock_data: Option<MockDataConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Test {
    pub name: String,
    pub assert: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockConfig {
    pub response: MockResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerfConfig {
    pub users: u32,
    pub duration: String,
    pub ramp_up: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockDataConfig {
    pub description: String,
    pub schema: Option<String>,
    pub examples: Option<Vec<String>>,
}

impl Collection {
    pub fn load(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let collection: Collection = serde_yaml::from_str(&content)?;
        Ok(collection)
    }

    pub fn save(&self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
