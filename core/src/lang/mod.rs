use crate::token::Token;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod cpp;
pub mod python;
pub mod rust;
pub mod verilog;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Language {
    Cpp,
    Rust,
    Verilog,
    Python,
}

pub fn tokenize(path: &Path) -> anyhow::Result<Vec<Token>> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default().to_ascii_lowercase();

    match extension.as_str() {
        "cpp" | "cc" | "cxx" | "c++" | "c" | "cu" => cpp::tokenize(path), // C, C++ or CUDA
        "rs" => rust::tokenize(path),
        "v" => verilog::tokenize(path),
        "py" => python::tokenize(path),
        _ => Err(anyhow!(
            "Unsupported file extension: {:?}",
            path
        )),
    }
}

pub fn tokenize_str(content: &str, language: Language) -> anyhow::Result<Vec<Token>> {
    match language {
        Language::Cpp => cpp::tokenize_str(content),
        Language::Rust => rust::tokenize_str(content),
        Language::Verilog => verilog::tokenize_str(content),
        Language::Python => verilog::tokenize_str(content),
    }
}
