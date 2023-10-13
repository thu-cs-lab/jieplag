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

pub trait AnalyzableLang {
    fn tokenize(&self, path: &Path) -> anyhow::Result<Vec<Token>> {
        self.tokenize_str(&std::fs::read_to_string(path)?)
    }
    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>>;
}

struct LangInfo {
    name: Language,
    supported_extensions: Vec<&'static str>,
    tokenizer: Box<dyn AnalyzableLang>,
}

fn get_lang_info() -> Vec<LangInfo> {
    vec![
        LangInfo {
            name: Language::Cpp,
            supported_extensions: vec![
                "cpp", "cc", "cxx", "c++", "c", "cu"
            ],
            tokenizer: Box::new(tokenizer::cpp::Cpp),
        },
        LangInfo {
            name: Language::Rust,
            supported_extensions: vec![
                "rs"
            ],
            tokenizer: Box::new(tokenizer::rust::Rust),
        },
        LangInfo {
            name: Language::Verilog,
            supported_extensions: vec![
                "v"
            ],
            tokenizer: Box::new(tokenizer::verilog::Verilog),
        },
        LangInfo {
            name: Language::Python,
            supported_extensions: vec![
                "py"
            ],
            tokenizer: Box::new(tokenizer::python::Python),
        },
        LangInfo {
            name: Language::SQL,
            supported_extensions: vec![
                "sql"
            ],
            tokenizer: Box::new(tokenizer::sql::SQL),
        },
        LangInfo {
            name: Language::JavaScript,
            supported_extensions: vec![
                "js"
            ],
            tokenizer: Box::new(tokenizer::javascript::JavaScript),
        },
        LangInfo {
            name: Language::Lua,
            supported_extensions: vec![
                "lua"
            ],
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
        if lang.supported_extensions.contains(&extension.as_str()) {
            return lang.tokenizer.tokenize(path);
        }
    }
    return Err(anyhow!("Unsupported file extension: {:?}", path));
}


pub fn tokenize_str(content: &str, language: Language) -> anyhow::Result<Vec<Token>> {
    for lang in get_lang_info() {
        if lang.name == language {
            return lang.tokenizer.tokenize_str(content);
        }
    }
    return Err(anyhow!("Unsupported supported language: {:?}", language));
}
