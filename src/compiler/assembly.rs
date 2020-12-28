use super::ast;

trait Assembly {
    fn generate_assembly(&self) -> Result<String, &'static str>;
}

impl Assembly for ast::Node {
    fn generate_assembly(&self) -> Result<String, &'static str> {
        let mut code = Vec::new();
        match self {
            ast::Node::Program(node) => code.push(node.generate_assembly()?),
            ast::Node::Function(id, node) => {
                code.push(format!(".globl _{}", id));
                code.push(format!("_{}:", id));
                code.push(node.generate_assembly()?);
            }
            ast::Node::Statement(st, expr) => match st {
                ast::Statement::Return => {
                    code.push(expr.generate_assembly()?);
                    code.push(String::from("  ret"));
                }
            },
        };
        Ok(code.join("\n"))
    }
}

impl Assembly for ast::Expr {
    fn generate_assembly(&self) -> Result<String, &'static str> {
        match self {
            ast::Expr::Term(t) => t.generate_assembly(),
        }
    }
}

impl Assembly for ast::Term {
    fn generate_assembly(&self) -> Result<String, &'static str> {
        match self {
            ast::Term::Factor(f) => f.generate_assembly(),
            ast::Term::BinOp(op, t1, t2) => generate_binary_op(op, &**t1, &**t2),
        }
    }
}

impl Assembly for ast::Factor {
    fn generate_assembly(&self) -> Result<String, &'static str> {
        match self {
            // Move the integer into %eax
            ast::Factor::IntLiteral(n) => Ok(format!("  movl\t${}, %eax", n)),
            ast::Factor::UnOp(op, f) => generate_unary_op(op, f),
            ast::Factor::BinOp(op, f1, f2) => generate_binary_op(op, &**f1, &**f2),
            ast::Factor::Expr(e) => e.generate_assembly(),
        }
    }
}

fn generate_unary_op(op: &ast::UnOp, factor: &ast::Factor) -> Result<String, &'static str> {
    let mut code = Vec::new();
    code.push(factor.generate_assembly()?);
    match op {
        ast::UnOp::Negate => code.push(String::from("  neg\t%eax")),
        ast::UnOp::Complement => code.push(String::from("  not\t%eax")),
        ast::UnOp::LogicalNegate => {
            code.push(String::from("  cmpl\t$0, %eax"));
            // Zero out the eax register
            code.push(String::from("  movl\t$0, %eax"));
            code.push(String::from("  sete\t%al"));
        }
    }
    Ok(code.join("\n"))
}

fn generate_binary_op(
    op: &ast::BinOp,
    a: &impl Assembly,
    b: &impl Assembly,
) -> Result<String, &'static str> {
    let mut code = Vec::new();
    // Evaluate a
    code.push(a.generate_assembly()?);
    // Push the value in %eax on to the stack
    code.push(String::from("  push\t%rax"));
    // Evaluate b
    code.push(b.generate_assembly()?);
    // Pop a's result from the stack to the %ecx register
    code.push(String::from("  pop\t%rcx"));
    match op {
        // Add %ecx to %eax and save the result in %eax
        ast::BinOp::Add => code.push(String::from("  addl\t%ecx, %eax")),
        ast::BinOp::Subtract => {
            code.push(String::from("  subl\t%eax, %ecx"));
            code.push(String::from("  movl\t%ecx, %eax"));
        }
        ast::BinOp::Multiply => code.push(String::from("  imul\t%ecx, %eax")),
        ast::BinOp::Divide => {
            // Move b's value to %ebx
            code.push(String::from("  movl\t%eax, %ebx"));
            // Move a's value to %eax
            code.push(String::from("  movl\t%ecx, %eax"));
            // Sign extend the value in %eax
            code.push(String::from("  cdq"));
            // Divide %edx:%eax by %ebx
            code.push(String::from("  idivl\t%ebx"));
        }
    };
    Ok(code.join("\n"))
}

pub fn generate(ast: &ast::Node) -> Result<String, &'static str> {
    ast.generate_assembly()
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
