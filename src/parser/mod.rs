#![allow(dead_code)]

use crate::{
    error::LanguageError, expression::Expression, statement::Statement,
    token::Token,
};

mod recursive_descent;

pub fn parse_expression(
    tokens: &Vec<Token>,
) -> Result<Expression, LanguageError> {
    recursive_descent::parse_expression(&tokens)
}

pub fn parse(tokens: &Vec<Token>) -> Result<Vec<Statement>, LanguageError> {
    recursive_descent::parse(&tokens)
}
