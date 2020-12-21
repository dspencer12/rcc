use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

mod assembly;
mod ast;
pub mod config;
mod error;
mod lexer;
mod parser;

fn replace_ext(input: &PathBuf, new_ext: &str) -> PathBuf {
    let mut new_path = input.clone();
    new_path.set_extension(new_ext);
    new_path
}

fn get_temp_assembly_file(input_file: &PathBuf) -> PathBuf {
    replace_ext(input_file, "s")
}

fn get_exe_file(input_file: &PathBuf) -> PathBuf {
    replace_ext(input_file, "")
}

pub fn compile(config: &config::Config) -> Result<(), Box<dyn Error>> {
    println!("Starting compilation...");

    let contents = fs::read_to_string(&config.filename)?;
    let tokens = lexer::tokenize(&contents)?;
    let ast = parser::parse(&tokens)?;
    let code = assembly::generate(&ast)?;

    // Output assembly to a temporary file
    let output_file = get_temp_assembly_file(&config.filename);
    fs::write(&output_file, code)?;

    let exe_path = get_exe_file(&config.filename);
    let exe_file = match exe_path.to_str() {
        Some(p) => p,
        None => return Err("Failed to parse path".into())
    };

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
        let cases = [
            ("test.c", "test.s"),
            ("mydir/src.c", "mydir/src.s"),
            ("src.c.c", "src.c.s"),
            ("/my/abs/path/to/file.c", "/my/abs/path/to/file.s"),
        ];
        for (input, output) in &cases {
            assert_eq!(
                get_temp_assembly_file(&PathBuf::from(input)),
                PathBuf::from(output)
            );
        }
    }

    #[test]
    fn exe_file_names() {
        let cases = [
            ("test.c", "test"),
            ("mydir/src.c", "mydir/src"),
            ("src.c.c", "src.c"),
            ("/my/abs/path/to/file.c", "/my/abs/path/to/file"),
        ];
        for (input, output) in &cases {
            assert_eq!(
                get_exe_file(&PathBuf::from(input)),
                PathBuf::from(output)
            );
        }
    }
}
