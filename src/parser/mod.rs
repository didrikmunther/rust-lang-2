use linked_list::LinkedList;
use linked_list::Cursor;
// use std::collections::LinkedList;

use super::error::*;
use super::lexer::*;

type Program = Result<Vec<Declaration>, Error>;

pub struct Declaration {
    pos: usize,
    width: usize,
    content: String,
    declaration_type: DeclarationType,
}

pub enum DeclarationType {
    Statement
}

pub struct Statement {
    pos: usize,
    width: usize,
    content: String,
    statement_type: StatementType
}

pub enum StatementType {
    Expression
}

pub struct Expression {
    pos: usize,
    width: usize,
    content: String,
    expression_type: ExpressionType
}

pub enum ExpressionType {
    Primary(Primary),
    Binary(Box<Expression>, Block, Box<Expression>)
}

pub enum Primary {
    Literal(Literal),
    Identifier(String)
}

pub struct Parser<'a> {
    lexed: Option<Cursor<'a, Block>>,
    current: Option<&'a Block>,
    index: usize
}

impl<'a> Parser<'a> {

    

    // fn get(&mut self, tokens: &[Token]) -> Option<Block> {


    //     // let val = self.lexed
    //     unimplemented!()
    // }

    // fn forward(&mut self) -> &Block {
    //     self.current = self.lexed.unwrap().next();
    //     self.current
    // }

    // fn backward(&mut self) -> &Block {
    //     self.current = self.lexed.unwrap().prev();
    //     self.current
    // }

    // fn primary() {

    // }

    // pub fn new() -> Self {
    //     Parser { lexed: None, index: 0, current:  }
    // }

    // pub fn parse(&mut self, lexed: &'a mut LinkedList<Block>) -> Program {
    //     self.index = 0;
    //     self.lexed = Some(lexed.cursor());



    //     self.lexed = None;
    //     unimplemented!();
    // }
}

// fn addition() {
//     let mut expr = next()?;

//     while let Some(operator) = self.do_match(&[Asterix, Slash]) {
//         expr = Expression::Binary(Box::new(expr), operator, next());
//     }
    
//     Ok(expr)
// }