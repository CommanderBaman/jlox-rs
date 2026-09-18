use std::process::ExitCode;

use thiserror::Error;

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
}

impl From<CliError> for ExitCode {
    fn from(value: CliError) -> Self {
        match value {
            CliError::Language { .. } => ExitCode::from(65),
            CliError::Parse(..) => ExitCode::from(64),
            CliError::PathDoesNotExist(..) => ExitCode::from(66),
            CliError::NotFile(..) => ExitCode::from(66),
            CliError::Io(..) => ExitCode::from(66),
        }
    }
}
