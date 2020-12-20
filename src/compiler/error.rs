use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    Unknown,
    MissingOpenParen,
    MissingCloseParen,
    MissingOpenBrace,
    MissingCloseBrace,
    MissingSemicolon,
    MissingIdentifier,
    MissingKeyword(String),
    InvalidIdentifier(String),
    InvalidExpression,
    UnexpectedToken,
}

// TODO: write_error! macro

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier(id) => write!(f, "Syntax Error: Invalid identifier: {}", id),
            Self::MissingKeyword(kw) => write!(f, "Syntax Error: Expected \"{}\" keyword", kw),
            Self::MissingOpenParen => write!(f, "Syntax Error: Expected opening parenthesis"),
            Self::MissingCloseParen => write!(f, "Syntax Error: Expected closing parenthesis"),
            Self::MissingOpenBrace => write!(f, "Syntax Error: Expected opening brace"),
            Self::MissingCloseBrace => write!(f, "Syntax Error: Expected closing brace"),
            Self::MissingSemicolon => write!(f, "Syntax Error: Expected semicolon"),
            Self::MissingIdentifier => write!(f, "Syntax Error: Expected identifier"),
            Self::InvalidExpression => write!(f, "Syntax Error: Invalid expression"),
            Self::UnexpectedToken => write!(f, "Syntax Error: Uexpected token"),
            SyntaxError::Unknown => write!(f, "Syntax Error: Unknown error"),
        }
    }
}

impl Error for SyntaxError {}
