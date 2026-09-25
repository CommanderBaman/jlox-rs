use crate::{error::LanguageError, expression::Expression, token::Token};

mod recursive_descent;

pub fn parse_expression(
    tokens: &Vec<Token>,
) -> Result<Expression, LanguageError> {
    recursive_descent::parse_expression(&tokens)
}
