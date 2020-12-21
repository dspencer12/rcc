use std::env;
use std::path::PathBuf;

pub struct Config {
    pub filename: PathBuf,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("No file path provided"),
        };

        Ok(Config {
            filename: PathBuf::from(filename),
        })
    }
}
