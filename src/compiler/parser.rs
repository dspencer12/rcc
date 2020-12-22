use std::error::Error;

use super::ast;
use super::error::SyntaxError;
use super::lexer::Token;

fn parse_expression<'a, I>(tokens: &mut I) -> Option<ast::Node>
where
    I: Iterator<Item = &'a Token>,
{
    match tokens.next() {
        Some(Token::IntLiteral(n)) => Some(ast::Node::Expression(ast::Expr::IntLiteral(*n))),
        _ => None,
    }
}

fn parse_statement<'a, I>(tokens: &mut I) -> Result<ast::Node, Box<dyn Error>>
where
    I: Iterator<Item = &'a Token>,
{
    match tokens.next() {
        Some(Token::ReturnKw) => match parse_expression(tokens) {
            Some(e) => match tokens.next() {
                Some(Token::Semicolon) => {
                    Ok(ast::Node::Statement(ast::Statement::Return, e.into()))
                }
                _ => Err(SyntaxError::MissingSemicolon.into()),
            },
            None => Err(SyntaxError::InvalidExpression.into()),
        },
        _ => Err(SyntaxError::UnexpectedToken.into()),
    }
}

fn parse_function<'a, I>(tokens: &mut I) -> Result<ast::Node, Box<dyn Error>>
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
                                Some(Token::CloseBrace) => {
                                    Ok(ast::Node::Function(String::from(id), s.into()))
                                }
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

pub fn parse(tokens: &Vec<Token>) -> Result<ast::Node, Box<dyn Error>> {
    Ok(ast::Node::Program(
        parse_function(&mut tokens.iter())?.into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn int_literal() {
        assert_eq!(
            parse_expression(&mut vec![IntLiteral(1)].iter()).unwrap(),
            ast::Node::Expression(ast::Expr::IntLiteral(1))
        );
    }

    #[test]
    fn return_statement() {
        assert_eq!(
            parse_statement(&mut vec![ReturnKw, IntLiteral(0), Semicolon].iter()).unwrap(),
            ast::Node::Statement(
                ast::Statement::Return,
                ast::Node::Expression(ast::Expr::IntLiteral(0)).into()
            )
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
            ast::Node::Function(
                func_name.clone(),
                ast::Node::Statement(
                    ast::Statement::Return,
                    ast::Node::Expression(ast::Expr::IntLiteral(0)).into()
                ).into()
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
            ast::Node::Program(
                ast::Node::Function(
                    func_name.clone(),
                    ast::Node::Statement(
                        ast::Statement::Return,
                        ast::Node::Expression(ast::Expr::IntLiteral(0)).into()
                    ).into()
                ).into()
            )
        );
    }

    macro_rules! assert_raises_syntax_error {
        ($left:expr, $err:expr) => {
            assert_eq!(
                *$left.err().unwrap().downcast::<SyntaxError>().unwrap(),
                $err
            );
        };
    }

    #[test]
    fn function_missing_closing_brace() {
        assert_raises_syntax_error!(
            parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon
            ]),
            SyntaxError::MissingCloseBrace
        );
    }

    #[test]
    fn function_missing_closing_paren() {
        assert_raises_syntax_error!(
            parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
                CloseBrace,
            ]),
            SyntaxError::MissingCloseParen
        );
    }

    #[test]
    fn function_missing_closing_paren_and_brace() {
        assert_raises_syntax_error!(
            parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(0),
                Semicolon,
            ]),
            SyntaxError::MissingCloseParen
        );
    }

    #[test]
    fn function_missing_return_value() {
        assert_raises_syntax_error!(
            parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Semicolon,
                CloseBrace,
            ]),
            SyntaxError::InvalidExpression
        );
    }

    #[test]
    fn function_missing_semicolon() {
        assert_raises_syntax_error!(
            parse(&vec![
                IntKw,
                Identifier(String::from("foo")),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                IntLiteral(5),
                CloseBrace,
            ]),
            SyntaxError::MissingSemicolon
        );
    }
}
