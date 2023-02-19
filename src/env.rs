use std::collections::HashMap;

use generational_arena::Index;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Index>,
}

impl Environment {
    pub fn insert(&self, name: String, value: Index) -> Environment {
        let mut new_env = self.clone();
        new_env.values.insert(name, value);
        new_env
    }

    pub fn enclose(&self) -> Environment {
        Environment {
            enclosing: Some(Box::new(self.clone())),
            ..Default::default()
        }
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
