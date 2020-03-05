use std::rc::Rc;

use super::{Error, ErrorType, VMErrorType};
use super::{Value, STACK_SIZE, Status, STATUS_OK, Instruction};

pub struct Stack {
    stacki: i32,
    stack: Vec<Option<Rc<Value>>>,
}

impl<'a> Stack {
    pub fn new() -> Self {
        Self {
            stacki: 0,
            stack: std::iter::repeat_with(|| None).take(STACK_SIZE).collect()
        }
    }

    // Always pass the instruction responsible for the range check
    pub fn check_range(&self, instruction: &'a Instruction, index_diff: i32) -> Status {
        let index = self.stacki + index_diff;
        if index < 0 || index >= STACK_SIZE as i32 {
            Err(
                Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::StackOverflow {
                    stack_size: STACK_SIZE as usize,
                    index
                })).with_description(format!("Stack overflow during operation [{:?}]", instruction.code))
            )
        } else {
            STATUS_OK
        }
    }

    pub fn push(&mut self, instruction: &'a Instruction, val: Rc<Value>) -> Status {
        self.check_range(instruction, 1)?;
        
        self.stacki += 1;
        self.stack[self.stacki as usize] = Some(val);
        
        STATUS_OK
    }

    pub fn pop(&mut self, instruction: &'a Instruction) -> Result<Rc<Value>, Error> {
        self.check_range(instruction, -1)?;

        self.stacki -= 1;
        Ok(Rc::clone(
            match self.stack[self.stacki as usize + 1] {
                Some(ref v) => v,
                None => return Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::StackElementUninitialized)))
            }
        ))
    }
}