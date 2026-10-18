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

// NOTE: In my version, I have made the classes, functions and instances
// own their environment because I won't be able to make closures otherwise.
// I have written asserts in order to ensure that I am correct

// NOTE: this should not have a LoxCallable implementation, because it makes
// it possible pass in the declaration statement inside the Value object. It
// is easy to make this mistake.
// While writing StatementVisitor, I made a call object using function
// declaration rather than the LoxFunction struct.
// Hardening the code to avoid this mistake will make things more
// complicated than they are now.
// Still, I have done this because it makes it easier for me to reason with
// my code. Any function declaration can be called, and lox function just
// provides the ability to overwrite the environment before running the
// function -> making it a closure
// Also, this provides excellent code reuse. Just take a look at the
// arity function implementations. All point here because function declaration
// is at the root of everything. It is breathtaking, marvelous.
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
        interpreter.push_environment();
        for (name, arg) in self.parameters.iter().zip(arguments.iter()) {
            interpreter
                .environment
                .borrow_mut()
                .define(name.to_owned(), arg.to_owned());
        }
        let value = interpreter.visit_block(self.body.clone());
        interpreter.pop_environment()?;
        match value {
            Err(RuntimeError::Return(v)) => Ok(v),
            v => v,
        }
    }
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
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.arity()
    }
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let old_environment =
            interpreter.assign_environment(self.environment.clone());
        let value = self.declaration.call(interpreter, arguments);
        let _ = interpreter.assign_environment(old_environment);
        if self.is_initializer {
            Ok(self
                .environment
                .borrow()
                .lookup(&VariableToken::This, Some(0))
                .expect("this to be defined in an initialier call"))
        } else {
            value
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
