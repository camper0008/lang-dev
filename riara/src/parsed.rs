use crate::pos::Node;

#[derive(Debug)]
pub enum UnaryType {
    Plus,
    Negate,
}

#[derive(Debug)]
pub enum BinaryType {
    Add,
    Subtract,
    Multiply,
    Divide,
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
