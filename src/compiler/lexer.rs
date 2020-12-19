use std::error::Error;
use std::fmt;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub struct SyntaxError {
    loc: String,
}

impl SyntaxError {
    fn new(loc: String) -> Self {
        SyntaxError{
            loc
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Syntax error in {}", self.loc)
    }
}

impl Error for SyntaxError {}

#[derive(Debug, PartialEq)]
pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Semicolon,
    IntKeyword,
    Return,
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

fn get_keyword_or_id(input: &str) -> Result<(Token, &str), Box<dyn Error>> {
    lazy_static! {
        static ref ID_REGEX: Regex = Regex::new(r"^[a-zA-Z]\w*").unwrap();
    }
    match ID_REGEX.find(input) {
        Some(m) => Ok((
            match m.as_str() {
                "int" => Token::IntKeyword,
                "return" => Token::Return,
                other => Token::Identifier(String::from(other)),
            }, &input[m.end()..]
        )),
        None => Err(SyntaxError::new(String::from(input)).into())
    }
}

fn tokenize_const_or_id(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    lazy_static! {
        static ref INT_REGEX: Regex = Regex::new(r"^[0-9]+").unwrap();
    }
    match INT_REGEX.find(input) {
        Some(m) => {
            let mut res = vec![Token::IntLiteral(m.as_str().parse().unwrap())];
            res.extend(tokenize(&input[m.end()..])?);
            return Ok(res)
        },
        None => (),
    };
    let (t, input) = get_keyword_or_id(input)?;
    let mut res = vec![t];
    res.extend(tokenize(&input)?);
    Ok(res)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    match input.chars().next() {
        Some(c) => {
            match char_to_token(&c) {
                Some(t) => {
                    let mut tokens = vec![t];
                    tokens.extend(tokenize(&input[1..])?);
                    Ok(tokens)
                },
                None => {
                    if c.is_whitespace() {
                        tokenize(&input[1..])
                    } else {
                        tokenize_const_or_id(input)
                    }
                }
            }
        },
        None => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use super::Token::*;

    #[test]
    fn int_literals() {
        for i in 0..11 {
            assert_eq!(
                tokenize(&i.to_string()).unwrap(),
                vec![IntLiteral(i)]
            );
        }
    }

    #[test]
    fn int_literals_line_breaks() {
        assert_eq!(
            tokenize("1\n2\n3\n").unwrap(),
            vec![IntLiteral(1), IntLiteral(2), IntLiteral(3)]
        );
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
        assert_eq!(
            tokenize("int").unwrap(),
            vec![IntKeyword]
        );
        assert_eq!(
            tokenize("return").unwrap(),
            vec![Return]
        );
    }

    #[test]
    fn return_statement() {
        assert_eq!(
            tokenize("return 0;").unwrap(),
            vec![Return, IntLiteral(0), Semicolon]
        );
    }

    #[test]
    fn empty_function_one_line() {
        assert_eq!(
            tokenize("int foo() {}").unwrap(),
            vec![
                IntKeyword, 
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
                IntKeyword, 
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
                IntKeyword,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                Return,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ]
        )
    }

    #[test]
    fn function_return_2() {
        let contents = fs::read_to_string("tests/testfiles/return_2.c").unwrap();
        assert_eq!(
            tokenize(&contents).unwrap(),
            vec![
                IntKeyword, 
                Identifier(String::from("main")), 
                OpenParen,
                CloseParen,
                OpenBrace, 
                Return, 
                IntLiteral(2), 
                Semicolon, 
                CloseBrace
            ]
        );
    }
}