use std::collections::LinkedList;

use super::error::*;
use super::lexer::*;

type Program<'a> = Result<Vec<Declaration<'a>>, Error>;
type ExpressionResult<'a> = Result<Expression<'a>, Error>;

#[derive(Debug)]
pub struct Declaration<'a> {
    offset: usize,
    width: usize,
    content: &'a str,
    pub declaration_type: DeclarationType<'a>,
}

#[derive(Debug)]
pub enum DeclarationType<'a> {
    Statement(Statement<'a>)
}

#[derive(Debug)]
pub struct Statement<'a> {
    offset: usize,
    width: usize,
    content: &'a str,
    end: bool, // ended with a semicolon
    statement_type: StatementType<'a>
}

#[derive(Debug)]
pub enum StatementType<'a> {
    Expression(Expression<'a>)
}

#[derive(Debug)]
pub struct Expression<'a> {
    offset: usize,
    width: usize,
    content: &'a str,
    expression_type: ExpressionType<'a>
}

#[derive(Debug)]
pub enum ExpressionType<'a> {
    Primary(Primary<'a>),
    Binary(Box<Expression<'a>>, Token, Box<Expression<'a>>)
}

#[derive(Debug)]
pub enum Primary<'a> {
    Literal(&'a Literal),
    Identifier(&'a str)
}

pub struct Parser<'a> {
    index: usize,
    lexed: Vec<&'a Block>
}

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        Parser {
            index: 0,
            lexed: vec![]
        }
    }

    fn get_at(&self, index: usize) -> Option<&'a Block> {
        self.lexed.get(index)
            .map(|v| *v)
    }
 
    fn peek(&self) -> Option<&'a Block> {
        self.get_at(self.index + 1)
    }

    // fn current(&self) -> Option<&'a Block> {
    //     self.get_at(self.index)
    // }

    fn is_end(&self) -> bool {
        self.check(Token::EOF).is_some()
    }

    fn check(&self, token: Token) -> Option<&'a Block> {
        if token != Token::EOF && self.is_end() {
            None
        } else {
            self.peek()
                .and_then(|v| if v.token == token {
                    Some(v)
                } else {
                    None
                })
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn get(&mut self, tokens: &'static [Token]) -> Option<&'a Block> {
        for token in tokens {
            if let Some(block) = self.check(*token) {
                self.advance();
                return Some(block);
            }
        }

        return None;
    }

    fn binary(expr: Expression<'a>, right: Expression<'a>, block: &'a Block) -> Expression<'a> {
        Expression {
            offset: expr.offset,
            width: right.offset + right.width - expr.offset,
            content: &block.content,
            expression_type: ExpressionType::Binary(Box::new(expr), block.token, Box::new(right))
        }
    }

    fn declaration(&mut self) -> Result<Declaration<'a>, Error> {
        let stmt = self.statement()?;

        Ok(Declaration {
            offset: stmt.offset,
            width: stmt.width,
            content: stmt.content,
            declaration_type: DeclarationType::Statement(stmt)
        })
    }

    fn statement(&mut self) -> Result<Statement<'a>, Error> {
        let expr = self.expression()?;

        Ok(Statement {
            offset: expr.offset,
            width: expr.width,
            content: expr.content,
            end: self.get(&[Token::SemiColon]).is_some(),
            statement_type: StatementType::Expression(expr)
        })
    }

    fn expression(&mut self) -> ExpressionResult<'a> {
        let primary = self.addition()?;
        Ok(primary)
    }

    fn addition(&mut self) -> ExpressionResult<'a> {
        let mut expr = self.multiplication()?;

        while let Some(block) = self.get(&[Token::Plus, Token::Minus]) {
            expr = Parser::binary(expr, self.multiplication()?, block);
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> ExpressionResult<'a> {
        let mut expr = self.primary()?;

        while let Some(block) = self.get(&[Token::Asterix, Token::FSlash]) {
            expr = Parser::binary(expr, self.primary()?, block);
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ExpressionResult<'a> {
        if let Some(block) = self.get(&[Token::Literal, Token::Identifier]) {
            Ok(Expression {
                offset: block.offset,
                width: block.width,
                content: &block.content,
                expression_type: match block.block_type {
                    BlockType::Literal(ref literal) => ExpressionType::Primary(Primary::Literal(literal)),
                    BlockType::Identifier(ref identifier) => ExpressionType::Primary(Primary::Identifier(identifier)),
                    _ => return Err(Error::new(0, 0, ErrorType::Unknown))
                }
            })
        } else {
            print!("{:?}\n", self.peek());

            let (offset, width) = self.peek()
                .map(|v| (v.offset, v.width))
                .or(Some((0, 0)))
                .unwrap();
            
            Err(
                Error::new(offset, width, ErrorType::ParserError(ParserErrorType::ExpectedPrimary))
                    .with_description(format!(
                        "Expected primary instead of: {}",
                        self.peek()
                            .map(|v| format!("{:?}", v.block_type))
                            .or(Some(String::from("Unknown block")))
                            .unwrap()
                    ))
            )
        }
    }

    pub fn parse(&mut self, lexed: &'a LinkedList<Block>) -> Program<'a> {
        self.index = 0;
        self.lexed = lexed.into_iter()
            .collect::<Vec<&'a Block>>();

        let mut program: Vec<Declaration<'a>> = vec![];

        while !self.is_end() {
            program.push(self.declaration()?);
        }

        return Ok(program);
    }
}