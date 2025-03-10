use std::collections::HashMap;

use crate::{error::Error, object::Object, token::Token};

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, Error> {
        let key = &*name.lexeme;
        if let Some(value) = self.values.get(key) {
            Ok((*value).clone())
        } else {
            Err(Error::Runtime {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", key),
            })
        }
    }
}
