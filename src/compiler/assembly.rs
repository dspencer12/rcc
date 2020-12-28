use super::ast;

fn stack_to_string(stack: &mut Vec<String>) -> String {
    stack.reverse();
    stack.join("\n")
}

fn generate_unary_op(op: &ast::UnOp, factor: &ast::Factor) -> Result<String, &'static str> {
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
    code.push(generate_factor(factor)?);
    Ok(stack_to_string(&mut code))
}

fn generate_factor(factor: &ast::Factor) -> Result<String, &'static str> {
    match factor {
        // Move the integer into %eax
        ast::Factor::IntLiteral(n) => Ok(format!("  movl\t${}, %eax", n)),
        ast::Factor::UnOp(op, f) => generate_unary_op(op, f),
        _ => Err("Unsupported factor"),
    }
}

fn generate_return(expr: &ast::Expr) -> Result<String, &'static str> {
    match expr {
        ast::Expr::Term(t) => match &**t {
            ast::Term::Factor(f) => generate_factor(f),
            _ => Err("Expected factor"),
        },
    }
}

fn generate_statement(
    statement: &ast::Statement,
    expr: &ast::Expr,
) -> Result<String, &'static str> {
    let mut code = Vec::new();
    match statement {
        ast::Statement::Return => {
            code.push(String::from("  ret"));
            code.push(generate_return(expr)?);
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
        ast::Node::Statement(st, expr) => code.push(generate_statement(st, expr)?),
    };
    Ok(stack_to_string(&mut code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;

    #[test]
    fn basic_function() {
        let ast = Node::Program(
            Node::Function(
                String::from("foo"),
                Node::Statement(
                    Statement::Return,
                    Expr::Term(Term::Factor(Factor::IntLiteral(0).into()).into()).into(),
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
        let ast = Node::Program(
            Node::Function(
                String::from("foo"),
                Node::Statement(
                    Statement::Return,
                    Expr::Term(
                        Term::Factor(
                            Factor::UnOp(UnOp::Negate, Factor::IntLiteral(1).into()).into(),
                        )
                        .into(),
                    )
                    .into(),
                )
                .into(),
            )
            .into(),
        )
        .into();
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
        let ast = Node::Program(
            Node::Function(
                String::from("foo"),
                Node::Statement(
                    Statement::Return,
                    Expr::Term(
                        Term::Factor(
                            Factor::UnOp(UnOp::Complement, Factor::IntLiteral(1).into()).into(),
                        )
                        .into(),
                    )
                    .into(),
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
        let ast = Node::Program(
            Node::Function(
                String::from("foo"),
                Node::Statement(
                    Statement::Return,
                    Expr::Term(
                        Term::Factor(
                            Factor::UnOp(UnOp::LogicalNegate, Factor::IntLiteral(1).into()).into(),
                        )
                        .into(),
                    )
                    .into(),
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
