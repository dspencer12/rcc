use std::error::Error;
use std::i32;

use lazy_static::lazy_static;
use regex::Regex;

use super::error::SyntaxError;

#[derive(Debug, PartialEq)]
pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Semicolon,
    IntKw,
    ReturnKw,
    Identifier(String),
    IntLiteral(i32),
}

fn char_to_token(c: &char) -> Option<Token> {
    match c {
        '{' => Some(Token::OpenBrace),
        '}' => Some(Token::CloseBrace),
        '(' => Some(Token::OpenParen),
        ')' => Some(Token::CloseParen),
        ';' => Some(Token::Semicolon),
        _ => None,
    }
}

fn get_keyword_or_id(input: &str) -> Result<(Token, &str), SyntaxError> {
    lazy_static! {
        static ref ID_REGEX: Regex = Regex::new(r"^[a-zA-Z]\w*").unwrap();
        static ref INVALID_ID_REGEX: Regex = Regex::new(r"^[^\(\)\{\}\s]+").unwrap();
    }
    match ID_REGEX.find(input) {
        Some(m) => Ok((
            match m.as_str() {
                "int" => Token::IntKw,
                "return" => Token::ReturnKw,
                other => Token::Identifier(String::from(other)),
            },
            &input[m.end()..],
        )),
        None => match INVALID_ID_REGEX.find(input) {
            Some(m) => Err(SyntaxError::InvalidIdentifier(
                String::from(m.as_str().split_whitespace().next().unwrap()).into(),
            )),
            None => Err(SyntaxError::Unknown),
        },
    }
}

fn tokenize_int_literal(input: &str) -> Result<Option<(i32, usize)>, Box<dyn Error>> {
    lazy_static! {
        static ref INT_REGEX: Regex =
            Regex::new(r"^(0x[0-9a-fA-F]+)|^(0[0-7]+)|^([0-9]+)").unwrap();
    }
    match INT_REGEX.captures(input) {
        Some(caps) => match caps.get(1) {
            Some(m) => Ok(Some((i32::from_str_radix(&m.as_str()[2..], 16)?, m.end()))),
            None => match caps.get(2) {
                Some(m) => Ok(Some((i32::from_str_radix(&m.as_str()[1..], 8)?, m.end()))),
                None => match caps.get(3) {
                    Some(m) => Ok(Some((m.as_str().parse()?, m.end()))),
                    None => Ok(None),
                },
            },
        },
        None => Ok(None),
    }
}

fn tokenize_const_or_id(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let int_match = tokenize_int_literal(input)?;
    match int_match {
        Some((num, end)) => {
            let mut res = vec![Token::IntLiteral(num)];
            res.extend(tokenize(&input[end..])?);
            return Ok(res);
        }
        None => (),
    }
    let (t, input) = get_keyword_or_id(input)?;
    let mut res = vec![t];
    res.extend(tokenize(&input)?);
    Ok(res)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    match input.chars().next() {
        Some(c) => match char_to_token(&c) {
            Some(t) => {
                let mut tokens = vec![t];
                tokens.extend(tokenize(&input[1..])?);
                Ok(tokens)
            }
            None => {
                if c.is_whitespace() {
                    tokenize(&input[1..])
                } else {
                    tokenize_const_or_id(input)
                }
            }
        },
        None => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Token::*;
    use super::*;

    #[test]
    fn decimal_literals() {
        for i in 0..11 {
            assert_eq!(tokenize(&i.to_string()).unwrap(), vec![IntLiteral(i)]);
        }
    }

    #[test]
    fn decimal_literals_line_breaks() {
        assert_eq!(
            tokenize("1\n2\n3\n").unwrap(),
            vec![IntLiteral(1), IntLiteral(2), IntLiteral(3)]
        );
    }

    #[test]
    fn hex_literals() {
        assert_eq!(tokenize("0x1").unwrap(), vec![IntLiteral(1)]);
        assert_eq!(tokenize("0xa").unwrap(), vec![IntLiteral(10)]);
        assert_eq!(tokenize("0xB").unwrap(), vec![IntLiteral(11)]);
        assert_eq!(tokenize("0xABC").unwrap(), vec![IntLiteral(2748)]);
    }

    #[test]
    fn oct_literals() {
        assert_eq!(tokenize("00").unwrap(), vec![IntLiteral(0)]);
        assert_eq!(tokenize("01").unwrap(), vec![IntLiteral(1)]);
        assert_eq!(tokenize("07").unwrap(), vec![IntLiteral(7)]);
        assert_eq!(tokenize("071").unwrap(), vec![IntLiteral(57)]);
        assert_eq!(tokenize("0777").unwrap(), vec![IntLiteral(511)]);
    }

    #[test]
    fn tab_separated_ints() {
        assert_eq!(
            tokenize("\t1\t2\t3\t").unwrap(),
            vec![IntLiteral(1), IntLiteral(2), IntLiteral(3)]
        );
    }

    #[test]
    fn basic_keywords() {
        assert_eq!(tokenize("int").unwrap(), vec![IntKw]);
        assert_eq!(tokenize("return").unwrap(), vec![ReturnKw]);
    }

    #[test]
    fn return_statement() {
        assert_eq!(
            tokenize("return 0;").unwrap(),
            vec![ReturnKw, IntLiteral(0), Semicolon]
        );
    }

    #[test]
    fn empty_function_one_line() {
        assert_eq!(
            tokenize("int foo() {}").unwrap(),
            vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                CloseBrace
            ]
        );
    }

    #[test]
    fn empty_function() {
        assert_eq!(
            tokenize("int foo() {\n}").unwrap(),
            vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                CloseBrace
            ]
        );
    }

    #[test]
    fn function_return_0() {
        assert_eq!(
            tokenize("int foo() {\n\treturn 0;\n}").unwrap(),
            vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        )
    }

    #[test]
    fn syntax_error_with_invalid_identifier() {
        assert_eq!(
            *tokenize("int $foo() {}")
                .err()
                .unwrap()
                .downcast::<SyntaxError>()
                .unwrap(),
            SyntaxError::InvalidIdentifier(String::from("$foo"))
        );
    }

    #[test]
    fn file_return_2() {
        let contents = fs::read_to_string("tests/testfiles/valid/return_2.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(2),
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_multi_digit() {
        let contents = fs::read_to_string("tests/testfiles/valid/multi_digit.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(100),
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_many_newlines() {
        let contents = fs::read_to_string("tests/testfiles/valid/many_newlines.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_minimal_whitespace() {
        let contents = fs::read_to_string("tests/testfiles/valid/minimal_whitespace.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_return_0() {
        let contents = fs::read_to_string("tests/testfiles/valid/return_0.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_abundant_spaces() {
        let contents = fs::read_to_string("tests/testfiles/valid/abundant_spaces.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_missing_paren() {
        let contents = fs::read_to_string("tests/testfiles/invalid/missing_paren.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_missing_return_val() {
        let contents = fs::read_to_string("tests/testfiles/invalid/missing_return_val.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Semicolon,
                CloseBrace
            ]
        );
    }

    #[test]
    fn file_missing_closing_brace() {
        let contents =
            fs::read_to_string("tests/testfiles/invalid/missing_closing_brace.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
            ]
        );
    }

    #[test]
    fn file_missing_semicolon() {
        let contents = fs::read_to_string("tests/testfiles/invalid/missing_semicolon.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                CloseBrace,
            ]
        );
    }

    #[test]
    fn file_missing_return_space() {
        let contents =
            fs::read_to_string("tests/testfiles/invalid/missing_return_space.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                Identifier(String::from("return0")),
                Semicolon,
                CloseBrace,
            ]
        );
    }

    #[test]
    fn file_wrong_return_case() {
        let contents = fs::read_to_string("tests/testfiles/invalid/wrong_return_case.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKw,
                Identifier(String::from("main")),
                OpenParen,
                CloseParen,
                OpenBrace,
                Identifier(String::from("RETURN")),
                IntLiteral(0),
                Semicolon,
                CloseBrace,
            ]
        );
    }
}
