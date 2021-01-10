use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Token {
    pub path: PathBuf,
    pub kind: u8,
    pub spelling: String,
    pub line: u32,
    pub column: u32,
}
