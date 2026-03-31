use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub companion: CompanionConfig,
    #[serde(default)]
    pub tools: HashMap<String, ToolConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_tool")]
    pub tool: String,
    pub fallback_shell: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub seed: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

fn default_tool() -> String { "claude".to_string() }
fn default_true() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: Defaults::default(),
            companion: CompanionConfig::default(),
            tools: HashMap::new(),
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self { tool: default_tool(), fallback_shell: None }
    }
}

impl Default for CompanionConfig {
    fn default() -> Self {
        Self { enabled: true, seed: None, name: None }
    }
}

impl Config {
    pub fn load_from(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() { return Ok(Self::default()); }
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = if let Ok(p) = std::env::var("CLAWDSHELL_CONFIG") {
            std::path::PathBuf::from(p)
        } else {
            Self::default_path()
        };
        Self::load_from(&path)
    }

    pub fn default_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("clawdshell")
            .join("config.toml")
    }
}
