use std::collections::HashMap;

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

pub struct Environment {
    blocks: Vec<EnvironmentBlock>,
}

struct EnvironmentBlock {
    store: HashMap<VariableToken, Value>,
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
            blocks: vec![EnvironmentBlock::new()],
        }
    }
    pub fn pop_child(&mut self) -> Result<(), RuntimeError> {
        // ensure that root is not popped
        if self.blocks.len() == 1 {
            return Err(RuntimeError::RootEnvironmentRemoved);
        }
        self.blocks.pop();
        assert!(self.blocks.len() >= 1);
        Ok(())
    }
    pub fn push_child(&mut self) {
        self.blocks.push(EnvironmentBlock::new());
    }
    fn top_block(&mut self) -> &mut EnvironmentBlock {
        match self.blocks.last_mut() {
            Some(block) => block,
            // unreachable because push and pop functions ensure
            // that root is always present
            None => unreachable!(),
        }
    }
    pub fn define(&mut self, name: VariableToken, value: Value) {
        self.top_block().define(name, value);
    }
    pub fn get(&self, name: &VariableToken) -> Option<&Value> {
        self.blocks.iter().rev().find_map(|block| block.get(name))
    }
    pub fn assign(
        &mut self,
        name: &VariableToken,
        value: Value,
    ) -> Result<(), RuntimeError> {
        let Some(block_to_update) = self
            .blocks
            .iter_mut()
            .rev()
            .find(|block| block.get(name).is_some())
        else {
            return Err(RuntimeError::VariableNotFound(name.to_string()));
        };
        let result = block_to_update.assign(name, value);
        assert!(result);
        Ok(())
    }
    pub fn lookup(
        &self,
        name: &VariableToken,
        hop: Option<usize>,
    ) -> Option<&Value> {
        let block = match hop {
            None => self.blocks.first(),
            Some(hop) => self.blocks.get(self.blocks.len() - hop - 1),
        };
        // TODO: remove this expect, with an error
        // assert as expect
        let block = block.expect("block at hop is present at lookup");
        block.get(name)
    }
    pub fn assign_at(
        &mut self,
        name: &VariableToken,
        value: Value,
        hop: Option<usize>,
    ) -> Result<(), RuntimeError> {
        if let Some(hop) = hop {
            let index = self.blocks.len() - hop - 1;
            let block = self
                .blocks
                .get_mut(index)
                .expect("block at hop is present at assign");
            let result = block.assign(name, value);
            assert!(result);
            Ok(())
        } else {
            self.assign(name, value)
        }
    }
}
