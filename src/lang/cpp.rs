use std::path::{Path, PathBuf};

use clang::token::TokenKind;

use crate::token::Token;

pub fn tokenize(path: &Path) -> Result<Vec<Token>, String> {
    let clang = clang::Clang::new()?;
    let index = clang::Index::new(&clang, true, false);
    let tu = index.parser(path).parse()?;
    let mut vector = vec![];
    if let Some(range) = tu.get_entity().get_range() {
        for token in range.tokenize() {
            if token.get_kind() == TokenKind::Comment {
                continue;
            }
            vector.push(Token {
                path: PathBuf::from(path),
                kind: token.get_kind() as u8,
                line: token.get_location().get_file_location().line,
                column: token.get_location().get_file_location().column,
            })
        }
    }
    Ok(vector)
}
