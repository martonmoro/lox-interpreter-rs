use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{error::Error, object::Object, token::Token};

pub struct Environment {
    values: HashMap<String, Object>,
    pub enclosing: Option<Rc<RefCell<Environment>>>, // Parent-pointer
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

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        // Get the first ancestor
        let parent = self
            .enclosing
            .clone()
            .expect(&format!("No enclosing environment at {}", 1));
        let mut environment = Rc::clone(&parent);

        // Get next ancestor
        for i in 1..distance {
            let parent = environment
                .borrow()
                .enclosing
                .clone()
                .expect(&format!("No enclosing environment at {}", i));
            environment = Rc::clone(&parent);
        }
        environment
    }

    // The older get() method dynamically walks the chain of enclosing
    // envrionments, scouring each one to see if the variable might be hiding in
    // there somewhere. But now we know exactly which environment in the chain
    // will have the variable.
    pub fn get_at(&self, distance: usize, name: &str) -> Result<Object, Error> {
        if distance > 0 {
            Ok(self
                .ancestor(distance)
                .borrow()
                .values
                .get(name)
                .expect(&format!("Undefined variable '{}'", name))
                .clone())
        } else {
            Ok(self
                .values
                .get(name)
                .expect(&format!("Undefined variable '{}'", name))
                .clone())
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

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Object) -> Result<(), Error> {
        if distance > 0 {
            self.ancestor(distance)
                .borrow_mut()
                .values
                .insert(name.lexeme.clone(), value);
        } else {
            self.values.insert(name.lexeme.clone(), value);
        }
        Ok(())
    }
}
