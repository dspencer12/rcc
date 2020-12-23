use super::ast;

fn stack_to_string(stack: &mut Vec<String>) -> String {
    stack.reverse();
    stack.join("\n")
}

fn generate_unary_op(op: &ast::UnOp, ast: &ast::Node) -> Result<String, &'static str> {
    let mut code = Vec::new();
    match op {
        ast::UnOp::Negate => code.push(String::from("  neg\t%eax")),
        ast::UnOp::Complement => code.push(String::from("  not\t%eax")),
        ast::UnOp::LogicalNegate => {
            code.push(String::from("  sete\t%al"));
            // Zero out the eax register
            code.push(String::from("  movl\t$0, %eax"));
            code.push(String::from("  cmpl\t$0, %eax"));
        }
    }
    code.push(generate_return(ast)?);
    Ok(stack_to_string(&mut code))
}

fn generate_return(ast: &ast::Node) -> Result<String, &'static str> {
    match ast {
        ast::Node::Expression(expr) => match expr {
            ast::Expr::IntLiteral(n) => Ok(format!("  movl\t${}, %eax", n)),
            ast::Expr::UnOp(op, node) => generate_unary_op(op, node),
        }
        _ => Err("Expected expression")
    }
}

fn generate_statement(statement: &ast::Statement, ast: &ast::Node) -> Result<String, &'static str> {
    let mut code = Vec::new();
    match statement {
        ast::Statement::Return => {
            code.push(String::from("  ret"));
            code.push(generate_return(ast)?);
        }
    }
    Ok(stack_to_string(&mut code))
}

pub fn generate(ast: &ast::Node) -> Result<String, &'static str> {
    let mut code = Vec::new();
    match ast {
        ast::Node::Program(node) => code.push(generate(node)?),
        ast::Node::Function(id, node) => {
            code.push(generate(node)?);
            code.push(format!("_{}:", id));
            code.push(format!(".globl _{}", id));
        }
        ast::Node::Statement(st, node) => code.push(generate_statement(st, node)?),
        ast::Node::Expression(_) => return Err("Unexpected expression"),
    };
    Ok(stack_to_string(&mut code))
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
                    ast::Node::Expression(ast::Expr::IntLiteral(0)).into(),
                )
                .into(),
            )
            .into(),
        );
        assert_eq!(
            generate(&ast).unwrap(),
            ".globl _foo
_foo:
  movl\t$0, %eax
  ret"
        );
    }

    #[test]
    fn function_return_negate_1() {
        let ast = ast::Node::Program(
            ast::Node::Function(
                String::from("foo"),
                ast::Node::Statement(
                    ast::Statement::Return,
                    ast::Node::Expression(
                        ast::Expr::UnOp(
                            ast::UnOp::Negate,
                            ast::Node::Expression(ast::Expr::IntLiteral(1)).into()
                        )
                    ).into()
                )
                .into(),
            )
            .into(),
        );
        assert_eq!(
            generate(&ast).unwrap(),
            ".globl _foo
_foo:
  movl\t$1, %eax
  neg\t%eax
  ret"
        );
    }

    #[test]
    fn function_return_complement_1() {
        let ast = ast::Node::Program(
            ast::Node::Function(
                String::from("foo"),
                ast::Node::Statement(
                    ast::Statement::Return,
                    ast::Node::Expression(
                        ast::Expr::UnOp(
                            ast::UnOp::Complement,
                            ast::Node::Expression(ast::Expr::IntLiteral(1)).into()
                        )
                    ).into()
                )
                .into(),
            )
            .into(),
        );
        assert_eq!(
            generate(&ast).unwrap(),
            ".globl _foo
_foo:
  movl\t$1, %eax
  not\t%eax
  ret"
        );
    }

    #[test]
    fn function_return_logical_negate_1() {
        let ast = ast::Node::Program(
            ast::Node::Function(
                String::from("foo"),
                ast::Node::Statement(
                    ast::Statement::Return,
                    ast::Node::Expression(
                        ast::Expr::UnOp(
                            ast::UnOp::LogicalNegate,
                            ast::Node::Expression(ast::Expr::IntLiteral(1)).into()
                        )
                    ).into()
                )
                .into(),
            )
            .into(),
        );
        assert_eq!(
            generate(&ast).unwrap(),
            ".globl _foo
_foo:
  movl\t$1, %eax
  cmpl\t$0, %eax
  movl\t$0, %eax
  sete\t%al
  ret"
        );
    }
}
