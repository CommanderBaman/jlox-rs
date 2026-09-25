use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
};

use strum::Display;

use crate::{
    error::RuntimeError,
    expression::{
        BinaryExpression, BinaryOperator, Expression, ExpressionVisitor,
        GroupingExpression, LiteralExpression, LiteralToken, UnaryExpression,
        UnaryOperator,
    },
    token::Token,
};

#[derive(Display, PartialEq, PartialOrd)]
pub enum Value {
    #[strum(to_string = "{0}")]
    Bool(bool),
    #[strum(to_string = "nil")]
    Nil,
    #[strum(to_string = "{0}")]
    Number(f64),
    #[strum(to_string = "{0}")]
    String(String),
}

impl Value {
    fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

// TODO: read why can't we derive Eq?
impl Eq for Value {}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => l.cmp(r),
            // NOTE: this total_cmp is different in behavior from partial_cmp
            (Value::Number(l), Value::Number(r)) => l.total_cmp(r),
            (Value::String(l), Value::String(r)) => l.cmp(r),
            (Value::Nil, Value::Nil) => Ordering::Equal,
            _ => unimplemented!(),
        }
    }
}

impl From<Value> for LiteralExpression {
    fn from(value: Value) -> Self {
        match value {
            Value::String(s) => LiteralExpression {
                literal: LiteralToken::String(s),
            },
            Value::Bool(b) => {
                let literal = if b {
                    LiteralToken::True
                } else {
                    LiteralToken::False
                };
                LiteralExpression { literal }
            }
            Value::Number(n) => LiteralExpression {
                literal: LiteralToken::Number(n),
            },
            Value::Nil => LiteralExpression {
                literal: LiteralToken::Nil,
            },
        }
    }
}

fn get_binary_expression(
    left: Value,
    right: Value,
    operator: Token,
) -> Expression {
    BinaryExpression::new(
        Expression::Literal(left.into()),
        operator,
        Expression::Literal(right.into()),
    )
    .expect("value to literal conversion failed or invalid operator passed for building binary expression for error")
}

impl Add for Value {
    type Output = Result<Value, RuntimeError>;
    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // (Value::Number(l), Value::Number(r)) => {}
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Add".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Plus),
            }),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, RuntimeError>;
    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Sub".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Minus),
            }),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, RuntimeError>;
    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Mul".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Star),
            }),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, RuntimeError>;
    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Div".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Slash),
            }),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }
    pub fn evaluate(
        &self,
        expression: &Expression,
    ) -> Result<Value, RuntimeError> {
        expression.accept(self)
    }
}

impl ExpressionVisitor<Result<Value, RuntimeError>> for Interpreter {
    fn visit_literal(
        &self,
        expression: &LiteralExpression,
    ) -> Result<Value, RuntimeError> {
        match expression.literal {
            LiteralToken::Nil => Ok(Value::Nil),
            LiteralToken::Number(n) => Ok(Value::Number(n)),
            LiteralToken::String(ref s) => Ok(Value::String(s.to_owned())),
            LiteralToken::True => Ok(Value::Bool(true)),
            LiteralToken::False => Ok(Value::Bool(false)),
        }
    }
    fn visit_grouping(
        &self,
        expression: &GroupingExpression,
    ) -> Result<Value, RuntimeError> {
        self.evaluate(expression.expression.as_ref())
    }
    fn visit_unary(
        &self,
        expression: &UnaryExpression,
    ) -> Result<Value, RuntimeError> {
        let mut value = self.evaluate(&expression.expression)?;
        value = match expression.operator {
            UnaryOperator::Bang => Value::Bool(!value.is_truthy()),
            UnaryOperator::Minus => {
                (Value::Number(-1.0) * value).map_err(|_| {
                    RuntimeError::InvalidOperation {
                        operation: UnaryOperator::Minus.to_string(),
                        expression: expression.clone().wrap(),
                    }
                })?
            }
        };
        Ok(value)
    }
    fn visit_binary(
        &self,
        expression: &BinaryExpression,
    ) -> Result<Value, RuntimeError> {
        let left_value = self.evaluate(&expression.left_expression)?;
        let right_value = self.evaluate(&expression.right_expression)?;

        match expression.operator {
            BinaryOperator::EqualEqual => {
                Ok(Value::Bool(left_value.eq(&right_value)))
            }
            BinaryOperator::BangEqual => {
                Ok(Value::Bool(left_value.ne(&right_value)))
            }
            BinaryOperator::Less => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_lt()))
            }
            BinaryOperator::LessEqual => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_le()))
            }
            BinaryOperator::Greater => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_gt()))
            }
            BinaryOperator::GreaterEqual => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_ge()))
            }
            BinaryOperator::Plus => left_value + right_value,
            BinaryOperator::Minus => left_value - right_value,
            BinaryOperator::Star => left_value * right_value,
            BinaryOperator::Slash => left_value / right_value,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn value_is_truthy() {
        let cases = vec![
            (Value::Nil, false),
            (Value::Bool(false), false),
            (Value::Bool(true), true),
            (Value::Number(0.0), true),
            (Value::Number(1.0), true),
            (Value::Number(-1.0), true),
            (Value::Number(f64::NAN), true),
            (Value::String("lkj".to_owned()), true),
        ];

        for (value, expected) in cases {
            assert_eq!(value.is_truthy(), expected)
        }
    }
}
