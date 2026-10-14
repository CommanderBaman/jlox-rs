use std::{
    fmt::{Debug, Display},
    rc::Rc,
    time::{self, UNIX_EPOCH},
};

use crate::{
    error::RuntimeError,
    expression::VariableToken,
    interpreter::{Interpreter, Value},
    statement::{FunctionDeclarationStatement, StatementVisitor},
};

pub type LoxFunction = FunctionDeclarationStatement;

pub trait LoxCallable: Display + Debug {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
}

impl LoxCallable for FunctionDeclarationStatement {
    fn arity(&self) -> usize {
        self.parameters.len()
    }
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if self.arity() != arguments.len() {
            return Err(RuntimeError::IncorrectNumberOfArguments {
                callee_name: self.name.to_string(),
                expected_arguments: self.arity(),
                received_arguments: arguments.len(),
            });
        }
        interpreter.environment.push_child();
        for (name, arg) in self.parameters.iter().zip(arguments.iter()) {
            interpreter
                .environment
                .define(name.to_owned(), arg.to_owned());
        }
        let value = interpreter.visit_block(self.body.clone());
        interpreter.environment.pop_child()?;
        match value {
            Err(RuntimeError::Return(v)) => Ok(v),
            v => v,
        }
    }
}

#[derive(Debug)]
pub struct ClockFunction {}

impl ClockFunction {
    fn as_value() -> Value {
        Value::Call(Rc::new(Self {}))
    }
}

impl LoxCallable for ClockFunction {
    fn arity(&self) -> usize {
        0
    }
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if self.arity() != arguments.len() {
            return Err(RuntimeError::IncorrectNumberOfArguments {
                callee_name: "clock".to_string(),
                expected_arguments: self.arity(),
                received_arguments: arguments.len(),
            });
        }
        Ok(time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|n| Value::Number(n.as_millis() as f64))?)
    }
}
impl Display for ClockFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "clock()")
    }
}

pub fn add_native_functions(interpreter: &mut Interpreter) {
    interpreter.environment.define(
        VariableToken::Identifier("clock".to_owned()),
        ClockFunction::as_value(),
    );
}
