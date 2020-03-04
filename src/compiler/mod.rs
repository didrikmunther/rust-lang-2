use super::error::*;
use super::parser::*;
use super::lexer::*;

mod instruction;
use instruction::{Instruction, OPCode, Instructions};

pub type Program = Vec<Instruction>;
type ProgramResult = Result<Program, Error>;

pub struct Compiler {

}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    fn declaration(&mut self, declaration: Declaration) -> ProgramResult {
        match declaration.declaration_type {
            DeclarationType::Statement(statement) => self.statement(statement)
        }
    }

    fn statement(&mut self, statement: Statement) -> ProgramResult {
        match statement.statement_type {
            StatementType::Expression(expression) => self.expression(expression)
        }
    }

    fn expression(&mut self, expression: Expression) -> ProgramResult {
        Ok(match &expression.expression_type {
            ExpressionType::Primary(primary) => match primary {
                Primary::Literal(literal) => {
                    let (code, instruction) = match literal {
                        Literal::Int(i) => (OPCode::PUSH_NUM, Instructions::PushNum(*i)),
                        Literal::Float(f) => (OPCode::PUSH_FLOAT, Instructions::PushFloat(*f)),
                        Literal::String(s) => (OPCode::PUSH_STRING, Instructions::PushString(String::from(s)))
                    };

                    vec![Instruction::from_expression(&expression, code).with_instruction(instruction)]
                },
                _ => unimplemented!()
            },
            _ => unimplemented!()
        })
    }

    pub fn compile(&mut self, ast: AST) -> ProgramResult {
        let mut program = vec![];

        for declaration in ast {
            program.append(&mut self.declaration(declaration)?);
        }

        Ok(program)
    }
}