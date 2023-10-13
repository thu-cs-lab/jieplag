use crate::lang::AnalyzableLang;
use crate::token::Token;
use anyhow::anyhow;
use clang::token::TokenKind;
use std::{
    hash::{Hash, Hasher},
    path::Path,
};
use tempfile::tempdir;

pub struct Cpp;

impl AnalyzableLang for Cpp {
    fn tokenize(&self, path: &Path) -> anyhow::Result<Vec<Token>> {
        tokenize(path)
    }

    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>> {
        tokenize_str(content)
    }
}

fn tokenize(path: &Path) -> anyhow::Result<Vec<Token>> {
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
                spelling: token.get_spelling(),
                kind: kind_u8,
                line: token.get_location().get_file_location().line,
                column: token.get_location().get_file_location().column,
            })
        }
    }
    Ok(vector)
}

fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let dir = tempdir()?;
    let path = dir.path().join("code.cpp");
    std::fs::write(&path, content)?;
    tokenize(&path)
}

#[cfg(test)]
mod tests {
    use super::tokenize_str;

    #[test]
    fn test_tokenize() {
        let code = "int main() { return 0; }";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "int");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].spelling, "main");
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 5);

        assert_eq!(tokens[2].spelling, "(");
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 9);

        assert_eq!(tokens[3].spelling, ")");
        assert_eq!(tokens[3].line, 1);
        assert_eq!(tokens[3].column, 10);

        assert_eq!(tokens[8].spelling, "}");
        assert_eq!(tokens[8].line, 1);
        assert_eq!(tokens[8].column, 24);
    }
}
