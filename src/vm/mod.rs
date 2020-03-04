use super::error::{Error, ErrorType, VMErrorType};
use super::compiler::{Program, Code, Instruction};
use std::rc::Rc;

const STACK_SIZE: usize = 512;

type Status = Result<(), Error>;
const STATUS_OK: Status = Ok(());

#[allow(dead_code)]
fn unimplemented(offset: usize, width: usize) -> Error {
    Error::new(offset, width, ErrorType::VMError(VMErrorType::NotImplemented))
}

#[allow(dead_code)]
fn operation_not_supported(instruction: &Instruction, first: &Value, second: &Value) -> Error {
    Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::OperationNotSupported))
        .with_description(format!("Operation {:?} not supported for operands of type {:?} and {:?}", instruction.code, first, second))
}

// pub struct Closure {

// }

struct Pool {
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

#[derive(Debug)]
enum Value {
    Int(i32),
    Float(f32)
}

pub struct VM {
    stacki: i32,
    stack: Vec<Option<Rc<Value>>>,
    pool: Pool
}

impl<'a> VM {
    pub fn new() -> Self {
        VM {
            stacki: 0,
            stack: std::iter::repeat_with(|| None).take(STACK_SIZE).collect(),
            pool: Pool::new()
        }
    }

    // Always pass the instruction responsible for the range check
    fn check_range(&self, instruction: &'a Instruction, index: i32) -> Status {
        if index < 0 || index >= STACK_SIZE as i32 {
            Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::StackOverflow {
                stack_size: STACK_SIZE as usize,
                index
            })).with_description(format!("Stack overflow during operation {:?}", instruction.code)))
        } else {
            STATUS_OK
        }
    }

    fn push(&mut self, instruction: &'a Instruction, val: Rc<Value>) -> Status {
        self.check_range(instruction, self.stacki + 1)?;
        
        self.stacki += 1;
        self.stack[self.stacki as usize] = Some(val);
        
        STATUS_OK
    }

    fn pop(&mut self, instruction: &'a Instruction) -> Result<Rc<Value>, Error> {
        self.check_range(instruction, self.stacki - 1)?;

        self.stacki -= 1;
        Ok(Rc::clone(
            match self.stack[self.stacki as usize + 1] {
                Some(ref v) => v,
                None => return Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::StackElementUninitialized)))
            }
        ))
    }

    // fn peek(&mut self, instruction: &'a Instruction) -> Result<i32, Error> {

    // }

    fn compute_two_operands(&mut self, instruction: &'a Instruction) -> Status {
        let (stack_second, stack_first) = (self.pop(instruction)?, self.pop(instruction)?);

        match (&*stack_first, &*stack_second) {
            (&Value::Int(first), &Value::Int(second)) => {
                let res = match instruction.code {
                    Code::Add => first + second,
                    Code::Subtract => first - second,
                    Code::Multiply => first * second,
                    Code::Divide => first / second,
                    _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
                };

                let val = self.pool.create(Value::Int(res));
                self.push(instruction, val)?;
            },
            _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
        };

        STATUS_OK
    }

    pub fn exec(&mut self, program: &'a Program) -> Result<String, Error> {
        for instruction in program {
            match instruction.code {
                Code::PushNum(i) => {
                    let val = self.pool.create(Value::Int(i));
                    self.push(instruction, val)?;
                },
                Code::PushFloat(f) => {
                    let val = self.pool.create(Value::Float(f));
                    self.push(instruction, val)?;
                },
                Code::Add |
                Code::Subtract |
                Code::Multiply |
                Code::Divide => self.compute_two_operands(instruction)?,

                // Code::PushVar => {},

                _ => return Err(
                    unimplemented(instruction.offset, instruction.width)
                        .with_description(format!("Operation not supported: {:?}", instruction.code))
                )
            }
        }

        Ok(format!("{:?}", self.stack[self.stacki as usize]))
    }
}