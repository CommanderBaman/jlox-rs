#![allow(dead_code)]

use crate::{
    error::LanguageError, expression::Expression, statement::Statement,
    token::Token,
};

mod recursive_descent;

pub fn parse_expression(tokens: &[Token]) -> Result<Expression, LanguageError> {
    recursive_descent::parse_expression(tokens)
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Statement>, LanguageError> {
    recursive_descent::parse(tokens)
}
