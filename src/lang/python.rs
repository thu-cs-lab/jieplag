use crate::token::Token;
use anyhow::anyhow;
use rustpython_parser::{lexer::make_tokenizer, token::Tok::*};
use std::path::Path;

pub fn tokenize(path: &Path) -> anyhow::Result<Vec<Token>> {
    Ok(tokenize_str(&std::fs::read_to_string(path)?)?)
}

pub fn tokenize_str(content: &str) -> anyhow::Result<Vec<Token>> {
    let tokens = make_tokenizer(content);
    let mut res = vec![];
    for item in tokens {
        let (start, token, _end) =
            item.map_err(|err| anyhow!("{} at {}", err.error, err.location))?;
        let kind = match &token {
            Name { name: _ } => 0,
            Int { value: _ } => 1,
            Float { value: _ } => 2,
            Complex { real: _, imag: _ } => 3,
            String {
                value: _,
                is_fstring: _,
            } => 4,
            Bytes { value: _ } => 5,
            // skip newline
            Newline => continue, // 6
            Indent => 7,
            Dedent => 8,
            StartProgram => 9,
            StartStatement => 10,
            StartExpression => 11,
            EndOfFile => 12,
            Lpar => 13,
            Rpar => 14,
            Lsqb => 15,
            Rsqb => 16,
            Colon => 17,
            Comma => 18,
            Semi => 19,
            Plus => 20,
            Minus => 21,
            Star => 22,
            Slash => 23,
            Vbar => 24,  // '|'
            Amper => 25, // '&'
            Less => 26,
            Greater => 27,
            Equal => 28,
            Dot => 29,
            Percent => 30,
            Lbrace => 31,
            Rbrace => 32,
            EqEqual => 33,
            NotEqual => 34,
            LessEqual => 35,
            GreaterEqual => 36,
            Tilde => 37,
            CircumFlex => 38,
            LeftShift => 39,
            RightShift => 40,
            DoubleStar => 41,
            DoubleStarEqual => 42, // '**='
            PlusEqual => 43,
            MinusEqual => 44,
            StarEqual => 45,
            SlashEqual => 46,
            PercentEqual => 47,
            AmperEqual => 48, // '&='
            VbarEqual => 49,
            CircumflexEqual => 50, // '^='
            LeftShiftEqual => 51,
            RightShiftEqual => 52,
            DoubleSlash => 53, // '//'
            DoubleSlashEqual => 54,
            ColonEqual => 55,
            At => 56,
            AtEqual => 57,
            Rarrow => 58,
            Ellipsis => 59,

            // Keywords (alphabetically):
            False => 60,
            None => 61,
            True => 62,

            And => 63,
            As => 64,
            Assert => 65,
            Async => 66,
            Await => 67,
            Break => 68,
            Class => 69,
            Continue => 70,
            Def => 71,
            Del => 72,
            Elif => 73,
            Else => 74,
            Except => 75,
            Finally => 76,
            For => 77,
            From => 78,
            Global => 79,
            If => 80,
            Import => 81,
            In => 82,
            Is => 83,
            Lambda => 84,
            Nonlocal => 85,
            Not => 86,
            Or => 87,
            Pass => 88,
            Raise => 89,
            Return => 90,
            Try => 91,
            While => 92,
            With => 93,
            Yield => 94,
        };
        res.push(Token {
            kind,
            spelling: format!("{}", token),
            line: start.row() as u32,
            column: start.column() as u32,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::tokenize_str;

    #[test]
    fn test_tokenize() {
        let code = "a = input()\nb = input()\nprint(a+b)";
        let tokens = tokenize_str(code).unwrap();

        eprintln!("{:?}", tokens);

        assert_eq!(tokens[0].spelling, "'a'");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].spelling, "'='");
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 3);

        assert_eq!(tokens[2].spelling, "'input'");
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 5);

        assert_eq!(tokens[13].spelling, "'+'");
        assert_eq!(tokens[13].line, 3);
        assert_eq!(tokens[13].column, 8);
    }
}
