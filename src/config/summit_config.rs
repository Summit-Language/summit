// File: src/config/summit_config.rs

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SummitConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub main: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    pub output_name: Option<String>,
    #[serde(default = "default_link_libc", alias = "link-libc")]
    pub link_libc: bool,
    #[serde(default = "default_link_summitstd", alias = "link-summitstd")]
    pub link_summitstd: bool,
    #[serde(default, alias = "link-libraries")]
    pub link_libraries: Option<Vec<String>>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        BuildConfig {
            output_dir: default_output_dir(),
            output_name: None,
            link_libc: default_link_libc(),
            link_summitstd: default_link_summitstd(),
            link_libraries: None,
        }
    }
}

fn default_output_dir() -> String {
    "build".to_string()
}

fn default_link_libc() -> bool {
    true
}

fn default_link_summitstd() -> bool {
    true
}

impl SummitConfig {
    /// Loads a Summit.toml configuration file
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read Summit.toml: {}", e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse Summit.toml: {}", e))
    }

    pub fn default_config(project_name: &str) -> Self {
        SummitConfig {
            project: ProjectConfig {
                name: project_name.to_string(),
                version: "0.1.0".to_string(),
                main: "src/main.sm".to_string(),
            },
            build: BuildConfig::default(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, content)
            .map_err(|e| format!("Failed to write Summit.toml: {}", e))
    }

    pub fn get_output_name(&self) -> String {
        self.build.output_name
            .clone()
            .unwrap_or_else(|| self.project.name.clone())
    }
}
