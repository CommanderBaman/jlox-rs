use std::{fmt::Debug, process::ExitCode, time::SystemTimeError};

use thiserror::Error;

use crate::{
    expression::Expression, interpreter::Value, statement::Statement,
    token::Token,
};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("invalid operation {operation} on expression {expression:?}")]
    InvalidOperation {
        operation: String,
        expression: Expression,
    },
    #[error("division by zero in expression {0:?}")]
    DivisionByZero(Expression),
    #[error("variable {0} not found in environment")]
    VariableNotFound(String),
    #[error("code tried to remove root environment")]
    RootEnvironmentRemoved,
    #[error("variable {0} used without initializing")]
    UnitializedVariableUsed(String),
    #[error("value is not callable: {0}")]
    NotCallable(String),
    #[error(
        "incorrect number of arguments received for {callee_name}. Expected {expected_arguments}, Received {received_arguments}"
    )]
    IncorrectNumberOfArguments {
        callee_name: String,
        expected_arguments: usize,
        received_arguments: usize,
    },
    #[error("time error: {0}")]
    IllegalTime(#[from] SystemTimeError),
    #[error("return called outside scope")]
    Return(Value),
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
    #[error("wrong statement for {statement_type} statement via {statement}")]
    WrongStatementForConversion {
        statement_type: &'static str,
        statement: Statement,
    },
    #[error("statement does not end with semi colon: {0}")]
    UnterminatedStatement(String),
    #[error("block does not end: {0}")]
    UnterminatedBlock(String),
    #[error("received token {0:?} instead of identifier after 'var' keyword")]
    IncompleteVariableDeclaration(Option<Token>),
    #[error(
        "received expresssion {0} instead of variable expression in assignment"
    )]
    InvalidAssignment(Expression),
    #[error("unknown error on token {0}")]
    Unknown(Token),
    #[error("malformed control statement: {0}")]
    MalformedControlStatement(String),
    #[error("call expression does not end: {0}")]
    UnterminatedCall(String),
    #[error("too many arguments given in function call")]
    TooManyArguments,
    #[error("malformed function declaration: {0}")]
    MalformedFunctionDeclaration(String),
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
    #[error(
        "the given statement {base_statement} was incorrectly converted to {converted_to}"
    )]
    IncorrectStatementConversion {
        base_statement: Statement,
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
