#![allow(unused_variables)]
use std::{iter::Peekable, slice::Iter};

use crate::{
    error::{LanguageError, ParseError},
    expression::{
        BinaryExpression, Expression, GroupingExpression, LiteralExpression,
        UnaryExpression,
    },
    token::Token,
};

// grammar
// expression → equality
// equality   → comparison ( ( "!=" | "==" ) comparison )*
// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
// term       → factor ( ( "-" | "+" ) factor )*
// factor     → unary ( ( "/" | "*" ) unary )*
// unary      → ( "!" | "-" ) unary | primary
// primary    → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")"
pub(super) fn parse_expression(
    tokens: &Vec<Token>,
) -> Result<Expression, LanguageError> {
    let mut tokens = tokens.iter().peekable();
    let mut errors = Vec::new();
    // add a dummy expression for rust checks
    let mut expr: Expression = LiteralExpression::new(Token::EndOfFile)
        .expect("eof is a literal expression");
    while let Some(token) = tokens.peek() {
        match expression(&mut tokens) {
            Ok(ex) => expr = ex,
            Err(err) => {
                errors.push(err);
                // consume tokens till a clear end
                while let Some(token) = tokens.peek() {
                    match token {
                        Token::Class
                        | Token::Fun
                        | Token::Var
                        | Token::For
                        | Token::If
                        | Token::While
                        | Token::Print
                        | Token::Return => {
                            break;
                        }
                        _ => {
                            _ = tokens.next();
                        }
                    }
                }
                continue;
            }
        }
    }
    if errors.is_empty() {
        Ok(expr)
    } else {
        Err(LanguageError::Parse(errors))
    }
}

// expression → equality
fn expression(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    equality(tokens)
}

// equality   → comparison ( ( "!=" | "==" ) comparison )*
fn equality(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expression = comparison(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::BangEqual | Token::EqualEqual => {
                let left_expression = expression;
                let operator = tokens
                    .next()
                    .expect("no token after peek check - equality");
                let right_expression = comparison(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
fn comparison(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expression = term(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual => {
                let left_expression = expression;
                let operator = tokens
                    .next()
                    .expect("no token after peek check - comparison");
                let right_expression = term(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// term → factor ( ( "-" | "+" ) factor )*
fn term(tokens: &mut Peekable<Iter<Token>>) -> Result<Expression, ParseError> {
    let mut expression = factor(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Minus | Token::Plus => {
                let left_expression = expression;
                let operator =
                    tokens.next().expect("no token after peek check - term");
                let right_expression = factor(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// factor → unary ( ( "/" | "*" ) unary )*
fn factor(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let mut expression = unary(tokens)?;
    while let Some(token) = tokens.peek() {
        match token {
            Token::Slash | Token::Star => {
                let left_expression = expression;
                let operator =
                    tokens.next().expect("no token after peek check - factor");
                let right_expression = unary(tokens)?;
                expression =
                    incorrect_token_error_wrap(BinaryExpression::new(
                        left_expression,
                        operator.to_owned(),
                        right_expression,
                    ))?;
            }
            _ => break,
        }
    }
    Ok(expression)
}

// unary → ( "!" | "-" ) unary | primary
fn unary(tokens: &mut Peekable<Iter<Token>>) -> Result<Expression, ParseError> {
    match tokens.peek() {
        Some(Token::Bang) | Some(Token::Minus) => {
            let token =
                tokens.next().expect("no token after peek check - unary");
            incorrect_token_error_wrap(UnaryExpression::new(
                token.to_owned(),
                unary(tokens)?,
            ))
        }
        _ => primary(tokens),
    }
}

// primary → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")"
fn primary(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expression, ParseError> {
    let Some(token) = tokens.next() else {
        return Err(ParseError::IncompletePrimaryExpression);
    };
    match token {
        Token::False => LiteralExpression::new(token.to_owned()).map_err(|e| {
            ParseError::WrongTokenForExpression {
                expression_type: "literal",
                token: token.clone(),
            }
        }),
        Token::True => {
            incorrect_token_error_wrap(LiteralExpression::new(token.to_owned()))
        }
        Token::Nil => {
            incorrect_token_error_wrap(LiteralExpression::new(token.to_owned()))
        }
        Token::Number(_) => {
            incorrect_token_error_wrap(LiteralExpression::new(token.to_owned()))
        }
        Token::String(_) => {
            incorrect_token_error_wrap(LiteralExpression::new(token.to_owned()))
        }
        Token::LeftParen => {
            let expression = expression(tokens)?;
            // should end with right parenthesis
            if !matches!(tokens.next(), Some(Token::RightParen)) {
                return Err(ParseError::IncompletePrimaryExpression);
            }
            Ok(GroupingExpression::new(expression))
        }
        _ => Err(ParseError::Unknown(token.to_owned())),
    }
}

fn incorrect_token_error_wrap(
    expression: Result<Expression, LanguageError>,
) -> Result<Expression, ParseError> {
    expression.map_err(|e| match e {
        LanguageError::IncorrectTokenConversion {
            base_token,
            converted_to,
        } => ParseError::WrongTokenForExpression {
            expression_type: converted_to,
            token: base_token,
        },
        _ => {
            unreachable!("found not handled error during expression wrap: {e}")
        }
    })
}

// solution to challenge 1
// I thought of three solutions
// 1. changing primary to
// primary → ... | "nil" | "(" expression ")" | "," expression
// but this allows expressions like ", 1 + 2"
// 2. changing equality to
// equality → comparison ( ( "!=" | "==" | "," ) comparison )*
// but this causes problems with cases like "x, y == z"
// not to mention we need to add , as a binary operator which might
// cause problems with evaluation later on - just an intuition
// 3. changing expression to
// expression -> equality ( "," equality )
// but in order to capture this we need to change the return values here
// I don't want to disturb the current type system so. otherwise,
// we have to return a vector everywhere instead of just expression

// solution to challenge 2
// ternary operator is right associative. ex,
// check1 ? true1 : check2 ? true2 : false2 boils down to
// check1 ? true1 : (check2 ? true2 : false2)
// as for precedence, it comes between equality and comparison

// solution to challenge 3
// adding error productions for binary operator is easy, just add them
// at the equality step accepting the operators
// problem is that with this typing, we can't form statements with them
// that is why i didn't add error production for + at unary
// for such cases i think having a very verbose parse error helps
