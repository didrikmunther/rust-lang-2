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
const GC_INSTRUCTION_COUNT: usize = 50; // At which amount of instructions to run the GC

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
        // offset: usize,
        // width: usize
    },

    Function {
        // instance: Rc<VMInstance>,
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

    pub fn garbage(&mut self) {
        if let Some(ref mut instance) = self.root_instance {
            println!("Running garbage collector...");
            instance.garbage();

            println!(
                "...garbage collection done.\nContents of pool after garbage collection:\n{:?}",
                instance.scope.borrow().pool.borrow().pool.iter()
                    .map(|v| (&**v, Rc::strong_count(v)))
                    .collect::<Vec<(&Value, usize)>>()
            );
        }
    }
}

pub struct VMInstance {
    scope: Rc<RefCell<Scope>>,
    instruction_count: Rc<RefCell<usize>>
}

impl std::fmt::Debug for VMInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "VMInstance at {:p}", self)
    }
}

impl<'a, 'r> VMInstance {
    pub fn new(parent_scope: Rc<RefCell<Scope>>) -> Self {
        Self::from_instruction_count(parent_scope, Rc::from(RefCell::from(0)))
    }

    fn from_instruction_count(parent_scope: Rc<RefCell<Scope>>, instruction_count: Rc<RefCell<usize>>) -> Self {
        Self {
            scope: Rc::from(RefCell::from(Scope::new(parent_scope))),
            instruction_count
        }
    }

    fn instance(&self) -> Self {
        VMInstance::from_instruction_count(Rc::clone(&self.scope), Rc::clone(&self.instruction_count))
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
            Value::Variable { identifier, .. } => {
                self.scope.borrow_mut().get_variable(identifier)
                    .or(Some(Rc::from(NULL)))
                    .map(|v| v)
                    .unwrap()
            },
            _ => Rc::clone(val)
        })
    }

    fn set_variable(&mut self, identifier: String, val: Rc<Value>) {
        self.scope.borrow_mut().set_variable(identifier, val);
    }

    fn assign(&mut self, instruction: &'a Instruction) -> Status {
        let (stack_second, stack_first) = (self.pop(instruction)?, self.pop(instruction)?);

        let identifier = match &*stack_first {
            Value::Variable { identifier } => identifier,
            _ => return Err(
                Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::AssignToNonVariable))
                    .with_description(format!("cannot assign to [{:?}]", stack_first))
            )
        };

        let second = self.get_variable(&stack_second)?;
        self.scope.borrow_mut().set_variable(String::from(identifier), second);

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

    pub fn do_exec(&mut self, program: &'a Program, from: usize) -> Result<(), Error> {
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
                        identifier: String::from(identifier)
                    });
                    self.push(instruction, val)?;
                },

                Code::PushFunction { body_len, .. } => {
                    let val = self.create(Value::Function {
                        position: index
                    });
                    self.push(instruction, val)?;
                    index += body_len; // Jump past the function body
                },

                Code::CallFunction { args_len } => {
                    let mut instance = self.instance();
                    instance.do_exec(program, index + 1)?;

                    let arg_count: usize = match &*instance.pop(instruction)? {
                        Value::Int(i) => *i as usize,
                        _ => return Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::InvalidArgumentCountType)))
                    };

                    let mut args: Vec<Rc<Value>> = Vec::new();
                    for _ in 0..arg_count {
                        let var = &instance.pop(instruction)?;
                        args.push(instance.get_variable(var)?);
                    }

                    let func = &self.pop(&instruction)?;

                    match &*self.get_variable(func)? {
                        Value::Function { position } => match &program[*position].code {
                            Code::PushFunction { pars, .. } => {
                                if pars.len() != args.len() {
                                    return Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::MismatchedArgumentCount)));
                                }

                                for i in 0..pars.len() {
                                    instance.set_variable(pars[i].clone(), Rc::clone(&args[pars.len() - 1 - i]));
                                }

                                instance.do_exec(program, *position + 1)?;
                                if let Some(val) = instance.pop(instruction).ok() {
                                    self.push(instruction, instance.get_variable(&val)?)?;
                                }
                            },
                            _ => {
                                // println!("{:?}", program[*position].code);
                                return Err(Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::InvalidFunctionValue)))
                            }
                        },
                        _ => {
                            // println!("{:?}: {:?}", func, &*self.get_variable(func)?);
                            println!("{:?}", instruction);
                            return Err(
                                Error::new(instruction.offset, instruction.width, ErrorType::VMError(VMErrorType::InvalidCast))
                                    .with_description(format!("Value [{:?}] could not be cast to [Function] type", func))
                            );
                        }
                    }

                    index += args_len; // Jump past the arguments
                }

                Code::Pop => { self.pop(instruction)?; },
                Code::Return => { break; },

                _ => return Err(
                    unimplemented(instruction.offset, instruction.width)
                        .with_description(format!("Operation not supported: [{:?}]", instruction.code))
                )
            }

            *self.instruction_count.borrow_mut() += 1;
            index += 1;

            if *self.instruction_count.borrow() > GC_INSTRUCTION_COUNT {
                self.garbage();
                *self.instruction_count.borrow_mut() = 0;
            }
        }

        Ok(())
    }

    pub fn garbage(&mut self) {
        self.scope.borrow_mut().garbage();
    }

    pub fn exec(&mut self, program: &'a Program, from: usize) -> Result<String, Error> {
        self.do_exec(program, from)?;

        // println!(
        //     "{:?}",
        //     self.scope.borrow().pool.borrow().pool.iter()
        //         .map(|v| (&**v, Rc::strong_count(v)))
        //         .collect::<Vec<(&Value, usize)>>()
        // );

        // {
        //     let scope = self.scope.borrow();
        //     let stack = scope.stack.borrow();
        //     println!("Stack: {}\n{:?}", stack.stacki, &stack.stack);
        // }

        Ok(format!(
            "{:?}",
            self.pop(&Instruction::new(0, 0, Code::Null))
                .as_ref()
                .map(|v| self.get_variable(&v))
                .unwrap_or(Ok(Rc::from(NULL)))?
        ))
    }
}