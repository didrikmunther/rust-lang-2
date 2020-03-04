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
    stacki: usize,
    stack: [i32; STACK_SIZE]
}

impl<'a> VM {
    pub fn new() -> Self {
        VM {
            stacki: 0,
            stack: [0; STACK_SIZE]
        }
    }

    fn push(&mut self, val: i32) -> Status {
        self.stacki += 1;
        self.stack[self.stacki] = val;
        
        STATUS_OK
    }

    fn pop(&mut self) -> Result<i32, Error> {
        self.stacki -= 1;
        Ok(self.stack[self.stacki + 1])
    }

    fn compute(&mut self, instruction: &'a Instruction) -> Status {
        let res = match instruction.code {
            Code::Add => self.pop()? + self.pop()?,
            _ => return Err(unimplemented(instruction.offset, instruction.width))
        };

        self.push(res)?;

        STATUS_OK
    }

    pub fn exec(&mut self, program: &'a Program) -> Result<String, Error> {
        for instruction in program {
            match instruction.code {
                Code::PushNum(i) => self.push(i)?,
                Code::Add => self.compute(instruction)?,
                _ => return Err(
                    unimplemented(instruction.offset, instruction.width)
                        .with_description(format!("Operation not supported: {:?}", instruction.code))
                )
            }
        }

        Ok(format!("{:?}", self.stack[self.stacki]))
    }
}