use super::ast::{ASTExpression, ASTStatement, AST};

pub fn generate(ast: &AST) -> Result<String, &'static str> {
    match ast {
        AST::Program(node) => match &**node {
            AST::Function(id, statement) => match statement {
                ASTStatement::Return(e) => match e {
                    ASTExpression::Literal(n) => Ok(format!(
                        ".globl _{}
_{}:
  movl    ${}, %eax
  ret
",
                        id, id, n
                    )),
                },
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
        let ast = AST::Program(
            AST::Function(
                String::from("foo"),
                ASTStatement::Return(ASTExpression::Literal(0)),
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
