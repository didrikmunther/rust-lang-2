use std::rc::Rc;
use std::cell::RefCell;

use super::error::{Error, ErrorType, VMErrorType};
use super::compiler::{Program, Code, Instruction};

mod scope;
mod stack;
mod pool;

use scope::Scope;
use stack::Stack;
use pool::Pool;

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

#[derive(Debug)]
pub enum Value {
    Null,

    Int(i32),
    Float(f64),
    String(String),

    Variable {
        identifier: String,
        offset: usize,
        width: usize
    },

    Function {
        position: usize
    }
}

pub struct VM {
    root_pool: Rc<RefCell<Pool>>,
    root_stack: Rc<RefCell<Stack>>,
    root_scope: Option<Rc<RefCell<Scope>>>,
    root_instance: Option<VMInstance>
}

impl<'a> VM {
    pub fn new() -> Self {
        Self {
            root_pool: Rc::from(RefCell::from(Pool::new())),
            root_stack: Rc::from(RefCell::from(Stack::new())),
            root_scope: None,
            root_instance: None
        }
    }

    pub fn exec(&mut self, program: &'a Program, offset: usize) -> Result<String, Error> {
        if let None = self.root_instance {
            self.root_scope = Some(Rc::from(RefCell::from(Scope::initial(
                Rc::clone(&self.root_pool),
                Rc::clone(&self.root_stack)
            ))));
            self.root_instance = Some(VMInstance::new(Rc::clone(self.root_scope.as_ref().unwrap())));
        }

        let root_instance = self.root_instance.as_mut().unwrap();
        root_instance.exec(program, offset)
    }
}

pub struct VMInstance {
    scope: Rc<RefCell<Scope>>
}

impl<'a, 'r> VMInstance {
    pub fn new(parent_scope: Rc<RefCell<Scope>>) -> Self {
        Self {
            scope: Rc::from(RefCell::from(Scope::new(parent_scope)))
        }
    }

    fn instance(&self) -> Self {
        VMInstance::new(Rc::clone(&self.scope))
    }

    fn push(&mut self, instruction: &'a Instruction, val: Rc<Value>) -> Status {
        self.scope.borrow_mut().stack.borrow_mut().push(instruction, val)
    }

    fn pop(&mut self, instruction: &'a Instruction) -> Result<Rc<Value>, Error> {
        self.scope.borrow_mut().stack.borrow_mut().pop(instruction)
    }

    fn create(&mut self, val: Value) -> Rc<Value> {
        self.scope.borrow_mut().pool.borrow_mut().create(val)
    }

    fn get_variable(&self, val: &Rc<Value>) -> Result<Rc<Value>, Error> {
        Ok(match &**val {
            Value::Variable { identifier, offset: _, width: _ } => {
                self.scope.borrow_mut().variables.get(identifier)
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

        let second = self.get_variable(&stack_second)?;
        self.scope.borrow_mut().variables.insert(String::from(identifier), second);

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
                    Code::Multiply => first * second,
                    Code::Divide => first / second,
                    _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
                };

                let val = self.create(Value::Int(res));
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

                let val = self.create(Value::Float(res));
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

                let val = self.create(Value::Float(res));
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

                let val = self.create(Value::Float(res));
                self.push(instruction, val)?;
            },
            _ => return Err(operation_not_supported(instruction, &*stack_first, &*stack_second))
        };

        STATUS_OK
    }

    pub fn do_exec(&mut self, program: &'a Program, from: usize, offset: usize) -> Result<(), Error> {
        let mut index = from;

        loop {
            if index >= program.len() {
                break;
            }

            let instruction = &program[index];

            match &instruction.code {
                Code::PushNum(i) => {
                    let val = self.create(Value::Int(*i));
                    self.push(instruction, val)?;
                },
                Code::PushFloat(f) => {
                    let val = self.create(Value::Float(*f));
                    self.push(instruction, val)?;
                },
                Code::PushString(ref s) => {
                    let val = self.create(Value::String(String::from(s)));
                    self.push(instruction, val)?;
                },
                Code::Add |
                Code::Subtract |
                Code::Multiply |
                Code::Divide => self.compute_two_operands(instruction)?,

                Code::Assign => self.assign(instruction)?,
                Code::PushVar(ref identifier) => {
                    let val = self.create(Value::Variable {
                        identifier: String::from(identifier),
                        offset: instruction.offset,
                        width: instruction.width
                    });
                    self.push(instruction, val)?;
                },

                Code::PushFunction { pars, body } => {
                    let val = self.create(Value::Function {
                        position: offset + index
                    });
                    self.push(instruction, val)?;
                },

                Code::CallFunction { func, args } => {
                    self.do_exec(func, 0, offset)?;

                    let var = &self.pop(&instruction)?;

                    // println!("{:?}", var);
                    // println!("{:?}", &*self.get_variable(var)?);
                    // println!("{:#?}", program);

                    let func = match &*self.get_variable(var)? {
                        Value::Function { position } => match &program[*position].code {
                            Code::PushFunction { pars, body } => {
                                let mut instance = self.instance();
                                instance.do_exec(body, 0, 0)?;
                                if let Some(val) = instance.pop(instruction).ok() {
                                    self.push(instruction, val)?;
                                }
                            },
                            _ => {
                                println!("{:?}", program[*position].code);
                                return Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::InvalidCast)))
                            }
                        },
                        _ => return Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::InvalidCast)))
                    };

                    // self.do_exec(args, 0, offset)?;
                    // let args = self.pop(&EMPTY_INSTRUCTION)?;
                }

                Code::Pop => { self.pop(instruction)?; },

                _ => return Err(
                    unimplemented(instruction.offset, instruction.width)
                        .with_description(format!("Operation not supported: [{:?}]", instruction.code))
                )
            }

            index += 1;
        }

        Ok(())
    }

    pub fn exec(&mut self, program: &'a Program, from: usize) -> Result<String, Error> {
        self.do_exec(program, from, 0)?;

        // println!("variables: {:?}", self.scope.variables);

        Ok(format!(
            "{:?}",
            self.pop(&Instruction::new(0, 0, Code::Null))
                .as_ref()
                .map(|v| self.get_variable(&v))
                .unwrap_or(Ok(Rc::from(NULL)))?
        ))
    }
}