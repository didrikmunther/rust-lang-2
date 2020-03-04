use std::collections::LinkedList;
// use linked_list::LinkedList;
use regex::Regex;

use super::error::*;

#[cfg(test)]
mod test;

mod definitions;
pub use definitions::*;

pub type LexerResult = Result<LinkedList<Block>, Error>;

#[derive(Debug)]
pub enum BlockType {
    Rest,
    Comment,

    Identifier(String),
    Literal(Literal),
    Token(Token)
}

#[derive(Debug)]
pub enum Literal {
    Null,
    String(String),
    Int(i32),
    Float(f64)
}

#[derive(Debug)]
pub struct Block {
    pub block_type: BlockType,
    pub token: Token,
    pub content: String,
    pub offset: usize,
    pub width: usize
}

impl Block {
    pub fn new(
        block_type: BlockType,
        token: Token,
        content: String,
        offset: usize
    ) -> Self {
        let width = content.len();
        Block { block_type, token, content, offset, width }
    }
}

fn get_last(positions: &Vec<usize>) -> usize {
    let len = positions.len();
    if len <= 0 { 0 } else { *positions.get(len - 1).or(Some(&0)).unwrap() }
}

pub struct Lexer {
    tokens: Vec<(String, Token)>,
    identifier_re: Regex
}

impl Lexer {
    pub fn new() -> Self {
        let mut tokens = TOKENS.iter()
            .map(|(k, v)| (String::from(*k), v.clone()))
            .collect::<Vec<(String, Token)>>();
            
        tokens.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        Lexer {
            tokens,
            identifier_re: Regex::new(r"(^[a-zA-Z_][0-9a-zA-Z_]+$)|(^[a-zA-Z_]+$)").unwrap()
        }
    }

    fn replace_rest(&self, list: LinkedList<Block>, f: &dyn Fn(&Self, Block) -> LexerResult) -> LexerResult {
        let mut result = LinkedList::<Block>::new();
    
        for v in list.into_iter() {
            match v.block_type {
                BlockType::Rest => f(self, v)?.into_iter().for_each(|v| result.push_back(v)),
                _ => result.push_back(v)
            }
        };
    
        Ok(result)
    }

    fn get_token(&self, content: &str, offset: usize) -> Option<Block> {
        let len = content.len();
        
        for (symbol, token) in self.tokens.iter().filter(|(s, _)| s.len() <= len) {
            let slice = &content[..symbol.len()];
            if *slice == **symbol {
                return Some(Block::new(
                    BlockType::Token(token.clone()),
                    token.clone(),
                    String::from(slice),
                    offset
                ));
            }
        }

        None
    }

    fn get_identifier(&self, content: &str, offset: usize) -> Option<Block> {
        let mut i = content.len();

        while i > 0 {
            let slice = &content[..i];
            if self.identifier_re.is_match(slice) {
                return Some(Block::new(
                    BlockType::Identifier(String::from(slice)),
                    Token::Identifier,
                    String::from(slice),
                    offset
                ));
            }
            i -= 1;
        }

        None
    }

    fn get_literal(&self, content: &str, offset: usize) -> Option<Block> {
        let mut i = content.len();

        while i > 0 {
            let slice = &content[..i];
            if let Ok(i) = slice.parse::<i32>() {
                return Some(Block::new(
                    BlockType::Literal(Literal::Int(i)),
                    Token::Literal,
                    String::from(slice),
                    offset
                ));
            } else if let Ok(f) = slice.parse::<f64>() {
                return Some(Block::new(
                    BlockType::Literal(Literal::Float(f)),
                    Token::Literal,
                    String::from(slice),
                    offset
                ));
            } else if "null" == slice {
                return Some(Block::new(
                    BlockType::Literal(Literal::Null),
                    Token::Literal,
                    String::from(slice),
                    offset
                ));
            }
            i -= 1;
        }

        None
    }

    fn tokenize(&self, block: Block) -> LexerResult {
        let mut result = LinkedList::<Block>::new();
    
        let mut i: usize = 0;
        let len = block.content.len();
        let offset = block.offset;
    
        while i < len {
            let slice = &block.content[i..];

            if let Some(token) = self.get_token(slice, i + offset) {
                i += token.width;
                result.push_back(token);
            } else if let Some(literal) = self.get_literal(slice, i + offset) {
                i += literal.width;
                result.push_back(literal);
            } else if let Some(identifier) = self.get_identifier(slice, i + offset) {
                i += identifier.width;
                result.push_back(identifier);
            } else {
                return Err(
                    Error::new(block.offset + i, 1, ErrorType::LexerError(LexerErrorType::UnknownToken))
                        .with_help(String::from("this token is not recognized"))
                );
            }
        }
    
        Ok(
            result.into_iter()
                .filter(|v| {
                    match v.block_type {
                        BlockType::Token(Token::Space) |
                        BlockType::Token(Token::Tab) |
                        BlockType::Token(Token::NewLine) => false,
                        _ => true
                    }
                })
                .collect()
        )
    }

    fn strip_comments(&self, block: Block) -> LexerResult {
        let mut result = LinkedList::<Block>::new();
        let mut buf: String = String::new();
    
        let mut positions: Vec<usize> = vec![];
    
        let mut comment_count = 0;
        let mut is_comment = false;
    
        for (i, v) in block.content.chars().enumerate() {
            if v != '/' {
                comment_count = 0;
            }
    
            match v {
                '/' => {
                    comment_count += 1;
                    if !is_comment && comment_count >= 2 {
                        buf.pop();
    
                        if buf.len() > 0 {
                            result.push_back(Block::new(
                                BlockType::Rest,
                                Token::Rest,
                                buf,
                                block.offset + get_last(&positions)
                            ));
                        }
    
                        positions.push(i + 1);
                        buf = String::new();
                        is_comment = true;
    
                        continue;
                    }
                },
                '\n' => {
                    if is_comment {
                        result.push_back(Block::new(
                            BlockType::Comment,
                            Token::Comment,
                            buf,
                            block.offset + get_last(&positions)
                        ));
    
                        positions.push(i + 1);
                        buf = String::new();
                        is_comment = false;
    
                        continue;
                    }
                },
                _ => {}
            }
    
            buf.push(v);
        }
    
        if buf.len() > 0 {
            result.push_back(Block::new(
                if is_comment { BlockType::Comment } else { BlockType::Rest },
                if is_comment { Token::Comment } else { Token::Rest },
                buf,
                block.offset + get_last(&positions)
            ));
        }
    
        Ok(result)
    }
    
    fn strip_strings(&self, block: Block) -> LexerResult {
        let mut escaped = false;
        let mut is_string = false;
        let mut comment_count = 0;
        let mut is_comment = false;
    
        let mut positions: Vec<usize> = vec![];
    
        let mut result = LinkedList::<Block>::new();
        let mut buf: String = String::new();
    
        for (i, v) in block.content.chars().enumerate() {
            if is_comment && v != '\n' {
                buf.push(v);
                continue;
            }
    
            if v != '/' {
                comment_count = 0;
            }
    
            match v {
                '/' => {
                    if !is_string {
                        comment_count += 1;
                        if comment_count >= 2 {
                            is_comment = true;
                        }
                    }
                },
                '\n' => {
                    comment_count = 0;
                    is_comment = false;
                },
                '\\' => {
                    escaped = !escaped && is_string;
                    if escaped { continue; }
                },
                '"' => {
                    if !escaped {
                        result.push_back(Block::new(
                            if is_string { BlockType::Literal(Literal::String(buf.clone())) } else { BlockType::Rest },
                            if is_string { Token::Literal } else { Token::Rest },
                            buf,
                            block.offset + get_last(&positions)
                        ));
    
                        positions.push(i + 1);
                        buf = String::new();
                        is_string = !is_string;
                        continue;
                    }
                },
                _ => {}
            }
    
            escaped = false;
            buf.push(v);
        }
    
        let last_pos = get_last(&positions);
    
        if is_string {
            return Err(
                Error::new(block.offset + last_pos - 1, 1, ErrorType::LexerError(LexerErrorType::UnexpectedEndOfString))
                    .with_help(String::from("unclosed quotation mark"))
            );
        }
    
        if buf.len() >= 1 {
            // println!("{}, {}, {}", buf, block.offset, last_pos);
            result.push_back(Block::new(
                BlockType::Rest,
                Token::Rest,
                buf,
                block.offset + last_pos
            ));
        }

        Ok(result)
    }

    pub fn lex(&self, query: String) -> LexerResult {
        let w_strings = self.strip_strings(Block::new(BlockType::Rest, Token::Rest, query.chars().collect(), 0))?;
        let w_comments = self.replace_rest(w_strings, &Self::strip_comments)?;
        let mut w_tokens = self.replace_rest(w_comments, &Self::tokenize)?;

        w_tokens.push_front(Block::new(
            BlockType::Token(Token::SOF),
            Token::SOF,
            String::from(""),
            0
        ));

        let last = w_tokens.back().unwrap();
        let (offset, width) = (last.offset, last.width);

        w_tokens.push_back(Block::new(
            BlockType::Token(Token::EOF),
            Token::EOF,
            String::from(" "),
            offset + width + 1
        ));
    
        Ok(w_tokens)
    }
}