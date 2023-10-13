use crate::lang::Tokenize;
use crate::token::Token;
use anyhow::anyhow;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::str::FromStr;

pub struct Rust;

impl Tokenize for Rust {
    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>> {
        tokenize_str(content)
    }
}

fn flatten(token_stream: TokenStream) -> Vec<Token> {
    let mut res = vec![];
    // https://doc.rust-lang.org/reference/keywords.html
    let keywords = [
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
        "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "Self", "self", "static", "struct", "super", "trait", "true", "try", "type",
        "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    ];
    for tokens in token_stream {
        match tokens {
            TokenTree::Group(group) => {
                // kind: [0, 2]
                let spelling = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => continue,
                };
                res.push(Token {
                    spelling: spelling.0.to_string(),
                    kind: group.delimiter() as u8,
                    line: group.span_open().start().line as u32,
                    column: group.span_open().start().column as u32 + 1,
                });
                res.extend(flatten(group.stream()));
                res.push(Token {
                    spelling: spelling.1.to_string(),
                    kind: group.delimiter() as u8,
                    line: group.span_close().start().line as u32,
                    column: group.span_close().start().column as u32 + 1,
                });
            }
            TokenTree::Literal(literal) => {
                // kind: 3
                res.push(Token {
                    spelling: format!("{}", literal),
                    kind: 3,
                    line: literal.span().start().line as u32,
                    column: literal.span().start().column as u32 + 1,
                });
            }
            TokenTree::Ident(ident) => {
                // kind: [4, 4+keywords.len()]
                let spelling = format!("{}", ident);
                if let Some(i) = keywords.iter().position(|s| s == &spelling) {
                    res.push(Token {
                        spelling: format!("{}", ident),
                        kind: 5 + i as u8,
                        line: ident.span().start().line as u32,
                        column: ident.span().start().column as u32 + 1,
                    });
                } else {
                    res.push(Token {
                        spelling: format!("{}", ident),
                        kind: 4,
                        line: ident.span().start().line as u32,
                        column: ident.span().start().column as u32 + 1,
                    });
                }
            }
            TokenTree::Punct(punct) => {
                // kind: [5+keywords.len(), ...]
                // skip semicolon
                if punct.as_char() == ';' {
                    continue;
                }

                res.push(Token {
                    spelling: format!("{}", punct),
                    kind: 5
                        + keywords.len() as u8
                        + (punct.as_char() as u8) % (251 - keywords.len() as u8),
                    line: punct.span().start().line as u32,
                    column: punct.span().start().column as u32 + 1,
                });
            }
        }
    }
    res
}

pub fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let token_stream = TokenStream::from_str(content).map_err(|err| anyhow!("{}", err))?;
    Ok(flatten(token_stream))
}

#[cfg(test)]
mod tests {
    use super::tokenize_str;

    #[test]
    fn test_tokenize() {
        let code = "fn main() { println!(\"Hello, world!\"); }";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "fn");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].spelling, "main");
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 4);

        assert_eq!(tokens[2].spelling, "(");
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 8);

        assert_eq!(tokens[3].spelling, ")");
        assert_eq!(tokens[3].line, 1);
        assert_eq!(tokens[3].column, 9);

        assert_eq!(tokens[4].spelling, "{");
        assert_eq!(tokens[4].line, 1);
        assert_eq!(tokens[4].column, 11);

        assert_eq!(tokens[5].spelling, "println");
        assert_eq!(tokens[5].line, 1);
        assert_eq!(tokens[5].column, 13);

        // semicolon is skipped
        assert_eq!(tokens[10].spelling, "}");
        assert_eq!(tokens[10].line, 1);
        assert_eq!(tokens[10].column, 40);
    }

    #[test]
    fn test_tokenize_comments() {
        let code = "// test \nfn main() { println!(\"Hello, world!\"); }";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "fn");
        assert_eq!(tokens[0].line, 2);
        assert_eq!(tokens[0].column, 1);
    }
}
