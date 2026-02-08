use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct RuleFile {
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    pub name: Option<String>,
    pub language: String,
    #[serde(default)]
    pub priority: i32,
    pub query: String,
    pub template: String,
}

impl RuleFile {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read rule file: {:?}", path))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse rule file: {:?}", path))
    }
}
