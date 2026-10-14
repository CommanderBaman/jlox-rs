use std::{collections::HashMap, rc::Rc};

use crate::{
    error::RuntimeError,
    expression::VariableToken,
    function::{LoxCallable, LoxFunction},
    interpreter::{self, Value},
    statement::ClassDeclarationStatement,
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: VariableToken,
    methods: HashMap<VariableToken, Rc<LoxFunction>>,
}
impl LoxClass {
    pub fn new(declaration: Rc<ClassDeclarationStatement>) -> Self {
        let mut methods = HashMap::new();
        for method in &declaration.functions {
            methods.insert(method.name.clone(), method.clone());
        }
        Self {
            name: declaration.name.clone(),
            methods,
        }
    }
    fn get_method(&self, name: &VariableToken) -> Option<Rc<LoxFunction>> {
        self.methods.get(name).cloned()
    }
}
impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} class", self.name.to_string())
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        0
    }
    fn call(
        &self,
        _interpreter: &mut interpreter::Interpreter,
        _arguments: Vec<interpreter::Value>,
    ) -> Result<interpreter::Value, RuntimeError> {
        Ok(interpreter::Value::ClassInstance(LoxInstance::new(
            self.clone(),
        )))
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<VariableToken, Value>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }
    // I really want to use an option here
    pub fn get(&self, property: &VariableToken) -> Result<Value, RuntimeError> {
        self.fields
            .get(property)
            .cloned()
            .or_else(|| self.class.get_method(property).map(|f| Value::Call(f)))
            .ok_or(RuntimeError::PropertyNotFound(
                property.clone(),
                self.to_string(),
            ))
    }
    pub fn set(&mut self, property: VariableToken, value: Value) {
        self.fields.insert(property, value);
    }
}
impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name.to_string())
    }
}
