use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct Pos {
    index: usize,
    line: usize,
    col: usize,
}

#[derive(Debug)]
enum ErrorType {
    Lexer,
    Parser,
    Runtime,
}

impl ErrorType {
    pub fn to_string(&self) -> String {
        match error.error_type {
            ErrorType::Lexer => "LexerError",
            ErrorType::Parser => "ParserError",
            ErrorType::Runtime => "RuntimeError",
        }.to_string()
    }
}

#[derive(Debug)]
struct Error {
    error_type: ErrorType,
    pos: Pos,
    message: String,
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

    pub fn merged_with(mut self, mut other: ErrorCollector) -> Self {
        self.errors.append(&mut other.errors);
        self
    }

    pub fn errors(mut self) -> Vec<Error> {
        self.errors
    }
}

struct Node<T> {
    value: T,
    pos: Pos,
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
