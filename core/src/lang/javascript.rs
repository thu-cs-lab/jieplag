use crate::token::Token;
use boa_interner::Interner;
use boa_parser::lexer::token::TokenKind::*;
use boa_parser::Lexer;
use std::io::Cursor;
use std::path::Path;

pub fn tokenize(path: &Path) -> anyhow::Result<Vec<Token>> {
    tokenize_str(&std::fs::read_to_string(path)?)
}

pub fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let mut res = vec![];
    let mut lexer = Lexer::new(Cursor::new(content));
    let mut interner = Interner::new();
    while let Some(token) = lexer.next(&mut interner)? {
        let kind = match token.kind() {
            BooleanLiteral(_) => 0,
            EOF => continue,
            IdentifierName(_) => 1,
            PrivateIdentifier(_) => 2,
            Keyword(_) => 3,
            NullLiteral(_) => 4,
            NumericLiteral(_) => 5,
            Punctuator(_) => 6,
            StringLiteral(_) => 7,
            TemplateNoSubstitution(_) => 8,
            TemplateMiddle(_) => 9,
            RegularExpressionLiteral(_, _) => 10,
            LineTerminator => continue,
            Comment => continue,
        };
        res.push(Token {
            kind,
            spelling: token.kind().to_string(&interner),
            line: token.span().start().line_number(),
            column: token.span().start().column_number(),
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::tokenize_str;

    #[test]
    fn test_tokenize() {
        // taken from https://www.w3schools.com/js/js_functions.asp
        let code = "function myFunction(p1, p2) {\nreturn p1 * p2;\n}";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "function");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].spelling, "myFunction");
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 10);

        assert_eq!(tokens[2].spelling, "(");
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 20);

        assert_eq!(tokens[3].spelling, "p1");
        assert_eq!(tokens[3].line, 1);
        assert_eq!(tokens[3].column, 21);

        assert_eq!(tokens[8].spelling, "return");
        assert_eq!(tokens[8].line, 2);
        assert_eq!(tokens[8].column, 1);
    }
}
