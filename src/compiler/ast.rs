use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Node {
    Program(Box<Node>),
    Function(String, Box<Node>),
    Statement(Statement, Box<Node>),
    Expression(Expr),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program(ast) => write!(f, "{}", *ast),
            Self::Function(name, node) => write!(
                f,
                "FUN INT {}:
    params: ()
    body:
        {}",
                name, node
            ),
            Self::Statement(s, node) => write!(f, "{} {}", s, node),
            Self::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return => write!(f, "RETURN"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    IntLiteral(i32),
    UnOp(UnOp, Box<Node>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntLiteral(n) => write!(f, "Int<{}>", n),
            Self::UnOp(op, node) => write!(f, "{}{}", op, node),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UnOp {
    Negate,
    Complement,
    LogicalNegate,
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negate => write!(f, "-"),
            Self::Complement => write!(f, "~"),
            Self::LogicalNegate => write!(f, "!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_int_literal() {
        assert_eq!(format!("{}", Expr::IntLiteral(1)), "Int<1>");
    }

    #[test]
    fn display_unary_operators() {
        assert_eq!(format!("{}", UnOp::Negate), "-");
        assert_eq!(format!("{}", UnOp::Complement), "~");
        assert_eq!(format!("{}", UnOp::LogicalNegate), "!");
    }

    #[test]
    fn display_unary_expressions() {
        assert_eq!(
            format!(
                "{}",
                Expr::UnOp(UnOp::Negate, Node::Expression(Expr::IntLiteral(5)).into())
            ),
            "-Int<5>"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::UnOp(
                    UnOp::Complement,
                    Node::Expression(Expr::IntLiteral(0)).into()
                )
            ),
            "~Int<0>"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::UnOp(
                    UnOp::LogicalNegate,
                    Node::Expression(Expr::IntLiteral(16)).into()
                )
            ),
            "!Int<16>"
        );
    }

    #[test]
    fn display_return_statement() {
        assert_eq!(
            format!(
                "{}",
                Node::Statement(
                    Statement::Return,
                    Node::Expression(Expr::IntLiteral(0)).into()
                )
            ),
            "RETURN Int<0>"
        );
    }

    #[test]
    fn display_function() {
        assert_eq!(
            format!(
                "{}",
                Node::Function(
                    String::from("foo"),
                    Node::Statement(
                        Statement::Return,
                        Node::Expression(Expr::IntLiteral(10)).into()
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

    #[test]
    fn display_program() {
        assert_eq!(
            format!(
                "{}",
                Node::Program(
                    Node::Function(
                        String::from("foo"),
                        Node::Statement(
                            Statement::Return,
                            Node::Expression(Expr::IntLiteral(10)).into()
                        )
                        .into()
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
