use std::error::Error;
use std::fs;

mod ast;
pub mod config;
mod error;
mod lexer;
mod parser;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    println!("Starting compilation...");

    let contents = fs::read_to_string(config.filename)?;
    let tokens = lexer::tokenize(&contents)?;
    let ast = parser::parse(&tokens);

    Ok(())
}
