use std::{
    rc::Rc,
    time::{self, UNIX_EPOCH},
};

use crate::{
    error::RuntimeError,
    expression::VariableToken,
    interpreter::{Interpreter, Value},
    statement::{FunctionDeclarationStatement, StatementVisitor},
};

pub trait LoxCallable {
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
        let value = interpreter.visit_block(&self.body);
        interpreter.environment.pop_child()?;
        value
    }
}

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

pub fn add_native_functions(interpreter: &mut Interpreter) {
    interpreter.environment.define(
        VariableToken::Identifer("clock".to_owned()),
        ClockFunction::as_value(),
    );
}
