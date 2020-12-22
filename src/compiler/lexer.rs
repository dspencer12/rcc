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
    Minus,
    Tilde,
    Bang,
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
        '-' => Some(Token::Minus),
        '~' => Some(Token::Tilde),
        '!' => Some(Token::Bang),
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
    use std::path::PathBuf;

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
    fn basic_operators() {
        assert_eq!(tokenize("-").unwrap(), vec![Minus]);
        assert_eq!(tokenize("~").unwrap(), vec![Tilde]);
        assert_eq!(tokenize("!").unwrap(), vec![Bang]);
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

    macro_rules! file_tests {
        ($
            (
                $test_dir:literal: (
                    $($name:ident: ($file:literal, $expected:expr),)+
                ),
            )+
        ) => {
            $(
                $(
                    #[test]
                    fn $name() {
                        let mut path = PathBuf::from($test_dir);
                        path.push($file);
                        let contents = fs::read_to_string(path).unwrap();
                        assert_eq!(
                            tokenize(&contents).unwrap(),
                            $expected
                        );
                    }
                )+
            )+
        }
    }

    file_tests! {
        "tests/testfiles/valid": (
            file_return_2: ("return_2.c",
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
            ),
            file_multi_digit: ("multi_digit.c",
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
            ),
            file_many_newlines: ("many_newlines.c",
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
            ),
            file_minimal_whitespace: ("minimal_whitespace.c",
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
            ),
            file_return_0: ("return_0.c",
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
            ),
            file_abundant_spaces: ("abundant_spaces.c",
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
            ),
            file_bitwise: ("bitwise.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Bang,
                    IntLiteral(12),
                    Semicolon,
                    CloseBrace
                ]
            ),
            file_bitwise_zero: ("bitwise_zero.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Tilde,
                    IntLiteral(0),
                    Semicolon,
                    CloseBrace
                ]
            ),
            file_neg: ("neg.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Minus,
                    IntLiteral(5),
                    Semicolon,
                    CloseBrace
                ]
            ),
            file_nested_ops: ("nested_ops.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Bang,
                    Minus,
                    IntLiteral(3),
                    Semicolon,
                    CloseBrace
                ]
            ),
            file_nested_ops_2: ("nested_ops_2.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Minus,
                    Tilde,
                    IntLiteral(0),
                    Semicolon,
                    CloseBrace
                ]
            ),
            file_not_0: ("not_0.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Bang,
                    IntLiteral(0),
                    Semicolon,
                    CloseBrace
                ]
            ),
            file_not_5: ("not_5.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Bang,
                    IntLiteral(5),
                    Semicolon,
                    CloseBrace
                ]
            ),
        ),
        "tests/testfiles/invalid": (
            file_missing_paren: ("missing_paren.c",
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
            ),
            file_missing_return_val: ("missing_return_val.c",
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
            ),
            file_missing_closing_brace: ("missing_closing_brace.c",
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
            ),
            file_missing_semicolon: ("missing_semicolon.c",
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
            ),
            file_missing_return_space: ("missing_return_space.c",
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
            ),
            file_wrong_return_case: ("wrong_return_case.c",
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
            ),
            file_missing_const: ("missing_const.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Bang,
                    Semicolon,
                    CloseBrace,
                ]
            ),
            file_missing_semicolon_2: ("missing_semicolon_2.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Bang,
                    IntLiteral(5),
                    CloseBrace,
                ]
            ),
            file_nested_missing_const: ("nested_missing_const.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    Bang,
                    Tilde,
                    Semicolon,
                    CloseBrace,
                ]
            ),
            file_wrong_unary_order: ("wrong_unary_order.c",
                vec![
                    IntKw,
                    Identifier(String::from("main")),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    IntLiteral(4),
                    Minus,
                    Semicolon,
                    CloseBrace,
                ]
            ),
        ),
    }
}
