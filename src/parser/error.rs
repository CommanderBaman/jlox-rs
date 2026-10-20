use thiserror::Error;

use crate::{expression::Expression, statement::Statement, token::Token};

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
