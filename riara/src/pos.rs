use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct Pos {
    pub index: usize,
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub fn value<'a>(&'a self, text: &'a str, length: usize) -> &str {
        &text[self.index..self.index + length]
    }
}

#[derive(Debug)]
pub enum ErrorType {
    Lexer,
    Parser,
    Runtime,
}

impl ErrorType {
    pub fn to_string(&self) -> String {
        match self {
            ErrorType::Lexer => "LexerError",
            ErrorType::Parser => "ParserError",
            ErrorType::Runtime => "RuntimeError",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct Error {
    pub error_type: ErrorType,
    pub pos: Pos,
    pub message: String,
}

pub struct ErrorCollector {
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

    pub fn merged_with(mut self, mut other: ErrorCollector) -> Self {
        self.errors.append(&mut other.errors);
        self
    }

    pub fn errors(self) -> Vec<Error> {
        self.errors
    }
}

pub struct Node<T> {
    pub value: T,
    pub pos: Pos,
}

impl<T> Node<T> {
    pub fn new(value: T, pos: Pos) -> Self {
        Self { value, pos }
    }
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!("{:#?}", self.value));
    }
}
