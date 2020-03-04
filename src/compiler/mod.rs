use super::error::*;
use super::parser::*;
use super::lexer::*;

mod instruction;
use instruction::{Instruction, OPCode, Instructions};

pub type Program = Vec<Instruction>;
type ProgramResult = Result<Program, Error>;

pub struct Compiler {

}

fn unimplemented(offset: usize, width: usize) -> Error {
    Error::new(offset, width, ErrorType::CompilerError(CompilerErrorType::NotImplemented))
}

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
        match &statement.statement_type {
            StatementType::Expression(expression) => self.expression(&expression)
        }
    }

    fn expression(&mut self, expr: &Expression) -> ProgramResult {
        Ok(match &expr.expression_type {
            ExpressionType::Primary(primary) => match primary {
                Primary::Literal(literal) => {
                    let (code, instruction) = match literal {
                        Literal::Int(i) => (OPCode::PUSH_NUM, Instructions::PushNum(*i)),
                        Literal::Float(f) => (OPCode::PUSH_FLOAT, Instructions::PushFloat(*f)),
                        Literal::String(s) => (OPCode::PUSH_STRING, Instructions::PushString(String::from(s)))
                    };

                    vec![Instruction::from_expression(&expr, code).with_instruction(instruction)]
                },
                _ => return Err(unimplemented_expr(&expr))
            },
            ExpressionType::Binary {left, right, operator, offset, width} => {
                let code = match operator {
                    Token::Plus => OPCode::ADD,
                    Token::Minus => OPCode::SUBTRACT,
                    Token::FSlash => OPCode::DIVIDE,
                    Token::Asterix => OPCode::MULTIPLY,
                    _ => return Err(
                        unimplemented(*offset, *width)
                            .with_description(format!("unimplemented operator {:?}", operator))
                    )
                };

                let mut result: Vec<Instruction> = Vec::new();
                result.append(&mut self.expression(&*left)?);
                result.append(&mut self.expression(&*right)?);
                result.push(Instruction::from_expression(&expr, code));
                result
            }
            _ => return Err(unimplemented_expr(&expr))
        })
    }

    pub fn compile(&mut self, ast: AST) -> ProgramResult {
        let mut program = vec![];

        for declaration in ast {
            program.append(&mut self.declaration(&declaration)?);
        }

        Ok(program)
    }
}