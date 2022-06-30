use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Result;

use crate::interpreter::RuntimeValue;

pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, RuntimeValue>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn define(&mut self, name: String, value: RuntimeValue) -> () {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &String) -> Result<RuntimeValue> {
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return Ok(enclosing.get(name)?);
        } else {
            Err(anyhow!("Undefined variable {}.", name))
        }
    }

    pub fn assign(&mut self, name: String, value: RuntimeValue) -> Result<()> {
        if let Some(_) = self.values.get(&name) {
            self.values.insert(name, value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return Ok(enclosing.assign(name, value)?);
        } else {
            Err(anyhow!("Undefined variable {}.", name))
        }
    }
}
