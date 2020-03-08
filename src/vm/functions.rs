use std::rc::Rc;
use std::cell::RefCell;

use super::{VMInstance, Value, Error};

type NativeInstance = Rc<RefCell<VMInstance>>;
type NativeValue = Rc<Value>;
type NativeReturn = Result<NativeValue, Error>;

pub type NativeFunction = fn(NativeInstance, Vec<NativeValue>) -> NativeReturn;

const NULL: Value = Value::Null;

pub fn print_value(_instance: NativeInstance, args: Vec<NativeValue>) -> NativeReturn {
    for arg in args {
        print!("{:?} ", arg);
    }

    Ok(Rc::from(NULL))
}

// pub fn range(instance: NativeInstance, args: NativeArgs) {
    
// }