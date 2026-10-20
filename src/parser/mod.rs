#![allow(dead_code)]

use crate::{expression::Expression, statement::Statement, token::Token};

mod error;
mod recursive_descent;

pub fn parse_expression(
    tokens: &[Token],
) -> Result<Expression, Vec<error::ParseError>> {
    recursive_descent::parse_expression(tokens)
}

pub fn parse(
    tokens: &[Token],
) -> Result<Vec<Statement>, Vec<error::ParseError>> {
    recursive_descent::parse(tokens)
}

pub type ParseError = error::ParseError;
