use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::NamedTempFile;

extern crate rcc;
use rcc::compiler::{self, config::Config, error::SyntaxError};

const VALID_TEST_DIR: &str = "tests/testfiles/valid";
const INVALID_TEST_DIR: &str = "tests/testfiles/invalid";

fn execute(file: &Path) -> io::Result<Output> {
    Command::new(file.to_str().expect("Failed to convert path to string")).output()
}

fn compile_and_execute_gcc(file: &PathBuf) -> io::Result<Output> {
    let temp_file = NamedTempFile::new()?;
    Command::new("gcc")
        .arg(file.to_str().expect("Failed to convert path to string"))
        .args(&["-o", &temp_file.path().to_str().expect("Bad path")])
        .output()?;
    let output = execute(temp_file.path());
    fs::remove_file(temp_file)?;
    output
}

macro_rules! file_compilation_tests {
    ($($name:ident: $test_file:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let mut path = PathBuf::from(VALID_TEST_DIR);
                path.push($test_file);
                let config = Config{ filename: path.clone() };

                compiler::compile(&config).expect("Compilation failed");

                let mut exe_path = PathBuf::from(&path);
                exe_path.set_extension("");
                let actual = execute(&exe_path).expect("Failed to execute rcc exe");
                fs::remove_file(exe_path.to_str().expect("Bad path")).expect("Failed to remove exe file");

                let expected = compile_and_execute_gcc(&path).expect("Failed to execute gcc exe");

                assert_eq!(actual, expected);
            }
        )*
    }
}

file_compilation_tests! {
    abundant_spaces: "abundant_spaces.c",
    add: "add.c",
    associativity_div: "associativity_div.c",
    associativity: "associativity.c",
    bitwise_zero: "bitwise_zero.c",
    bitwise: "bitwise.c",
    div_neg: "div_neg.c",
    div: "div.c",
    many_newlines: "many_newlines.c",
    minimal_whitespace: "minimal_whitespace.c",
    mult: "mult.c",
    multi_digit: "multi_digit.c",
    neg: "neg.c",
    nested_ops_2: "nested_ops_2.c",
    nested_ops: "nested_ops.c",
    not_0: "not_0.c",
    not_5: "not_5.c",
    parens: "parens.c",
    precedence: "precedence.c",
    return_0: "return_0.c",
    return_2: "return_2.c",
    sub_neg: "sub_neg.c",
    sub: "sub.c",
    unop_add: "unop_add.c",
    unop_parens: "unop_parens.c",
}

macro_rules! assert_raises_syntax_error {
    ($left:expr, $err:expr) => {
        assert_eq!(
            *$left.err().unwrap().downcast::<SyntaxError>().unwrap(),
            $err
        );
    };
}

macro_rules! file_error_tests {
    ($($name:ident: ($test_file:expr, $err:expr),)*) => {
        $(
            #[test]
            fn $name() {
                let mut path = PathBuf::from(INVALID_TEST_DIR);
                path.push($test_file);
                let config = Config{ filename: path.clone() };

                assert_raises_syntax_error!(
                    compiler::compile(&config),
                    $err
                );
            }
        )*
    }
}

file_error_tests! {
    malformed_paren: ("malformed_paren.c", SyntaxError::MissingSemicolon),
    missing_closing_brace: ("missing_closing_brace.c", SyntaxError::MissingCloseBrace),
    missing_const: ("missing_const.c", SyntaxError::InvalidFactor),
    missing_first_op: ("missing_first_op.c", SyntaxError::InvalidFactor),
    missing_paren: ("missing_paren.c", SyntaxError::MissingCloseParen),
    missing_return_space: ("missing_return_space.c", SyntaxError::UnexpectedToken),
    missing_return_val: ("missing_return_val.c", SyntaxError::InvalidFactor),
    missing_second_op: ("missing_second_op.c", SyntaxError::InvalidFactor),
    missing_semicolon: ("missing_semicolon.c", SyntaxError::MissingSemicolon),
    missing_semicolon_2: ("missing_semicolon_2.c", SyntaxError::MissingSemicolon),
    nested_missing_const: ("nested_missing_const.c", SyntaxError::InvalidFactor),
    no_semicolon: ("no_semicolon.c", SyntaxError::MissingSemicolon),
    wrong_return_case: ("wrong_return_case.c", SyntaxError::UnexpectedToken),
    wrong_unary_order: ("wrong_unary_order.c", SyntaxError::InvalidFactor),
}
