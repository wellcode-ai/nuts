use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

pub mod manager;
pub use manager::CollectionManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAPISpec {
    pub openapi: String,
    pub info: Info,
    pub servers: Vec<Server>,
    pub paths: HashMap<String, PathItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mock_data: Option<MockDataConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    #[serde(default)]
    pub responses: HashMap<String, Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mock_data: Option<MockDataConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_: String,
    pub description: Option<String>,
    pub required: bool,
    pub schema: Schema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: Option<bool>,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MockDataConfig {
    pub description: String,
    pub schema: Option<String>,
    pub examples: Option<Vec<String>>,
}

impl OpenAPISpec {
    pub fn new(name: &str) -> Self {
        Self {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: name.to_string(),
                version: "1.0.0".to_string(),
                description: Some(format!("API flow for {}", name)),
            },
            servers: vec![Server {
                url: "http://localhost:3000".to_string(),
                description: Some("Default server".to_string()),
            }],
            paths: HashMap::new(),
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let spec = serde_yaml::from_str(&contents)?;
        Ok(spec)
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(&self)?;
        fs::write(path, yaml)?;
        Ok(())
    }
}

impl PathItem {
    pub fn new() -> Self {
        Self {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
            mock_data: None,
        }
    }

    pub fn get_operation(&self) -> Option<(&'static str, &Operation)> {
        if let Some(op) = &self.get { return Some(("GET", op)) }
        if let Some(op) = &self.post { return Some(("POST", op)) }
        if let Some(op) = &self.put { return Some(("PUT", op)) }
        if let Some(op) = &self.delete { return Some(("DELETE", op)) }
        if let Some(op) = &self.patch { return Some(("PATCH", op)) }
        None
    }
}

