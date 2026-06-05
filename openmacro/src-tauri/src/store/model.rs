//! Data model for loaded snippets.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub id: String,
    pub trigger: String,
    pub replace: String,
    pub vars: Vec<VarDecl>,
    pub source_file: PathBuf,
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
