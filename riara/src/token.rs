use crate::pos::Pos;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Eof,
    Error,
    Id,
    Int,
    String,
    False,
    True,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Dot,
    Comma,
    Colon,
    Semicolon,
    Plus,
    Minus,
    Asterisk,
    Slash,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub pos: Pos,
    pub length: usize,
}

impl Token {
    pub fn value<'a>(&'a self, text: &'a str) -> &str {
        &text[self.pos.index..self.pos.index + self.length]
    }
}
