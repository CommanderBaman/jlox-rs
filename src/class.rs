use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    environment::Environment,
    error::RuntimeError,
    expression::VariableToken,
    function::{LoxCallable, LoxFunction},
    interpreter::{Interpreter, InterpreterEnvironment},
    statement::{ClassDeclarationStatement, FunctionDeclarationStatement},
    value::Value,
};

#[derive(Debug)]
pub struct LoxClass {
    declaration: Rc<ClassDeclarationStatement>,
    environment: InterpreterEnvironment,
    superclass: Option<Rc<LoxClass>>,
}
impl LoxClass {
    pub fn new(
        declaration: Rc<ClassDeclarationStatement>,
        environment: InterpreterEnvironment,
        superclass: Option<Rc<LoxClass>>,
    ) -> Self {
        // if superclass is defined, we need
        // the super defined in the current environment
        if superclass.is_some() {
            assert!(
                environment
                    .borrow()
                    .lookup(&VariableToken::Super, Some(0))
                    .is_some()
            );
        }
        Self {
            declaration,
            environment,
            superclass,
        }
    }
    pub fn get_method(
        &self,
        name: &VariableToken,
    ) -> Option<Rc<FunctionDeclarationStatement>> {
        self.declaration.functions.get(name).cloned().or_else(|| {
            self.superclass.as_ref().and_then(|c| c.get_method(name))
        })
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} class", self.declaration.name)
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        self.declaration
            .functions
            .get(&VariableToken::Identifier("init".to_string()))
            .map(|f| f.arity())
            .unwrap_or(0)
    }
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // extra 1 because of reference to self at last
        if self.arity() + 1 != arguments.len() {
            return Err(RuntimeError::IncorrectNumberOfArguments {
                callee_name: self.declaration.name.to_string(),
                expected_arguments: self.arity(),
                received_arguments: arguments.len(),
            });
        }
        let Some(Value::ClassDeclaration(class)) = arguments.last() else {
            unreachable!("last element of class call should be self reference");
        };
        Ok(Value::ClassInstance(Rc::new(LoxInstance::new(
            class.clone(),
            Rc::new(RefCell::new(Environment::child(self.environment.clone()))),
        ))))
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    fields: RefCell<HashMap<VariableToken, Value>>,
    environment: InterpreterEnvironment,
    class: Rc<LoxClass>,
}

impl LoxInstance {
    pub fn new(
        class: Rc<LoxClass>,
        environment: InterpreterEnvironment,
    ) -> Self {
        Self {
            fields: RefCell::new(HashMap::new()),
            class,
            environment,
        }
    }
    pub fn get(&self, property: &VariableToken) -> Result<Value, RuntimeError> {
        self.fields
            .borrow()
            .get(property)
            .cloned()
            .or_else(|| {
                self.class.get_method(property).map(|f| {
                    Value::Call(LoxFunction::new(
                        f.clone(),
                        self.environment.clone(),
                        f.name == VariableToken::Identifier("init".to_string()),
                    ))
                })
            })
            .ok_or(RuntimeError::PropertyNotFound(
                property.clone(),
                self.to_string(),
            ))
    }
    pub fn set(&self, property: VariableToken, value: Value) {
        self.fields.borrow_mut().insert(property, value);
    }
    pub fn init(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        // 'this' is defined
        assert!(
            self.environment
                .borrow()
                .lookup(&VariableToken::This, Some(0))
                .is_some()
        );
        match self.get(&VariableToken::Identifier("init".to_string())) {
            Ok(call) => {
                call.call(interpreter, arguments)?;
                Ok(())
            }
            Err(RuntimeError::PropertyNotFound(_, _)) => Ok(()),
            Err(e) => Err(e),
        }
    }
    pub fn environment(&self) -> InterpreterEnvironment {
        // 'this' is defined
        assert!(
            self.environment
                .borrow()
                .lookup(&VariableToken::This, Some(0))
                .is_some()
        );
        self.environment.clone()
    }
    pub fn bind(&self, this: Value) {
        self.environment
            .borrow_mut()
            .define(VariableToken::This, this);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields_string = self
            .fields
            .borrow()
            .iter()
            .map(|(variable, value)| format!("{variable} : {value} "))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "{} instance with fields {{ {}}}",
            self.class.declaration.name, fields_string
        )
    }
}
