use super::ast;

fn stack_to_string(stack: &mut Vec<String>) -> String {
    stack.reverse();
    let mut s = stack.join("\n");
    s.push('\n');
    s
}

fn generate_return(ast: &ast::Node) -> Result<String, &'static str> {
    match ast {
        ast::Node::Expression(expr) => match expr {
            ast::Expr::IntLiteral(n) => Ok(format!("  movl    ${}, %eax", n)),
            _ => Err("Expected int literal"),
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
    code.reverse();
    Ok(code.join("\n"))
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
  movl    $0, %eax
  ret
"
        );
    }
}
