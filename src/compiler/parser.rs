use std::error::Error;
use std::iter::Peekable;

use super::ast;
use super::error::SyntaxError;
use super::lexer::Token;

fn token_to_unop(t: &Token) -> Result<ast::UnOp, Box<dyn Error>> {
    match t {
        Token::Bang => Ok(ast::UnOp::LogicalNegate),
        Token::Minus => Ok(ast::UnOp::Negate),
        Token::Tilde => Ok(ast::UnOp::Complement),
        _ => Err("Invalid unary operator".into()),
    }
}

fn token_to_binop(t: &Token) -> Result<ast::BinOp, Box<dyn Error>> {
    match t {
        Token::Plus => Ok(ast::BinOp::Add),
        Token::Minus => Ok(ast::BinOp::Subtract),
        Token::Asterisk => Ok(ast::BinOp::Multiply),
        Token::Slash => Ok(ast::BinOp::Divide),
        _ => Err("Invalid binary operator".into()),
    }
}

fn parse_factor<'a, I>(tokens: &mut Peekable<I>) -> Result<ast::Factor, Box<dyn Error>>
where
    I: Iterator<Item = &'a Token>,
{
    match tokens.next() {
        Some(Token::IntLiteral(n)) => Ok(ast::Factor::IntLiteral(*n)),
        Some(t @ Token::Bang) | Some(t @ Token::Minus) | Some(t @ Token::Tilde) => Ok(
            ast::Factor::UnOp(token_to_unop(t)?, parse_factor(tokens)?.into()),
        ),
        Some(Token::OpenParen) => {
            let expr = parse_expression(tokens)?;
            match tokens.next() {
                Some(Token::CloseParen) => Ok(ast::Factor::Expr(expr.into())),
                _ => Err(SyntaxError::MissingCloseParen.into()),
            }
        }
        _ => Err(SyntaxError::InvalidExpression.into()),
    }
}

fn parse_term<'a, I>(tokens: &mut Peekable<I>) -> Result<ast::Term, Box<dyn Error>>
where
    I: Iterator<Item = &'a Token>,
{
    let mut factor = parse_factor(tokens)?;
    loop {
        let next = tokens.peek();
        match next {
            Some(t) => match t {
                Token::Asterisk | Token::Slash => {
                    let op = token_to_binop(tokens.next().unwrap())?;
                    let next_factor = parse_factor(tokens)?;
                    factor = ast::Factor::BinOp(op, factor.into(), next_factor.into());
                }
                _ => break,
            },
            None => break,
        };
    }
    Ok(ast::Term::Factor(factor.into()))
}

fn parse_expression<'a, I>(tokens: &mut Peekable<I>) -> Result<ast::Expr, Box<dyn Error>>
where
    I: Iterator<Item = &'a Token>,
{
    let mut term = parse_term(tokens)?;
    loop {
        let next = tokens.peek();
        match next {
            Some(t) => match t {
                Token::Plus | Token::Minus => {
                    let op = token_to_binop(tokens.next().unwrap())?;
                    let next_term = parse_term(tokens)?;
                    term = ast::Term::BinOp(op, term.into(), next_term.into());
                }
                _ => break,
            },
            None => break,
        };
    }
    Ok(ast::Expr::Term(term.into()))
}

fn parse_statement<'a, I>(tokens: &mut Peekable<I>) -> Result<ast::Node, Box<dyn Error>>
where
    I: Iterator<Item = &'a Token>,
{
    match tokens.next() {
        Some(Token::ReturnKw) => {
            let expr = parse_expression(tokens)?;
            match tokens.next() {
                Some(Token::Semicolon) => {
                    Ok(ast::Node::Statement(ast::Statement::Return, expr.into()))
                }
                _ => Err(SyntaxError::MissingSemicolon.into()),
            }
        }
        _ => Err(SyntaxError::UnexpectedToken.into()),
    }
}

fn parse_function<'a, I>(tokens: &mut Peekable<I>) -> Result<ast::Node, Box<dyn Error>>
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
        parse_function(&mut tokens.iter().peekable())?.into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;
    use Token::*;

    #[test]
    fn int_literal() {
        assert_eq!(
            parse_expression(&mut vec![IntLiteral(1)].iter().peekable()).unwrap(),
            Expr::Term(Term::Factor(Factor::IntLiteral(1).into()).into())
        );
    }

    #[test]
    fn unary_operators() {
        assert_eq!(
            parse_expression(&mut vec![Tilde, IntLiteral(0)].iter().peekable()).unwrap(),
            Expr::Term(
                Term::Factor(Factor::UnOp(UnOp::Complement, Factor::IntLiteral(0).into()).into())
                    .into()
            )
        )
    }

    #[test]
    fn return_statement() {
        assert_eq!(
            parse_statement(&mut vec![ReturnKw, IntLiteral(0), Semicolon].iter().peekable())
                .unwrap(),
            Node::Statement(
                Statement::Return,
                Expr::Term(Term::Factor(Factor::IntLiteral(0).into()).into()).into()
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
                .peekable()
            )
            .unwrap(),
            Node::Function(
                func_name.clone(),
                Node::Statement(
                    Statement::Return,
                    Expr::Term(Term::Factor(Factor::IntLiteral(0).into()).into()).into()
                )
                .into()
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
            Node::Program(
                Node::Function(
                    func_name.clone(),
                    Node::Statement(
                        Statement::Return,
                        Expr::Term(Term::Factor(Factor::IntLiteral(0).into()).into()).into()
                    )
                    .into()
                )
                .into()
            )
        );
    }

    #[test]
    fn program_function_return_complement_0() {
        let func_name = String::from("foo");
        assert_eq!(
            parse(&vec![
                IntKw,
                Identifier(func_name.clone()),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Tilde,
                IntLiteral(0),
                Semicolon,
                CloseBrace
            ])
            .unwrap(),
            Node::Program(
                Node::Function(
                    func_name.clone(),
                    Node::Statement(
                        Statement::Return,
                        Expr::Term(
                            Term::Factor(
                                Factor::UnOp(UnOp::Complement, Factor::IntLiteral(0).into()).into()
                            )
                            .into()
                        )
                        .into()
                    )
                    .into()
                )
                .into()
            )
        );
    }

    #[test]
    fn return_unary_on_unary_expr() {
        let func_name = String::from("foo");
        assert_eq!(
            parse(&vec![
                IntKw,
                Identifier(func_name.clone()),
                OpenParen,
                CloseParen,
                OpenBrace,
                ReturnKw,
                Tilde,
                OpenParen,
                Bang,
                IntLiteral(1),
                CloseParen,
                Semicolon,
                CloseBrace
            ])
            .unwrap(),
            Node::Program(
                Node::Function(
                    func_name.clone(),
                    Node::Statement(
                        Statement::Return,
                        Expr::Term(
                            Term::Factor(
                                Factor::UnOp(
                                    UnOp::Complement,
                                    Factor::Expr(
                                        Expr::Term(
                                            Term::Factor(
                                                Factor::UnOp(
                                                    UnOp::LogicalNegate,
                                                    Factor::IntLiteral(1).into()
                                                )
                                                .into()
                                            )
                                            .into()
                                        )
                                        .into()
                                    )
                                    .into()
                                )
                                .into()
                            )
                            .into()
                        )
                        .into()
                    )
                    .into()
                )
                .into()
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
