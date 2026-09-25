mod cli;
mod error;
mod expression;
mod lox;
mod parser;
mod token;

use crate::error::CliError;

pub fn run() -> Result<(), CliError> {
    cli::run()
}
