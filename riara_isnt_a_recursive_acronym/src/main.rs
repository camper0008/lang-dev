#![allow(dead_code)]

use std::{fmt::Debug, str::Chars};

#[derive(Debug, Clone, PartialEq)]
struct Pos {
    index: usize,
    line: usize,
    col: usize,
}

#[derive(Debug)]
enum ErrorType {
    Lexer,
    Parser,
}

#[derive(Debug)]
struct Error {
    error_type: ErrorType,
    pos: Pos,
    text: String,
}

struct ErrorCollector {
    errors: Vec<Error>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn merge(&mut self, mut other: ErrorCollector) {
        self.errors.append(&mut other.errors)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
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
struct Token {
    token_type: TokenType,
    pos: Pos,
    length: usize,
}

impl Token {
    fn value<'a>(&'a self, text: &'a str) -> &str {
        let current = &text[self.pos.index..self.pos.index + self.length];
        println!("c: {current}");
        current
    }
}

struct Lexer<'a> {
    chars: Chars<'a>,
    current: Option<char>,
    index: usize,
    line: usize,
    col: usize,
    error_collector: &'a mut ErrorCollector,
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

    fn add_error(&mut self, pos: Pos, text: String) {
        self.error_collector.add(Error {
            error_type: ErrorType::Lexer,
            pos,
            text,
        })
    }
}

#[derive(Debug)]
enum UnaryType {
    Plus,
    Negate,
}

#[derive(Debug)]
enum BinaryType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

struct Node<T> {
    value: T,
    pos: Pos,
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!("{:#?}", self.value));
    }
}

impl<T> Node<T> {
    pub fn new(value: T, pos: Pos) -> Self {
        Self { value, pos }
    }
}

#[derive(Debug)]
enum Expr {
    Error,
    Id(String),
    Int(i64),
    String(String),
    Unary {
        unary_type: UnaryType,
        subject: Box<Node<Expr>>,
    },
    Binary {
        binary_type: BinaryType,
        left: Box<Node<Expr>>,
        right: Box<Node<Expr>>,
    },
}

struct Parser<'a> {
    text: &'a str,
    lexer: &'a mut Lexer<'a>,
    current: Token,
    error_collector: &'a mut ErrorCollector,
}

impl<'a> Parser<'a> {
    pub fn new(
        text: &'a str,
        lexer: &'a mut Lexer<'a>,
        error_collector: &'a mut ErrorCollector,
    ) -> Self {
        let current = lexer.next_token();
        Self {
            text,
            lexer,
            current,
            error_collector,
        }
    }

    pub fn parse(&mut self) -> Node<Expr> {
        let expr = self.parse_expr();
        match &self.current.token_type {
            TokenType::Eof => expr,
            tt => {
                let pos = self.current.pos.clone();
                self.add_error(pos, format!("expected end, got {tt:#?}"));
                expr
            }
        }
    }

    fn parse_expr(&mut self) -> Node<Expr> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Node<Expr> {
        let mut left = self.parse_factor();
        loop {
            match self.current.token_type {
                TokenType::Plus => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Add,
                            left: Box::new(left),
                            right: Box::new(self.parse_factor()),
                        },
                        pos,
                    )
                }
                TokenType::Minus => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Subtract,
                            left: Box::new(left),
                            right: Box::new(self.parse_factor()),
                        },
                        pos,
                    )
                }
                _ => break left,
            }
        }
    }

    fn parse_factor(&mut self) -> Node<Expr> {
        let mut left = self.parse_unary();
        loop {
            match self.current.token_type {
                TokenType::Asterisk => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Multiply,
                            left: Box::new(left),
                            right: Box::new(self.parse_unary()),
                        },
                        pos,
                    )
                }
                TokenType::Slash => {
                    self.step();
                    let pos = left.pos.clone();
                    left = Node::new(
                        Expr::Binary {
                            binary_type: BinaryType::Divide,
                            left: Box::new(left),
                            right: Box::new(self.parse_unary()),
                        },
                        pos,
                    )
                }
                _ => break left,
            }
        }
    }

    fn parse_unary(&mut self) -> Node<Expr> {
        let pos = self.pos();
        match self.current.token_type {
            TokenType::Plus => {
                self.step();
                Node::new(
                    Expr::Unary {
                        unary_type: UnaryType::Plus,
                        subject: Box::new(self.parse_unary()),
                    },
                    pos,
                )
            }
            TokenType::Minus => {
                self.step();
                Node::new(
                    Expr::Unary {
                        unary_type: UnaryType::Negate,
                        subject: Box::new(self.parse_unary()),
                    },
                    pos,
                )
            }
            _ => self.parse_operand(),
        }
    }

    fn parse_operand(&mut self) -> Node<Expr> {
        match &self.current.token_type {
            TokenType::Int => {
                let current = self.current.clone();
                let number = current.value(self.text).parse().unwrap();
                self.step();
                Node::new(Expr::Int(number), current.pos)
            }
            TokenType::String => {
                let current = self.current.clone();
                let str = String::from(current.value(self.text));
                self.step();
                Node::new(Expr::String(str), current.pos)
            }
            TokenType::LParen => {
                self.step();
                let left = self.parse_expr();
                match self.current.token_type {
                    TokenType::RParen => self.step(),
                    _ => self.add_error(
                        self.pos(),
                        "you forgot a closing parenthesis idiot".to_string(),
                    ),
                };
                left
            }
            token_type => {
                self.add_error(self.pos(), format!("expected operand, got {token_type:#?}"));
                Node::new(Expr::Error, self.pos())
            }
        }
    }

    fn pos(&self) -> Pos {
        self.current.pos.clone()
    }

    fn step(&mut self) {
        self.current = self.lexer.next_token();
    }

    fn add_error(&mut self, pos: Pos, text: String) {
        self.error_collector.add(Error {
            error_type: ErrorType::Parser,
            pos,
            text,
        })
    }
}

#[derive(Debug)]
enum Value {
    Int(i64),
    String(String),
}

fn eval_expr(expr: Node<Expr>) -> Result<Value, ()> {
    match expr.value {
        Expr::Error => Err(()),
        Expr::Id(_) => todo!(),
        Expr::Int(v) => Ok(Value::Int(v)),
        Expr::String(v) => Ok(Value::String(v)),
        Expr::Unary {
            unary_type,
            subject,
        } => {
            let value = eval_expr(*subject)?;
            match (unary_type, value) {
                (UnaryType::Plus, v @ Value::Int(_)) => Ok(v),
                (UnaryType::Negate, Value::Int(v)) => Ok(Value::Int(-v)),
                _ => Err(()),
            }
        }
        Expr::Binary {
            binary_type,
            left,
            right,
        } => {
            let left = eval_expr(*left)?;
            let right = eval_expr(*right)?;
            match (binary_type, left, right) {
                (BinaryType::Add, Value::Int(left), Value::Int(right)) => {
                    Ok(Value::Int(left + right))
                }
                (BinaryType::Subtract, Value::Int(left), Value::Int(right)) => {
                    Ok(Value::Int(left - right))
                }
                (BinaryType::Multiply, Value::Int(left), Value::Int(right)) => {
                    Ok(Value::Int(left * right))
                }
                (BinaryType::Divide, Value::Int(left), Value::Int(right)) => {
                    Ok(Value::Int(left / right))
                }
                _ => Err(()),
            }
        }
    }
}

fn main() {
    let text = "1 + 2 * -(3 - 4) + 1";
    let mut lexer_collector = ErrorCollector::new();
    let mut lexer = Lexer::new(text, &mut lexer_collector);
    let mut parser_collector = ErrorCollector::new();
    let mut parser = Parser::new(text, &mut lexer, &mut parser_collector);
    let expr = parser.parse();
    println!("Parsed: {expr:#?}");
    let value = eval_expr(expr);
    println!("Value: {value:#?}");

    parser_collector.merge(lexer_collector);
    println!("{:#?}", parser_collector.errors);
}
