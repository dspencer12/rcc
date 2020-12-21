use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::process::Command;

use lazy_static::lazy_static;
use regex::Regex;

mod assembly;
mod ast;
pub mod config;
mod error;
mod lexer;
mod parser;

fn get_temp_assembly_file(input_file: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\.c$").unwrap();
    }
    String::from(RE.replace(input_file, ".s"))
}

fn get_exe_file(input_file: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\.c$").unwrap();
    }
    String::from(RE.replace(input_file, ""))
}

pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    println!("Starting compilation...");

    let contents = fs::read_to_string(&config.filename)?;
    let tokens = lexer::tokenize(&contents)?;
    let ast = parser::parse(&tokens)?;
    let code = assembly::generate(&ast)?;

    // Output assembly to a temporary file
    let output_file = get_temp_assembly_file(&config.filename);
    fs::write(&output_file, code)?;

    let exe_file = get_exe_file(&config.filename);

    // Execute gcc to compile the assembly to machine code and link
    let output = Command::new("gcc")
        .arg(&output_file)
        .args(&["-o", &exe_file])
        .output()?;

    // Remove the temporary file
    match fs::remove_file(&output_file) {
        Ok(()) => (),
        // Ignore file not found error
        Err(ref e) if e.kind() == ErrorKind::NotFound => (),
        // Return other errors to the caller
        Err(e) => return Err(e.into()),
    }

    if !output.stderr.is_empty() {
        Err(String::from_utf8(output.stderr).unwrap().into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembly_file_names() {
        assert_eq!(get_temp_assembly_file("test.c"), "test.s");
        assert_eq!(get_temp_assembly_file("mydir/src.c"), "mydir/src.s");
        assert_eq!(get_temp_assembly_file("src.c.c"), "src.c.s");
        assert_eq!(
            get_temp_assembly_file("/my/abs/path/to/file.c"),
            "/my/abs/path/to/file.s"
        );
    }

    #[test]
    fn exe_file_names() {
        assert_eq!(get_exe_file("test.c"), "test");
        assert_eq!(get_exe_file("mydir/src.c"), "mydir/src");
        assert_eq!(get_exe_file("src.c.c"), "src.c");
        assert_eq!(
            get_exe_file("/my/abs/path/to/file.c"),
            "/my/abs/path/to/file"
        );
    }
}
