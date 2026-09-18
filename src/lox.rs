use crate::{error::LanguageError, token::scan_tokens};

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Lox {}
    }
    pub fn run(&self, line: &str, _line_number: &u64) -> Result<(), LanguageError> {
        let tokens = scan_tokens(line)?;
        for token in tokens {
            println!("{token}");
        }
        Ok(())
    }
}
