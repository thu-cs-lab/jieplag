use crate::lang::AnalyzableLang;
use crate::token::Token;
use sqlparser::{dialect::GenericDialect, tokenizer::Token::*, tokenizer::Tokenizer};

pub struct SQL;

impl AnalyzableLang for SQL {
    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>> {
        tokenize_str(content)
    }
}

pub fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let dialect = GenericDialect {};
    let mut res = vec![];
    for token in Tokenizer::new(&dialect, content).tokenize_with_location()? {
        let kind = match token.token {
            EOF => continue,
            Word(_) => 1,
            Number(_, _) => 2,
            Char(_) => 3,
            SingleQuotedString(_) => 4,
            DoubleQuotedString(_) => 5,
            DollarQuotedString(_) => 6,
            SingleQuotedByteStringLiteral(_) => 7,
            DoubleQuotedByteStringLiteral(_) => 8,
            RawStringLiteral(_) => 9,
            NationalStringLiteral(_) => 10,
            EscapedStringLiteral(_) => 11,
            HexStringLiteral(_) => 12,
            Comma => 13,
            Whitespace(_) => continue,
            DoubleEq => 15,
            Eq => 16,
            Neq => 17,
            Lt => 18,
            Gt => 19,
            LtEq => 20,
            GtEq => 21,
            Spaceship => 22,
            Plus => 23,
            Minus => 24,
            Mul => 25,
            Div => 26,
            DuckIntDiv => 27,
            Mod => 28,
            StringConcat => 29,
            LParen => 30,
            RParen => 31,
            Period => 32,
            Colon => 33,
            DoubleColon => 34,
            DuckAssignment => 35,
            SemiColon => 36,
            Backslash => 37,
            LBracket => 38,
            RBracket => 39,
            Ampersand => 40,
            Pipe => 41,
            Caret => 42,
            LBrace => 43,
            RBrace => 44,
            RArrow => 45,
            Sharp => 46,
            Tilde => 47,
            TildeAsterisk => 48,
            ExclamationMarkTilde => 49,
            ExclamationMarkTildeAsterisk => 50,
            ShiftLeft => 51,
            ShiftRight => 52,
            Overlap => 53,
            ExclamationMark => 54,
            DoubleExclamationMark => 55,
            AtSign => 56,
            PGSquareRoot => 57,
            PGCubeRoot => 58,
            Placeholder(_) => 59,
            Arrow => 60,
            LongArrow => 61,
            HashArrow => 62,
            HashLongArrow => 63,
            AtArrow => 64,
            ArrowAt => 65,
            HashMinus => 66,
            AtQuestion => 67,
            AtAt => 68,
        };
        res.push(Token {
            kind,
            spelling: token.to_string(),
            line: token.location.line as u32,
            column: token.location.column as u32,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::tokenize_str;

    #[test]
    fn test_tokenize() {
        // example taken from https://crates.io/crates/sqlparser
        let code =
            "SELECT a, b, 123, myfunc(b)\nFROM table_1\nWHERE a > b AND b < 100\nORDER BY a DESC, b";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "SELECT");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].spelling, "a");
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 8);

        assert_eq!(tokens[2].spelling, ",");
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 9);

        assert_eq!(tokens[3].spelling, "b");
        assert_eq!(tokens[3].line, 1);
        assert_eq!(tokens[3].column, 11);

        assert_eq!(tokens[13].spelling, "WHERE");
        assert_eq!(tokens[13].line, 3);
        assert_eq!(tokens[13].column, 1);
    }
}
