use crate::lang::Tokenize;
use crate::token::Token;
use sqlparser::{dialect::GenericDialect, tokenizer::Token::*, tokenizer::Tokenizer};

pub struct SQL;

impl Tokenize for SQL {
    fn tokenize_str(&self, content: &str) -> anyhow::Result<Vec<Token>> {
        tokenize_str(content)
    }
}

pub fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let dialect = GenericDialect {};
    let mut res = vec![];
    for token in Tokenizer::new(&dialect, content).tokenize_with_location()? {
        let spelling = token.token.to_string();
        let kind = match token.token {
            EOF => continue,
            Word(_) => 1,
            Number(_, _) => 2,
            Char(_) => 3,
            SingleQuotedString(_) => 4,
            DoubleQuotedString(_) => 5,
            TripleSingleQuotedString(_) => 6,
            TripleDoubleQuotedString(_) => 7,
            DollarQuotedString(_) => 8,
            SingleQuotedByteStringLiteral(_) => 9,
            DoubleQuotedByteStringLiteral(_) => 10,
            TripleSingleQuotedByteStringLiteral(_) => 11,
            TripleDoubleQuotedByteStringLiteral(_) => 12,
            SingleQuotedRawStringLiteral(_) => 13,
            DoubleQuotedRawStringLiteral(_) => 14,
            TripleSingleQuotedRawStringLiteral(_) => 15,
            TripleDoubleQuotedRawStringLiteral(_) => 16,
            NationalStringLiteral(_) => 17,
            EscapedStringLiteral(_) => 18,
            UnicodeStringLiteral(_) => 19,
            HexStringLiteral(_) => 20,
            Comma => 21,
            Whitespace(_) => continue,
            DoubleEq => 22,
            Eq => 23,
            Neq => 24,
            Lt => 25,
            Gt => 26,
            LtEq => 27,
            GtEq => 28,
            Spaceship => 29,
            Plus => 30,
            Minus => 31,
            Mul => 32,
            Div => 33,
            DuckIntDiv => 34,
            Mod => 35,
            StringConcat => 36,
            LParen => 37,
            RParen => 38,
            Period => 39,
            Colon => 40,
            DoubleColon => 41,
            Assignment => 42,
            SemiColon => 43,
            Backslash => 44,
            LBracket => 45,
            RBracket => 46,
            Ampersand => 47,
            Pipe => 48,
            Caret => 49,
            LBrace => 50,
            RBrace => 51,
            RArrow => 52,
            Sharp => 53,
            DoubleSharp => 54,
            Tilde => 55,
            TildeAsterisk => 56,
            ExclamationMarkTilde => 57,
            ExclamationMarkTildeAsterisk => 58,
            DoubleTilde => 59,
            DoubleTildeAsterisk => 60,
            ExclamationMarkDoubleTilde => 61,
            ExclamationMarkDoubleTildeAsterisk => 62,
            ShiftLeft => 63,
            ShiftRight => 64,
            Overlap => 65,
            ExclamationMark => 66,
            DoubleExclamationMark => 67,
            AtSign => 68,
            CaretAt => 69,
            PGSquareRoot => 70,
            PGCubeRoot => 71,
            Placeholder(_) => 72,
            Arrow => 73,
            LongArrow => 74,
            HashArrow => 75,
            AtDashAt => 76,
            QuestionMarkDash => 77,
            AmpersandLeftAngleBracket => 78,
            AmpersandRightAngleBracket => 79,
            AmpersandLeftAngleBracketVerticalBar => 80,
            VerticalBarAmpersandRightAngleBracket => 81,
            TwoWayArrow => 82,
            LeftAngleBracketCaret => 83,
            RightAngleBracketCaret => 84,
            QuestionMarkSharp => 85,
            QuestionMarkDashVerticalBar => 86,
            QuestionMarkDoubleVerticalBar => 87,
            TildeEqual => 88,
            ShiftLeftVerticalBar => 89,
            VerticalBarShiftRight => 90,
            VerticalBarRightAngleBracket => 91,
            HashLongArrow => 92,
            AtArrow => 93,
            ArrowAt => 94,
            HashMinus => 95,
            AtQuestion => 96,
            AtAt => 97,
            Question => 98,
            QuestionAnd => 99,
            QuestionPipe => 100,
            CustomBinaryOperator(_) => 101,
            QuoteDelimitedStringLiteral(_) => 102,
            NationalQuoteDelimitedStringLiteral(_) => 103,
        };
        res.push(Token {
            kind,
            spelling,
            line: token.span.start.line as u32,
            column: token.span.start.column as u32,
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
