#[derive(Clone, Debug)]
pub struct Token {
    pub kind: u8,
    pub spelling: String,
    // 1-based
    pub line: u32,
    pub column: u32,
}
