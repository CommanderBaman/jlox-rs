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

type Link = Option<Rc<RefCell<EnvironmentBlock>>>;

#[derive(Debug)]
struct EnvironmentBlock {
    // I have to make it option because of root node
    parent: Link,
    store: HashMap<VariableToken, Value>,
}

#[derive(Debug)]
pub struct Environment {
    // we can always access the tail end of the linked list
    tail: Link,
    num_blocks: usize,
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        let mut tail = self.tail.clone();
        while let Some(block) = tail.take() {
            let block = block.borrow();
            string = format!("{} -> {}", block, string);
            tail = block.parent.clone();
        }
        write!(f, "{}", string.split_at(string.len() - 4).0)
    }
}

impl Display for EnvironmentBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .store
            .iter()
            .map(|(variable, value)| format!(" {} : {}", variable, value))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{{{} }}", string)
    }
}

impl EnvironmentBlock {
    fn new(parent: Link) -> Rc<RefCell<EnvironmentBlock>> {
        Rc::new(RefCell::new(Self {
            parent,
            store: HashMap::new(),
        }))
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
            tail: Some(EnvironmentBlock::new(None)),
            num_blocks: 1,
        }
    }
    pub fn pop_child(&mut self) -> Result<(), RuntimeError> {
        let parent = self
            .tail
            .as_ref()
            .ok_or(RuntimeError::RootEnvironmentRemoved)
            .map(|block| block.borrow().parent.clone())?;
        match parent {
            Some(_) => {
                self.tail = parent;
                self.num_blocks -= 1;
                assert!(self.num_blocks >= 1);
                Ok(())
            }
            None => Err(RuntimeError::RootEnvironmentRemoved),
        }
    }
    pub fn push_child(&mut self) {
        let new_block = EnvironmentBlock::new(self.tail.clone());
        self.tail = Some(new_block);
        self.num_blocks += 1;
    }
    pub fn define(
        &mut self,
        name: VariableToken,
        value: Value,
    ) -> Result<(), RuntimeError> {
        let Some(tail) = &self.tail else {
            return Err(RuntimeError::RootEnvironmentRemoved);
        };
        tail.borrow_mut().define(name, value);
        Ok(())
    }
    pub fn get(&self, name: &VariableToken) -> Option<Value> {
        let mut tail = self.tail.clone();
        while let Some(block) = tail.take() {
            let block = block.borrow();
            match block.get(name) {
                Some(v) => return Some(v.clone()),
                None => tail = block.parent.clone(),
            }
        }
        None
    }

    pub fn assign(
        &mut self,
        name: &VariableToken,
        value: Value,
    ) -> Result<(), RuntimeError> {
        let mut tail = self.tail.clone();
        while let Some(block) = tail.take() {
            let mut block = block.borrow_mut();
            match block.get(name) {
                Some(_) => {
                    let result = block.assign(name, value);
                    assert!(result);
                    return Ok(());
                }
                None => {
                    tail = block.parent.clone();
                }
            }
        }
        Err(RuntimeError::VariableNotFound(name.to_string()))
    }

    pub fn lookup(
        &self,
        name: &VariableToken,
        hop: Option<usize>,
    ) -> Option<Value> {
        let num_hops = hop.unwrap_or(self.num_blocks - 1);
        let mut tail = self.tail.clone();
        for _ in 1..=num_hops {
            let Some(block) = tail.take() else {
                break;
            };
            tail = block.borrow().parent.clone();
        }
        tail.and_then(|block| block.borrow().get(name).cloned())
    }
    pub fn assign_at(
        &mut self,
        name: &VariableToken,
        value: Value,
        hop: Option<usize>,
    ) -> Result<(), RuntimeError> {
        let num_hops = hop.unwrap_or(self.num_blocks - 1);
        let mut tail = self.tail.clone();
        for _ in 1..=num_hops {
            let Some(block) = tail.take() else {
                break;
            };
            tail = block.borrow().parent.clone();
        }
        let block = tail.take().ok_or(RuntimeError::EnvironmentNotFound)?;
        let result = block.borrow_mut().assign(name, value);
        assert!(result);
        Ok(())
    }
}
