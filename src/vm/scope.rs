use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use super::{Pool, Stack, Value};

pub struct Scope {
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub variables: HashMap<String, Rc<Value>>,
    pub pool: Rc<RefCell<Pool>>,
    pub stack: Rc<RefCell<Stack>>,
}

impl Scope {
    pub fn new(parent: Rc<RefCell<Scope>>) -> Self {
        let p = parent.borrow_mut();

        Scope {
            parent: Some(Rc::clone(&parent)),
            variables: HashMap::new(),
            pool: Rc::clone(&p.pool),
            stack: Rc::clone(&p.stack),
        }
    }

    pub fn initial(pool: Rc<RefCell<Pool>>, stack: Rc<RefCell<Stack>>) -> Self {
        Scope {
            parent: None,
            variables: HashMap::new(),
            pool,
            stack,
        }
    }
}