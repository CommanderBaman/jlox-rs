use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
};

use strum::Display;

use crate::{
    environment::Environment,
    error::RuntimeError,
    expression::{
        AssignmentExpression, BinaryExpression, BinaryOperator, Expression,
        ExpressionVisitor, GroupingExpression, LiteralExpression, LiteralToken,
        UnaryExpression, UnaryOperator, VariableExpression,
    },
    statement::{
        BlockStatement, ExpressionStatement, PrintStatement, Statement,
        StatementVisitor, VariableDeclarationStatement,
    },
    token::Token,
};

#[derive(Display, PartialEq, PartialOrd, Clone)]
pub enum Value {
    #[strum(to_string = "{0}")]
    Bool(bool),
    #[strum(to_string = "nil")]
    Nil,
    #[strum(to_string = "{0}")]
    Number(f64),
    #[strum(to_string = "{0}")]
    String(String),
    #[strum(to_string = "uninitialized")]
    Unintialized,
}

impl Value {
    fn is_truthy(&self) -> bool {
        !matches!(self, Value::Unintialized | Value::Nil | Value::Bool(false))
    }
}

// TODO: research why can't we derive Eq?
impl Eq for Value {}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => l.cmp(r),
            // NOTE: this total_cmp is different in behavior from partial_cmp
            (Value::Number(l), Value::Number(r)) => l.total_cmp(r),
            (Value::String(l), Value::String(r)) => l.cmp(r),
            // NOTE: solution to challenge 1
            //
            // I can easily bring conversions between numbers and strings
            // and same for bool
            // (Value::Number(l), Value::String(r)) => l.to_string().cmp(r),
            // (Value::String(l), Value::Number(r)) => l.cmp(&r.to_string()),
            // But I would not introduce these
            // because it brings confusion in the language
            // ex, "02" < 1 because "0" < "1"
            // Python also gives us type error when we do int vs str comparison
            (Value::Nil, Value::Nil) => Ordering::Equal,
            _ => unimplemented!(),
        }
    }
}

impl TryFrom<Value> for LiteralExpression {
    type Error = RuntimeError;
    fn try_from(value: Value) -> Result<Self, RuntimeError> {
        match value {
            Value::String(s) => Ok(LiteralExpression {
                literal: LiteralToken::String(s),
            }),
            Value::Bool(b) => {
                let literal = if b {
                    LiteralToken::True
                } else {
                    LiteralToken::False
                };
                Ok(LiteralExpression { literal })
            }
            Value::Number(n) => Ok(LiteralExpression {
                literal: LiteralToken::Number(n),
            }),
            Value::Nil => Ok(LiteralExpression {
                literal: LiteralToken::Nil,
            }),
            Value::Unintialized => {
                Err(RuntimeError::UnitializedVariableUsed("?".to_owned()))
            }
        }
    }
}

fn get_binary_expression(
    left: Value,
    right: Value,
    operator: Token,
) -> Expression {
    BinaryExpression::new(
        Expression::Literal(left.try_into().expect("left value to be initialized")),
        operator,
        Expression::Literal(right.try_into().expect("right value to be initialized")),
    )
    .expect("value to literal conversion failed or invalid operator passed for building binary expression for error")
}

impl Add for Value {
    type Output = Result<Value, RuntimeError>;
    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // (Value::Number(l), Value::Number(r)) => {}
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
            // NOTE: solution to challenge 2
            (Value::String(l), Value::String(r)) => {
                Ok(Value::String(format!("{l}{r}")))
            }
            (Value::String(l), Value::Number(r)) => {
                Ok(Value::String(format!("{l}{r}")))
            }
            (Value::Number(l), Value::String(r)) => {
                Ok(Value::String(format!("{l}{r}")))
            }
            (Value::Unintialized, _) | (_, Value::Unintialized) => {
                Err(RuntimeError::UnitializedVariableUsed("?".to_owned()))
            }
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
            (Value::Unintialized, _) | (_, Value::Unintialized) => {
                Err(RuntimeError::UnitializedVariableUsed("?".to_owned()))
            }
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
            (Value::Unintialized, _) | (_, Value::Unintialized) => {
                Err(RuntimeError::UnitializedVariableUsed("?".to_owned()))
            }
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
            (Value::Number(l), Value::Number(r)) => {
                // NOTE: solution to challenge 3
                //
                // previously when divided by zero, due to rust internals,
                // the number becamse infinite
                // now we will give a runtime error
                if r.eq(&0.0) {
                    return Err(RuntimeError::DivisionByZero(
                        get_binary_expression(self, rhs, Token::Slash),
                    ));
                }
                Ok(Value::Number(l / r))
            }
            (Value::Unintialized, _) | (_, Value::Unintialized) => {
                Err(RuntimeError::UnitializedVariableUsed("?".to_owned()))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Div".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Slash),
            }),
        }
    }
}

pub struct Interpreter {
    environment: Environment,
    print_expression: bool,
}

impl Interpreter {
    pub fn new(print_expression: bool) -> Self {
        Interpreter {
            environment: Environment::new(),
            print_expression,
        }
    }
    pub fn evaluate(
        &mut self,
        expression: &Expression,
    ) -> Result<Value, RuntimeError> {
        expression.accept(self)
    }
    pub fn interpret(
        &mut self,
        statements: &Vec<Statement>,
    ) -> Result<(), RuntimeError> {
        let mut last_value = Value::Nil;
        for statement in statements {
            last_value = self.execute(statement)?;
        }
        if self.print_expression
            && let Some(Statement::Expression(statement)) = statements.last()
        {
            // could have used matches! macro
            match statement.expression {
                Expression::Assignment(_) => {}
                _ => {
                    println!("{last_value}");
                }
            }
        };
        Ok(())
    }
    fn execute(
        &mut self,
        statement: &Statement,
    ) -> Result<Value, RuntimeError> {
        statement.accept(self)
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
        &mut self,
        expression: &GroupingExpression,
    ) -> Result<Value, RuntimeError> {
        self.evaluate(expression.expression.as_ref())
    }
    fn visit_unary(
        &mut self,
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
        &mut self,
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
    fn visit_variable(
        &self,
        expression: &VariableExpression,
    ) -> Result<Value, RuntimeError> {
        self.environment
            .get(&expression.variable)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::VariableNotFound(expression.variable.to_string())
            })
    }
    fn visit_assignment(
        &mut self,
        expression: &AssignmentExpression,
    ) -> Result<Value, RuntimeError> {
        let value = self.evaluate(&expression.expression)?;
        self.environment
            .assign(&expression.variable, value.clone())?;
        Ok(value)
    }
}

impl StatementVisitor<Result<Value, RuntimeError>> for Interpreter {
    fn visit_print(
        &mut self,
        statement: &PrintStatement,
    ) -> Result<Value, RuntimeError> {
        let value = self.evaluate(&statement.expression)?;
        println!("{value}");
        Ok(value)
    }
    fn visit_expression(
        &mut self,
        statement: &ExpressionStatement,
    ) -> Result<Value, RuntimeError> {
        self.evaluate(&statement.expression)
    }
    fn visit_variable_declaration(
        &mut self,
        statement: &VariableDeclarationStatement,
    ) -> Result<Value, RuntimeError> {
        let value = statement
            .expression
            .as_ref()
            .map(|ex| self.evaluate(ex))
            .transpose()?
            .unwrap_or(Value::Unintialized);
        self.environment
            .define(statement.name.clone(), value.clone());
        Ok(value)
    }
    fn visit_block(
        &mut self,
        statement: &BlockStatement,
    ) -> Result<Value, RuntimeError> {
        self.environment.push_child();
        for statement in &statement.statements {
            match self.execute(&statement) {
                Ok(_) => {}
                Err(e) => {
                    // how much I wish there was a defer
                    self.environment.pop_child()?;
                    return Err(e);
                }
            }
        }
        self.environment.pop_child()?;
        Ok(Value::Nil)
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
