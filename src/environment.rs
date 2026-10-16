#![allow(unused_variables)]
use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    error::RuntimeError, expression::VariableToken, interpreter::Value,
};

// NOTE:
// It took some tries to implement this environment
//
// In the first try, I used lifetimes for environment. But I found that
// &mut T is invariant over T. Hence, we can't use lifetimes. We have
// to use Box
// It makes sense though, as we can change Environment
// from a child environment using that pointer
// Reference: https://doc.rust-lang.org/nomicon/subtyping.html
//
// Then I thought, maybe a linked list would work. It becomes complicated.
// I learned from too-many-lists tutorial so I think I should someday
// update this without changing the pub return types
//
// In the end, holding a simple vector of all blocks,
// and pushing and popping environment blocks is enough
//
// Edit: It was not enough. In order to properly implement closures,
// I have to ensure that environments remain connected to one another
// even after they are popped off. I keep thinking what data
// structure other than linked lists would work best here. Linked List
// are the obvious answer. I can not reorganize via a vector because
// environments are like a cactus. Linked lists it is.
// Good bye method chaining (╥﹏╥)
//
// Edit 2: I made a mistake. I thought we need to just pass environment
// into function and we are done. I forgot that environment moves as we
// progress through code. So the function that hold it also move. This
// causes function environment to change as soon as we do pop or push
// instructions on the environment.
// So, I finally implemented here - block for storing variables and
// environment for placing several such stores in memory

type Link = Option<Rc<RefCell<Environment>>>;

#[derive(Debug)]
struct EnvironmentBlock {
    store: HashMap<VariableToken, Value>,
}

#[derive(Debug)]
pub struct Environment {
    pub id: u8,
    pub parent: Link,
    num_parents: usize,
    store: EnvironmentBlock,
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = self.format_stored();
        let mut tail = self.parent.clone();
        while let Some(block) = tail {
            let block = block.borrow();
            string = format!("{} -> {}", string, block.format_stored());
            tail = block.parent.clone();
        }
        write!(f, "{string}")
    }
}

impl Display for EnvironmentBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .store
            .iter()
            .map(|(variable, value)| format!(" {} : {} ", variable, value))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{{{}}}", string)
    }
}

impl EnvironmentBlock {
    fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
    fn define(&mut self, name: VariableToken, value: Value) {
        self.store.insert(name, value);
    }
    fn get(&self, name: &VariableToken) -> Option<&Value> {
        self.store.get(name)
    }
    fn assign(&mut self, name: &VariableToken, value: Value) -> bool {
        if let Some(old_value) = self.store.get_mut(name) {
            *old_value = value;
            return true;
        }
        false
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            num_parents: 0,
            id: rand::random(),
            store: EnvironmentBlock::new(),
        }
    }
    pub fn child(environment: Rc<RefCell<Environment>>) -> Self {
        let num_parents = environment.borrow().num_parents + 1;
        Self {
            parent: Some(environment),
            num_parents,
            id: rand::random(),
            store: EnvironmentBlock::new(),
        }
    }
    pub fn format_stored(&self) -> String {
        format!(
            "[id = {}, num_parents = {}, store = {}]",
            self.id, self.num_parents, self.store
        )
    }
    pub fn define(
        &mut self,
        name: VariableToken,
        value: Value,
    ) -> Result<(), RuntimeError> {
        self.store.define(name, value);
        Ok(())
    }
    // NOTE: this function is not used anywhere
    pub fn get(&self, name: &VariableToken) -> Option<Value> {
        self.store.get(name).cloned().or_else(|| {
            let mut parent = self.parent.clone();
            while let Some(block) = parent {
                let env = block.borrow();
                match env.get(name) {
                    Some(v) => return Some(v),
                    None => parent = env.parent.clone(),
                }
            }
            None
        })
    }
    pub fn lookup(
        &self,
        name: &VariableToken,
        hop: Option<usize>,
    ) -> Option<Value> {
        let num_hops = hop.unwrap_or(self.num_parents);
        if num_hops == 0 {
            return self.store.get(name).cloned();
        }
        // first hop occurs here
        let mut parent = self.parent.clone();
        for _ in 1..=(num_hops - 1) {
            let Some(block) = parent else {
                break;
            };
            parent = block.borrow().parent.clone();
        }
        parent.and_then(|block| block.borrow().store.get(name).cloned())
    }

    pub fn assign(
        &mut self,
        name: &VariableToken,
        value: Value,
    ) -> Result<(), RuntimeError> {
        if self.store.assign(name, value.clone()) {
            return Ok(());
        }
        let mut parent = self.parent.clone();
        while let Some(block) = parent {
            let mut block = block.borrow_mut();
            match block.store.get(name) {
                Some(_) => {
                    let result = block.store.assign(name, value);
                    assert!(result);
                    return Ok(());
                }
                None => {
                    parent = block.parent.clone();
                }
            }
        }
        Err(RuntimeError::VariableNotFound(name.to_string()))
    }
    pub fn assign_at(
        &mut self,
        name: &VariableToken,
        value: Value,
        hop: Option<usize>,
    ) -> Result<(), RuntimeError> {
        let num_hops = hop.unwrap_or(self.num_parents);
        if num_hops == 0 {
            return self.assign(name, value);
        }
        // first hop occurs here
        let mut parent = self.parent.clone();
        for _ in 1..=(num_hops - 1) {
            let Some(block) = parent else {
                break;
            };
            parent = block.borrow().parent.clone();
        }
        let block = parent.ok_or(RuntimeError::EnvironmentNotFound)?;
        let result = block.borrow_mut().store.assign(name, value);
        assert!(result);
        Ok(())
    }
}
