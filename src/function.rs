use std::{
    fmt::{Debug, Display},
    rc::Rc,
    time::{self, UNIX_EPOCH},
};

use crate::{
    error::RuntimeError,
    expression::VariableToken,
    interpreter::{Interpreter, InterpreterEnvironment, Value},
    statement::{FunctionDeclarationStatement, StatementVisitor},
};

pub trait LoxCallable: Display + Debug {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
}

#[derive(Debug)]
pub struct LoxFunction {
    declaration: Rc<FunctionDeclarationStatement>,
    environment: InterpreterEnvironment,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: Rc<FunctionDeclarationStatement>,
        environment: InterpreterEnvironment,
        is_initializer: bool,
    ) -> Rc<Self> {
        Rc::new(Self {
            declaration,
            environment,
            is_initializer,
        })
    }
    pub fn using(&self, environment: InterpreterEnvironment) -> Rc<Self> {
        Rc::new(Self {
            declaration: self.declaration.clone(),
            is_initializer: self.is_initializer,
            environment,
        })
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if self.arity() != arguments.len() {
            return Err(RuntimeError::IncorrectNumberOfArguments {
                callee_name: self.declaration.name.to_string(),
                expected_arguments: self.arity(),
                received_arguments: arguments.len(),
            });
        }
        let old_environment =
            interpreter.assign_environment(self.environment.clone());
        interpreter.push_environment();
        for (name, arg) in
            self.declaration.parameters.iter().zip(arguments.iter())
        {
            interpreter
                .environment
                .borrow_mut()
                .define(name.to_owned(), arg.to_owned());
        }
        let value = interpreter.visit_block(self.declaration.body.clone());
        interpreter.pop_environment()?;
        let _ = interpreter.assign_environment(old_environment);
        if self.is_initializer {
            Ok(interpreter
                .environment
                .borrow()
                .get(&VariableToken::This)
                .expect("this to be defined in an init call"))
        } else {
            match value {
                Err(RuntimeError::Return(v)) => Ok(v),
                v => v,
            }
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // NOTE: can not print whole environment or will be stuck in
        // recursive loop
        let initializer_string = if self.is_initializer {
            String::from("initializer ")
        } else {
            String::new()
        };
        write!(
            f,
            "[{}call ({}) in environment with id {})]",
            initializer_string,
            self.declaration,
            self.environment.borrow().id
        )
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
        write!(f, "inbuilt fun clock()")
    }
}

pub fn add_native_functions(
    interpreter: &mut Interpreter,
) -> Result<(), RuntimeError> {
    interpreter.environment.borrow_mut().define(
        VariableToken::Identifier("clock".to_owned()),
        ClockFunction::as_value(),
    );
    Ok(())
}
