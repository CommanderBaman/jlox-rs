use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use thiserror::Error;

use crate::{
    class::{LoxClass, LoxInstance},
    expression::{BinaryOperator, UnaryOperator},
    function::LoxCallable,
};

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
    Uninitialized,
    Call(Rc<dyn LoxCallable>),
    ClassDeclaration(Rc<LoxClass>),
    ClassInstance(Rc<LoxInstance>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Bool(b) => format!("{b}"),
            Self::Nil => "nil".to_string(),
            Self::Number(n) => format!("{n}"),
            Self::String(s) => s.to_owned(),
            Self::Uninitialized => "uinitialized".to_string(),
            Self::Call(c) => format!("{c}"),
            Self::ClassDeclaration(c) => format!("{c}"),
            Self::ClassInstance(c) => format!("{c}"),
        };
        write!(f, "{}", string)
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Uninitialized | Value::Nil | Value::Bool(false))
    }

    pub fn do_binary_operation(
        self,
        right: Self,
        operator: BinaryOperator,
    ) -> Result<Self, ValueError> {
        match operator {
            BinaryOperator::EqualEqual => Ok(Self::Bool(self.eq(&right))),
            BinaryOperator::BangEqual => Ok(Self::Bool(self.ne(&right))),
            BinaryOperator::Less => Ok(Self::Bool(
                self.partial_cmp(&right)
                    .map(|cmp| cmp.is_lt())
                    .ok_or_else(|| invalid_operation(self, right, operator))?,
            )),
            BinaryOperator::LessEqual => Ok(Self::Bool(
                self.partial_cmp(&right)
                    .map(|cmp| cmp.is_le())
                    .ok_or_else(|| invalid_operation(self, right, operator))?,
            )),
            BinaryOperator::Greater => Ok(Self::Bool(
                self.partial_cmp(&right)
                    .map(|cmp| cmp.is_gt())
                    .ok_or_else(|| invalid_operation(self, right, operator))?,
            )),
            BinaryOperator::GreaterEqual => Ok(Self::Bool(
                self.partial_cmp(&right)
                    .map(|cmp| cmp.is_ge())
                    .ok_or_else(|| invalid_operation(self, right, operator))?,
            )),
            BinaryOperator::Plus => self + right,
            BinaryOperator::Minus => self - right,
            BinaryOperator::Star => self * right,
            BinaryOperator::Slash => self / right,
        }
    }
    pub fn do_unary_operation(
        self,
        operator: UnaryOperator,
    ) -> Result<Self, ValueError> {
        match operator {
            UnaryOperator::Bang => Ok(Value::Bool(!self.is_truthy())),
            UnaryOperator::Minus => self * Value::Number(-1.0),
        }
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ValueError {
    #[error(
        "can not perform operation {} between the values {} and {}",
        operation,
        left,
        right
    )]
    InvalidOperation {
        operation: String,
        left: Value,
        right: Value,
    },
    #[error("can not use an uninitialized value")]
    UninitializedValueUsed,
    #[error("division by zero of {0}")]
    DivisionByZero(Value),
}

// we have to specify the == here for each pair because
// (l, r) => l.eq(r) does not work
// it just loops back this function again, causing infinite recursion
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Uninitialized, Value::Uninitialized) => true,
            (Value::Call(l), Value::Call(r)) => Rc::ptr_eq(l, r),
            _ => false,
        }
    }
}

// we do not face the same problem as PartialEq because we use Ord for
// all the ordering that we do. Hence, I believe this is never called.
// I also checked this via lldb when I couldn't set a breakpoint in this
// block but could set it for Ord block
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => Some(l.cmp(r)),
            // NOTE: this total_cmp is different in behavior from partial_cmp
            (Value::Number(l), Value::Number(r)) => Some(l.total_cmp(r)),
            (Value::String(l), Value::String(r)) => Some(l.cmp(r)),
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
            (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

// research why can't we derive Eq?
// we can't derive eq because of f64 and later Rc
// f64 is not eq, because it is not reflexive
// NaN == NaN in rust comes out as false when it should be true
// when we manually implement it we are saying that we are okay with it
impl Eq for Value {}

impl Add for Value {
    type Output = Result<Value, ValueError>;
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
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(ValueError::UninitializedValueUsed)
            }
            _ => Err(ValueError::InvalidOperation {
                operation: "+".to_string(),
                left: self,
                right: rhs,
            }),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, ValueError>;
    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(ValueError::UninitializedValueUsed)
            }
            _ => Err(ValueError::InvalidOperation {
                operation: "-".to_string(),
                left: self,
                right: rhs,
            }),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, ValueError>;
    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(ValueError::UninitializedValueUsed)
            }
            _ => Err(ValueError::InvalidOperation {
                operation: "*".to_string(),
                left: self,
                right: rhs,
            }),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, ValueError>;
    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => {
                // NOTE: solution to challenge 3
                //
                // previously when divided by zero, due to rust internals,
                // the number becamse infinite
                // now we will give a runtime error
                if r.eq(&0.0) {
                    return Err(ValueError::DivisionByZero(self));
                }
                Ok(Value::Number(l / r))
            }
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(ValueError::UninitializedValueUsed)
            }
            _ => Err(ValueError::InvalidOperation {
                operation: "/".to_string(),
                left: self,
                right: rhs,
            }),
        }
    }
}

fn invalid_operation(
    left: Value,
    right: Value,
    operator: BinaryOperator,
) -> ValueError {
    ValueError::InvalidOperation {
        operation: operator.to_string(),
        left,
        right,
    }
}
