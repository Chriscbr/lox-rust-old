use std::collections::HashMap;

use generational_arena::Index;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
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

impl Environment {
    pub fn insert(&self, name: String, value: Index) -> Environment {
        let mut new_env = self.clone();
        new_env.values.insert(name, value);
        new_env
    }

    pub fn enclose(&self) -> Environment {
        let mut new_env = Environment::default();
        new_env.enclosing = Some(Box::new(self.clone()));
        new_env
    }

    pub fn get(&self, name: &String) -> Option<Index> {
        if let Some(idx) = self.values.get(name) {
            return Some(*idx);
        }

        if let Some(enclosing) = &self.enclosing {
            Some(enclosing.get(name)?)
        } else {
            None
        }
    }
}
