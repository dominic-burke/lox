use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// A handle to a scope. Scopes are shared so closures can keep their defining
/// environment alive after the enclosing function has returned.
pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<EnvRef>,
}

impl Environment {
    pub fn new() -> Self {
        Environment::default()
    }

    /// A fresh global scope.
    pub fn new_ref() -> EnvRef {
        Rc::new(RefCell::new(Environment::new()))
    }

    /// A scope nested inside `enclosing`.
    pub fn new_child(enclosing: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    /// Looks `name` up in this scope and then outwards. `None` means the
    /// variable is undefined.
    pub fn get(&self, name: &str) -> Option<Value> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => self.enclosing.as_ref()?.borrow().get(name),
        }
    }

    /// Assigns to an existing variable, walking outwards. Returns `false` if
    /// the variable was never defined.
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if let Some(slot) = self.values.get_mut(name) {
            *slot = value;
            return true;
        }
        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign(name, value),
            None => false,
        }
    }
}
