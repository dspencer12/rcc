use super::ast;

pub fn generate(ast: &ast::Node) -> Result<String, &'static str> {
    match ast {
        ast::Node::Program(node) => match &**node {
            ast::Node::Function(id, node) => match &**node {
                ast::Node::Statement(st, node) => match st {
                    ast::Statement::Return => match &**node {
                        ast::Node::Expression(expr) => match expr {
                            ast::Expr::IntLiteral(n) => Ok(format!(
                                ".globl _{}
_{}:
  movl    ${}, %eax
  ret
",
                                id, id, n
                            )),
                            _ => Err("Expected int literal"),
                        },
                        _ => Err("Expected expression"),
                    },
                    _ => Err("Expected return statement"),
                },
                _ => Err("Expected statement"),
            },
            _ => Err("Expected AST::Function"),
        },
        _ => Err("Expected AST::Program"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_function() {
        let ast = ast::Node::Program(
            ast::Node::Function(
                String::from("foo"),
                ast::Node::Statement(
                    ast::Statement::Return,
                    ast::Node::Expression(ast::Expr::IntLiteral(0)).into()
                ).into()
            )
            .into(),
        );
        assert_eq!(
            generate(&ast).unwrap(),
            ".globl _foo
_foo:
  movl    $0, %eax
  ret
"
        );
    }
}
