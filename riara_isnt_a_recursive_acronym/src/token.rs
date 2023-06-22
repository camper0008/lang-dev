use crate::pos::{Pos}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Eof,
    Error,
    Id,
    Int,
    String,
    Plus,
    Minus,
    Asterisk,
    Slash,
    LParen,
    RParen,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    pos: Pos,
    length: usize,
}

impl Token {
    fn value<'a>(&'a self, text: &'a str) -> &str {
        &text[self.pos.index..self.pos.index + self.length]
    }
}
