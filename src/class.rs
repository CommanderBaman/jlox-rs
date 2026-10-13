use std::rc::Rc;

use crate::{
    error::RuntimeError, function::LoxCallable, interpreter,
    statement::ClassDeclarationStatement,
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    declaration: Rc<ClassDeclarationStatement>,
}
impl LoxClass {
    pub fn new(declaration: Rc<ClassDeclarationStatement>) -> Self {
        Self { declaration }
    }
}
impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} class", self.declaration.name.to_string())
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
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self { class }
    }
}
impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.declaration.name.to_string())
    }
}
