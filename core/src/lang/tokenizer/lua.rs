use crate::lang::Tokenize;
use crate::token::Token;
use full_moon::tokenizer::tokens;
use full_moon::tokenizer::TokenKind::*;

pub struct Lua;

impl Tokenize for Lua {
    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>> {
        tokenize_str(content)
    }
}

fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let mut res = vec![];
    for token in tokens(content)? {
        let kind = match token.token_kind() {
            Eof => continue,
            Identifier => 0,
            MultiLineComment => continue,
            Number => 1,
            Shebang => 2,
            SingleLineComment => continue,
            StringLiteral => 3,
            Symbol => 4,
            Whitespace => continue,
            _ => todo!(),
        };
        res.push(Token {
            kind,
            spelling: token.to_string(),
            line: token.start_position().line() as u32,
            column: token.start_position().character() as u32,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::tokenize_str;

    #[test]
    fn test_tokenize() {
        // example taken from https://www.lua.org/pil/1.html
        let code =
            "function fact (n)\nif n == 0 then\nreturn 1\nelse\nreturn n * fact(n-1)\nend\nend";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "function");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].spelling, "fact");
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 10);

        assert_eq!(tokens[2].spelling, "(");
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 15);

        assert_eq!(tokens[3].spelling, "n");
        assert_eq!(tokens[3].line, 1);
        assert_eq!(tokens[3].column, 16);

        assert_eq!(tokens[5].spelling, "if");
        assert_eq!(tokens[5].line, 2);
        assert_eq!(tokens[5].column, 1);
    }
}
