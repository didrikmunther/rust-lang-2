use std::collections::HashMap;

use super::super::parser::Expression;

#[derive(Debug)]
pub struct Instruction {
    offset: usize,
    width: usize,
    code: Code
}

impl Instruction {
    pub fn new(offset: usize, width: usize, code: Code) -> Self {
        Instruction {
            offset,
            width,
            code
        }
    }

    pub fn from_expression(expr: &Expression, code: Code) -> Self {
        Instruction::new(expr.offset, expr.width, code)
    }

    // pub fn to_u8(&self, mut content: Option<Vec<u8>>) -> Vec<u8> {
    //     let mut res = vec![self.code as u8, self.offset as u8, self.width as u8];

    //     if let Some(content) = content.as_mut() {
    //         let a: InstructionParser = push_num;
    //         res.append(content);
    //     }

    //     res
    // }
}

type InstructionParser = fn(&[u8]) -> Code;

#[derive(Debug)]
pub enum Code {
    Null,
    
    Add,
    Subtract,
    Multiply,
    Divide,
    Assign,

    PushNum(i32),
    PushFloat(f32),
    PushString(String),
    PushVar(String),
    PushFunction {
        args: Vec<String>,
        body: Vec<Instruction>
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum OPCode {
    NULL = 0x00,
    END = 0x01,

    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
    ASSIGN,

    PUSH_NUM,
    PUSH_FLOAT,
    PUSH_STRING,
    PUSH_FUNCTION
}

use OPCode::*;

lazy_static! {
    pub static ref h: HashMap<OPCode, InstructionParser> = hashmap!{
        PUSH_NUM => push_num as InstructionParser
    };
}

fn push_num(i: &[u8]) -> Code {
    Code::PushNum(5)
}