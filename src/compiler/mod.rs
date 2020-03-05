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
                Builder::from(Instruction::from_expression(&expr, Code::PushFunction {
                    pars: pars.into_iter()
                        .map(|v| String::from(*v))
                        .collect::<Vec<String>>(),
                    body: self.compile(body)?
                }))
            }
            _ => return Err(unimplemented_expr(&expr))
        })
    }

    pub fn compile(&mut self, ast: &AST) -> Result<Program, Error> {
        let mut program = Builder::new();

        for declaration in ast {
            program = program.append(self.declaration(&declaration)?);
        }

        Ok(program.to_vec())
    }
}