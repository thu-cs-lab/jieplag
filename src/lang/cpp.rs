use crate::token::Token;
use anyhow::anyhow;
use clang::token::TokenKind;
use std::{
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub fn tokenize(path: &Path) -> anyhow::Result<Vec<Token>> {
    let clang = clang::Clang::new().map_err(|err| anyhow!("{}", err))?;
    let index = clang::Index::new(&clang, true, false);
    let tu = index.parser(path).parse()?;
    let mut vector = vec![];
    if let Some(range) = tu.get_entity().get_range() {
        for token in range.tokenize() {
            let kind = token.get_kind();
            let kind_u8 = match kind {
                TokenKind::Comment => continue,
                TokenKind::Identifier => 0x0,
                TokenKind::Literal => 0x1,
                TokenKind::Keyword | TokenKind::Punctuation => {
                    // Keyword or Punctuation
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    token.get_spelling().hash(&mut hasher);
                    let hash = hasher.finish() as u8 % 127;
                    // Keyword: [2, 128]
                    // Punctuation: [129, 255]
                    hash + if kind == TokenKind::Keyword { 2 } else { 129 }
                }
            };

            vector.push(Token {
                path: PathBuf::from(path),
                spelling: token.get_spelling(),
                kind: kind_u8,
                line: token.get_location().get_file_location().line,
                column: token.get_location().get_file_location().column,
            })
        }
    }
    Ok(vector)
}

pub fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let mut temp = NamedTempFile::new()?;
    write!(temp, "{}", content)?;
    tokenize(&temp.into_temp_path())
}
