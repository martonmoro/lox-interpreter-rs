use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{error::Error, object::Object, token::Token};

pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>, // Parent-pointer
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn from(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Rc::clone(enclosing)),
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
            if let Some(ref enclosing) = self.enclosing {
                // it is probably faster to iteratively walk the chain but recursion here is prettier
                enclosing.borrow().get(name)
            } else {
                Err(Error::Runtime {
                    token: name.clone(),
                    message: format!("Undefined variable '{}'.", key),
                })
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), Error> {
        let key = &*name.lexeme;
        if self.values.contains_key(key) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else {
            if let Some(ref enclosing) = self.enclosing {
                enclosing.borrow_mut().assign(name, value)
            } else {
                Err(Error::Runtime {
                    token: name.clone(),
                    message: format!("Undefined variable '{}'.", key),
                })
            }
        }
    }
}
