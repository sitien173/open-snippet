//! Data model for loaded snippets.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_max_files() -> usize {
    7
}

fn default_trigger_prefix() -> String {
    ";".to_string()
}

fn default_expand_mode() -> ExpandMode {
    ExpandMode::Manual
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub id: String,
    pub trigger: String,
    pub raw_trigger: String,
    pub trigger_literal: bool,
    pub replace: String,
    pub vars: Vec<VarDecl>,
    pub source_file: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExpandMode {
    Auto,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreSettings {
    #[serde(default = "default_trigger_prefix")]
    pub trigger_prefix: String,
    #[serde(default = "default_expand_mode")]
    pub expand_mode: ExpandMode,
}

impl Default for StoreSettings {
    fn default() -> Self {
        Self {
            trigger_prefix: default_trigger_prefix(),
            expand_mode: default_expand_mode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VarDecl {
    pub name: String,
    pub kind: VarKind,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub cmd: Vec<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub confirm: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarKind {
    Text,
    Textarea,
    Choice,
    Number,
    Datetime,
    Clipboard,
    Cursor,
    Shell,
    Form,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub modules: HashMap<String, String>,
    #[serde(default)]
    pub file: LoggingFileConfig,
    #[serde(default)]
    pub verbose_content: bool,
    #[serde(default)]
    pub frontend: LoggingFrontendConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            modules: HashMap::new(),
            file: LoggingFileConfig::default(),
            verbose_content: false,
            frontend: LoggingFrontendConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingFileConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_log_max_files")]
    pub max_files: usize,
}

impl Default for LoggingFileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_files: default_log_max_files(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingFrontendConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub modules: HashMap<String, String>,
}

impl Default for LoggingFrontendConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            modules: HashMap::new(),
        }
    }
}

fn default_true() -> bool {
    true
}
