mod cli;
mod error;
mod lox;
mod token;

use crate::error::CliError;

pub fn run() -> Result<(), CliError> {
    cli::run()
}
