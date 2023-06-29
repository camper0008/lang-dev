use crate::{
    pos::{Error, ErrorCollector, ErrorType, Pos},
    token::{Token, TokenType},
};
use std::str::Chars;

pub struct Lexer<'a> {
    text: &'a str,
    chars: Chars<'a>,
    current: Option<char>,
    index: usize,
    line: usize,
    col: usize,
    error_collector: &'a mut ErrorCollector,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str, error_collector: &'a mut ErrorCollector) -> Self {
        let mut chars = text.chars();
        let current = chars.next();
        Self {
            text,
            chars,
            current,
            index: 0,
            line: 1,
            col: 1,
            error_collector,
        }
    }

    pub fn next_token(&mut self) -> Token {
        let pos = self.pos();
        match self.current {
            Some('a'..='z' | 'A'..='Z' | '_') => {
                self.step();
                loop {
                    match self.current {
                        Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') => self.step(),
                        _ => {
                            break self.token(
                                match pos.value(self.text, self.index - pos.index) {
                                    "false" => TokenType::False,
                                    "true" => TokenType::True,
                                    "not" => TokenType::Not,
                                    "or" => TokenType::Or,
                                    "and" => TokenType::And,
                                    "in" => TokenType::In,
                                    "if" => TokenType::If,
                                    "else" => TokenType::Else,
                                    "let" => TokenType::Let,
                                    _ => TokenType::Id,
                                },
                                pos,
                            )
                        }
                    }
                }
            }
            Some(' ' | '\t' | '\n' | '\r') => {
                self.step();
                loop {
                    match self.current {
                        Some(' ' | '\t' | '\n' | '\r') => self.step(),
                        _ => break self.next_token(),
                    }
                }
            }
            Some('0'..='9') => {
                self.step();
                loop {
                    match self.current {
                        Some('0'..='9') => self.step(),
                        _ => break self.token(TokenType::Int, pos),
                    }
                }
            }
            Some('"') => {
                self.step();
                loop {
                    match self.current {
                        Some('\\') => {
                            self.step();
                            match self.current {
                                Some(_) => self.step(),
                                None => {
                                    self.add_error(
                                        pos.clone(),
                                        "malformed string literal".to_string(),
                                    );
                                    break self.token(TokenType::Error, pos);
                                }
                            }
                        }
                        Some('"') => {
                            self.step();
                            break self.token(TokenType::String, pos);
                        }
                        Some(_) => {
                            self.step();
                        }
                        None => {
                            self.add_error(pos.clone(), "malformed string literal".to_string());
                            break self.token(TokenType::Error, pos);
                        }
                    }
                }
            }
            Some('(') => self.step_and_token(TokenType::LParen),
            Some(')') => self.step_and_token(TokenType::RParen),
            Some('{') => self.step_and_token(TokenType::LBrace),
            Some('}') => self.step_and_token(TokenType::RBrace),
            Some('[') => self.step_and_token(TokenType::LBracket),
            Some(']') => self.step_and_token(TokenType::RBracket),
            Some('.') => self.step_and_token(TokenType::Dot),
            Some(',') => self.step_and_token(TokenType::Comma),
            Some(':') => self.step_and_token(TokenType::Colon),
            Some(';') => self.step_and_token(TokenType::Semicolon),
            Some('+') => self.step_and_token(TokenType::Plus),
            Some('-') => self.step_and_token(TokenType::Minus),
            Some('*') => self.step_and_token(TokenType::Asterisk),
            Some('/') => self.step_and_token(TokenType::Slash),
            Some('=') => self.step_and_token(TokenType::Equal),
            Some(c) => {
                self.step();
                self.add_error(pos.clone(), format!("invalid char '{}'", c));
                self.token(TokenType::Error, pos)
            }
            None => self.token(TokenType::Eof, pos),
        }
    }

    fn step_and_token(&mut self, token_type: TokenType) -> Token {
        let pos = self.pos();
        self.step();
        self.token(token_type, pos)
    }

    fn done(&self) -> bool {
        self.current.is_none()
    }

    fn pos(&self) -> Pos {
        Pos {
            index: self.index,
            line: self.line,
            col: self.col,
        }
    }

    fn token(&self, token_type: TokenType, pos: Pos) -> Token {
        Token {
            token_type,
            length: self.index - pos.index,
            pos,
        }
    }

    fn step(&mut self) {
        self.current = self.chars.next();
        self.index += 1;
        match self.current {
            Some('\n') => {
                self.line += 1;
                self.col = 1;
            }
            Some(_) => {
                self.col += 1;
            }
            None => {}
        }
    }

    fn add_error(&mut self, pos: Pos, message: String) {
        self.error_collector.add(Error {
            error_type: ErrorType::Lexer,
            pos,
            message,
        })
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next_token();
        match next.token_type {
            TokenType::Eof => None,
            _ => Some(next),
        }
    }
}
