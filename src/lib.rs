mod cli;
mod environment;
mod error;
mod expression;
mod interpreter;
mod lox;
mod parser;
mod statement;
mod token;

use crate::error::CliError;

pub fn run() -> Result<(), CliError> {
    cli::run()
}
