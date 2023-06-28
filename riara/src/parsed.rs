use crate::pos::Node;

#[derive(Debug)]
pub enum UnaryType {
    Plus,
    Negate,
    Not,
}

#[derive(Debug)]
pub enum BinaryType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Or,
    And,
    Includes,
    Excludes,
}

#[derive(Debug)]
pub enum Expr {
    Error,
    Unit,
    Id(String),
    Int(i64),
    String(String),
    Bool(bool),
    Tuple(Vec<Node<Expr>>),
    Array(Vec<Node<Expr>>),
    Block {
        statements: Vec<Node<Expr>>,
        expr: Option<Box<Node<Expr>>>,
    },
    If {
        condition: Box<Node<Expr>>,
        truthy: Box<Node<Expr>>,
        falsy: Option<Box<Node<Expr>>>,
    },
    Index {
        subject: Box<Node<Expr>>,
        value: Box<Node<Expr>>,
    },
    Call {
        subject: Box<Node<Expr>>,
        arguments: Vec<Node<Expr>>,
    },
    Unary {
        unary_type: UnaryType,
        subject: Box<Node<Expr>>,
    },
    Binary {
        binary_type: BinaryType,
        left: Box<Node<Expr>>,
        right: Box<Node<Expr>>,
    },
    Let {
        id: String,
        value: Box<Node<Expr>>,
    },
}
