use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ASTExpression {
    Literal(i32),
}

impl fmt::Display for ASTExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(n) => write!(f, "Int<{}>", n),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ASTStatement {
    Return(ASTExpression),
}

impl fmt::Display for ASTStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(e) => write!(f, "RETURN {}", e),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AST {
    Function(String, ASTStatement),
    Program(Box<AST>),
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function(name, body) => {
                write!(
                    f,
                    "FUN INT {}:
    params: ()
    body:
        {}",
                    name, body
                )
            }
            Self::Program(ast) => write!(f, "{}", *ast),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_int_literal() {
        assert_eq!(format!("{}", ASTExpression::Literal(1)), "Int<1>");
    }

    #[test]
    fn display_return_statement() {
        assert_eq!(
            format!("{}", ASTStatement::Return(ASTExpression::Literal(0))),
            "RETURN Int<0>"
        );
    }

    #[test]
    fn display_function() {
        assert_eq!(
            format!(
                "{}",
                AST::Function(
                    String::from("foo"),
                    ASTStatement::Return(ASTExpression::Literal(10))
                )
            ),
            "FUN INT foo:
    params: ()
    body:
        RETURN Int<10>"
        );
    }

    #[test]
    fn display_program() {
        assert_eq!(
            format!(
                "{}",
                AST::Program(
                    AST::Function(
                        String::from("foo"),
                        ASTStatement::Return(ASTExpression::Literal(10))
                    )
                    .into()
                )
            ),
            "FUN INT foo:
    params: ()
    body:
        RETURN Int<10>"
        );
    }
}
