use std::error::Error;
use std::i32;

use lazy_static::lazy_static;
use regex::Regex;

use super::error::SyntaxError;

#[derive(Debug, PartialEq)]
pub enum Token {
    // Syntax elements
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Semicolon,
    // Operators
    Minus,
    Tilde,
    Bang,
    Plus,
    Slash,
    Asterisk,
    DoubleAmpersand,
    DoubleBar,
    DoubleEqual,
    BangEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    // Keywords
    IntKw,
    ReturnKw,
    // Identifiers and literals
    Identifier(String),
    IntLiteral(i32),
}

fn symbols_to_token(s: &str) -> Option<Token> {
    match s {
        "{" => Some(Token::OpenBrace),
        "}" => Some(Token::CloseBrace),
        "(" => Some(Token::OpenParen),
        ")" => Some(Token::CloseParen),
        ";" => Some(Token::Semicolon),
        "-" => Some(Token::Minus),
        "~" => Some(Token::Tilde),
        "!" => Some(Token::Bang),
        "+" => Some(Token::Plus),
        "/" => Some(Token::Slash),
        "*" => Some(Token::Asterisk),
        "&&" => Some(Token::DoubleAmpersand),
        "||" => Some(Token::DoubleBar),
        "==" => Some(Token::DoubleEqual),
        "!=" => Some(Token::BangEqual),
        "<" => Some(Token::LessThan),
        ">" => Some(Token::GreaterThan),
        "<=" => Some(Token::LessThanEqual),
        ">=" => Some(Token::GreaterThanEqual),
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

fn tokenize_symbol(input: &str) -> Result<Option<(Token, &str)>, Box<dyn Error>> {
    lazy_static! {
        static ref SYMBOL_REGEX: Regex = Regex::new(
            r"^(?:&&|\|\||==|!=|>=|<=|>|<|\{|\}|\(|\)|;|-|~|!|\+|/|\*)"
        ).unwrap();
    }
    match SYMBOL_REGEX.find(input) {
        Some(m) => match symbols_to_token(m.as_str()) {
            Some(t) => Ok(Some((t, &input[m.end()..]))),
            None => Err("Unexpected symbols".into()),
        },
        None => Ok(None),
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    match tokenize_symbol(input)? {
        Some((t, input)) => {
            let mut tokens = vec![t];
            tokens.extend(tokenize(input)?);
            Ok(tokens)
        },
        None => match input.chars().next() {
            Some(c) => {
                if c.is_whitespace() {
                    tokenize(&input[1..])
                } else {
                    tokenize_const_or_id(input)
                }
            }
            None => Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests;
