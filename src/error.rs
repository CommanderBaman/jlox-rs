use std::{fmt::Debug, process::ExitCode, time::SystemTimeError};

use thiserror::Error;

use crate::{
    expression::{Expression, VariableToken},
    statement::Statement,
    token::Token,
    value::{Value, ValueError},
};

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("given variable is already declared: {0}")]
    AlreadyDeclared(VariableToken),
    #[error("given variable is already defined: {0}")]
    AlreadyDefined(VariableToken),
    #[error("can not read local variable {0} before its initializer")]
    UsageBeforeDeclaration(VariableToken),
    #[error("can not use local variable {0} before its initializer")]
    UsageBeforeDefinition(VariableToken),
    #[error("given variable {0} is not declared")]
    NotDeclared(VariableToken),
    #[error("return called from top level of code")]
    TopLevelReturn,
    #[error("unused variables: {}", .0.iter().fold(String::new(), |acc, e| format!("{acc}, {e}")).split_off(2))]
    UnusedVariables(Vec<VariableToken>),
    #[error("unknown: {0}")]
    Unknown(String),
    #[error("invalid usage of this")]
    InvalidThis,
    #[error("invalid return from initializer")]
    InvalidReturnFromInitializer,
    #[error("self inheritance is not allowed: {0}")]
    SelfInheritanceNotAllowed(VariableToken),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("error operating on values of expression {expression}")]
    ValueError {
        expression: Expression,
        source: ValueError,
    },
    #[error("variable {0} not found in environment")]
    VariableNotFound(String),
    #[error("code tried to remove root environment")]
    RootEnvironmentRemoved,
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
    #[error("return called from somewhere unknown")]
    Return(Value),
    #[error("no properties on types other than instance: {0}")]
    NoProperties(Value),
    #[error("no property named {0} on instance: {1}")]
    PropertyNotFound(VariableToken, String),
    #[error("can not set properties on instance: {0}")]
    NotSettable(String),
    #[error("environment not found")]
    EnvironmentNotFound,
    #[error("expected class value for superclass for class {0}")]
    InvalidSuperClass(VariableToken),
    #[error("invalid super state: {0}")]
    InvalidSuperState(String),
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
        "received expression {0} instead of variable expression in assignment"
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
    #[error("malformed declaration: {0}")]
    MalformedDeclaration(String),
    #[error("get expression does not contain identifier: {0}")]
    MalformedGet(String),
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
}

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("could not recognize token: {0}")]
    UnrecognizedToken(char),
    #[error("token was incomplete. buffer: {0}")]
    IncompleteToken(String),
    #[error("could not parse token as number: {0}")]
    UnparsableNumber(String),
    #[error("the given string was not terminated: \"{0}\"")]
    UnfinishedString(String),
}

#[derive(Error, Debug)]
pub enum LanguageError {
    #[error("parse errors:\n{}", .0.iter().fold(String::new(), |acc, e| format!("{acc}\n* {e}")).split_off(1))]
    Parse(Vec<ParseError>),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Resolver(#[from] ResolverError),
    #[error(transparent)]
    Token(#[from] TokenError),
}

impl From<LanguageError> for ExitCode {
    fn from(value: LanguageError) -> Self {
        let error_code = match value {
            LanguageError::Token(_)
            | LanguageError::Parse(_)
            | LanguageError::Resolver(_) => 65,
            LanguageError::Runtime(_) => 70,
        };
        ExitCode::from(error_code)
    }
}

#[derive(Error, Debug)]
pub enum IoError {
    #[error(transparent)]
    BaseIo(#[from] std::io::Error),
    #[error("too many arguments provided. need at most one. received {0}")]
    TooManyArguments(usize),
    #[error("path does not exist: {0}")]
    PathDoesNotExist(String),
    #[error("given path is not a file: {0}")]
    NotFile(String),
}

impl From<IoError> for ExitCode {
    fn from(value: IoError) -> Self {
        let error_code = match value {
            IoError::BaseIo(_)
            | IoError::NotFile(_)
            | IoError::PathDoesNotExist(_) => 66,
            IoError::TooManyArguments(_) => 64,
        };
        ExitCode::from(error_code)
    }
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    Language(#[from] LanguageError),
    #[error(transparent)]
    Io(#[from] IoError),
}

impl From<CliError> for ExitCode {
    fn from(value: CliError) -> Self {
        match value {
            CliError::Language(l) => l.into(),
            CliError::Io(i) => i.into(),
        }
    }
}
