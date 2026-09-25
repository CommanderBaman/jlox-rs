use std::{fmt::Debug, process::ExitCode};

use thiserror::Error;

use crate::{expression::Expression, token::Token};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("invalid operation {operation} on expression {expression:?}")]
    InvalidOperation {
        operation: String,
        expression: Expression,
    },
    #[error("division by zero in expression {0:?}")]
    DivisionByZero(Expression),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("primary expression is incomplete")]
    IncompletePrimaryExpression,
    #[error("wrong token for {expression_type} expression for token {token}")]
    WrongTokenForExpression {
        expression_type: &'static str,
        token: Token,
    },
    #[error("unknown error on token {0}")]
    Unknown(Token),
}

#[derive(Error, Debug)]
pub enum LanguageError {
    #[error("could not recognize token: {0}")]
    UnrecognizedToken(char),
    #[error("token was incomplete. buffer: {0}")]
    IncompleteToken(String),
    #[error("could not parse token as number: {0}")]
    UnparseableNumber(String),
    #[error("the given string was not terminated: \"{0}\"")]
    UnfinishedString(String),
    #[error(
        "the given token {base_token} was incorrectly converted to {converted_to}"
    )]
    IncorrectTokenConversion {
        base_token: Token,
        converted_to: &'static str,
    },
    #[error("parse errors:\n{}", .0.iter().fold(String::new(), |acc, e| format!("{acc}\n* {e}")).split_off(1))]
    Parse(Vec<ParseError>),
    #[error("runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("language error at line {line}: {error}")]
    Language { line: u64, error: LanguageError },
    #[error("parse error: {0}")]
    Parse(&'static str),
    #[error("path does not exist: {0}")]
    PathDoesNotExist(String),
    #[error("given path is not a file: {0}")]
    NotFile(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("runtime error at line {line}: {error}")]
    Runtime { line: u64, error: RuntimeError },
}

impl From<CliError> for ExitCode {
    fn from(value: CliError) -> Self {
        match value {
            CliError::Language { .. } => ExitCode::from(65),
            CliError::Parse(..) => ExitCode::from(64),
            CliError::PathDoesNotExist(..) => ExitCode::from(66),
            CliError::NotFile(..) => ExitCode::from(66),
            CliError::Io(..) => ExitCode::from(66),
            CliError::Runtime { .. } => ExitCode::from(70),
        }
    }
}
