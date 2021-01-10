use std::{
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use clang::token::TokenKind;

use crate::token::Token;

pub fn tokenize(path: &Path) -> Result<Vec<Token>, String> {
    let clang = clang::Clang::new()?;
    let index = clang::Index::new(&clang, true, false);
    let tu = index.parser(path).parse()?;
    let mut vector = vec![];
    if let Some(range) = tu.get_entity().get_range() {
        for token in range.tokenize() {
            let kind = token.get_kind();
            let mut kind_u8 = kind as u8;
            // only four kind of tokens, too small
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            if token.get_kind() == TokenKind::Comment {
                continue;
            } else if token.get_kind() == TokenKind::Punctuation
                || token.get_kind() == TokenKind::Keyword
            {
                token.get_spelling().hash(&mut hasher);
                kind_u8 ^= hasher.finish() as u8;
            }

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
