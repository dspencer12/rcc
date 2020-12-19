use std::fs;
use std::error::Error;

pub mod config;
mod lexer;

pub fn run(config: config::Config) -> Result<(), Box<dyn Error>> {
    println!("Starting compilation...");

    let contents = fs::read_to_string(config.filename)?;
    let tokens = lexer::tokenize(&contents);

    Ok(())
}