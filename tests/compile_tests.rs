use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::NamedTempFile;

extern crate rcc;
use rcc::compiler::{self, config::Config};

const VALID_TEST_DIR: &str = "tests/testfiles/valid";

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
    many_newlines: "many_newlines.c",
    minimal_whitespace: "minimal_whitespace.c",
    multi_digit: "multi_digit.c",
    return_0: "return_0.c",
    return_2: "return_2.c",
}
