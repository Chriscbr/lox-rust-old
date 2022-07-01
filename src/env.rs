use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Result;
use generational_arena::Arena;
use generational_arena::Index;

use crate::interpreter::RuntimeValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, Index>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }
}

// TODO: extract out "arena" from methods

impl Environment {
    pub fn define(
        &mut self,
        arena: &mut Arena<RuntimeValue>,
        name: String,
        value: RuntimeValue,
    ) -> () {
        let index = arena.insert(value);
        self.values.insert(name, index);
    }

    pub fn get_idx(&self, arena: &Arena<RuntimeValue>, name: &String) -> Result<Index> {
        if let Some(idx) = self.values.get(name) {
            return Ok(*idx);
        }

        if let Some(enclosing) = &self.enclosing {
            return Ok(enclosing.get_idx(arena, name)?);
        } else {
            Err(anyhow!("Undefined variable {}.", name))
        }
    }

    pub fn get(&self, arena: &Arena<RuntimeValue>, name: &String) -> Result<RuntimeValue> {
        let index = self.get_idx(arena, name)?;
        if let Some(value) = arena.get(index) {
            return Ok(value.clone());
        } else {
            return Err(anyhow!("Variable {} unexpectedly deallocated.", name));
        }
    }

    pub fn assign(
        &mut self,
        arena: &mut Arena<RuntimeValue>,
        name: String,
        value: RuntimeValue,
    ) -> Result<()> {
        if let Some(index) = self.values.get(&name) {
            if let Some(old_value) = arena.get_mut(*index) {
                *old_value = value;
                return Ok(());
            } else {
                return Err(anyhow!("Variable {} unexpectedly deallocated.", name));
            }
        }

        if let Some(enclosing) = &mut self.enclosing {
            return Ok(enclosing.assign(arena, name, value)?);
        } else {
            Err(anyhow!("Undefined variable {}.", name))
        }
    }
}
