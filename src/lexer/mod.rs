use std::collections::LinkedList;
use regex::Regex;

use super::error::*;

#[cfg(test)]
mod test;

mod definitions;
use definitions::*;

type LexerResult = Result<LinkedList<Block>, Error>;

#[derive(Debug)]
pub enum BlockType {
    Rest,
    Comment,

    Identifier(String),
    LiteralString(String),
    LiteralInt(i32),
    LiteralFloat(f32),
    Token(Token)
}

#[derive(Debug)]
pub struct Block {
    block_type: BlockType,
    content: String,
    offset: usize,
    width: usize
}

impl Block {
    pub fn new(block_type: BlockType, content: String, offset: usize) -> Self {
        let width = content.len();
        Block { block_type, content, offset, width }
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
            identifier_re: Regex::new(r"[a-zA-Z_]+$").unwrap()
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
        for (symbol, token) in self.tokens.iter() {
            let slice = &content[..symbol.len()];
            if *slice == **symbol {
                return Some(Block::new(BlockType::Token(token.clone()), String::from(slice), offset));
            }
        }

        None
    }

    fn get_identifier(&self, content: &str, offset: usize) -> Option<Block> {
        let mut i = content.len();

        while i > 0 {
            let slice = &content[..i];
            if self.identifier_re.is_match(slice) {
                return Some(Block::new(BlockType::Identifier(String::from(slice)), String::from(slice), offset))
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
                return Some(Block::new(BlockType::LiteralInt(i), String::from(slice), offset))
            } else if let Ok(i) = slice.parse::<f32>() {
                return Some(Block::new(BlockType::LiteralFloat(i), String::from(slice), offset))
            }
            i -= 1;
        }

        None
    }

    fn tokenize(&self, block: Block) -> LexerResult {
        let mut result = LinkedList::<Block>::new();
    
        let mut i: usize = 0;
        let len = block.content.len();
    
        while i < len {
            let slice = &block.content[i..];

            if let Some(token) = self.get_token(slice, i) {
                i += token.width;
                result.push_back(token);
            } else if let Some(literal) = self.get_literal(slice, i) {
                i += literal.width;
                result.push_back(literal);
            } else if let Some(identifier) = self.get_identifier(slice, i) {
                i += identifier.width;
                result.push_back(identifier);
            } else {
                return Err(Error::new(block.offset + i, 0, ErrorType::LexerError(LexerErrorType::UnknownToken)));
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
        let len = block.content.len();
    
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
                            if is_string { BlockType::LiteralString(buf.clone()) } else { BlockType::Rest },
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
            return Err(Error::new(block.offset + last_pos, len - last_pos + 1, ErrorType::LexerError(LexerErrorType::UnexpectedEndOfString)));
        }
    
        if buf.len() >= 1 {
            result.push_back(Block::new(BlockType::Rest, buf, block.offset + last_pos));
        }
    
        Ok(result)
    }

    pub fn lex(&self, query: String) -> LexerResult {
        let w_strings = self.strip_strings(Block::new(BlockType::Rest, query.chars().collect(), 0))?;
        let w_comments = self.replace_rest(w_strings, &Self::strip_comments)?;
        let w_tokens = self.replace_rest(w_comments, &Self::tokenize)?;
    
        Ok(w_tokens)
    }
}