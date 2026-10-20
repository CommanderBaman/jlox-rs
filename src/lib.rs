mod class;
mod cli;
mod environment;
mod error;
mod expression;
mod function;
mod interpreter;
mod lox;
mod parser;
mod resolver;
mod statement;
mod token;
mod value;

use crate::error::CliError;

pub fn run() -> Result<(), CliError> {
    cli::run()
}
