use std::collections::LinkedList;

use super::error::*;
use super::parser::*;
use super::lexer::*;

mod instruction;
pub use instruction::{Instruction, Code};

pub type Program = Vec<Instruction>;
type ProgramResult = Result<Builder, Error>;

struct Builder {
    list: LinkedList<Instruction>
}

impl Builder {
    pub fn new() -> Self {
        Builder { list: LinkedList::new() }
    }

    pub fn from(instruction: Instruction) -> Self {
        let mut list = LinkedList::new();
        list.push_back(instruction);

        Builder { list }
    }

    pub fn push_back(mut self, instruction: Instruction) -> Self {
        self.list.push_back(instruction);
        self
    }

    pub fn append(mut self, mut builder: Builder) -> Self {
        self.list.append(&mut builder.list);
        self
    }

    pub fn into_iter(self) -> impl Iterator<Item = Instruction> {
        self.list.into_iter()
    }

    pub fn to_vec(self) -> Vec<Instruction> {
        self.into_iter()
            .collect::<Vec<Instruction>>()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}

pub struct Compiler { }

#[allow(dead_code)]
fn unimplemented(offset: usize, width: usize) -> Error {
    Error::new(offset, width, ErrorType::CompilerError(CompilerErrorType::NotImplemented))
}

#[allow(dead_code)]
fn unimplemented_expr(expr: &Expression) -> Error {
    unimplemented(expr.offset, expr.width)
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    fn declaration(&mut self, declaration: &Declaration) -> ProgramResult {
        match &declaration.declaration_type {
            DeclarationType::Statement(statement) => self.statement(&statement)
        }
    }

    fn statement(&mut self, statement: &Statement) -> ProgramResult {
        let mut stmt = match &statement.statement_type {
            StatementType::Expression(expression) => self.expression(&expression)?
        };

        if statement.end {
            stmt = stmt.push_back(Instruction::new(statement.offset, statement.width, Code::Pop));
        }

        Ok(stmt)
    }

    fn expression(&mut self, expr: &Expression) -> ProgramResult {
        Ok(match &expr.expression_type {
            ExpressionType::Primary(primary) => match primary {
                Primary::Literal(literal) => {
                    Builder::from(Instruction::from_expression(&expr, match literal {
                        Literal::Null => Code::PushNull,
                        Literal::Int(i) => Code::PushNum(*i),
                        Literal::Float(f) => Code::PushFloat(*f),
                        Literal::String(s) => Code::PushString(String::from(s))
                    }))
                },
                Primary::Identifier(identifier) => {
                    Builder::from(Instruction::from_expression(&expr, Code::PushVar(String::from(*identifier))))
                }
            },
            ExpressionType::Binary {left, right, operator, offset, width} => {
                let code = match operator {
                    Token::Plus => Code::Add,
                    Token::Minus => Code::Subtract,
                    Token::FSlash => Code::Divide,
                    Token::Asterix => Code::Multiply,
                    Token::Equals => Code::Assign,
                    _ => return Err(
                        unimplemented(*offset, *width)
                            .with_description(format!("unimplemented operator {:?}", operator))
                    )
                };

                Builder::new()
                    .append(self.expression(&*left)?)
                    .append(self.expression(&*right)?)
                    .push_back(Instruction::new(*offset, *width, code))
            },
            ExpressionType::Function {pars, body} => {
                let body = self.get_compiled(body)?;

                Builder::from(Instruction::from_expression(&expr, Code::PushFunction {
                    pars: pars.into_iter()
                        .map(|v| String::from(*v))
                        .collect::<Vec<String>>(),
                    body_len: body.len() + 1 // 1 is the Code::Return
                }))
                .append(body)
                .push_back(Instruction::from_expression(&expr, Code::Return))

            },
            ExpressionType::FunctionCall { func, args } => {
                let args = Builder::new()
                    .append({
                        let mut instructions = Builder::new();
                        for arg in args {
                            instructions = instructions.append(self.expression(&*arg)?);
                        }
                        
                        // Push back amount of arguments
                        instructions.push_back(Instruction::from_expression(&expr, Code::PushNum(args.len() as i32)))
                    })
                    .push_back(Instruction::from_expression(&expr, Code::Return));


                self.expression(&*func)?
                    .push_back(Instruction::from_expression(&expr, Code::CallFunction {
                        args_len: args.len()
                    }))
                    .append(args)
            },
            ExpressionType::List(list) => {
                Builder::new()
                    .append({
                        let mut instructions = Builder::new();
                        for item in list {
                            instructions = instructions.append(self.expression(&*item)?);
                        }

                        instructions
                    })
                    .push_back(Instruction::from_expression(&expr, Code::PushList(list.len() as i32)))
            },
            ExpressionType::Empty => Builder::new(),
            // _ => return Err(unimplemented_expr(&expr))
        })
    }

    fn get_compiled(&mut self, ast: &AST) -> ProgramResult {
        let mut program = Builder::new();

        for declaration in ast {
            program = program.append(self.declaration(&declaration)?);
        }

        Ok(program)
    }

    pub fn compile(&mut self, ast: &AST) -> Result<Program, Error> {
        Ok(self.get_compiled(ast)?.to_vec())
    }
}