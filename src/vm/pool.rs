use std::rc::Rc;
use std::vec::Vec;

use super::Value;

pub struct Pool {
    pub pool: Vec<Rc<Value>>
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

    pub fn garbage(&mut self) {
        self.pool = self.pool.iter()
            .filter(|v| Rc::strong_count(v) > 1)
            .map(|v| Rc::clone(v))
            .collect::<Vec<Rc<Value>>>();
    }
}