use std::rc::Rc;
use std::vec::Vec;

use super::Value;

pub struct Pool {
    pool: Vec<Rc<Value>>
}

impl Pool {
    pub fn new() -> Self {
        Pool {
            pool: Vec::new()
        }
    }

    pub fn create(&mut self, val: Value) -> Rc<Value> {
        let p = Rc::new(val);
        self.pool.push(Rc::clone(&p));
        p
    }
}