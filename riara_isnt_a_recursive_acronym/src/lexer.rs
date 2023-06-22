use std::str::Chars;
use crate::pos::{Pos, ErrorType, ErrorCollector};
use crate::token::{TokenType, Token};

pub struct Lexer<'a> {
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
                        _ => break self.token(TokenType::Id, pos),
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
                println!("before: {:#?}", self.current);
                self.step();
                println!("after: {:#?}", self.current);
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
            Some('+') => {
                self.step();
                self.token(TokenType::Plus, pos)
            }
            Some('-') => {
                self.step();
                self.token(TokenType::Minus, pos)
            }
            Some('*') => {
                self.step();
                self.token(TokenType::Asterisk, pos)
            }
            Some('/') => {
                self.step();
                self.token(TokenType::Slash, pos)
            }
            Some('(') => {
                self.step();
                self.token(TokenType::LParen, pos)
            }
            Some(')') => {
                self.step();
                self.token(TokenType::RParen, pos)
            }
            Some(c) => {
                self.step();
                self.add_error(pos.clone(), format!("invalid char '{}'", c));
                self.token(TokenType::Error, pos)
            }
            None => self.token(TokenType::Eof, pos),
        }
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
