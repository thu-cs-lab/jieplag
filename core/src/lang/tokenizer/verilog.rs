use crate::lang::AnalyzableLang;
use crate::token::Token;
use verilog_lang::lexer::Lexer;


pub struct Verilog;

impl AnalyzableLang for Verilog {
    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>> {
        tokenize_str(content)
    }
}

pub fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let lexer = Lexer::lex(content);
    let mut res = vec![];
    for token in lexer.tokens {
        res.push(Token {
            kind: token.token as u8,
            spelling: token.text.to_string(),
            line: token.span.from.row as u32 + 1,
            column: token.span.from.col as u32 + 1,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::tokenize_str;

    #[test]
    fn test_tokenize() {
        let code = "module test (); reg test_reg;\nendmodule";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "module");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].spelling, "test");
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 8);

        assert_eq!(tokens[2].spelling, "(");
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 13);

        assert_eq!(tokens[8].spelling, "endmodule");
        assert_eq!(tokens[8].line, 2);
        assert_eq!(tokens[8].column, 1);
    }
}
