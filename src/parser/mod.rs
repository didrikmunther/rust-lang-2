use std::collections::LinkedList;

use super::error::*;
use super::lexer::*;

pub type AST<'a> = Vec<Declaration<'a>>;
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
    pub statement_type: StatementType<'a>
}

#[derive(Debug)]
pub enum StatementType<'a> {
    Expression(Expression<'a>)
}

#[derive(Debug)]
pub struct Expression<'a> {
    pub offset: usize,
    pub width: usize,
    pub content: &'a str,
    pub expression_type: ExpressionType<'a>
}

#[derive(Debug)]
pub enum ExpressionType<'a> {
    Primary(Primary<'a>),
    Binary {
        left: Box<Expression<'a>>,
        right: Box<Expression<'a>>,
        operator: Token,
        offset: usize, // operator offset
        width: usize // operator width
    },
    Function {
        args: Vec<&'a str>,
        body: AST<'a>
    }
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

    fn reverse(&mut self, index: usize) {
        self.index = index;
    }

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
            expression_type: ExpressionType::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator: block.token,
                offset: block.offset,
                width: block.width
            }
        }
    }

    fn match_lambda(&mut self) -> Result<Option<Expression<'a>>, Error> {
        let start;
        let mut args: Vec<&'a str> = vec![];

        if let Some(parenthesis) = self.get(&[Token::ParOpen]) {
            start = parenthesis.offset;
            while let Some(arg) = self.get(&[Token::Identifier]) {
                args.push(&arg.content);
                if let None = self.get(&[Token::Comma]) {
                    break;
                }
            }
            if let None = self.get(&[Token::ParClosed]) {
                return Ok(None);
            }
        } else if let Some(arg) = self.get(&[Token::Identifier]) {
            start = arg.offset;
            args.push(&arg.content);
        } else {
            return Ok(None);
        }

        if let None = self.get(&[Token::Lambda]) {
            return Ok(None);
        }

        let end: usize;
        let body = if let Some(open_bracket) = self.get(&[Token::BracketOpen]) {
            let mut declarations = vec![];

            loop {
                if self.is_end() {
                    // return Ok(None);
                    return Err(Error::new(
                        open_bracket.offset,
                        open_bracket.width,
                        ErrorType::ParserError(ParserErrorType::UnclosedBracket)
                    ))
                } else if let Some(close_bracket) = self.get(&[Token::BracketClosed]) {
                    end = close_bracket.offset + close_bracket.width;
                    break;
                }

                declarations.push(self.declaration()?);
            }

            declarations
        } else {
            let declaration = self.declaration()?;
            end = declaration.offset + declaration.width;
            vec![declaration]
        };

        Ok(Some(Expression {
            offset: start,
            width: end - start,
            content: "",
            expression_type: ExpressionType::Function {
                args,
                body
            }
        }))
    }

    fn ast(&mut self) -> Result<AST<'a>, Error> {
        let mut ast = vec![];

        while !self.is_end() {
            ast.push(self.declaration()?);
        }

        return Ok(ast);
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
        Ok(self.assign()?)
    }

    fn assign(&mut self) -> ExpressionResult<'a> {
        let mut expr = self.function()?;

        while let Some(block) = self.get(&[Token::Equals]) {
            expr = Parser::binary(expr, self.function()?, block);
        }

        Ok(expr)
    }

    fn function(&mut self) -> ExpressionResult<'a> {
        let reverse = self.index;

        if let Some(function) = self.match_lambda()? {
            return Ok(function);
        } else {
            self.reverse(reverse);
            return self.addition();
        }
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
            return Ok(Expression {
                offset: block.offset,
                width: block.width,
                content: &block.content,
                expression_type: match block.block_type {
                    BlockType::Literal(ref literal) => ExpressionType::Primary(Primary::Literal(literal)),
                    BlockType::Identifier(ref identifier) => ExpressionType::Primary(Primary::Identifier(identifier)),
                    _ => return Err(Error::new(0, 0, ErrorType::Unknown))
                }
            });
        }

        return self.parenthesis();
    }

    fn parenthesis(&mut self) -> ExpressionResult<'a> {
        if let Some(parenthesis) = self.get(&[Token::ParOpen]) {
            let expr = self.expression()?;
            if let None = self.get(&[Token::ParClosed]) {
                return Err(Error::new(
                    parenthesis.offset,
                    parenthesis.width,
                    ErrorType::ParserError(ParserErrorType::UnclosedParenthesis)
                ));
            }
            return Ok(expr);
        }

        return self.error();
    }

    fn error(&mut self) -> ExpressionResult<'a> {
        // print!("{:?}\n", self.peek());

        let (offset, width) = self.peek()
            .map(|v| (v.offset, v.width))
            .or(Some((0, 0)))
            .unwrap();
        
        return Err(
            Error::new(offset, width, ErrorType::ParserError(ParserErrorType::UnexpectedToken))
                .with_description(format!(
                    "Did not expect token [{}]",
                    self.peek()
                        .map(|v| format!("{:?}", v.block_type))
                        .or(Some(String::from("Unknown block")))
                        .unwrap()
                ))
        );
    }

    pub fn parse(&mut self, lexed: &'a LinkedList<Block>) -> Result<AST<'a>, Error> {
        self.index = 0;
        self.lexed = lexed.into_iter()
            .collect::<Vec<&'a Block>>();

        return Ok(self.ast()?);
    }
}