use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Node {
    Program(Box<Node>),
    Function(String, Box<Node>),
    Statement(Statement, Box<Expr>),
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
    Term(Box<Term>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Term(t) => write!(f, "{}", *t),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Term {
    Factor(Box<Factor>),
    // High precedence binary operators
    BinOp(BinOp, Box<Term>, Box<Term>),
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Factor(fac) => write!(f, "{}", *fac),
            Self::BinOp(op, t1, t2) => write!(f, "{} {} {}", t1, op, t2),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Factor {
    Expr(Box<Expr>),
    UnOp(UnOp, Box<Factor>),
    IntLiteral(i32),
    // Low precedence binary operators
    BinOp(BinOp, Box<Factor>, Box<Factor>),
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expr(e) => write!(f, "{}", *e),
            Self::UnOp(op, factor) => write!(f, "{}{}", op, *factor),
            Self::IntLiteral(n) => write!(f, "Int<{}>", n),
            Self::BinOp(op, f1, f2) => write!(f, "{} {} {}", *f1, op, *f2),
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

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Subtract => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_int_literal() {
        assert_eq!(format!("{}", Factor::IntLiteral(1)), "Int<1>");
    }

    #[test]
    fn display_unary_operators() {
        assert_eq!(format!("{}", UnOp::Negate), "-");
        assert_eq!(format!("{}", UnOp::Complement), "~");
        assert_eq!(format!("{}", UnOp::LogicalNegate), "!");
    }

    #[test]
    fn display_unary_factors() {
        assert_eq!(
            format!(
                "{}",
                Factor::UnOp(UnOp::Negate, Factor::IntLiteral(5).into())
            ),
            "-Int<5>"
        );
        assert_eq!(
            format!(
                "{}",
                Factor::UnOp(UnOp::Complement, Factor::IntLiteral(0).into())
            ),
            "~Int<0>"
        );
        assert_eq!(
            format!(
                "{}",
                Factor::UnOp(UnOp::LogicalNegate, Factor::IntLiteral(16).into())
            ),
            "!Int<16>"
        );
    }

    #[test]
    fn display_high_precedence_binary_exprs() {
        assert_eq!(
            format!(
                "{}",
                Expr::Term(
                    Term::BinOp(
                        BinOp::Add,
                        Term::Factor(Factor::IntLiteral(1).into()).into(),
                        Term::Factor(Factor::IntLiteral(2).into()).into()
                    )
                    .into()
                )
            ),
            "Int<1> + Int<2>"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Term(
                    Term::BinOp(
                        BinOp::Subtract,
                        Term::Factor(Factor::IntLiteral(1).into()).into(),
                        Term::Factor(Factor::IntLiteral(2).into()).into()
                    )
                    .into()
                )
            ),
            "Int<1> - Int<2>"
        );
    }

    #[test]
    fn display_low_precedence_binary_exprs() {
        assert_eq!(
            format!(
                "{}",
                Expr::Term(
                    Term::Factor(
                        Factor::BinOp(
                            BinOp::Multiply,
                            Factor::IntLiteral(1).into(),
                            Factor::IntLiteral(2).into()
                        )
                        .into()
                    )
                    .into()
                )
            ),
            "Int<1> * Int<2>"
        );
        assert_eq!(
            format!(
                "{}",
                Expr::Term(
                    Term::Factor(
                        Factor::BinOp(
                            BinOp::Divide,
                            Factor::IntLiteral(1).into(),
                            Factor::IntLiteral(2).into()
                        )
                        .into()
                    )
                    .into()
                )
            ),
            "Int<1> / Int<2>"
        );
    }

    #[test]
    fn display_return_statement() {
        assert_eq!(
            format!(
                "{}",
                Node::Statement(
                    Statement::Return,
                    Expr::Term(Term::Factor(Factor::IntLiteral(0).into()).into()).into()
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
                        Expr::Term(Term::Factor(Factor::IntLiteral(10).into()).into()).into()
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
                            Expr::Term(Term::Factor(Factor::IntLiteral(10).into()).into()).into()
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
