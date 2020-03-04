use super::error::{Error, ErrorType, VMErrorType};
use super::compiler::{Program, Code, Instruction};

const STACK_SIZE: usize = 512;

type Status = Result<(), Error>;
const STATUS_OK: Status = Ok(());

#[allow(dead_code)]
fn unimplemented(offset: usize, width: usize) -> Error {
    Error::new(offset, width, ErrorType::VMError(VMErrorType::NotImplemented))
}

pub struct VM {
    stacki: i32,
    stack: [i32; STACK_SIZE]
}

impl<'a> VM {
    pub fn new() -> Self {
        VM {
            stacki: 0,
            stack: [0; STACK_SIZE]
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

    fn push(&mut self, instruction: &'a Instruction, val: i32) -> Status {
        self.check_range(instruction, self.stacki + 1)?;
        
        self.stacki += 1;
        self.stack[self.stacki as usize] = val;
        
        STATUS_OK
    }

    fn pop(&mut self, instruction: &'a Instruction) -> Result<i32, Error> {
        self.check_range(instruction, self.stacki - 1)?;

        self.stacki -= 1;
        Ok(self.stack[self.stacki as usize + 1])
    }

    // fn peek(&mut self, instruction: &'a Instruction) -> Result<i32, Error> {

    // }

    fn compute_two_operands(&mut self, instruction: &'a Instruction) -> Status {
        let (second, first) = (self.pop(instruction)?, self.pop(instruction)?);

        let res = match instruction.code {
            Code::Add => first + second,
            Code::Subtract => first - second,
            Code::Multiply => first * second,
            Code::Divide => first / second,
            _ => return Err(unimplemented(instruction.offset, instruction.width))
        };

        self.push(instruction, res)?;

        STATUS_OK
    }

    pub fn exec(&mut self, program: &'a Program) -> Result<String, Error> {
        for instruction in program {
            match instruction.code {
                Code::PushNum(i) => self.push(instruction, i)?,
                Code::Add |
                Code::Subtract |
                Code::Multiply |
                Code::Divide => self.compute_two_operands(instruction)?,
                _ => return Err(
                    unimplemented(instruction.offset, instruction.width)
                        .with_description(format!("Operation not supported: {:?}", instruction.code))
                )
            }
        }

        Ok(format!("{:?}", self.stack[self.stacki as usize]))
    }
}