use crate::token::Token;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod tokenizer;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Cpp,
    Rust,
    Verilog,
    Python,
    SQL,
    JavaScript,
    Lua,
}

pub trait Tokenize {
    fn tokenize(&self, path: &Path) -> anyhow::Result<Vec<Token>> {
        self.tokenize_str(&std::fs::read_to_string(path)?)
    }
    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>>;
}

struct LangInfo {
    name: Language,
    extensions: Vec<&'static str>,
    tokenizer: Box<dyn Tokenize>,
}

fn get_lang_info() -> Vec<LangInfo> {
    vec![
        #[cfg(feature = "cpp")]
        LangInfo {
            name: Language::Cpp,
            extensions: vec!["cpp", "cc", "cxx", "c++", "c", "cu"],
            tokenizer: Box::new(tokenizer::cpp::Cpp),
        },
        #[cfg(feature = "rust")]
        LangInfo {
            name: Language::Rust,
            extensions: vec!["rs"],
            tokenizer: Box::new(tokenizer::rust::Rust),
        },
        #[cfg(feature = "verilog")]
        LangInfo {
            name: Language::Verilog,
            extensions: vec!["v"],
            tokenizer: Box::new(tokenizer::verilog::Verilog),
        },
        #[cfg(feature = "python")]
        LangInfo {
            name: Language::Python,
            extensions: vec!["py"],
            tokenizer: Box::new(tokenizer::python::Python),
        },
        #[cfg(feature = "sql")]
        LangInfo {
            name: Language::SQL,
            extensions: vec!["sql"],
            tokenizer: Box::new(tokenizer::sql::SQL),
        },
        #[cfg(feature = "javascript")]
        LangInfo {
            name: Language::JavaScript,
            extensions: vec!["js"],
            tokenizer: Box::new(tokenizer::javascript::JavaScript),
        },
        #[cfg(feature = "lua")]
        LangInfo {
            name: Language::Lua,
            extensions: vec!["lua"],
            tokenizer: Box::new(tokenizer::lua::Lua),
        },
    ]
}

pub fn tokenize(path: &Path) -> anyhow::Result<Vec<Token>> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    for lang in get_lang_info() {
        if lang.extensions.contains(&extension.as_str()) {
            return lang.tokenizer.tokenize(path);
        }
    }
    Err(anyhow!("Unsupported file extension: {:?}. \
    Did you enable a corresponding feature?", path))
}

pub fn tokenize_str(content: &str, language: Language) -> anyhow::Result<Vec<Token>> {
    for lang in get_lang_info() {
        if lang.name == language {
            return lang.tokenizer.tokenize_str(content);
        }
    }
    Err(anyhow!("Unsupported language: {:?}. \
    Did you enable a corresponding feature?", language))
}
