mod cli;
mod error;
mod lox;
mod scanner;

use crate::error::CliError;

pub fn run() -> Result<(), CliError> {
    cli::run()
}
