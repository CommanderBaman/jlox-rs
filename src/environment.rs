use std::collections::HashMap;

use crate::{
    error::RuntimeError, expression::VariableToken, interpreter::Value,
};

pub struct Environment {
    store: HashMap<VariableToken, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: VariableToken, value: Value) {
        self.store.insert(name, value);
    }
    pub fn get(&self, name: &VariableToken) -> Option<&Value> {
        self.store.get(name)
    }
    pub fn assign(
        &mut self,
        name: &VariableToken,
        value: Value,
    ) -> Result<(), RuntimeError> {
        if let Some(old_value) = self.store.get_mut(name) {
            *old_value = value;
            Ok(())
        } else {
            Err(RuntimeError::VariableNotFound(name.to_string()))
        }
    }
}
