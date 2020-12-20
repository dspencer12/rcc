use std::error::Error;

use super::error::SyntaxError;
use super::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum ASTExpression {
    Literal(i32),
}

#[derive(Debug, PartialEq)]
pub enum ASTStatement {
    Return(ASTExpression),
}

#[derive(Debug, PartialEq)]
pub enum AST {
    Function(String, ASTStatement),
    Program(Box<AST>),
}

fn parse_expression<'a, I>(tokens: &mut I) -> Option<ASTExpression>
where
    I: Iterator<Item = &'a Token>,
{
    match tokens.next() {
        Some(Token::IntLiteral(n)) => Some(ASTExpression::Literal(*n)),
        _ => None,
    }
}

fn parse_statement<'a, I>(tokens: &mut I) -> Result<ASTStatement, Box<dyn Error>>
where
    I: Iterator<Item = &'a Token>,
{
    match tokens.next() {
        Some(Token::ReturnKw) => match parse_expression(tokens) {
            Some(e) => match tokens.next() {
                Some(Token::Semicolon) => Ok(ASTStatement::Return(e)),
                _ => Err(SyntaxError::MissingSemicolon.into()),
            },
            None => Err(SyntaxError::InvalidExpression.into()),
        },
        _ => Err(SyntaxError::UnexpectedToken.into()),
    }
}

fn parse_function<'a, I>(tokens: &mut I) -> Result<AST, Box<dyn Error>>
where
    I: Iterator<Item = &'a Token>,
{
    match tokens.next() {
        Some(Token::IntKw) => match tokens.next() {
            Some(Token::Identifier(id)) => match tokens.next() {
                Some(Token::OpenParen) => match tokens.next() {
                    Some(Token::CloseParen) => match tokens.next() {
                        Some(Token::OpenBrace) => {
                            let s = parse_statement(tokens)?;
                            match tokens.next() {
                                Some(Token::CloseBrace) => Ok(AST::Function(String::from(id), s)),
                                _ => Err(SyntaxError::MissingCloseBrace.into()),
                            }
                        }
                        _ => Err(SyntaxError::MissingOpenBrace.into()),
                    },
                    _ => Err(SyntaxError::MissingCloseParen.into()),
                },
                _ => Err(SyntaxError::MissingOpenParen.into()),
            },
            _ => Err(SyntaxError::MissingIdentifier.into()),
        },
        _ => Err(SyntaxError::MissingKeyword(String::from("int")).into()),
    }
}

pub fn parse(tokens: &Vec<Token>) -> Result<AST, Box<dyn Error>> {
    Ok(AST::Program(parse_function(&mut tokens.iter())?.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn int_literal() {
        assert_eq!(
            parse_expression(&mut vec![IntLiteral(1)].iter()).unwrap(),
            ASTExpression::Literal(1)
        );
    }

    #[test]
    fn return_statement() {
        assert_eq!(
            parse_statement(&mut vec![ReturnKw, IntLiteral(0), Semicolon].iter()).unwrap(),
            ASTStatement::Return(ASTExpression::Literal(0))
        );
    }

    #[test]
    fn basic_function() {
        let func_name = String::from("foo");
        assert_eq!(
            parse_function(
                &mut vec![
                    IntKw,
                    Identifier(func_name.clone()),
                    OpenParen,
                    CloseParen,
                    OpenBrace,
                    ReturnKw,
                    IntLiteral(0),
                    Semicolon,
                    CloseBrace
                ]
                .iter()
            )
            .unwrap(),
            AST::Function(
                func_name.clone(),
                ASTStatement::Return(ASTExpression::Literal(0))
            )
        );
    }

    #[test]
    fn program_function_return_0() {
        let func_name = String::from("foo");
        assert_eq!(
            parse(&vec![
                IntKw,
                Identifier(func_name.clone()),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ])
            .unwrap(),
            AST::Program(
                AST::Function(
                    func_name.clone(),
                    ASTStatement::Return(ASTExpression::Literal(0))
                )
                .into()
            )
        );
    }

    #[test]
    fn function_missing_closing_brace() {
        assert_eq!(
            *parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon
            ])
            .err()
            .unwrap()
            .downcast::<SyntaxError>()
            .unwrap(),
            SyntaxError::MissingCloseBrace
        );
    }

    #[test]
    fn function_missing_closing_paren() {
        assert_eq!(
            *parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace,
            ])
            .err()
            .unwrap()
            .downcast::<SyntaxError>()
            .unwrap(),
            SyntaxError::MissingCloseParen
        );
    }

    #[test]
    fn function_missing_closing_paren_and_brace() {
        assert_eq!(
            *parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
            ])
            .err()
            .unwrap()
            .downcast::<SyntaxError>()
            .unwrap(),
            SyntaxError::MissingCloseParen
        );
    }

    #[test]
    fn function_missing_return_value() {
        assert_eq!(
            *parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Semicolon,
                CloseBrace,
            ])
            .err()
            .unwrap()
            .downcast::<SyntaxError>()
            .unwrap(),
            SyntaxError::InvalidExpression
        );
    }

    #[test]
    fn function_missing_semicolon() {
        assert_eq!(
            *parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(5),
                CloseBrace,
            ])
            .err()
            .unwrap()
            .downcast::<SyntaxError>()
            .unwrap(),
            SyntaxError::MissingSemicolon
        );
    }
}
