use crate::token::Token;
use anyhow::anyhow;
use serde::Deserialize;
use std::path::Path;

pub mod cpp;

#[derive(Deserialize, Clone, Copy, Debug)]
pub enum Language {
    Cpp,
}

pub fn tokenize(path: &Path) -> anyhow::Result<Vec<Token>> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if extension == "cpp" {
        cpp::tokenize(path)
    } else {
        Err(anyhow!(
            "Unsupported file extension: {:?}",
            path.extension()
        ))
    }
}

pub fn tokenize_str(content: &str, language: Language) -> anyhow::Result<Vec<Token>> {
    match language {
        Language::Cpp => cpp::tokenize_str(content),
    }
}
