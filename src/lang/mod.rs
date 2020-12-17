use std::path::Path;

use crate::token::Token;

pub mod cpp;

pub fn tokenize(path: &Path) -> Result<Vec<Token>, String> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if extension == "cpp" {
        cpp::tokenize(path)
    } else {
        Err(format!(
            "Unsupported file extension: {:?}",
            path.extension()
        ))
    }
}
