use std::collections::HashMap;

use super::error::{Error, ErrorType, VMErrorType};
use super::compiler::{Program, Code, Instruction};
use std::rc::Rc;

const STACK_SIZE: usize = 512;
const NULL: Value = Value::Null;

type Status = Result<(), Error>;
const STATUS_OK: Status = Ok(());

#[allow(dead_code)]
fn unimplemented(offset: usize, width: usize) -> Error {
    Error::new(offset, width, ErrorType::VMError(VMErrorType::NotImplemented))
}

#[allow(dead_code)]
fn operation_not_supported(instruction: &Instruction, first: &Value, second: &Value) -> Error {
    Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::OperationNotSupported))
        .with_description(format!("Operation [{:?}] not supported for operands of type [{:?}] and [{:?}]", instruction.code, first, second))
}

pub struct Scope {
    // root: Option<&'s Scope<'s>>,
    variables: HashMap<String, Rc<Value>>
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new()
        }
    }
}

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
    Null,

    Int(i32),
    Float(f64),
    String(String),

    Variable {
        identifier: String,
        offset: usize,
        width: usize
    }
}

pub struct VM {
    stacki: i32,
    stack: Vec<Option<Rc<Value>>>,
    pool: Pool,
    scope: Scope
}

impl<'a> VM {
    pub fn new() -> Self {
        VM {
            stacki: 0,
            stack: std::iter::repeat_with(|| None).take(STACK_SIZE).collect(),
            pool: Pool::new(),
            scope: Scope::new()
        }
    }

    // Always pass the instruction responsible for the range check
    fn check_range(&self, instruction: &'a Instruction, index: i32) -> Status {
        if index < 0 || index >= STACK_SIZE as i32 {
            Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::StackOverflow {
                stack_size: STACK_SIZE as usize,
                index
            })).with_description(format!("Stack overflow during operation [{:?}]", instruction.code)))
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

    fn get_variable(&self, val: &Rc<Value>) -> Result<Rc<Value>, Error> {
        Ok(match &**val {
            Value::Variable { identifier, offset: _, width: _ } => {
                self.scope.variables.get(identifier)
                    .or(Some(&Rc::from(NULL)))
                    .map(|v| Rc::clone(&v))
                    .unwrap()
            },
            _ => Rc::clone(val)
        })
    }

    fn assign(&mut self, instruction: &'a Instruction) -> Status {
        let (stack_second, stack_first) = (self.pop(instruction)?, self.pop(instruction)?);

        let (identifier, _offset, _width) = match &*stack_first {
            Value::Variable { identifier, offset, width } => (identifier, offset, width),
            _ => return Err(
                Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::AssignToNonVariable))
                    .with_description(format!("cannot assign to [{:?}]", stack_first))
            )
        };

        // check if identifier is valid ie not ""

        let second = self.get_variable(&stack_second)?;
        self.scope.variables.insert(String::from(identifier), second);

        self.push(instruction, Rc::clone(&stack_first))?;
    
        STATUS_OK
    }

    fn compute_two_operands(&mut self, instruction: &'a Instruction) -> Status {
        let (pop_second, pop_first) = (self.pop(instruction)?, self.pop(instruction)?);
        let (stack_second, stack_first) = (self.get_variable(&pop_second)?, self.get_variable(&pop_first)?);

        match (&*stack_first, &*stack_second) {
            (&Value::Int(first), &Value::Int(second)) => {
                let res = match instruction.code {
                    Code::Add => first + second,
                    Code::Subtract => first - second,
                    // Code::Multiply => first * second,
                    Code::Divide => first / second,
                    _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
                };

                let val = self.pool.create(Value::Int(res));
                self.push(instruction, val)?;
            },
            (&Value::Float(first), &Value::Float(second)) => {
                let res = match instruction.code {
                    Code::Add => first + second,
                    Code::Subtract => first - second,
                    Code::Multiply => first * second,
                    Code::Divide => first / second,
                    _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
                };

                let val = self.pool.create(Value::Float(res));
                self.push(instruction, val)?;
            },
            (&Value::Int(first), &Value::Float(second)) => {
                let first = f64::from(first);

                let res = match instruction.code {
                    Code::Add => first + second,
                    Code::Subtract => first - second,
                    Code::Multiply => first * second,
                    Code::Divide => first / second,
                    _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
                };

                let val = self.pool.create(Value::Float(res));
                self.push(instruction, val)?;
            },
            (&Value::Float(first), &Value::Int(second)) => {
                let second = f64::from(second);

                let res = match instruction.code {
                    Code::Add => first + second,
                    Code::Subtract => first - second,
                    Code::Multiply => first * second,
                    Code::Divide => first / second,
                    _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
                };

                let val = self.pool.create(Value::Float(res));
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
                Code::PushString(ref s) => {
                    let val = self.pool.create(Value::String(String::from(s)));
                    self.push(instruction, val)?;
                },
                Code::Add |
                Code::Subtract |
                Code::Multiply |
                Code::Divide => self.compute_two_operands(instruction)?,

                Code::Assign => self.assign(instruction)?,
                Code::PushVar(ref identifier) => {
                    let val = self.pool.create(Value::Variable {
                        identifier: String::from(identifier),
                        offset: instruction.offset,
                        width: instruction.width
                    });
                    self.push(instruction, val)?;
                },

                Code::Pop => { self.pop(instruction)?; },

                _ => return Err(
                    unimplemented(instruction.offset, instruction.width)
                        .with_description(format!("Operation not supported: [{:?}]", instruction.code))
                )
            }
        }

        // println!("variables: {:?}", self.scope.variables);

        Ok(format!(
            "{:?}",
            self.stack[self.stacki as usize]
                .as_ref()
                .map(|v| self.get_variable(&v))
                .unwrap_or(Ok(Rc::from(NULL)))?
        ))
    }
}